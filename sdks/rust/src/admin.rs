use crate::client::parse_error_message;
use crate::error::SynapseError;
use crate::retry::retry_with_backoff;
use serde::de::DeserializeOwned;

/// HTTP client for admin-only Synapse API endpoints.
///
/// Obtain via [`crate::SynapseClient::as_admin`]. Sends
/// `Authorization: Bearer <admin_key>` on every request, mirroring the
/// server's `admin_auth` middleware. Public resource methods are not
/// available on this type, preventing accidental mix-up of admin and
/// public-API scopes.
#[derive(Clone)]
pub struct AdminClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) admin_key: String,
    pub(crate) max_attempts: u32,
    pub(crate) base_delay_ms: u64,
}

impl AdminClient {
    pub(crate) fn new(
        http: reqwest::Client,
        base_url: String,
        admin_key: String,
        max_attempts: u32,
        base_delay_ms: u64,
    ) -> Self {
        Self {
            http,
            base_url,
            admin_key,
            max_attempts,
            base_delay_ms,
        }
    }

    /// Issue an admin-authenticated GET request to `path` and deserialize the JSON response.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, SynapseError> {
        let url = format!("{}{}", self.base_url, path);
        let key = self.admin_key.clone();
        let http = self.http.clone();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            async move {
                let resp = http
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", key))
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    let message = parse_error_message(&body).unwrap_or(body);
                    return Err(SynapseError::Api { status, message });
                }
                resp.json::<T>().await.map_err(|e| SynapseError::Decode(e.to_string()))
            }
        })
        .await
    }

    /// Issue an admin-authenticated GET request with query parameters and deserialize the JSON response.
    pub async fn get_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, SynapseError> {
        let url = format!("{}{}", self.base_url, path);
        let key = self.admin_key.clone();
        let http = self.http.clone();
        let query: Vec<(String, String)> = query
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            let query = query.clone();
            async move {
                let resp = http
                    .get(&url)
                    .query(&query)
                    .header("Authorization", format!("Bearer {}", key))
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    let message = parse_error_message(&body).unwrap_or(body);
                    return Err(SynapseError::Api { status, message });
                }
                resp.json::<T>().await.map_err(|e| SynapseError::Decode(e.to_string()))
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SynapseClient;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn admin_get_sends_bearer_token() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/admin/status"))
            .and(header("Authorization", "Bearer admin-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "public-key");
        let admin = client.as_admin("admin-secret");
        let result: Result<serde_json::Value, _> = admin.get("/admin/status").await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    }

    #[tokio::test]
    async fn admin_get_does_not_send_api_key_header() {
        let server = MockServer::start().await;

        // Only match requests WITHOUT X-API-Key — if the client sends it, this mock won't fire
        // and the test will get a 404 (unmatched), revealing the bug.
        Mock::given(method("GET"))
            .and(path("/admin/status"))
            .and(header("Authorization", "Bearer admin-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "public-key");
        let admin = client.as_admin("admin-secret");
        let result: Result<serde_json::Value, _> = admin.get("/admin/status").await;

        assert!(result.is_ok(), "admin client must use Bearer, not X-API-Key");
    }

    #[tokio::test]
    async fn admin_get_returns_api_error_on_401() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/admin/secret"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_string("Unauthorized"),
            )
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "public-key");
        let admin = client.as_admin("wrong-key");
        let result: Result<serde_json::Value, _> = admin.get("/admin/secret").await;

        assert!(
            matches!(result, Err(SynapseError::Api { status: 401, .. })),
            "expected Api(401), got: {:?}",
            result
        );
    }
}
