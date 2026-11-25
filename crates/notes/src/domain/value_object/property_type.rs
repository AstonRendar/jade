use super::Data;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyType {
    Text,
    List,
    Number,
    Checkbox,
    Date,
    DateTime,
    Tags,
    Aliases,
}

impl PropertyType {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "text" => Self::Text,
            "multitext" => Self::List,
            "number" => Self::Number,
            "checkbox" => Self::Checkbox,
            "date" => Self::Date,
            "datetime" => Self::DateTime,
            "tags" => Self::Tags,
            "aliases" => Self::Aliases,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::List => "multitext",
            Self::Number => "number",
            Self::Checkbox => "checkbox",
            Self::Date => "date",
            Self::DateTime => "datetime",
            Self::Tags => "tags",
            Self::Aliases => "aliases",
        }
    }

    pub fn is_assignable(self) -> bool {
        !matches!(self, Self::Tags | Self::Aliases)
    }

    pub fn infer(key: &str, value: &Data) -> Option<Self> {
        match key {
            "tags" => return Some(Self::Tags),
            "aliases" => return Some(Self::Aliases),
            _ => {}
        }
        match value {
            Data::Null | Data::Map(_) => None,
            Data::Boolean(_) => Some(Self::Checkbox),
            Data::Number(_) => Some(Self::Number),
            Data::List(_) => Some(Self::List),
            Data::Text(text) if is_date(text) => Some(Self::Date),
            Data::Text(text) if is_date_time(text) => Some(Self::DateTime),
            Data::Text(_) => Some(Self::Text),
        }
    }
}

fn is_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

fn is_date_time(text: &str) -> bool {
    text.len() >= 16
        && text.is_char_boundary(10)
        && is_date(&text[..10])
        && matches!(text.as_bytes()[10], b'T' | b' ')
        && text[11..]
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, ':' | '.' | 'Z' | '+' | '-'))
}
