#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    English,
    Spanish,
    SimplifiedChinese,
    Catalan,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("unsupported locale `{0}`")]
pub struct UnsupportedLocaleError(pub String);

impl Locale {
    pub fn parse(raw: &str) -> Result<Self, UnsupportedLocaleError> {
        match raw {
            "en" => Ok(Self::English),
            "es" => Ok(Self::Spanish),
            "zh-CN" => Ok(Self::SimplifiedChinese),
            "ca" => Ok(Self::Catalan),
            _ => Err(UnsupportedLocaleError(raw.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::SimplifiedChinese => "zh-CN",
            Self::Catalan => "ca",
        }
    }
}
