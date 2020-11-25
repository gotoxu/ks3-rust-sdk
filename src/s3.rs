use crate::core::error::Ks3Error;
use crate::core::Client;
use crate::core::{BufferedHttpResponse, DispatchSignedRequest, HttpResponse};
use crate::credential::ProvideAwsCredentials;
use crate::request::*;
use crate::signature::{Region, SignedRequest};

use async_trait::async_trait;
use xml::EventWriter;

/// Trait representing the capabilities of the Amazon S3 API. Amazon S3 clients implement this trait.
#[async_trait]
pub trait S3 {
    /// <p><p>Creates a new bucket. To create a bucket, you must register with Amazon S3 and have a valid AWS Access Key ID to authenticate requests. Anonymous requests are never allowed to create buckets. By creating the bucket, you become the bucket owner.</p> <p>Not every string is an acceptable bucket name. For information on bucket naming restrictions, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingBucket.html">Working with Amazon S3 Buckets</a>.</p> <p>By default, the bucket is created in the US East (N. Virginia) Region. You can optionally specify a Region in the request body. You might choose a Region to optimize latency, minimize costs, or address regulatory requirements. For example, if you reside in Europe, you will probably find it advantageous to create buckets in the Europe (Ireland) Region. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingBucket.html#access-bucket-intro">How to Select a Region for Your Buckets</a>.</p> <note> <p>If you send your create bucket request to the <code>s3.amazonaws.com</code> endpoint, the request goes to the us-east-1 Region. Accordingly, the signature calculations in Signature Version 4 must use us-east-1 as the Region, even if the location constraint in the request specifies another Region where the bucket is to be created. If you create a bucket in a Region other than US East (N. Virginia), your application must be able to handle 307 redirect. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/VirtualHosting.html">Virtual Hosting of Buckets</a>.</p> </note> <p>When creating a bucket using this operation, you can optionally specify the accounts or groups that should be granted specific permissions on the bucket. There are two ways to grant the appropriate permissions using the request headers.</p> <ul> <li> <p>Specify a canned ACL using the <code>x-amz-acl</code> request header. Amazon S3 supports a set of predefined ACLs, known as <i>canned ACLs</i>. Each canned ACL has a predefined set of grantees and permissions. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html#CannedACL">Canned ACL</a>.</p> </li> <li> <p>Specify access permissions explicitly using the <code>x-amz-grant-read</code>, <code>x-amz-grant-write</code>, <code>x-amz-grant-read-acp</code>, <code>x-amz-grant-write-acp</code>, and <code>x-amz-grant-full-control</code> headers. These headers map to the set of permissions Amazon S3 supports in an ACL. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html">Access Control List (ACL) Overview</a>.</p> <p>You specify each grantee as a type=value pair, where the type is one of the following:</p> <ul> <li> <p> <code>id</code> – if the value specified is the canonical user ID of an AWS account</p> </li> <li> <p> <code>uri</code> – if you are granting permissions to a predefined group</p> </li> <li> <p> <code>emailAddress</code> – if the value specified is the email address of an AWS account</p> <note> <p>Using email addresses to specify a grantee is only supported in the following AWS Regions: </p> <ul> <li> <p>US East (N. Virginia)</p> </li> <li> <p>US West (N. California)</p> </li> <li> <p> US West (Oregon)</p> </li> <li> <p> Asia Pacific (Singapore)</p> </li> <li> <p>Asia Pacific (Sydney)</p> </li> <li> <p>Asia Pacific (Tokyo)</p> </li> <li> <p>Europe (Ireland)</p> </li> <li> <p>South America (São Paulo)</p> </li> </ul> <p>For a list of all the Amazon S3 supported Regions and endpoints, see <a href="https://docs.aws.amazon.com/general/latest/gr/rande.html#s3_region">Regions and Endpoints</a> in the AWS General Reference.</p> </note> </li> </ul> <p>For example, the following <code>x-amz-grant-read</code> header grants the AWS accounts identified by account IDs permissions to read object data and its metadata:</p> <p> <code>x-amz-grant-read: id=&quot;11112222333&quot;, id=&quot;444455556666&quot; </code> </p> </li> </ul> <note> <p>You can use either a canned ACL or specify access permissions explicitly. You cannot do both.</p> </note> <p>The following operations are related to <code>CreateBucket</code>:</p> <ul> <li> <p> <a>PutObject</a> </p> </li> <li> <p> <a>DeleteBucket</a> </p> </li> </ul></p>
    async fn create_bucket(
        &self,
        input: CreateBucketRequest,
    ) -> Result<CreateBucketOutput, Ks3Error<CreateBucketError>>;

