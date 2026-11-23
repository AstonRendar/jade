use yaml_rust2::{Yaml, YamlLoader};

use crate::domain::port::{DataFormat, DataFormatError};
use crate::domain::value_object::Data;

#[derive(Debug, Clone, Copy, Default)]
pub struct YamlDataFormat;

impl DataFormat for YamlDataFormat {
    fn parse(&self, text: &str) -> Result<Data, DataFormatError> {
        let documents =
            YamlLoader::load_from_str(text).map_err(|error| DataFormatError(error.to_string()))?;
        Ok(documents.first().map_or(Data::Null, data))
    }
}

fn data(yaml: &Yaml) -> Data {
    match yaml {
        Yaml::Real(raw) => raw
            .parse()
            .map_or_else(|_| Data::Text(raw.clone()), Data::Number),
        Yaml::Integer(number) => Data::Number(*number as f64),
        Yaml::String(text) => Data::Text(text.clone()),
        Yaml::Boolean(flag) => Data::Boolean(*flag),
        Yaml::Array(items) => Data::List(items.iter().map(data).collect()),
        Yaml::Hash(entries) => Data::Map(
            entries
                .iter()
                .map(|(key, value)| (key_text(key), data(value)))
                .collect(),
        ),
        Yaml::Alias(_) | Yaml::Null | Yaml::BadValue => Data::Null,
    }
}

fn key_text(key: &Yaml) -> String {
    match key {
        Yaml::String(text) | Yaml::Real(text) => text.clone(),
        Yaml::Integer(number) => number.to_string(),
        Yaml::Boolean(flag) => flag.to_string(),
        _ => String::new(),
    }
}
