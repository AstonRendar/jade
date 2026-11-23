use super::frontmatter::Frontmatter;
use crate::domain::value_object::Data;

const RESERVED_WORDS: [&str; 10] = [
    "true", "false", "yes", "no", "on", "off", "null", "~", "True", "False",
];
const INDICATORS: [char; 18] = [
    '-', '?', ':', ',', '[', ']', '{', '}', '#', '&', '*', '!', '|', '>', '\'', '"', '%', '@',
];

#[derive(Debug, Clone, Copy, Default)]
pub struct FrontmatterWriter;

impl FrontmatterWriter {
    pub fn write(&self, content: &str, properties: &[(String, Data)]) -> String {
        let body = Frontmatter::split(content).body;
        if properties.is_empty() {
            return body.to_string();
        }
        let mut yaml = String::from("---\n");
        for (key, value) in properties {
            yaml.push_str(&scalar(key));
            yaml.push(':');
            entry(&mut yaml, value, 0);
        }
        yaml.push_str("---\n");
        yaml + body
    }
}

fn entry(yaml: &mut String, value: &Data, indent: usize) {
    match value {
        Data::List(items) if items.is_empty() => yaml.push_str(" []\n"),
        Data::List(items) => {
            yaml.push('\n');
            for item in items {
                yaml.push_str(&" ".repeat(indent + 2));
                yaml.push('-');
                entry(yaml, item, indent + 2);
            }
        }
        Data::Map(entries) if entries.is_empty() => yaml.push_str(" {}\n"),
        Data::Map(entries) => {
            yaml.push('\n');
            for (key, nested) in entries {
                yaml.push_str(&" ".repeat(indent + 2));
                yaml.push_str(&scalar(key));
                yaml.push(':');
                entry(yaml, nested, indent + 2);
            }
        }
        Data::Null => yaml.push('\n'),
        other => {
            yaml.push(' ');
            yaml.push_str(&inline(other));
            yaml.push('\n');
        }
    }
}

fn inline(value: &Data) -> String {
    match value {
        Data::Boolean(flag) => flag.to_string(),
        Data::Number(number) if number.fract() == 0.0 && number.abs() < 1e15 => {
            format!("{}", *number as i64)
        }
        Data::Number(number) => number.to_string(),
        Data::Text(text) => scalar(text),
        _ => String::new(),
    }
}

fn scalar(text: &str) -> String {
    if needs_quotes(text) {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        format!("\"{escaped}\"")
    } else {
        text.to_string()
    }
}

fn needs_quotes(text: &str) -> bool {
    text.is_empty()
        || text.trim() != text
        || text.starts_with(INDICATORS)
        || text.contains(": ")
        || text.contains(" #")
        || text.ends_with(':')
        || text.contains(['\n', '\t'])
        || RESERVED_WORDS.contains(&text)
        || text.parse::<f64>().is_ok()
}
