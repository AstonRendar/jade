use crate::domain::value_object::Data;

#[derive(Debug, Clone, Copy, Default)]
pub struct JsonWriter;

impl JsonWriter {
    pub fn write(&self, data: &Data) -> String {
        let mut json = String::new();
        value(&mut json, data, 0);
        json.push('\n');
        json
    }
}

fn value(json: &mut String, data: &Data, indent: usize) {
    match data {
        Data::Null => json.push_str("null"),
        Data::Boolean(flag) => json.push_str(&flag.to_string()),
        Data::Number(number) if number.is_finite() => {
            if number.fract() == 0.0 && number.abs() < 1e15 {
                json.push_str(&(*number as i64).to_string());
            } else {
                json.push_str(&number.to_string());
            }
        }
        Data::Number(_) => json.push_str("null"),
        Data::Text(text) => string(json, text),
        Data::List(items) if items.is_empty() => json.push_str("[]"),
        Data::List(items) => {
            json.push('[');
            for (index, item) in items.iter().enumerate() {
                separator(json, index, indent + 2);
                value(json, item, indent + 2);
            }
            close(json, indent, ']');
        }
        Data::Map(entries) if entries.is_empty() => json.push_str("{}"),
        Data::Map(entries) => {
            json.push('{');
            for (index, (key, item)) in entries.iter().enumerate() {
                separator(json, index, indent + 2);
                string(json, key);
                json.push_str(": ");
                value(json, item, indent + 2);
            }
            close(json, indent, '}');
        }
    }
}

fn separator(json: &mut String, index: usize, indent: usize) {
    if index > 0 {
        json.push(',');
    }
    json.push('\n');
    json.push_str(&" ".repeat(indent));
}

fn close(json: &mut String, indent: usize, bracket: char) {
    json.push('\n');
    json.push_str(&" ".repeat(indent));
    json.push(bracket);
}

fn string(json: &mut String, text: &str) {
    json.push('"');
    for character in text.chars() {
        match character {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            control if control.is_control() => {
                json.push_str(&format!("\\u{:04x}", u32::from(control)));
            }
            other => json.push(other),
        }
    }
    json.push('"');
}
