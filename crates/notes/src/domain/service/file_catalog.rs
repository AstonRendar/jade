use super::frontmatter::Frontmatter;
use super::tag_parser::TagParser;
use super::wiki_link_parser::WikiLinkParser;
use crate::domain::model::{FileMetadata, FileRecord};
use crate::domain::value_object::{Data, VaultPath};

#[derive(Debug, Clone, Copy, Default)]
pub struct FileCatalog;

impl FileCatalog {
    pub fn file(&self, path: VaultPath, metadata: FileMetadata) -> FileRecord {
        FileRecord::new(path, metadata)
    }

    pub fn note(
        &self,
        path: VaultPath,
        metadata: FileMetadata,
        frontmatter: Option<&Data>,
        content: &str,
    ) -> FileRecord {
        let properties = frontmatter
            .and_then(Data::as_map)
            .map(<[(String, Data)]>::to_vec)
            .unwrap_or_default();
        let body = Frontmatter::split(content).body;
        let mut tags = tags_property(frontmatter);
        for tag in TagParser.parse(body) {
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }
        let links = WikiLinkParser
            .parse(body)
            .into_iter()
            .map(|located| located.link.target().to_string())
            .collect();
        FileRecord::new(path, metadata).with_contents(properties, tags, links)
    }
}

fn tags_property(frontmatter: Option<&Data>) -> Vec<String> {
    match frontmatter.and_then(|data| data.get("tags")) {
        Some(Data::List(items)) => items
            .iter()
            .filter_map(Data::as_text)
            .map(clean_tag)
            .collect(),
        Some(Data::Text(text)) => text
            .split([',', ' '])
            .filter(|tag| !tag.is_empty())
            .map(clean_tag)
            .collect(),
        _ => Vec::new(),
    }
}

fn clean_tag(tag: &str) -> String {
    tag.trim().trim_start_matches('#').to_string()
}
