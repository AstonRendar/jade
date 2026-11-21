use crate::domain::value_object::WikiLink;

const FENCES: [&str; 2] = ["```", "~~~"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedLink {
    pub line: usize,
    pub link: WikiLink,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WikiLinkParser;

impl WikiLinkParser {
    pub fn parse(&self, content: &str) -> Vec<LocatedLink> {
        let mut inside_fence = false;
        let mut links = Vec::new();
        for (index, line) in content.lines().enumerate() {
            if FENCES
                .iter()
                .any(|fence| line.trim_start().starts_with(fence))
            {
                inside_fence = !inside_fence;
                continue;
            }
            if inside_fence {
                continue;
            }
            links.extend(
                links_in(&without_inline_code(line))
                    .into_iter()
                    .map(|link| LocatedLink {
                        line: index + 1,
                        link,
                    }),
            );
        }
        links
    }
}

fn without_inline_code(line: &str) -> String {
    let mut inside_code = false;
    line.chars()
        .map(|character| {
            if character == '`' {
                inside_code = !inside_code;
                ' '
            } else if inside_code {
                ' '
            } else {
                character
            }
        })
        .collect()
}

fn links_in(line: &str) -> Vec<WikiLink> {
    let mut links = Vec::new();
    let mut rest = line;
    let mut consumed = 0;
    while let Some(close) = rest.find("]]") {
        if let Some(open) = rest[..close].rfind("[[") {
            let embed = line[..consumed + open].ends_with('!');
            if let Some(link) = parse_inner(&rest[open + 2..close], embed) {
                links.push(link);
            }
        }
        consumed += close + 2;
        rest = &rest[close + 2..];
    }
    links
}

fn parse_inner(inner: &str, embed: bool) -> Option<WikiLink> {
    let (reference, alias) = match inner.split_once('|') {
        Some((reference, alias)) => (reference, Some(alias)),
        None => (inner, None),
    };
    let (target, heading) = match reference.split_once('#') {
        Some((target, heading)) => (target, Some(heading)),
        None => (reference, None),
    };
    WikiLink::new(target, heading, alias, embed).ok()
}
