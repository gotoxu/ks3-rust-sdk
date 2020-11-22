use serde::Deserialize;
use chrono::{DateTime, Duration as ChronoDuration, Utc};

use std::collections::BTreeMap;
use std::fmt;

/// AWS API access credentials, including access key, secret key, token (for IAM profiles),
/// expiration timestamp, and claims from federated login.
///
/// # Anonymous example
///
/// Some AWS services, like [s3](https://docs.aws.amazon.com/AmazonS3/latest/API/Welcome.html) do
/// not require authenticated credentials. For these cases you can use `AwsCredentials::default`
/// with `StaticProvider`.
#[derive(Clone, Deserialize, Default)]
pub struct AwsCredentials {
    #[serde(rename = "AccessKeyId")]
    key: String,
    #[serde(rename = "SecretAccessKey")]
    secret: String,
    #[serde(rename = "SessionToken", alias = "Token")]
    token: Option<String>,
    #[serde(rename = "Expiration")]
    expires_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    claims: BTreeMap<String, String>,
}

impl AwsCredentials {
    /// Create a new `AwsCredentials` from a key ID, secret key, optional access token, and expiry
    /// time.
    pub fn new<K, S>(
        key: K,
        secret: S,
        token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> AwsCredentials
    where
        K: Into<String>,
        S: Into<String>,
    {
        AwsCredentials {
            key: key.into(),
            secret: secret.into(),
            token,
            expires_at,
            claims: BTreeMap::new(),
        }
    }

    /// Get a reference to the access key ID.
    pub fn aws_access_key_id(&self) -> &str {
        &self.key
    }

    /// Get a reference to the secret access key.
    pub fn aws_secret_access_key(&self) -> &str {
        &self.secret
    }

    /// Get a reference to the expiry time.
    pub fn expires_at(&self) -> &Option<DateTime<Utc>> {
        &self.expires_at
    }

    /// Get a reference to the access token.
    pub fn token(&self) -> &Option<String> {
        &self.token
    }

    /// Determine whether or not the credentials are expired.
    fn credentials_are_expired(&self) -> bool {
        match self.expires_at {
            Some(ref e) =>
            // This is a rough hack to hopefully avoid someone requesting creds then sitting on them
            // before issuing the request:
            {
                *e < Utc::now() + ChronoDuration::seconds(20)
            }
            None => false,
        }
    }

    /// Get the token claims
    pub fn claims(&self) -> &BTreeMap<String, String> {
        &self.claims
    }

    /// Get the mutable token claims
    pub fn claims_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.claims
    }
}

impl fmt::Debug for AwsCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AwsCredentials")
            .field("key", &self.key)
            .field("secret", &"**********")
            .field("token", &self.token.as_ref().map(|_| "**********"))
            .field("expires_at", &self.expires_at)
            .field("claims", &self.claims)
            .finish()
    }
}