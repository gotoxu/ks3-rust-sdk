//! Provides a way to create static/programmatically generated AWS Credentials.
//! For those who can't get them from an environment, or a file.
use async_trait::async_trait;
use chrono::{Duration, Utc};

use crate::credential::{AwsCredentials, CredentialsError, ProvideAwsCredentials};

/// Provides AWS credentials from statically/programmatically provided strings.
#[derive(Clone, Debug)]
pub struct StaticProvider {
    /// AWS credentials
    credentials: AwsCredentials,

    /// The time in seconds for which each issued token is valid.
    valid_for: Option<i64>,
}

impl StaticProvider {
    /// Creates a new Static Provider. This should be used when you want to statically, or programmatically
    /// provide access to AWS.
    ///
    /// `valid_for` is the number of seconds for which issued tokens are valid.
    pub fn new(
        access_key: String,
        secret_access_key: String,
        token: Option<String>,
        valid_for: Option<i64>,
    ) -> StaticProvider {
        StaticProvider {
            credentials: AwsCredentials::new(access_key, secret_access_key, token, None),
            valid_for,
        }
    }

    /// Creates a new minimal Static Provider. This will set the token as optional none.
    pub fn new_minimal(access_key: String, secret_access_key: String) -> StaticProvider {
        StaticProvider {
            credentials: AwsCredentials::new(access_key, secret_access_key, None, None),
            valid_for: None,
        }
    }

    /// Gets the AWS Access Key ID for this Static Provider.
    pub fn get_aws_access_key_id(&self) -> &str {
        &self.credentials.key
    }

    /// Gets the AWS Secret Access Key for this Static Provider.
    pub fn get_aws_secret_access_key(&self) -> &str {
        &self.credentials.secret
    }

    /// Determines if this Static Provider was given a Token.
    pub fn has_token(&self) -> bool {
        self.credentials.token.is_some()
    }

    /// Gets The Token this Static Provider was given.
    pub fn get_token(&self) -> &Option<String> {
        &self.credentials.token
    }

    /// Returns the length in seconds this Static Provider will be valid for.
    pub fn is_valid_for(&self) -> &Option<i64> {
        &self.valid_for
    }
}

#[async_trait]
impl ProvideAwsCredentials for StaticProvider {
    async fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
        let mut creds = self.credentials.clone();
        creds.expires_at = self.valid_for.map(|v| Utc::now() + Duration::seconds(v));
        Ok(creds)
    }
}

impl From<AwsCredentials> for StaticProvider {
    fn from(credentials: AwsCredentials) -> Self {
        StaticProvider {
            credentials,
            valid_for: None,
        }
    }
}
