use std::fmt;

use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub access_key: String,
    pub secret_key: SecretKey,
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    /// \<date\> value is specified using YYYYMMDD format.
    pub date: String,
    /// region
    pub region: String,
    /// \<aws-service\> value is `s3` when sending request to Amazon S3.
    pub service: String,

    pub request: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretKey(Box<str>);

impl SecretKey {
    fn new(s: impl Into<Box<str>>) -> Self {
        Self(s.into())
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl Zeroize for SecretKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl From<String> for SecretKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<Box<str>> for SecretKey {
    fn from(value: Box<str>) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = "[SECRET-KEY]";
        f.debug_tuple("SecretKey").field(&inner).finish()
    }
}
