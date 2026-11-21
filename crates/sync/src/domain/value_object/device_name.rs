const MAX_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceName(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidDeviceNameError {
    #[error("device name is empty")]
    Empty,
    #[error("device name contains control characters")]
    ContainsControlCharacters,
    #[error("device name is longer than {MAX_LENGTH} characters")]
    TooLong,
}

impl DeviceName {
    pub fn parse(raw: &str) -> Result<Self, InvalidDeviceNameError> {
        let name = raw.trim();
        if name.is_empty() {
            return Err(InvalidDeviceNameError::Empty);
        }
        if name.chars().any(char::is_control) {
            return Err(InvalidDeviceNameError::ContainsControlCharacters);
        }
        if name.chars().count() > MAX_LENGTH {
            return Err(InvalidDeviceNameError::TooLong);
        }
        Ok(Self(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
