use std::error::Error;
use std::fmt;
use std::io;

use crate::core::client::SignAndDispatchError;
use crate::core::proto::xml::util::XmlParseError;
use crate::core::request::BufferedHttpResponse;
use crate::core::request::HttpDispatchError;
use crate::credential::CredentialsError;

/// Generic error type returned by all rusoto requests.
#[derive(Debug, PartialEq)]
pub enum Ks3Error<E> {
    /// A service-specific error occurred.
    Service(E),
    /// An error occurred dispatching the HTTP request
    HttpDispatch(HttpDispatchError),
    /// An error was encountered with AWS credentials.
    Credentials(CredentialsError),
    /// A validation error occurred.  Details from AWS are provided.
    Validation(String),
    /// An error occurred parsing the response payload.
    ParseError(String),
    /// An unknown error occurred.  The raw HTTP response is provided.
    Unknown(BufferedHttpResponse),
    /// An error occurred when attempting to run a future as blocking
    Blocking,
}

/// Result carrying a generic `Ks3Error`.
pub type Ks3Result<T, E> = Result<T, Ks3Error<E>>;

/// Header used by AWS on responses to identify the request
pub const AWS_REQUEST_ID_HEADER: &str = "x-amzn-requestid";

impl<E> From<XmlParseError> for Ks3Error<E> {
    fn from(err: XmlParseError) -> Self {
        let XmlParseError(message) = err;
        Ks3Error::ParseError(message)
    }
}

impl<E> From<serde_json::error::Error> for Ks3Error<E> {
    fn from(err: serde_json::error::Error) -> Self {
        Ks3Error::ParseError(err.to_string())
    }
}

impl<E> From<CredentialsError> for Ks3Error<E> {
    fn from(err: CredentialsError) -> Self {
        Ks3Error::Credentials(err)
    }
}

impl<E> From<HttpDispatchError> for Ks3Error<E> {
    fn from(err: HttpDispatchError) -> Self {
        Ks3Error::HttpDispatch(err)
    }
}

impl<E> From<SignAndDispatchError> for Ks3Error<E> {
    fn from(err: SignAndDispatchError) -> Self {
        match err {
            SignAndDispatchError::Credentials(e) => Self::from(e),
            SignAndDispatchError::Dispatch(e) => Self::from(e),
        }
    }
}

impl<E> From<io::Error> for Ks3Error<E> {
    fn from(err: io::Error) -> Self {
        Ks3Error::HttpDispatch(HttpDispatchError::from(err))
    }
}

impl<E: Error + 'static> fmt::Display for Ks3Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ks3Error::Service(ref err) => write!(f, "{}", err),
            Ks3Error::Validation(ref cause) => write!(f, "{}", cause),
            Ks3Error::Credentials(ref err) => write!(f, "{}", err),
            Ks3Error::HttpDispatch(ref dispatch_error) => write!(f, "{}", dispatch_error),
            Ks3Error::ParseError(ref cause) => write!(f, "{}", cause),
            Ks3Error::Unknown(ref cause) => write!(
                f,
                "Request ID: {:?} Body: {}",
                cause.headers.get(AWS_REQUEST_ID_HEADER),
                cause.body_as_str()
            ),
            Ks3Error::Blocking => write!(f, "Failed to run blocking future"),
        }
    }
}

impl<E: Error + 'static> Error for Ks3Error<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Ks3Error::Service(ref err) => Some(err),
            Ks3Error::Credentials(ref err) => Some(err),
            Ks3Error::HttpDispatch(ref err) => Some(err),
            _ => None,
        }
    }
}
