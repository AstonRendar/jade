use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct AccessToken(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("access token is empty or contains whitespace")]
pub struct InvalidAccessTokenError;

impl AccessToken {
    pub fn new(raw: &str) -> Result<Self, InvalidAccessTokenError> {
        if raw.is_empty() || raw.chars().any(char::is_whitespace) {
            return Err(InvalidAccessTokenError);
        }
        Ok(Self(raw.to_string()))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccessToken(***)")
    }
}
