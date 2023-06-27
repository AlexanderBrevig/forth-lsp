pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;

// Usual wrapper, but evidently not needed this time
// pub struct W<T>(pub T);
