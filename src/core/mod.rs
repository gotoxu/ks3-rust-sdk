#[cfg(feature = "rustls")]
use hyper_rustls as tls;
#[cfg(feature = "native-tls")]
use hyper_tls as tls;

pub mod client;
#[doc(hidden)]
pub mod encoding;
pub mod error;
#[doc(hidden)]
pub mod proto;
pub mod region;
pub mod request;

pub use crate::core::client::Client;
pub use crate::core::region::Region;
pub use crate::core::request::HttpClient;
pub use crate::core::request::{BufferedHttpResponse, DispatchSignedRequest, HttpResponse};
