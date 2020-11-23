//! The Credentials Provider to read from Environment Variables.
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};

use crate::credential::{
    non_empty_env_var, AwsCredentials, CredentialsError, ProvideAwsCredentials,
};

/// Provides AWS credentials from environment variables.
///
/// # Available Environment Variables
///
/// * `AWS_ACCESS_KEY_ID`:
///
///   [Access key ID](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html#access-keys-and-secret-access-keys)
///
/// * `AWS_SECRET_ACCESS_KEY`:
///
///   [Secret access key](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html#access-keys-and-secret-access-keys)
///
/// * `AWS_SESSION_TOKEN`:
///
///   [Session token](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_temp.html)
///
/// * `AWS_CREDENTIAL_EXPIRATION`:
///
///   Expiration time in RFC 3339 format (e.g. `1996-12-19T16:39:57-08:00`). If unset, credentials
///   won't expire.
///
/// # Example
///
/// ```rust
/// use futures::future::Future;
/// use rusoto_credential::{EnvironmentProvider, ProvideAwsCredentials};
/// use std::env;
///
/// #[tokio::main]
/// async fn main() {
///     env::set_var("AWS_ACCESS_KEY_ID", "ANTN35UAENTS5UIAEATD");
///     env::set_var("AWS_SECRET_ACCESS_KEY", "TtnuieannGt2rGuie2t8Tt7urarg5nauedRndrur");
///     env::set_var("AWS_SESSION_TOKEN", "DfnGs8Td4rT8r4srxAg6Td4rT8r4srxAg6GtkTir");
///
///     let creds = EnvironmentProvider::default().credentials().await.unwrap();
///
///     assert_eq!(creds.aws_access_key_id(), "ANTN35UAENTS5UIAEATD");
///     assert_eq!(creds.aws_secret_access_key(), "TtnuieannGt2rGuie2t8Tt7urarg5nauedRndrur");
///     assert_eq!(creds.token(), &Some("DfnGs8Td4rT8r4srxAg6Td4rT8r4srxAg6GtkTir".to_string()));
///     assert!(creds.expires_at().is_none()); // doesn't expire
///
///     env::set_var("AWS_CREDENTIAL_EXPIRATION", "2018-04-21T01:13:02Z");
///     let creds = EnvironmentProvider::default().credentials().await.unwrap();
///     assert_eq!(creds.expires_at().unwrap().to_rfc3339(), "2018-04-21T01:13:02+00:00");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct EnvironmentProvider {
    prefix: String,
}

impl Default for EnvironmentProvider {
    fn default() -> Self {
        EnvironmentProvider {
            prefix: "AWS".to_owned(),
        }
    }
}

impl EnvironmentProvider {
    /// Create an EnvironmentProvider with a non-standard variable prefix.
    ///
    /// ```rust
    /// use std::future::Future;
    /// use rusoto_credential::{EnvironmentProvider, ProvideAwsCredentials};
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() -> () {
    ///     env::set_var("MYAPP_ACCESS_KEY_ID", "ANTN35UAENTS5UIAEATD");
    ///     env::set_var("MYAPP_SECRET_ACCESS_KEY", "TtnuieannGt2rGuie2t8Tt7urarg5nauedRndrur");
    ///     env::set_var("MYAPP_SESSION_TOKEN", "DfnGs8Td4rT8r4srxAg6Td4rT8r4srxAg6GtkTir");
    ///
    ///     let creds = EnvironmentProvider::with_prefix("MYAPP").credentials().await.unwrap();
    ///
    ///     assert_eq!(creds.aws_access_key_id(), "ANTN35UAENTS5UIAEATD");
    ///     assert_eq!(creds.aws_secret_access_key(), "TtnuieannGt2rGuie2t8Tt7urarg5nauedRndrur");
    ///     assert_eq!(creds.token(), &Some("DfnGs8Td4rT8r4srxAg6Td4rT8r4srxAg6GtkTir".to_string()));
    ///     assert!(creds.expires_at().is_none()); // doesn't expire
    ///
    ///     env::set_var("MYAPP_CREDENTIAL_EXPIRATION", "2018-04-21T01:13:02Z");
    ///     let creds = EnvironmentProvider::with_prefix("MYAPP").credentials().await.unwrap();
    ///     assert_eq!(creds.expires_at().unwrap().to_rfc3339(), "2018-04-21T01:13:02+00:00");
    /// }
    /// ```
    pub fn with_prefix(prefix: &str) -> Self {
        EnvironmentProvider {
            prefix: prefix.to_owned(),
        }
    }
}

/// A private trait for building the environment variable names based
/// on a provided prefix. Smallest subset of functionality needed for
/// Credentials building (see `credentials_from_environment` below).
trait EnvironmentVariableProvider {
    fn prefix(&self) -> &str;

    fn access_key_id_var(&self) -> String {
        format!("{}_ACCESS_KEY_ID", self.prefix())
    }

    fn secret_access_key_var(&self) -> String {
        format!("{}_SECRET_ACCESS_KEY", self.prefix())
    }

    fn session_token_var(&self) -> String {
        format!("{}_SESSION_TOKEN", self.prefix())
    }

    fn credential_expiration_var(&self) -> String {
        format!("{}_CREDENTIAL_EXPIRATION", self.prefix())
    }
}

impl EnvironmentVariableProvider for EnvironmentProvider {
    fn prefix(&self) -> &str {
        self.prefix.as_str()
    }
}

#[async_trait]
impl ProvideAwsCredentials for EnvironmentProvider {
    async fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
        let env_key = get_critical_variable(self.access_key_id_var())?;
        let env_secret = get_critical_variable(self.secret_access_key_var())?;
        // Present when using temporary credentials, e.g. on Lambda with IAM roles
        let token = non_empty_env_var(&self.session_token_var());
        // Mimic botocore's behavior, see https://github.com/boto/botocore/pull/1187.
        let var_name = self.credential_expiration_var();
        let expires_at = match non_empty_env_var(&var_name) {
            Some(val) => Some(
                DateTime::<FixedOffset>::parse_from_rfc3339(&val)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| {
                        CredentialsError::new(format!(
                            "Invalid {} in environment '{}': {}",
                            var_name, val, e
                        ))
                    })?,
            ),
            _ => None,
        };
        Ok(AwsCredentials::new(env_key, env_secret, token, expires_at))
    }
}

/// Force an error if we do not see the particular variable name in the env.
fn get_critical_variable(var_name: String) -> Result<String, CredentialsError> {
    non_empty_env_var(&var_name)
        .ok_or_else(|| CredentialsError::new(format!("No (or empty) {} in environment", var_name)))
}
