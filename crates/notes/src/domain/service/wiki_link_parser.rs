use std::ops::Range;

use crate::domain::value_object::WikiLink;

const FENCES: [&str; 2] = ["```", "~~~"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedLink {
    pub line: usize,
    pub link: WikiLink,
    pub target_span: Range<usize>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WikiLinkParser;

impl WikiLinkParser {
    pub fn parse(&self, content: &str) -> Vec<LocatedLink> {
        let mut inside_fence = false;
        let mut links = Vec::new();
        let mut line_start = 0;
        for (index, raw_line) in content.split_inclusive('\n').enumerate() {
            let line = raw_line.trim_end_matches(['\n', '\r']);
            if FENCES
                .iter()
                .any(|fence| line.trim_start().starts_with(fence))
            {
                inside_fence = !inside_fence;
            } else if !inside_fence {
                links.extend(links_in(&without_inline_code(line)).into_iter().map(
                    |(link, span)| LocatedLink {
                        line: index + 1,
                        link,
                        target_span: line_start + span.start..line_start + span.end,
                    },
                ));
            }
            line_start += raw_line.len();
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
                " ".to_string()
            } else if inside_code {
                " ".repeat(character.len_utf8())
            } else {
                character.to_string()
            }
        })
        .collect()
}

fn links_in(line: &str) -> Vec<(WikiLink, Range<usize>)> {
    let mut links = Vec::new();
    let mut consumed = 0;
    while let Some(close) = line[consumed..].find("]]") {
        let close = consumed + close;
        if let Some(open) = line[consumed..close].rfind("[[") {
            let inner_start = consumed + open + 2;
            let embed = line[..consumed + open].ends_with('!');
            if let Some(parsed) = parse_inner(&line[inner_start..close], inner_start, embed) {
                links.push(parsed);
            }
        }
        consumed = close + 2;
    }
    links
}

fn parse_inner(inner: &str, offset: usize, embed: bool) -> Option<(WikiLink, Range<usize>)> {
    let (reference, alias) = match inner.split_once('|') {
        Some((reference, alias)) => (reference, Some(alias)),
        None => (inner, None),
    };
    let (target, heading) = match reference.split_once('#') {
        Some((target, heading)) => (target, Some(heading)),
        None => (reference, None),
    };
    let link = WikiLink::new(target, heading, alias, embed).ok()?;
    let leading = target.len() - target.trim_start().len();
    let start = offset + leading;
    Some((link, start..start + target.trim().len()))
}