    /// <p><p>Adds an object to a bucket. You must have WRITE permissions on a bucket to add an object to it.</p> <p>Amazon S3 never adds partial objects; if you receive a success response, Amazon S3 added the entire object to the bucket.</p> <p>Amazon S3 is a distributed system. If it receives multiple write requests for the same object simultaneously, it overwrites all but the last object written. Amazon S3 does not provide object locking; if you need this, make sure to build it into your application layer or use versioning instead.</p> <p>To ensure that data is not corrupted traversing the network, use the <code>Content-MD5</code> header. When you use this header, Amazon S3 checks the object against the provided MD5 value and, if they do not match, returns an error. Additionally, you can calculate the MD5 while putting an object to Amazon S3 and compare the returned ETag to the calculated MD5 value.</p> <note> <p> The <code>Content-MD5</code> header is required for any request to upload an object with a retention period configured using Amazon S3 Object Lock. For more information about Amazon S3 Object Lock, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/object-lock-overview.html">Amazon S3 Object Lock Overview</a> in the <i>Amazon Simple Storage Service Developer Guide</i>. </p> </note> <p> <b>Server-side Encryption</b> </p> <p>You can optionally request server-side encryption. With server-side encryption, Amazon S3 encrypts your data as it writes it to disks in its data centers and decrypts the data when you access it. You have the option to provide your own encryption key or use AWS managed encryption keys. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingServerSideEncryption.html">Using Server-Side Encryption</a>.</p> <p> <b>Access Control List (ACL)-Specific Request Headers</b> </p> <p>You can use headers to grant ACL- based permissions. By default, all objects are private. Only the owner has full access control. When adding a new object, you can grant permissions to individual AWS accounts or to predefined groups defined by Amazon S3. These permissions are then added to the ACL on the object. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html">Access Control List (ACL) Overview</a> and <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-using-rest-api.html">Managing ACLs Using the REST API</a>. </p> <p> <b>Storage Class Options</b> </p> <p>By default, Amazon S3 uses the STANDARD storage class to store newly created objects. The STANDARD storage class provides high durability and high availability. Depending on performance needs, you can specify a different storage class. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/storage-class-intro.html">Storage Classes</a> in the <i>Amazon S3 Service Developer Guide</i>.</p> <p> <b>Versioning</b> </p> <p>If you enable versioning for a bucket, Amazon S3 automatically generates a unique version ID for the object being stored. Amazon S3 returns this ID in the response. When you enable versioning for a bucket, if Amazon S3 receives multiple write requests for the same object simultaneously, it stores all of the objects.</p> <p>For more information about versioning, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/AddingObjectstoVersioningEnabledBuckets.html">Adding Objects to Versioning Enabled Buckets</a>. For information about returning the versioning state of a bucket, see <a>GetBucketVersioning</a>. </p> <p class="title"> <b>Related Resources</b> </p> <ul> <li> <p> <a>CopyObject</a> </p> </li> <li> <p> <a>DeleteObject</a> </p> </li> </ul></p>
    async fn put_object(
        &self,
        input: PutObjectRequest,
    ) -> Result<PutObjectOutput, Ks3Error<PutObjectError>>;
}

/// A client for the Amazon S3 API.
#[derive(Clone)]
pub struct S3Client {
    client: Client,
    region: Region,
}

impl S3Client {
    /// Creates a client backed by the default tokio event loop.
    ///
    /// The client will use the default credentials provider and tls client.
    pub fn new(region: Region) -> Self {
        S3Client {
            client: Client::shared(),
            region,
        }
    }

    pub fn new_with<P, D>(request_dispatcher: D, credentials_provider: P, region: Region) -> Self
    where
        P: ProvideAwsCredentials + Send + Sync + 'static,
        D: DispatchSignedRequest + Send + Sync + 'static,
    {
        S3Client {
            client: Client::new_with(credentials_provider, request_dispatcher),
            region,
        }
    }

    pub fn new_with_client(client: Client, region: Region) -> S3Client {
        S3Client { client, region }
    }
}

