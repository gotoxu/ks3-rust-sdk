//! AWS Regions and helper functions.
//!
//! Mostly used for translating the Region enum to a string AWS accepts.
//!
//! For example: `UsEast1` to "us-east-1"

use crate::credential::ProfileProvider;
use serde::ser::SerializeTuple;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt::{self, Display, Error as FmtError, Formatter};
use std::str::FromStr;

/// An AWS region.
///
/// # Default
///
/// `Region` implements the `Default` trait. Calling `Region::default()` will attempt to read the
/// `AWS_DEFAULT_REGION` or `AWS_REGION` environment variable. If it is malformed, it will fall back to `Region::UsEast1`.
/// If it is not present it will fallback on the value associated with the current profile in `~/.aws/config` or the file
/// specified by the `AWS_CONFIG_FILE` environment variable. If that is malformed of absent it will fall back on `Region::UsEast1`
///
/// # AWS-compatible services
///
/// `Region::Custom` can be used to connect to AWS-compatible services such as DynamoDB Local or Ceph.
///
/// ```
///     # use rusoto_signature::Region;
///     Region::Custom {
///         name: "eu-east-3".to_owned(),
///         endpoint: "http://localhost:8000".to_owned(),
///     };
/// ```
///
/// # Caveats
///
/// `CnNorth1` is currently untested due to Rusoto maintainers not having access to AWS China.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Region {
    /// Region that covers the Eastern part of Asia Pacific
    ApEast1,

    /// Region that covers the North-Eastern part of Asia Pacific
    ApNortheast1,

    /// Region that covers the North-Eastern part of Asia Pacific
    ApNortheast2,

    /// Region that covers the North-Eastern part of Asia Pacific
    ApNortheast3,

    /// Region that covers the Southern part of Asia Pacific
    ApSouth1,

    /// Region that covers the South-Eastern part of Asia Pacific
    ApSoutheast1,

    /// Region that covers the South-Eastern part of Asia Pacific
    ApSoutheast2,

    /// Region that covers Canada
    CaCentral1,

    /// Region that covers Central Europe
    EuCentral1,

    /// Region that covers Western Europe
    EuWest1,

    /// Region that covers Western Europe
    EuWest2,

    /// Region that covers Western Europe
    EuWest3,

    /// Region that covers Northern Europe
    EuNorth1,

    /// Region that covers Southern Europe
    EuSouth1,

    /// Bahrain, Middle East South
    MeSouth1,

    /// Region that covers South America
    SaEast1,

    /// Region that covers the Eastern part of the United States
    UsEast1,

    /// Region that covers the Eastern part of the United States
    UsEast2,

    /// Region that covers the Western part of the United States
    UsWest1,

    /// Region that covers the Western part of the United States
    UsWest2,

    /// Region that covers the Eastern part of the United States for the US Government
    UsGovEast1,

    /// Region that covers the Western part of the United States for the US Government
    UsGovWest1,

    /// Region that covers China
    CnNorth1,

    /// Region that covers North-Western part of China
    CnNorthwest1,

    /// Region that covers southern part Africa
    AfSouth1,

    /// Specifies a custom region, such as a local Ceph target
    Custom {
        /// Name of the endpoint (e.g. `"eu-east-2"`).
        name: String,

        /// Endpoint to be used. For instance, `"https://s3.my-provider.net"` or just
        /// `"s3.my-provider.net"` (default scheme is https).
        endpoint: String,
    },
}

impl Region {
    /// Name of the region
    ///
    /// ```
    ///     # use rusoto_signature::Region;
    ///     assert_eq!(Region::CaCentral1.name(), "ca-central-1");
    ///     assert_eq!(
    ///         Region::Custom { name: "eu-east-3".to_owned(), endpoint: "s3.net".to_owned() }.name(),
    ///         "eu-east-3"
    ///     );
    /// ```
    pub fn name(&self) -> &str {
        match *self {
            Region::ApEast1 => "ap-east-1",
            Region::ApNortheast1 => "ap-northeast-1",
            Region::ApNortheast2 => "ap-northeast-2",
            Region::ApNortheast3 => "ap-northeast-3",
            Region::ApSouth1 => "ap-south-1",
            Region::ApSoutheast1 => "ap-southeast-1",
            Region::ApSoutheast2 => "ap-southeast-2",
            Region::CaCentral1 => "ca-central-1",
            Region::EuCentral1 => "eu-central-1",
            Region::EuWest1 => "eu-west-1",
            Region::EuWest2 => "eu-west-2",
            Region::EuWest3 => "eu-west-3",
            Region::EuNorth1 => "eu-north-1",
            Region::EuSouth1 => "eu-south-1",
            Region::MeSouth1 => "me-south-1",
            Region::SaEast1 => "sa-east-1",
            Region::UsEast1 => "us-east-1",
            Region::UsEast2 => "us-east-2",
            Region::UsWest1 => "us-west-1",
            Region::UsWest2 => "us-west-2",
            Region::UsGovEast1 => "us-gov-east-1",
            Region::UsGovWest1 => "us-gov-west-1",
            Region::CnNorth1 => "cn-north-1",
            Region::CnNorthwest1 => "cn-northwest-1",
            Region::AfSouth1 => "af-south-1",
            Region::Custom { ref name, .. } => name,
        }
    }
}

