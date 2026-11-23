#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Null,
    Boolean(bool),
    Number(f64),
    Text(String),
    List(Vec<Data>),
    Map(Vec<(String, Data)>),
}

impl Data {
    pub fn get(&self, key: &str) -> Option<&Data> {
        match self {
            Self::Map(entries) => entries
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Data]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&[(String, Data)]> {
        match self {
            Self::Map(entries) => Some(entries),
            _ => None,
        }
    }
}