impl S3Client {
    async fn sign_and_dispatch<E>(
        &self,
        request: SignedRequest,
        from_response: fn(BufferedHttpResponse) -> Ks3Error<E>,
    ) -> Result<HttpResponse, Ks3Error<E>> {
        let mut response = self.client.sign_and_dispatch(request).await?;
        if !response.status.is_success() {
            let response = response.buffer().await.map_err(Ks3Error::HttpDispatch)?;
            return Err(from_response(response));
        }

        Ok(response)
    }
}

#[async_trait]
impl S3 for S3Client {
    /// <p><p>Creates a new bucket. To create a bucket, you must register with Amazon S3 and have a valid AWS Access Key ID to authenticate requests. Anonymous requests are never allowed to create buckets. By creating the bucket, you become the bucket owner.</p> <p>Not every string is an acceptable bucket name. For information on bucket naming restrictions, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingBucket.html">Working with Amazon S3 Buckets</a>.</p> <p>By default, the bucket is created in the US East (N. Virginia) Region. You can optionally specify a Region in the request body. You might choose a Region to optimize latency, minimize costs, or address regulatory requirements. For example, if you reside in Europe, you will probably find it advantageous to create buckets in the Europe (Ireland) Region. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingBucket.html#access-bucket-intro">How to Select a Region for Your Buckets</a>.</p> <note> <p>If you send your create bucket request to the <code>s3.amazonaws.com</code> endpoint, the request goes to the us-east-1 Region. Accordingly, the signature calculations in Signature Version 4 must use us-east-1 as the Region, even if the location constraint in the request specifies another Region where the bucket is to be created. If you create a bucket in a Region other than US East (N. Virginia), your application must be able to handle 307 redirect. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/VirtualHosting.html">Virtual Hosting of Buckets</a>.</p> </note> <p>When creating a bucket using this operation, you can optionally specify the accounts or groups that should be granted specific permissions on the bucket. There are two ways to grant the appropriate permissions using the request headers.</p> <ul> <li> <p>Specify a canned ACL using the <code>x-amz-acl</code> request header. Amazon S3 supports a set of predefined ACLs, known as <i>canned ACLs</i>. Each canned ACL has a predefined set of grantees and permissions. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html#CannedACL">Canned ACL</a>.</p> </li> <li> <p>Specify access permissions explicitly using the <code>x-amz-grant-read</code>, <code>x-amz-grant-write</code>, <code>x-amz-grant-read-acp</code>, <code>x-amz-grant-write-acp</code>, and <code>x-amz-grant-full-control</code> headers. These headers map to the set of permissions Amazon S3 supports in an ACL. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html">Access Control List (ACL) Overview</a>.</p> <p>You specify each grantee as a type=value pair, where the type is one of the following:</p> <ul> <li> <p> <code>id</code> – if the value specified is the canonical user ID of an AWS account</p> </li> <li> <p> <code>uri</code> – if you are granting permissions to a predefined group</p> </li> <li> <p> <code>emailAddress</code> – if the value specified is the email address of an AWS account</p> <note> <p>Using email addresses to specify a grantee is only supported in the following AWS Regions: </p> <ul> <li> <p>US East (N. Virginia)</p> </li> <li> <p>US West (N. California)</p> </li> <li> <p> US West (Oregon)</p> </li> <li> <p> Asia Pacific (Singapore)</p> </li> <li> <p>Asia Pacific (Sydney)</p> </li> <li> <p>Asia Pacific (Tokyo)</p> </li> <li> <p>Europe (Ireland)</p> </li> <li> <p>South America (São Paulo)</p> </li> </ul> <p>For a list of all the Amazon S3 supported Regions and endpoints, see <a href="https://docs.aws.amazon.com/general/latest/gr/rande.html#s3_region">Regions and Endpoints</a> in the AWS General Reference.</p> </note> </li> </ul> <p>For example, the following <code>x-amz-grant-read</code> header grants the AWS accounts identified by account IDs permissions to read object data and its metadata:</p> <p> <code>x-amz-grant-read: id=&quot;11112222333&quot;, id=&quot;444455556666&quot; </code> </p> </li> </ul> <note> <p>You can use either a canned ACL or specify access permissions explicitly. You cannot do both.</p> </note> <p>The following operations are related to <code>CreateBucket</code>:</p> <ul> <li> <p> <a>PutObject</a> </p> </li> <li> <p> <a>DeleteBucket</a> </p> </li> </ul></p>
    #[allow(unused_variables, warnings)]
    async fn create_bucket(
        &self,
        input: CreateBucketRequest,
    ) -> Result<CreateBucketOutput, Ks3Error<CreateBucketError>> {
        let request_uri = format!("/{bucket}", bucket = input.bucket);
        let mut request = SignedRequest::new("PUT", "s3", &self.region, &request_uri);

        request.add_optional_header("x-amz-acl", input.acl.as_ref());
        request.add_optional_header(
            "x-amz-grant-full-control",
            input.grant_full_control.as_ref(),
        );
        request.add_optional_header("x-amz-grant-read", input.grant_read.as_ref());
        request.add_optional_header("x-amz-grant-read-acp", input.grant_read_acp.as_ref());
        request.add_optional_header("x-amz-grant-write", input.grant_write.as_ref());
        request.add_optional_header("x-amz-grant-write-acp", input.grant_write_acp.as_ref());
        request.add_optional_header(
            "x-amz-bucket-object-lock-enabled",
            input.object_lock_enabled_for_bucket.as_ref(),
        );

        if input.create_bucket_configuration.is_some() {
            let mut writer = EventWriter::new(Vec::new());
            CreateBucketConfigurationSerializer::serialize(
                &mut writer,
                "CreateBucketConfiguration",
                input.create_bucket_configuration.as_ref().unwrap(),
            );
            request.set_payload(Some(writer.into_inner()));
        } else {
            request.set_payload(Some(Vec::new()));
        }

        let mut response = self
            .sign_and_dispatch(request, CreateBucketError::from_response)
            .await?;

        let result = CreateBucketOutput::default();
        let mut result = result;
        result.location = response.headers.remove("Location"); // parse non-payload
        Ok(result)
    }

