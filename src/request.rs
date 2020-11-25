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

pub type StreamingBody = crate::signature::ByteStream;

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

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize_structs", derive(Serialize))]
pub struct PutObjectOutput {
    /// <p>Entity tag for the uploaded object.</p>
    pub e_tag: Option<String>,
    /// <p> If the expiration is configured for the object (see <a>PutBucketLifecycleConfiguration</a>), the response includes this header. It includes the expiry-date and rule-id key-value pairs that provide information about object expiration. The value of the rule-id is URL encoded.</p>
    pub expiration: Option<String>,
    pub request_charged: Option<String>,
    /// <p>If server-side encryption with a customer-provided encryption key was requested, the response will include this header confirming the encryption algorithm used.</p>
    pub sse_customer_algorithm: Option<String>,
    /// <p>If server-side encryption with a customer-provided encryption key was requested, the response will include this header to provide round-trip message integrity verification of the customer-provided encryption key.</p>
    pub sse_customer_key_md5: Option<String>,
    /// <p>If present, specifies the AWS KMS Encryption Context to use for object encryption. The value of this header is a base64-encoded UTF-8 string holding JSON with the encryption context key-value pairs.</p>
    pub ssekms_encryption_context: Option<String>,
    /// <p>If <code>x-amz-server-side-encryption</code> is present and has the value of <code>aws:kms</code>, this header specifies the ID of the AWS Key Management Service (AWS KMS) symmetric customer managed customer master key (CMK) that was used for the object. </p>
    pub ssekms_key_id: Option<String>,
    /// <p>If you specified server-side encryption either with an AWS KMS customer master key (CMK) or Amazon S3-managed encryption key in your PUT request, the response includes this header. It confirms the encryption algorithm that Amazon S3 used to encrypt the object.</p>
    pub server_side_encryption: Option<String>,
    /// <p>Version of the object.</p>
    pub version_id: Option<String>,
}

#[allow(dead_code)]
struct PutObjectOutputDeserializer;
impl PutObjectOutputDeserializer {
    #[allow(dead_code, unused_variables)]
    fn deserialize<T: Peek + Next>(
        tag_name: &str,
        stack: &mut T,
    ) -> Result<PutObjectOutput, XmlParseError> {
        xml_util::start_element(tag_name, stack)?;

        let obj = PutObjectOutput::default();

        xml_util::end_element(tag_name, stack)?;

        Ok(obj)
    }
}

