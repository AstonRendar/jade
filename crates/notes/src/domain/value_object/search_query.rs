use super::fold;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    text: String,
    folded: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("search query is empty")]
pub struct EmptySearchQueryError;

impl SearchQuery {
    pub fn parse(raw: &str) -> Result<Self, EmptySearchQueryError> {
        let text = raw.trim();
        if text.is_empty() {
            return Err(EmptySearchQueryError);
        }
        Ok(Self {
            text: text.to_string(),
            folded: fold(text),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn matches(&self, text: &str) -> bool {
        fold(text).contains(&self.folded)
    }
}