    /// <p><p>Adds an object to a bucket. You must have WRITE permissions on a bucket to add an object to it.</p> <p>Amazon S3 never adds partial objects; if you receive a success response, Amazon S3 added the entire object to the bucket.</p> <p>Amazon S3 is a distributed system. If it receives multiple write requests for the same object simultaneously, it overwrites all but the last object written. Amazon S3 does not provide object locking; if you need this, make sure to build it into your application layer or use versioning instead.</p> <p>To ensure that data is not corrupted traversing the network, use the <code>Content-MD5</code> header. When you use this header, Amazon S3 checks the object against the provided MD5 value and, if they do not match, returns an error. Additionally, you can calculate the MD5 while putting an object to Amazon S3 and compare the returned ETag to the calculated MD5 value.</p> <note> <p> The <code>Content-MD5</code> header is required for any request to upload an object with a retention period configured using Amazon S3 Object Lock. For more information about Amazon S3 Object Lock, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/object-lock-overview.html">Amazon S3 Object Lock Overview</a> in the <i>Amazon Simple Storage Service Developer Guide</i>. </p> </note> <p> <b>Server-side Encryption</b> </p> <p>You can optionally request server-side encryption. With server-side encryption, Amazon S3 encrypts your data as it writes it to disks in its data centers and decrypts the data when you access it. You have the option to provide your own encryption key or use AWS managed encryption keys. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingServerSideEncryption.html">Using Server-Side Encryption</a>.</p> <p> <b>Access Control List (ACL)-Specific Request Headers</b> </p> <p>You can use headers to grant ACL- based permissions. By default, all objects are private. Only the owner has full access control. When adding a new object, you can grant permissions to individual AWS accounts or to predefined groups defined by Amazon S3. These permissions are then added to the ACL on the object. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html">Access Control List (ACL) Overview</a> and <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-using-rest-api.html">Managing ACLs Using the REST API</a>. </p> <p> <b>Storage Class Options</b> </p> <p>By default, Amazon S3 uses the STANDARD storage class to store newly created objects. The STANDARD storage class provides high durability and high availability. Depending on performance needs, you can specify a different storage class. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/storage-class-intro.html">Storage Classes</a> in the <i>Amazon S3 Service Developer Guide</i>.</p> <p> <b>Versioning</b> </p> <p>If you enable versioning for a bucket, Amazon S3 automatically generates a unique version ID for the object being stored. Amazon S3 returns this ID in the response. When you enable versioning for a bucket, if Amazon S3 receives multiple write requests for the same object simultaneously, it stores all of the objects.</p> <p>For more information about versioning, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/AddingObjectstoVersioningEnabledBuckets.html">Adding Objects to Versioning Enabled Buckets</a>. For information about returning the versioning state of a bucket, see <a>GetBucketVersioning</a>. </p> <p class="title"> <b>Related Resources</b> </p> <ul> <li> <p> <a>CopyObject</a> </p> </li> <li> <p> <a>DeleteObject</a> </p> </li> </ul></p>
    #[allow(unused_variables, warnings)]
    async fn put_object(
        &self,
        input: PutObjectRequest,
    ) -> Result<PutObjectOutput, Ks3Error<PutObjectError>> {
        let request_uri = format!("/{bucket}/{key}", bucket = input.bucket, key = input.key);

        let mut request = SignedRequest::new("PUT", "s3", &self.region, &request_uri);

        request.add_optional_header("x-amz-acl", input.acl.as_ref());
        request.add_optional_header("Cache-Control", input.cache_control.as_ref());
        request.add_optional_header("Content-Disposition", input.content_disposition.as_ref());
        request.add_optional_header("Content-Encoding", input.content_encoding.as_ref());
        request.add_optional_header("Content-Language", input.content_language.as_ref());
        request.add_optional_header("Content-Length", input.content_length.as_ref());
        request.add_optional_header("Content-MD5", input.content_md5.as_ref());
        request.add_optional_header("Content-Type", input.content_type.as_ref());
        request.add_optional_header("Expires", input.expires.as_ref());
        request.add_optional_header(
            "x-amz-grant-full-control",
            input.grant_full_control.as_ref(),
        );
        request.add_optional_header("x-amz-grant-read", input.grant_read.as_ref());
        request.add_optional_header("x-amz-grant-read-acp", input.grant_read_acp.as_ref());
        request.add_optional_header("x-amz-grant-write-acp", input.grant_write_acp.as_ref());

        if let Some(ref metadata) = input.metadata {
            for (header_name, header_value) in metadata.iter() {
                let header = format!("x-amz-meta-{}", header_name);
                request.add_header(header, header_value);
            }
        }
        request.add_optional_header(
            "x-amz-object-lock-legal-hold",
            input.object_lock_legal_hold_status.as_ref(),
        );
        request.add_optional_header("x-amz-object-lock-mode", input.object_lock_mode.as_ref());
        request.add_optional_header(
            "x-amz-object-lock-retain-until-date",
            input.object_lock_retain_until_date.as_ref(),
        );
        request.add_optional_header("x-amz-request-payer", input.request_payer.as_ref());
        request.add_optional_header(
            "x-amz-server-side-encryption-customer-algorithm",
            input.sse_customer_algorithm.as_ref(),
        );
        request.add_optional_header(
            "x-amz-server-side-encryption-customer-key",
            input.sse_customer_key.as_ref(),
        );
        request.add_optional_header(
            "x-amz-server-side-encryption-customer-key-MD5",
            input.sse_customer_key_md5.as_ref(),
        );
        request.add_optional_header(
            "x-amz-server-side-encryption-context",
            input.ssekms_encryption_context.as_ref(),
        );
        request.add_optional_header(
            "x-amz-server-side-encryption-aws-kms-key-id",
            input.ssekms_key_id.as_ref(),
        );
        request.add_optional_header(
            "x-amz-server-side-encryption",
            input.server_side_encryption.as_ref(),
        );
        request.add_optional_header("x-amz-storage-class", input.storage_class.as_ref());
        request.add_optional_header("x-amz-tagging", input.tagging.as_ref());
        request.add_optional_header(
            "x-amz-website-redirect-location",
            input.website_redirect_location.as_ref(),
        );

        if let Some(__body) = input.body {
            request.set_payload_stream(__body);
        }

        let mut response = self
            .sign_and_dispatch(request, PutObjectError::from_response)
            .await?;

        let result = PutObjectOutput::default();
        let mut result = result;
        result.e_tag = response.headers.remove("ETag");
        result.expiration = response.headers.remove("x-amz-expiration");
        result.request_charged = response.headers.remove("x-amz-request-charged");
        result.sse_customer_algorithm = response
            .headers
            .remove("x-amz-server-side-encryption-customer-algorithm");
        result.sse_customer_key_md5 = response
            .headers
            .remove("x-amz-server-side-encryption-customer-key-MD5");
        result.ssekms_encryption_context = response
            .headers
            .remove("x-amz-server-side-encryption-context");
        result.ssekms_key_id = response
            .headers
            .remove("x-amz-server-side-encryption-aws-kms-key-id");
        result.server_side_encryption = response.headers.remove("x-amz-server-side-encryption");
        result.version_id = response.headers.remove("x-amz-version-id"); // parse non-payload
        Ok(result)
    }
}
