//! Telemetry module with input validation, reconnection logic, and connection pooling.
//!
//! # Health Checks
//!
//! The telemetry module implements several health checks to ensure the OpenTelemetry exporter
//! remains healthy and resilient:
//!
//! - **Connection Pool Health**: [`ConnectionPool`] enforces a hard cap on pool size and evicts
//!   stale idle connections, preventing resource exhaustion.
//! - **Exporter Connectivity**: [`ReconnectionManager`] monitors connection attempts with
//!   exponential backoff and circuit breaker pattern. When the circuit is open, requests are
//!   rejected immediately; when closed, the circuit auto-resets after a configured duration.
//! - **Error Handling**: [`ErrorHandler`] tracks telemetry operation errors and determines
//!   whether to continue or fail fast based on error type and threshold. Graceful degradation
//!   ensures the application continues even if the exporter is unavailable.
//!
//! See [`docs/telemetry-health-checks.md`](https://github.com/Synapse-bridgez/synapse-core/blob/main/docs/telemetry-health-checks.md)
//! for integration details and how to add new health checks.

pub mod connection_pool;
pub mod error_handling;
pub mod input_validation;
pub mod reconnection;

pub use connection_pool::{ConnectionPool, PoolConfig, PoolError};
pub use error_handling::{ErrorAction, ErrorHandler, TelemetryError, TelemetryResult};
pub use input_validation::InputValidator;
pub use reconnection::ReconnectionManager;
