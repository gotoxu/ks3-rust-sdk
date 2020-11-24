//! AWS API request signatures.
//!
//! Follows [AWS Signature 2](https://docs.aws.amazon.com/general/latest/gr/signature-version-2.html)
//! algorithm.
//!
//! If needed, the request will be re-issued to a temporary redirect endpoint.  This can happen with
//! newly created S3 buckets not in us-standard/us-east-1.
//!
//! Please note that this module does not expect URIs to already be encoded.
//!

use bytes::Bytes;
use hmac::{Hmac, Mac, NewMac};
use hyper::Body;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use sha1::Sha1;
use time::OffsetDateTime;

use crate::credential::AwsCredentials;
use crate::signature::ks_time::rfc1123;
use crate::signature::ByteStream;
use crate::signature::Region;
use std::collections::BTreeMap;
use std::fmt;
use std::str;

pub type Params = BTreeMap<String, Option<String>>;

/// Possible payloads included in a `SignedRequest`.
pub enum SignedRequestPayload {
    /// Transfer payload in a single chunk
    Buffer(Bytes),
    /// Transfer payload in multiple chunks
    Stream(ByteStream),
}

impl SignedRequestPayload {
    /// Convert `SignedRequestPayload` into a hyper `Body`
    pub fn into_body(self) -> Body {
        match self {
            SignedRequestPayload::Buffer(bytes) => Body::from(bytes),
            SignedRequestPayload::Stream(stream) => Body::wrap_stream(stream),
        }
    }
}

impl fmt::Debug for SignedRequestPayload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SignedRequestPayload::Buffer(ref buf) => {
                write!(f, "SignedRequestPayload::Buffer(len = {})", buf.len())
            }
            SignedRequestPayload::Stream(ref stream) => write!(
                f,
                "SignedRequestPayload::Stream(size_hint = {:?})",
                stream.size_hint()
            ),
        }
    }
}

/// A data structure for all the elements of an HTTP request that are involved in
/// the Amazon Signature Version 2 signing process
#[derive(Debug)]
pub struct SignedRequest {
    /// The HTTP Method
    pub method: String,
    /// The AWS Service
    pub service: String,
    /// The AWS Region
    pub region: Region,
    /// The HTTP request path
    pub path: String,
    /// The HTTP Request Headers
    pub headers: BTreeMap<String, Vec<Vec<u8>>>,
    /// The HTTP request paramaters
    pub params: Params,
    /// The HTTP/HTTPS protocol
    pub scheme: Option<String>,
    /// The AWS hostname
    pub hostname: Option<String>,
    /// The HTTP Content
    pub payload: Option<SignedRequestPayload>,
    /// The Standardised query string
    pub canonical_query_string: String,
    /// The Standardised URI
    pub canonical_uri: String,
}

impl SignedRequest {
    /// Default constructor
    pub fn new(method: &str, service: &str, region: &Region, path: &str) -> SignedRequest {
        SignedRequest {
            method: method.to_string(),
            service: service.to_string(),
            region: region.clone(),
            path: path.to_string(),
            headers: BTreeMap::new(),
            params: Params::new(),
            scheme: None,
            hostname: None,
            payload: None,
            canonical_query_string: String::new(),
            canonical_uri: String::new(),
        }
    }

    /// Sets the value of the "content-type" header.
    pub fn set_content_type(&mut self, content_type: String) {
        self.add_header("content-type", &content_type);
    }

    /// Sets the target hostname
    pub fn set_hostname(&mut self, hostname: Option<String>) {
        self.hostname = hostname;
    }

    /// Sets the target hostname using the current service type and region
    ///
    /// See the implementation of build_hostname to see how this is done
    pub fn set_endpoint_prefix(&mut self, endpoint_prefix: String) {
        self.hostname = Some(build_hostname(&endpoint_prefix, &self.region));
    }

    /// Sets the new body (payload)
    pub fn set_payload<B: Into<Bytes>>(&mut self, payload: Option<B>) {
        self.payload = payload.map(|chunk| SignedRequestPayload::Buffer(chunk.into()));
    }

    /// Sets the new body (payload) as a stream
    pub fn set_payload_stream(&mut self, stream: ByteStream) {
        self.payload = Some(SignedRequestPayload::Stream(stream));
    }

    /// Computes and sets the Content-MD5 header based on the current payload.
    ///
    /// Has no effect if the payload is not set, or is not a buffer.
    pub fn set_content_md5_header(&mut self) {
        let digest;
        if let Some(SignedRequestPayload::Buffer(ref payload)) = self.payload {
            digest = Some(md5::compute(payload));
        } else {
            digest = None;
        }
        if let Some(digest) = digest {
            // need to deref digest and then pass that reference:
            self.add_header("Content-MD5", &base64::encode(&(*digest)));
        }
    }