/// An error produced when attempting to convert a `str` into a `Region` fails.
#[derive(Debug, PartialEq)]
pub struct ParseRegionError {
    message: String,
}

// Manually created for lack of a way to flatten the `Region::Custom` variant
// Related: https://github.com/serde-rs/serde/issues/119
impl Serialize for Region {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(2)?;
        if let Region::Custom {
            ref endpoint,
            ref name,
        } = *self
        {
            seq.serialize_element(&name)?;
            seq.serialize_element(&Some(&endpoint))?;
        } else {
            seq.serialize_element(self.name())?;
            seq.serialize_element(&None as &Option<&str>)?;
        }
        seq.end()
    }
}

struct RegionVisitor;

impl<'de> de::Visitor<'de> for RegionVisitor {
    type Value = Region;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("sequence of (name, Some(endpoint))")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let name: String = seq
            .next_element::<String>()?
            .ok_or_else(|| de::Error::custom("region is missing name"))?;
        let endpoint: Option<String> = seq.next_element::<Option<String>>()?.unwrap_or_default();
        match (name, endpoint) {
            (name, Some(endpoint)) => Ok(Region::Custom { name, endpoint }),
            (name, None) => name.parse().map_err(de::Error::custom),
        }
    }
}

// Manually created for lack of a way to flatten the `Region::Custom` variant
// Related: https://github.com/serde-rs/serde/issues/119
impl<'de> Deserialize<'de> for Region {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, RegionVisitor)
    }
}

impl FromStr for Region {
    type Err = ParseRegionError;

    fn from_str(s: &str) -> Result<Region, ParseRegionError> {
        let v: &str = &s.to_lowercase();
        match v {
            "ap-east-1" | "apeast1" => Ok(Region::ApEast1),
            "ap-northeast-1" | "apnortheast1" => Ok(Region::ApNortheast1),
            "ap-northeast-2" | "apnortheast2" => Ok(Region::ApNortheast2),
            "ap-northeast-3" | "apnortheast3" => Ok(Region::ApNortheast3),
            "ap-south-1" | "apsouth1" => Ok(Region::ApSouth1),
            "ap-southeast-1" | "apsoutheast1" => Ok(Region::ApSoutheast1),
            "ap-southeast-2" | "apsoutheast2" => Ok(Region::ApSoutheast2),
            "ca-central-1" | "cacentral1" => Ok(Region::CaCentral1),
            "eu-central-1" | "eucentral1" => Ok(Region::EuCentral1),
            "eu-west-1" | "euwest1" => Ok(Region::EuWest1),
            "eu-west-2" | "euwest2" => Ok(Region::EuWest2),
            "eu-west-3" | "euwest3" => Ok(Region::EuWest3),
            "eu-north-1" | "eunorth1" => Ok(Region::EuNorth1),
            "eu-south-1" | "eusouth1" => Ok(Region::EuSouth1),
            "me-south-1" | "mesouth1" => Ok(Region::MeSouth1),
            "sa-east-1" | "saeast1" => Ok(Region::SaEast1),
            "us-east-1" | "useast1" => Ok(Region::UsEast1),
            "us-east-2" | "useast2" => Ok(Region::UsEast2),
            "us-west-1" | "uswest1" => Ok(Region::UsWest1),
            "us-west-2" | "uswest2" => Ok(Region::UsWest2),
            "us-gov-east-1" | "usgoveast1" => Ok(Region::UsGovEast1),
            "us-gov-west-1" | "usgovwest1" => Ok(Region::UsGovWest1),
            "cn-north-1" | "cnnorth1" => Ok(Region::CnNorth1),
            "cn-northwest-1" | "cnnorthwest1" => Ok(Region::CnNorthwest1),
            "af-south-1" | "afsouth1" => Ok(Region::AfSouth1),
            s => Err(ParseRegionError::new(s)),
        }
    }
}

impl ParseRegionError {
    /// Parses a region given as a string literal into a type `Region'
    pub fn new(input: &str) -> Self {
        ParseRegionError {
            message: format!("Not a valid AWS region: {}", input),
        }
    }
}

impl Error for ParseRegionError {}

impl Display for ParseRegionError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{}", self.message)
    }
}

impl Default for Region {
    fn default() -> Region {
        match std::env::var("AWS_DEFAULT_REGION").or_else(|_| std::env::var("AWS_REGION")) {
            Ok(ref v) => Region::from_str(v).unwrap_or(Region::UsEast1),
            Err(_) => match ProfileProvider::region() {
                Ok(Some(region)) => Region::from_str(&region).unwrap_or(Region::UsEast1),
                _ => Region::UsEast1,
            },
        }
    }
}
