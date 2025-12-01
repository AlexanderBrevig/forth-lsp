pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;

// Logging macros
pub use crate::{log_debug, log_handler_error, log_request, log_request_msg};

// Usual wrapper, but evidently not needed this time
// pub struct W<T>(pub T);