    /// Returns the current HTTP method
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the current path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Invokes `canonical_uri(path)` to return a canonical path
    pub fn canonical_path(&self) -> String {
        canonical_uri(&self.path, &self.region)
    }

    /// Returns the current canonical URI
    pub fn canonical_uri(&self) -> &str {
        &self.canonical_uri
    }

    /// Returns the current query string
    ///
    /// Converts a paramater such as "example param": "examplekey" into "&example+param=examplekey"
    pub fn canonical_query_string(&self) -> &str {
        &self.canonical_query_string
    }

    /// Returns the current headers
    pub fn headers(&self) -> &BTreeMap<String, Vec<Vec<u8>>> {
        &self.headers
    }

    /// Returns the current http scheme (https or http)
    pub fn scheme(&self) -> String {
        match self.scheme {
            Some(ref p) => p.to_string(),
            None => match self.region {
                Region::Custom { ref endpoint, .. } => {
                    if endpoint.starts_with("http://") {
                        "http".to_owned()
                    } else {
                        "https".to_owned()
                    }
                }
                _ => "https".to_owned(),
            },
        }
    }

    /// Modify the region used for signing if needed, such as for AWS Organizations
    pub fn region_for_service(&self) -> String {
        match self.service.as_str() {
            "organizations" => {
                // Matches https://docs.aws.amazon.com/general/latest/gr/ao.html
                match self.region {
                    Region::CnNorth1 | Region::CnNorthwest1 => {
                        Region::CnNorthwest1.name().to_string()
                    }
                    Region::UsGovEast1 | Region::UsGovWest1 => {
                        Region::UsGovWest1.name().to_string()
                    }
                    _ => Region::UsEast1.name().to_string(),
                }
            }
            _ => self.region.name().to_string(),
        }
    }

    /// Converts hostname to String if it exists, else it invokes build_hostname()
    pub fn hostname(&self) -> String {
        // hostname may be already set by an endpoint prefix
        match self.hostname {
            Some(ref h) => h.to_string(),
            None => build_hostname(&self.service, &self.region),
        }
    }

    /// If the key exists in headers, set it to blank/unoccupied:
    pub fn remove_header(&mut self, key: &str) {
        let key_lower = key.to_ascii_lowercase();
        self.headers.remove(&key_lower);
    }

    /// Add a value to the array of headers for the specified key.
    /// Headers are kept sorted by key name for use at signing (BTreeMap)
    pub fn add_header<K: ToString>(&mut self, key: K, value: &str) {
        let mut key_lower = key.to_string();
        key_lower.make_ascii_lowercase();

        let value_vec = value.as_bytes().to_vec();

        self.headers.entry(key_lower).or_default().push(value_vec);
    }

    pub fn add_optional_header<K: ToString, V: ToString>(&mut self, key: K, value: Option<V>) {
        if let Some(ref value) = value {
            self.add_header(key, &value.to_string());
        }
    }

    /// Adds parameter to the HTTP Request
    pub fn add_param<S>(&mut self, key: S, value: S)
    where
        S: Into<String>,
    {
        self.params.insert(key.into(), Some(value.into()));
    }

    /// Sets paramaters with a given variable of `Params` type
    pub fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    /// Complement SignedRequest by ensuring the following HTTP headers are set accordingly:
    /// - host
    /// - content-type
    /// - content-length (if applicable)
    pub fn complement(&mut self) {
        // build the canonical request
        self.canonical_uri = self.canonical_path();
        self.canonical_query_string = build_canonical_query_string(&self.params);
        // Gotta remove and re-add headers since by default they append the value.  If we're following
        // a 307 redirect we end up with Three Stooges in the headers with duplicate values.
        self.remove_header("Host");
        self.add_header("Host", &self.hostname());
        // if there's no content-type header set, set it to the default value
        // if let Entry::Vacant(entry) = self.headers.entry("Content-Type".to_owned()) {
        //     let mut values = Vec::new();
        //     values.push(b"application/octet-stream".to_vec());
        //     entry.insert(values);
        // }
        let len = match self.payload {
            None => Some(0),
            Some(SignedRequestPayload::Buffer(ref payload)) => Some(payload.len()),
            Some(SignedRequestPayload::Stream(ref stream)) => stream.size_hint(),
        };
        if let Some(len) = len {
            self.remove_header("Content-Length");
            self.add_header("Content-Length", &format!("{}", len));
        }
    }

