const FENCES: [&str; 2] = ["```", "~~~"];

#[derive(Debug, Clone, Copy, Default)]
pub struct TagParser;

impl TagParser {
    pub fn parse(&self, body: &str) -> Vec<String> {
        let mut inside_fence = false;
        let mut tags = Vec::new();
        for line in body.lines() {
            if FENCES
                .iter()
                .any(|fence| line.trim_start().starts_with(fence))
            {
                inside_fence = !inside_fence;
            } else if !inside_fence {
                tags.extend(tags_in(line));
            }
        }
        tags
    }
}

fn tags_in(line: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_code = false;
    let mut previous: Option<char> = None;
    let mut characters = line.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if character == '`' {
            in_code = !in_code;
        } else if character == '#' && !in_code && previous.is_none_or(char::is_whitespace) {
            let name: String = line[index + 1..]
                .chars()
                .take_while(|&c| is_tag_character(c))
                .collect();
            if name.chars().any(|c| !c.is_ascii_digit() && c != '/') {
                tags.push(name.trim_end_matches('/').to_string());
            }
            for _ in 0..name.chars().count() {
                characters.next();
            }
        }
        previous = Some(character);
    }
    tags
}

fn is_tag_character(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '-' | '/')
}
