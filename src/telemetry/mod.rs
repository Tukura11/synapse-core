//! Telemetry module with input validation and reconnection logic.

pub mod error_handling;
pub mod input_validation;
pub mod reconnection;

pub use error_handling::{ErrorAction, ErrorHandler, TelemetryError, TelemetryResult};
pub use input_validation::InputValidator;
pub use reconnection::ReconnectionManager;