    /// Signs the request using Amazon Signature version 2 to verify identity.
    pub fn sign(&mut self, creds: &AwsCredentials) {
        self.complement();
        if self.is_request_signed() && !creds.credentials_are_expired() {
            // If the request is already signed, and the credentials have not
            // expired yet ignore the signing request.
            return;
        }

        // build time
        let date = OffsetDateTime::now_utc();
        self.remove_header("Date");
        let formatted_time = rfc1123(&date);
        self.add_header("Date", &formatted_time);

        // build canonical headers
        let canonical_headers = canonical_headers(&self.headers);

        // build canonical resource
        let mut uri = self.canonical_uri().to_owned();
        if !uri.is_empty() {
            uri = uri.strip_prefix("/").unwrap().to_owned();
            let uris: Vec<&str> = uri.split('/').collect();

            let mut append = false;
            if uris.len() == 1 && !uris[0].is_empty() {
                append = true;
            }
            uri = format!("/{}", uris.join("/"));
            if append {
                uri = format!("{}/", uri);
            }
        }
        if uri.is_empty() {
            uri = String::from("/");
        }

        let canonical_resource = if self.canonical_query_string.is_empty() {
            uri
        } else {
            format!("{}?{}", &uri, &self.canonical_query_string)
        };

        let md5_list = self.headers.get("Content-Md5");
        let mut md5_str = String::new();
        if md5_list.is_some() {
            let list = match md5_list {
                Some(a) => a,
                _ => unreachable!(),
            };
            if !list.is_empty() {
                md5_str = String::from_utf8(list[0].clone()).unwrap();
            }
        }

        let type_list = self.headers.get("Content-Type");
        let mut type_str = String::new();
        if type_list.is_some() {
            let list = match type_list {
                Some(a) => a,
                _ => unreachable!(),
            };
            if !list.is_empty() {
                type_str = String::from_utf8(list[0].clone()).unwrap();
            }
        }

        let mut canonical_request = format!(
            "{}\n{}\n{}\n{}",
            &self.method, md5_str, type_str, formatted_time
        );
        if !canonical_headers.is_empty() {
            canonical_request.push('\n');
            canonical_request.push_str(&canonical_headers);
        }
        canonical_request.push('\n');
        canonical_request.push_str(&canonical_resource);
        println!("{}", canonical_request);

        let signature = sign_string(&canonical_request, creds.aws_secret_access_key());
        let auth_header = format!("AWS {}:{}", &creds.aws_access_key_id(), signature);
        self.remove_header("Authorization");
        self.add_header("Authorization", &auth_header);
        println!("{}", auth_header);
    }

    /// is_request_signed returns if the request is currently signed or presigned
    fn is_request_signed(&self) -> bool {
        if self.params.get("signature").is_some() {
            return true;
        }
        if self.headers.get("authorization").is_some() {
            return true;
        }
        false
    }
}

/// Takes a message and signs it using AWS secret, time, region keys and service keys.
fn sign_string(string_to_sign: &str, secret: &str) -> String {
    let signing_hmac = hmac(secret.as_ref(), string_to_sign.as_ref())
        .finalize()
        .into_bytes();

    base64::encode_config(signing_hmac, base64::STANDARD)
}

#[inline]
fn hmac(secret: &[u8], message: &[u8]) -> Hmac<Sha1> {
    let mut hmac = Hmac::<Sha1>::new_varkey(secret).expect("failed to create hmac");
    hmac.update(message);
    hmac
}

/// Returns standardised URI
fn canonical_uri(path: &str, region: &Region) -> String {
    let endpoint_path = match region {
        Region::Custom { ref endpoint, .. } => extract_endpoint_path(endpoint),
        _ => None,
    };

    match (endpoint_path, path) {
        (Some(prefix), "") => prefix.to_string(),
        (None, "") => "/".to_string(),
        (Some(prefix), _) => encode_uri_path(&(prefix.to_owned() + path)),
        _ => encode_uri_path(path),
    }
}

/// Canonicalizes query while iterating through the given paramaters
fn build_canonical_query_string(params: &Params) -> String {
    if params.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    for (key, val) in params.iter() {
        if !output.is_empty() {
            output.push('&');
        }
        output.push_str(&encode_uri_strict(&key));
        output.push('=');

        if let Some(ref unwrapped_val) = *val {
            output.push_str(&encode_uri_strict(&unwrapped_val));
        }
    }

    output
}

// Do not URI-encode any of the unreserved characters that RFC 3986 defines:
// A-Z, a-z, 0-9, hyphen ( - ), underscore ( _ ), period ( . ), and tilde ( ~ ).
//
// Percent-encode all other characters with %XY, where X and Y are hexadecimal
// characters (0-9 and uppercase A-F). For example, the space character must be
// encoded as %20 (not using '+', as some encoding schemes do) and extended UTF-8
// characters must be in the form %XY%ZA%BC
/// This constant is used to maintain the strict URI encoding standard as proposed by RFC 3986
pub const STRICT_ENCODE_SET: AsciiSet = NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// This struct is used to maintain the URI path encoding
pub const STRICT_PATH_ENCODE_SET: AsciiSet = STRICT_ENCODE_SET.remove(b'/');

