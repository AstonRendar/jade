#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown theme `{0}`")]
pub struct UnknownThemeError(pub String);

impl Theme {
    pub fn parse(raw: &str) -> Result<Self, UnknownThemeError> {
        match raw {
            "system" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(UnknownThemeError(raw.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}
