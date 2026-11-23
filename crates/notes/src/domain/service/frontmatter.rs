const DELIMITER: &str = "---";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frontmatter<'a> {
    pub yaml: Option<&'a str>,
    pub body: &'a str,
}

impl<'a> Frontmatter<'a> {
    pub fn split(content: &'a str) -> Self {
        let Some(rest) = opening(content) else {
            return Self {
                yaml: None,
                body: content,
            };
        };
        let mut offset = 0;
        for line in rest.split_inclusive('\n') {
            if line.trim_end_matches(['\n', '\r']) == DELIMITER {
                return Self {
                    yaml: Some(&rest[..offset]),
                    body: &rest[offset + line.len()..],
                };
            }
            offset += line.len();
        }
        Self {
            yaml: None,
            body: content,
        }
    }
}

fn opening(content: &str) -> Option<&str> {
    content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
}