#[inline]
#[doc(hidden)]
pub fn encode_uri_path(uri: &str) -> String {
    utf8_percent_encode(uri, &STRICT_PATH_ENCODE_SET).collect::<String>()
}

#[inline]
fn encode_uri_strict(uri: &str) -> String {
    utf8_percent_encode(uri, &STRICT_ENCODE_SET).collect::<String>()
}

fn extract_endpoint_path(endpoint: &str) -> Option<&str> {
    extract_endpoint_components(endpoint).1
}

/// Canonicalizes headers into the AWS Canonical Form.
fn canonical_headers(headers: &BTreeMap<String, Vec<Vec<u8>>>) -> String {
    let mut canonical = String::new();

    for (key, value) in headers.iter() {
        if !key.starts_with("x-amz-") {
            continue;
        }

        canonical.push_str(format!("{}:{}\n", key, canonical_values(value)).as_ref());
    }
    if !canonical.is_empty() {
        canonical.remove(canonical.len() - 1);
    }
    canonical
}

/// Canonicalizes values into the AWS Canonical Form.
fn canonical_values(values: &[Vec<u8>]) -> String {
    let mut st = String::new();
    for v in values {
        let s = str::from_utf8(v).unwrap();
        if !st.is_empty() {
            st.push(',');
        }
        st.push_str(s);
    }
    st
}

fn extract_endpoint_components(endpoint: &str) -> (&str, Option<&str>) {
    let unschemed = endpoint
        .find("://")
        .map(|p| &endpoint[p + 3..])
        .unwrap_or(endpoint);
    unschemed
        .find('/')
        .map(|p| (&unschemed[..p], Some(&unschemed[p..])))
        .unwrap_or((unschemed, None))
}

fn extract_hostname(endpoint: &str) -> &str {
    extract_endpoint_components(endpoint).0
}

/// Takes a `Region` enum and a service and formas a vaild DNS name.
/// E.g. `Region::ApNortheast1` and `s3` produces `s3.ap-northeast-1.amazonaws.com.cn`
fn build_hostname(service: &str, region: &Region) -> String {
    // Any of these that modify the region will need to have their signature adjusted as well: sign for destination region
    //iam & cloudfront have only 1 endpoint, other services have region-based endpoints
    match service {
        "organizations" => match *region {
            // organizations is routed specially: see https://docs.aws.amazon.com/organizations/latest/APIReference/Welcome.html and https://docs.aws.amazon.com/general/latest/gr/ao.html
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            Region::CnNorth1 | Region::CnNorthwest1 => {
                "organizations.cn-northwest-1.amazonaws.com.cn".to_owned()
            }
            Region::UsGovEast1 | Region::UsGovWest1 => {
                "organizations.us-gov-west-1.amazonaws.com".to_owned()
            }
            _ => "organizations.us-east-1.amazonaws.com".to_owned(),
        },
        "iam" => match *region {
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            Region::CnNorth1 | Region::CnNorthwest1 => {
                format!("{}.{}.amazonaws.com.cn", service, region.name())
            }
            _ => format!("{}.amazonaws.com", service),
        },
        "chime" => match *region {
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            _ => format!("service.{}.aws.amazon.com", service),
        },
        "cloudfront" => match *region {
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            _ => format!("{}.amazonaws.com", service),
        },
        "importexport" => match *region {
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            _ => "importexport.amazonaws.com".to_owned(),
        },
        "s3" => match *region {
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            Region::CnNorth1 | Region::CnNorthwest1 => {
                format!("s3.{}.amazonaws.com.cn", region.name())
            }
            _ => format!("s3.{}.amazonaws.com", region.name()),
        },
        "route53" => match *region {
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            _ => "route53.amazonaws.com".to_owned(),
        },
        "sdb" => match *region {
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            Region::UsEast1 => "sdb.amazonaws.com".to_string(),
            _ => format!("sdb.{}.amazonaws.com", region.name()),
        },
        _ => match *region {
            Region::Custom { ref endpoint, .. } => extract_hostname(endpoint).to_owned(),
            Region::CnNorth1 | Region::CnNorthwest1 => {
                format!("{}.{}.amazonaws.com.cn", service, region.name())
            }
            _ => format!("{}.{}.amazonaws.com", service, region.name()),
        },
    }
}
