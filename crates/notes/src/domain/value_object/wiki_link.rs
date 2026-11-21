#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLink {
    target: String,
    heading: Option<String>,
    alias: Option<String>,
    embed: bool,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("wikilink target is empty")]
pub struct EmptyWikiLinkTargetError;

impl WikiLink {
    pub fn new(
        target: &str,
        heading: Option<&str>,
        alias: Option<&str>,
        embed: bool,
    ) -> Result<Self, EmptyWikiLinkTargetError> {
        let target = target.trim();
        if target.is_empty() {
            return Err(EmptyWikiLinkTargetError);
        }
        Ok(Self {
            target: target.to_string(),
            heading: non_blank(heading),
            alias: non_blank(alias),
            embed,
        })
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn heading(&self) -> Option<&str> {
        self.heading.as_deref()
    }

    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    pub fn is_embed(&self) -> bool {
        self.embed
    }
}

fn non_blank(text: Option<&str>) -> Option<String> {
    text.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}
