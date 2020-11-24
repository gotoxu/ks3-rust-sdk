use std::default::Default;
use std::error::Error;
use std::fmt;
use std::io::Write;

#[cfg(feature = "deserialize_structs")]
use serde::Deserialize;
#[cfg(feature = "serialize_structs")]
use serde::Serialize;
use xml::EventReader;
use xml::EventWriter;

use crate::core::error::Ks3Error;
use crate::core::proto::xml::error::{XmlError, XmlErrorDeserializer};
use crate::core::proto::xml::util::{self as xml_util, Next, Peek, XmlParseError, XmlResponse};
use crate::core::proto::xml::util::{find_start_element, write_characters_element};
use crate::core::request::BufferedHttpResponse;

/// <p>The configuration information for the bucket.</p>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "deserialize_structs", derive(Deserialize))]
pub struct CreateBucketConfiguration {
    /// <p>Specifies the Region where the bucket will be created. If you don't specify a Region, the bucket is created in the US East (N. Virginia) Region (us-east-1).</p>
    pub location_constraint: Option<String>,
}

pub struct CreateBucketConfigurationSerializer;
impl CreateBucketConfigurationSerializer {
    #[allow(unused_variables, warnings)]
    pub fn serialize<W>(
        mut writer: &mut EventWriter<W>,
        name: &str,
        obj: &CreateBucketConfiguration,
    ) -> Result<(), xml::writer::Error>
    where
        W: Write,
    {
        writer.write(xml::writer::XmlEvent::start_element(name))?;
        if let Some(ref value) = obj.location_constraint {
            write_characters_element(writer, "LocationConstraint", &value.to_string())?;
        }
        writer.write(xml::writer::XmlEvent::end_element())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize_structs", derive(Serialize))]
pub struct CreateBucketOutput {
    /// <p>Specifies the Region where the bucket will be created. If you are creating a bucket on the US East (N. Virginia) Region (us-east-1), you do not need to specify the location.</p>
    pub location: Option<String>,
}

#[allow(dead_code)]
struct CreateBucketOutputDeserializer;
impl CreateBucketOutputDeserializer {
    #[allow(dead_code, unused_variables)]
    fn deserialize<T: Peek + Next>(
        tag_name: &str,
        stack: &mut T,
    ) -> Result<CreateBucketOutput, XmlParseError> {
        xml_util::start_element(tag_name, stack)?;

        let obj = CreateBucketOutput::default();

        xml_util::end_element(tag_name, stack)?;

        Ok(obj)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "deserialize_structs", derive(Deserialize))]
pub struct CreateBucketRequest {
    /// <p>The canned ACL to apply to the bucket.</p>
    pub acl: Option<String>,
    /// <p>The name of the bucket to create.</p>
    pub bucket: String,
    /// <p>The configuration information for the bucket.</p>
    pub create_bucket_configuration: Option<CreateBucketConfiguration>,
    /// <p>Allows grantee the read, write, read ACP, and write ACP permissions on the bucket.</p>
    pub grant_full_control: Option<String>,
    /// <p>Allows grantee to list the objects in the bucket.</p>
    pub grant_read: Option<String>,
    /// <p>Allows grantee to read the bucket ACL.</p>
    pub grant_read_acp: Option<String>,
    /// <p>Allows grantee to create, overwrite, and delete any object in the bucket.</p>
    pub grant_write: Option<String>,
    /// <p>Allows grantee to write the ACL for the applicable bucket.</p>
    pub grant_write_acp: Option<String>,
    /// <p>Specifies whether you want S3 Object Lock to be enabled for the new bucket.</p>
    pub object_lock_enabled_for_bucket: Option<bool>,
}

/// Errors returned by CreateBucket
#[derive(Debug, PartialEq)]
pub enum CreateBucketError {
    /// <p>The requested bucket name is not available. The bucket namespace is shared by all users of the system. Please select a different name and try again.</p>
    BucketAlreadyExists(String),
    /// <p>The bucket you tried to create already exists, and you own it. Amazon S3 returns this error in all AWS Regions except in the North Virginia Region. For legacy compatibility, if you re-create an existing bucket that you already own in the North Virginia Region, Amazon S3 returns 200 OK and resets the bucket access control lists (ACLs).</p>
    BucketAlreadyOwnedByYou(String),
}

impl CreateBucketError {
    pub fn from_response(res: BufferedHttpResponse) -> Ks3Error<CreateBucketError> {
        {
            let reader = EventReader::new(res.body.as_ref());
            let mut stack = XmlResponse::new(reader.into_iter().peekable());
            find_start_element(&mut stack);
            if let Ok(parsed_error) = Self::deserialize(&mut stack) {
                match &parsed_error.code[..] {
                    "BucketAlreadyExists" => {
                        return Ks3Error::Service(CreateBucketError::BucketAlreadyExists(
                            parsed_error.message,
                        ))
                    }
                    "BucketAlreadyOwnedByYou" => {
                        return Ks3Error::Service(CreateBucketError::BucketAlreadyOwnedByYou(
                            parsed_error.message,
                        ))
                    }
                    _ => {}
                }
            }
        }
        Ks3Error::Unknown(res)
    }

    fn deserialize<T>(stack: &mut T) -> Result<XmlError, XmlParseError>
    where
        T: Peek + Next,
    {
        XmlErrorDeserializer::deserialize("Error", stack)
    }
}

impl fmt::Display for CreateBucketError {
    #[allow(unused_variables)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CreateBucketError::BucketAlreadyExists(ref cause) => write!(f, "{}", cause),
            CreateBucketError::BucketAlreadyOwnedByYou(ref cause) => write!(f, "{}", cause),
        }
    }
}

impl Error for CreateBucketError {}