#[derive(Debug, Default)]
pub struct PutObjectRequest {
    /// <p>The canned ACL to apply to the object. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html#CannedACL">Canned ACL</a>.</p>
    pub acl: Option<String>,
    /// <p>Object data.</p>
    pub body: Option<StreamingBody>,
    /// <p>Bucket name to which the PUT operation was initiated. </p> <p>When using this API with an access point, you must direct requests to the access point hostname. The access point hostname takes the form <i>AccessPointName</i>-<i>AccountId</i>.s3-accesspoint.<i>Region</i>.amazonaws.com. When using this operation using an access point through the AWS SDKs, you provide the access point ARN in place of the bucket name. For more information about access point ARNs, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/using-access-points.html">Using Access Points</a> in the <i>Amazon Simple Storage Service Developer Guide</i>.</p>
    pub bucket: String,
    /// <p> Can be used to specify caching behavior along the request/reply chain. For more information, see <a href="http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.9">http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.9</a>.</p>
    pub cache_control: Option<String>,
    /// <p>Specifies presentational information for the object. For more information, see <a href="http://www.w3.org/Protocols/rfc2616/rfc2616-sec19.html#sec19.5.1">http://www.w3.org/Protocols/rfc2616/rfc2616-sec19.html#sec19.5.1</a>.</p>
    pub content_disposition: Option<String>,
    /// <p>Specifies what content encodings have been applied to the object and thus what decoding mechanisms must be applied to obtain the media-type referenced by the Content-Type header field. For more information, see <a href="http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.11">http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.11</a>.</p>
    pub content_encoding: Option<String>,
    /// <p>The language the content is in.</p>
    pub content_language: Option<String>,
    /// <p>Size of the body in bytes. This parameter is useful when the size of the body cannot be determined automatically. For more information, see <a href="http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.13">http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.13</a>.</p>
    pub content_length: Option<i64>,
    /// <p>The base64-encoded 128-bit MD5 digest of the message (without the headers) according to RFC 1864. This header can be used as a message integrity check to verify that the data is the same data that was originally sent. Although it is optional, we recommend using the Content-MD5 mechanism as an end-to-end integrity check. For more information about REST request authentication, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/RESTAuthentication.html">REST Authentication</a>.</p>
    pub content_md5: Option<String>,
    /// <p>A standard MIME type describing the format of the contents. For more information, see <a href="http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.17">http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.17</a>.</p>
    pub content_type: Option<String>,
    /// <p>The date and time at which the object is no longer cacheable. For more information, see <a href="http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.21">http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.21</a>.</p>
    pub expires: Option<String>,
    /// <p>Gives the grantee READ, READ_ACP, and WRITE_ACP permissions on the object.</p>
    pub grant_full_control: Option<String>,
    /// <p>Allows grantee to read the object data and its metadata.</p>
    pub grant_read: Option<String>,
    /// <p>Allows grantee to read the object ACL.</p>
    pub grant_read_acp: Option<String>,
    /// <p>Allows grantee to write the ACL for the applicable object.</p>
    pub grant_write_acp: Option<String>,
    /// <p>Object key for which the PUT operation was initiated.</p>
    pub key: String,
    /// <p>A map of metadata to store with the object in S3.</p>
    pub metadata: Option<::std::collections::HashMap<String, String>>,
    /// <p>Specifies whether a legal hold will be applied to this object. For more information about S3 Object Lock, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/object-lock.html">Object Lock</a>.</p>
    pub object_lock_legal_hold_status: Option<String>,
    /// <p>The Object Lock mode that you want to apply to this object.</p>
    pub object_lock_mode: Option<String>,
    /// <p>The date and time when you want this object's Object Lock to expire.</p>
    pub object_lock_retain_until_date: Option<String>,
    pub request_payer: Option<String>,
    /// <p>Specifies the algorithm to use to when encrypting the object (for example, AES256).</p>
    pub sse_customer_algorithm: Option<String>,
    /// <p>Specifies the customer-provided encryption key for Amazon S3 to use in encrypting data. This value is used to store the object and then it is discarded; Amazon S3 does not store the encryption key. The key must be appropriate for use with the algorithm specified in the <code>x-amz-server-side​-encryption​-customer-algorithm</code> header.</p>
    pub sse_customer_key: Option<String>,
    /// <p>Specifies the 128-bit MD5 digest of the encryption key according to RFC 1321. Amazon S3 uses this header for a message integrity check to ensure that the encryption key was transmitted without error.</p>
    pub sse_customer_key_md5: Option<String>,
    /// <p>Specifies the AWS KMS Encryption Context to use for object encryption. The value of this header is a base64-encoded UTF-8 string holding JSON with the encryption context key-value pairs.</p>
    pub ssekms_encryption_context: Option<String>,
    /// <p>If <code>x-amz-server-side-encryption</code> is present and has the value of <code>aws:kms</code>, this header specifies the ID of the AWS Key Management Service (AWS KMS) symmetrical customer managed customer master key (CMK) that was used for the object.</p> <p> If the value of <code>x-amz-server-side-encryption</code> is <code>aws:kms</code>, this header specifies the ID of the symmetric customer managed AWS KMS CMK that will be used for the object. If you specify <code>x-amz-server-side-encryption:aws:kms</code>, but do not provide<code> x-amz-server-side-encryption-aws-kms-key-id</code>, Amazon S3 uses the AWS managed CMK in AWS to protect the data.</p>
    pub ssekms_key_id: Option<String>,
    /// <p>The server-side encryption algorithm used when storing this object in Amazon S3 (for example, AES256, aws:kms).</p>
    pub server_side_encryption: Option<String>,
    /// <p>If you don't specify, S3 Standard is the default storage class. Amazon S3 supports other storage classes.</p>
    pub storage_class: Option<String>,
    /// <p>The tag-set for the object. The tag-set must be encoded as URL Query parameters. (For example, "Key1=Value1")</p>
    pub tagging: Option<String>,
    /// <p>If the bucket is configured as a website, redirects requests for this object to another object in the same bucket or to an external URL. Amazon S3 stores the value of this header in the object metadata. For information about object metadata, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingMetadata.html">Object Key and Metadata</a>.</p> <p>In the following example, the request header sets the redirect to an object (anotherPage.html) in the same bucket:</p> <p> <code>x-amz-website-redirect-location: /anotherPage.html</code> </p> <p>In the following example, the request header sets the object redirect to another website:</p> <p> <code>x-amz-website-redirect-location: http://www.example.com/</code> </p> <p>For more information about website hosting in Amazon S3, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/WebsiteHosting.html">Hosting Websites on Amazon S3</a> and <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/how-to-page-redirect.html">How to Configure Website Page Redirects</a>. </p>
    pub website_redirect_location: Option<String>,
}

/// Errors returned by PutObject
#[derive(Debug, PartialEq)]
pub enum PutObjectError {}

impl PutObjectError {
    pub fn from_response(res: BufferedHttpResponse) -> Ks3Error<PutObjectError> {
        {
            let reader = EventReader::new(res.body.as_ref());
            let mut stack = XmlResponse::new(reader.into_iter().peekable());
            find_start_element(&mut stack);
            if let Ok(parsed_error) = Self::deserialize(&mut stack) {
                match &parsed_error.code[..] {
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

impl fmt::Display for PutObjectError {
    #[allow(unused_variables)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl Error for PutObjectError {}
