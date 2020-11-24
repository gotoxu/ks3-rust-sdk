pub mod core;
pub mod credential;
mod request;
mod s3;
mod signature;

pub use crate::request::*;
pub use crate::s3::{S3Client, S3};
