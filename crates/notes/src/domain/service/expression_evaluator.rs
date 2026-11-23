use std::cmp::Ordering;

use crate::domain::model::FileRecord;
use crate::domain::value_object::{fold, BinaryOperator, Data, Expression};

const MAX_FORMULA_DEPTH: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    Text(String),
    Date(i64),
    List(Vec<Value<'a>>),
    Object(Vec<(String, Value<'a>)>),
    Link {
        target: String,
        label: Option<String>,
    },
    File(&'a FileRecord),
}

impl Value<'_> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Boolean(flag) => *flag,
            Self::Number(number) => *number != 0.0 && !number.is_nan(),
            Self::Text(text) => !text.is_empty(),
            Self::List(items) => !items.is_empty(),
            Self::Object(entries) => !entries.is_empty(),
            Self::Date(_) | Self::Link { .. } | Self::File(_) => true,
        }
    }

    pub fn display(&self) -> String {
        match self {
            Self::Null => String::new(),
            Self::Boolean(flag) => flag.to_string(),
            Self::Number(number) => format_number(*number),
            Self::Text(text) => text.clone(),
            Self::Date(milliseconds) => milliseconds.to_string(),
            Self::List(items) => items
                .iter()
                .map(Value::display)
                .collect::<Vec<_>>()
                .join(", "),
            Self::Object(entries) => entries
                .iter()
                .map(|(key, value)| format!("{key}: {}", value.display()))
                .collect::<Vec<_>>()
                .join(", "),
            Self::Link { target, label } => label.clone().unwrap_or_else(|| target.clone()),
            Self::File(record) => record.basename().to_string(),
        }
    }
}

pub struct Scope<'a> {
    pub file: &'a FileRecord,
    pub this: &'a FileRecord,
    pub formulas: &'a [(String, Expression)],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    pub fn evaluate<'a>(&self, expression: &Expression, scope: &Scope<'a>) -> Value<'a> {
        Evaluation {
            scope,
            locals: Vec::new(),
            depth: 0,
        }
        .evaluate(expression)
    }
}

pub fn data_to_value<'a>(data: &Data) -> Value<'a> {
    match data {
        Data::Null => Value::Null,
        Data::Boolean(flag) => Value::Boolean(*flag),
        Data::Number(number) => Value::Number(*number),
        Data::Text(text) => link_in(text).unwrap_or_else(|| Value::Text(text.clone())),
        Data::List(items) => Value::List(items.iter().map(data_to_value).collect()),
        Data::Map(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), data_to_value(value)))
                .collect(),
        ),
    }
}

fn link_in<'a>(text: &str) -> Option<Value<'a>> {
    let inner = text.trim().strip_prefix("[[")?.strip_suffix("]]")?;
    let (reference, label) = match inner.split_once('|') {
        Some((reference, label)) => (reference, Some(label.trim().to_string())),
        None => (inner, None),
    };
    Some(Value::Link {
        target: reference.trim().to_string(),
        label,
    })
}

struct Evaluation<'s, 'a> {
    scope: &'s Scope<'a>,
    locals: Vec<(String, Value<'a>)>,
    depth: usize,
}

impl<'a> Evaluation<'_, 'a> {
    fn evaluate(&mut self, expression: &Expression) -> Value<'a> {
        match expression {
            Expression::Null => Value::Null,
            Expression::Boolean(flag) => Value::Boolean(*flag),
            Expression::Number(number) => Value::Number(*number),
            Expression::Text(text) => Value::Text(text.clone()),
            Expression::List(items) => {
                Value::List(items.iter().map(|item| self.evaluate(item)).collect())
            }
            Expression::Identifier(name) => self.identifier(name),
            Expression::Member(object, name) => self.member(object, name),
            Expression::Index(object, index) => {
                let object = self.evaluate(object);
                let index = self.evaluate(index);
                index_into(object, &index)
            }
            Expression::Call(callee, arguments) => self.call(callee, arguments),
            Expression::Not(inner) => Value::Boolean(!self.evaluate(inner).is_truthy()),
            Expression::Negate(inner) => match self.evaluate(inner) {
                Value::Number(number) => Value::Number(-number),
                _ => Value::Null,
            },
            Expression::Binary(operator, left, right) => self.binary(*operator, left, right),
        }
    }

    fn identifier(&self, name: &str) -> Value<'a> {
        if let Some((_, value)) = self.locals.iter().rev().find(|(local, _)| local == name) {
            return value.clone();
        }
        match name {
            "file" => Value::File(self.scope.file),
            "this" => Value::File(self.scope.this),
            "note" => properties_of(self.scope.file),
            _ => self
                .scope
                .file
                .property(name)
                .map_or(Value::Null, data_to_value),
        }
    }

    fn member(&mut self, object: &Expression, name: &str) -> Value<'a> {
        if let Expression::Identifier(root) = object {
            match root.as_str() {
                "formula" => return self.formula(name),
                "note" => {
                    return self
                        .scope
                        .file
                        .property(name)
                        .map_or(Value::Null, data_to_value)
                }
                _ => {}
            }
        }
        let object = self.evaluate(object);
        field(&object, name)
    }

    fn formula(&mut self, name: &str) -> Value<'a> {
        let Some((_, expression)) = self.scope.formulas.iter().find(|(key, _)| key == name) else {
            return Value::Null;
        };
        if self.depth >= MAX_FORMULA_DEPTH {
            return Value::Null;
        }
        self.depth += 1;
        let value = self.evaluate(expression);
        self.depth -= 1;
        value
    }

    fn call(&mut self, callee: &Expression, arguments: &[Expression]) -> Value<'a> {
        match callee {
            Expression::Identifier(name) => self.global(name, arguments),
            Expression::Member(object, method) => {
                let receiver = self.evaluate(object);
                self.method(receiver, method, arguments)
            }
            _ => Value::Null,
        }
    }

    fn arguments(&mut self, arguments: &[Expression]) -> Vec<Value<'a>> {
        arguments
            .iter()
            .map(|argument| self.evaluate(argument))
            .collect()
    }

    fn global(&mut self, name: &str, arguments: &[Expression]) -> Value<'a> {
        if name == "if" {
            let condition = arguments.first().map(|argument| self.evaluate(argument));
            let branch = if condition.is_some_and(|value| value.is_truthy()) {
                arguments.get(1)
            } else {
                arguments.get(2)
            };
            return branch.map_or(Value::Null, |branch| self.evaluate(branch));
        }
        let values = self.arguments(arguments);
        match name {
            "list" => match values.into_iter().next() {
                Some(Value::List(items)) => Value::List(items),
                Some(Value::Null) | None => Value::List(Vec::new()),
                Some(value) => Value::List(vec![value]),
            },
            "number" => values.first().map_or(Value::Null, to_number),
            "max" | "min" => extreme(values, name == "max"),
            "link" => match values.as_slice() {
                [target, rest @ ..] => Value::Link {
                    target: link_target(target),
                    label: rest.first().map(Value::display),
                },
                [] => Value::Null,
            },
            "date" => values
                .first()
                .and_then(|value| parse_date(&value.display()))
                .map_or(Value::Null, Value::Date),
            _ => Value::Null,
        }
    }

    fn method(&mut self, receiver: Value<'a>, name: &str, arguments: &[Expression]) -> Value<'a> {
        match name {
            "isTruthy" => return Value::Boolean(receiver.is_truthy()),
            "toString" => return Value::Text(receiver.display()),
            "isEmpty" => return Value::Boolean(is_empty(&receiver)),
            _ => {}
        }
        if let Value::List(items) = &receiver {
            if matches!(name, "filter" | "map") {
                return self.iterate(items, name, arguments);
            }
        }
        let values = self.arguments(arguments);
        match receiver {
            Value::File(record) => file_method(record, name, &values),
            Value::Text(text) => text_method(&text, name, &values),
            Value::Link { target, .. } => match name {
                "linksTo" => Value::Boolean(
                    values
                        .first()
                        .is_some_and(|value| same_target(&target, &link_target(value))),
                ),
                _ => text_method(&target, name, &values),
            },
            Value::Number(number) => number_method(number, name, &values),
            Value::List(items) => list_method(items, name, &values),
            Value::Object(entries) => match name {
                "keys" => Value::List(
                    entries
                        .into_iter()
                        .map(|(key, _)| Value::Text(key))
                        .collect(),
                ),
                "values" => Value::List(entries.into_iter().map(|(_, value)| value).collect()),
                _ => Value::Null,
            },
            _ => Value::Null,
        }
    }

    fn iterate(&mut self, items: &[Value<'a>], name: &str, arguments: &[Expression]) -> Value<'a> {
        let Some(body) = arguments.first() else {
            return Value::Null;
        };
        let mut results = Vec::new();
        for (index, item) in items.iter().enumerate() {
            self.locals.push(("value".to_string(), item.clone()));
            self.locals
                .push(("index".to_string(), Value::Number(index as f64)));
            let outcome = self.evaluate(body);
            self.locals.truncate(self.locals.len() - 2);
            if name == "map" {
                results.push(outcome);
            } else if outcome.is_truthy() {
                results.push(item.clone());
            }
        }
        Value::List(results)
    }

    fn binary(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Value<'a> {
        match operator {
            BinaryOperator::Or => {
                let left = self.evaluate(left);
                if left.is_truthy() {
                    left
                } else {
                    self.evaluate(right)
                }
            }
            BinaryOperator::And => {
                let left = self.evaluate(left);
                if left.is_truthy() {
                    self.evaluate(right)
                } else {
                    left
                }
            }
            _ => {
                let left = self.evaluate(left);
                let right = self.evaluate(right);
                apply(operator, &left, &right)
            }
        }
    }
}

fn apply<'a>(operator: BinaryOperator, left: &Value<'a>, right: &Value<'a>) -> Value<'a> {
    match operator {
        BinaryOperator::Equal => Value::Boolean(equals(left, right)),
        BinaryOperator::NotEqual => Value::Boolean(!equals(left, right)),
        BinaryOperator::Less => ordered(left, right, Ordering::is_lt),
        BinaryOperator::LessOrEqual => ordered(left, right, Ordering::is_le),
        BinaryOperator::Greater => ordered(left, right, Ordering::is_gt),
        BinaryOperator::GreaterOrEqual => ordered(left, right, Ordering::is_ge),
        BinaryOperator::Add => add(left, right),
        _ => arithmetic(operator, left, right),
    }
}

fn add<'a>(left: &Value<'a>, right: &Value<'a>) -> Value<'a> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
        (Value::List(a), Value::List(b)) => Value::List(a.iter().chain(b).cloned().collect()),
        (Value::Text(_), _) | (_, Value::Text(_)) => {
            Value::Text(format!("{}{}", left.display(), right.display()))
        }
        _ => Value::Null,
    }
}

fn arithmetic<'a>(operator: BinaryOperator, left: &Value<'a>, right: &Value<'a>) -> Value<'a> {
    let (Value::Number(a), Value::Number(b)) = (left, right) else {
        return Value::Null;
    };
    Value::Number(match operator {
        BinaryOperator::Subtract => a - b,
        BinaryOperator::Multiply => a * b,
        BinaryOperator::Divide => a / b,
        _ => a % b,
    })
}

fn ordered<'a>(left: &Value<'a>, right: &Value<'a>, accepts: fn(Ordering) -> bool) -> Value<'a> {
    compare(left, right).map_or(Value::Boolean(false), |ordering| {
        Value::Boolean(accepts(ordering))
    })
}

pub fn compare(left: &Value<'_>, right: &Value<'_>) -> Option<Ordering> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
        (Value::Date(a), Value::Date(b)) => Some(a.cmp(b)),
        (Value::Boolean(a), Value::Boolean(b)) => Some(a.cmp(b)),
        (Value::Null, Value::Null) => Some(Ordering::Equal),
        (Value::Null, _) | (_, Value::Null) => None,
        _ => Some(fold(&left.display()).cmp(&fold(&right.display()))),
    }
}

fn equals(left: &Value<'_>, right: &Value<'_>) -> bool {
    match (left, right) {
        (Value::Null, Value::Null) => true,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Date(a), Value::Date(b)) => a == b,
        (Value::Text(a), Value::Text(b)) => a == b,
        (Value::List(a), Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| equals(x, y))
        }
        (Value::File(a), Value::File(b)) => a.path() == b.path(),
        (Value::Link { target, .. }, other) | (other, Value::Link { target, .. }) => {
            !matches!(other, Value::Null) && same_target(target, &link_target(other))
        }
        (Value::File(record), Value::Text(text)) | (Value::Text(text), Value::File(record)) => {
            same_target(record.path().as_str(), text)
        }
        _ => false,
    }
}

fn link_target(value: &Value<'_>) -> String {
    match value {
        Value::File(record) => record.path().as_str().to_string(),
        Value::Link { target, .. } => target.clone(),
        other => other.display(),
    }
}

fn same_target(a: &str, b: &str) -> bool {
    let key = |raw: &str| {
        let folded = fold(raw.trim());
        folded
            .strip_suffix(".md")
            .map_or(folded.clone(), str::to_string)
    };
    let (a, b) = (key(a), key(b));
    a == b || a.ends_with(&format!("/{b}")) || b.ends_with(&format!("/{a}"))
}

fn field<'a>(object: &Value<'a>, name: &str) -> Value<'a> {
    match object {
        Value::File(record) => file_field(record, name),
        Value::Object(entries) => entries
            .iter()
            .find(|(key, _)| key == name)
            .map_or(Value::Null, |(_, value)| value.clone()),
        Value::Text(text) if name == "length" => Value::Number(text.chars().count() as f64),
        Value::List(items) if name == "length" => Value::Number(items.len() as f64),
        Value::Date(milliseconds) => date_field(*milliseconds, name),
        _ => Value::Null,
    }
}

fn file_field<'a>(record: &'a FileRecord, name: &str) -> Value<'a> {
    let metadata = record.metadata();
    match name {
        "name" => Value::Text(record.name().to_string()),
        "basename" => Value::Text(record.basename().to_string()),
        "path" => Value::Text(record.path().as_str().to_string()),
        "folder" => Value::Text(record.folder()),
        "ext" => Value::Text(record.extension().to_string()),
        "size" => Value::Number(metadata.size as f64),
        "ctime" => Value::Date(metadata.created_ms),
        "mtime" => Value::Date(metadata.modified_ms),
        "properties" => properties_of(record),
        "tags" => Value::List(
            record
                .tags()
                .iter()
                .map(|tag| Value::Text(tag.clone()))
                .collect(),
        ),
        "links" => Value::List(
            record
                .links()
                .iter()
                .map(|target| Value::Link {
                    target: target.clone(),
                    label: None,
                })
                .collect(),
        ),
        "file" => Value::File(record),
        _ => Value::Null,
    }
}

fn properties_of<'a>(record: &FileRecord) -> Value<'a> {
    Value::Object(
        record
            .properties()
            .iter()
            .map(|(key, value)| (key.clone(), data_to_value(value)))
            .collect(),
    )
}

fn file_method<'a>(record: &'a FileRecord, name: &str, arguments: &[Value<'a>]) -> Value<'a> {
    let texts: Vec<String> = arguments.iter().map(Value::display).collect();
    Value::Boolean(match name {
        "inFolder" => texts
            .first()
            .is_some_and(|folder| in_folder(record, folder)),
        "hasTag" => texts.iter().any(|wanted| has_tag(record, wanted)),
        "hasProperty" => texts
            .first()
            .is_some_and(|property| record.property(property).is_some()),
        "hasLink" => arguments.first().is_some_and(|target| {
            let target = link_target(target);
            record.links().iter().any(|link| same_target(link, &target))
        }),
        "asLink" => {
            return Value::Link {
                target: record.path().as_str().to_string(),
                label: texts.first().cloned(),
            }
        }
        _ => return Value::Null,
    })
}

fn in_folder(record: &FileRecord, folder: &str) -> bool {
    let folder = folder.trim_matches('/');
    folder.is_empty() || {
        let parent = record.folder();
        parent == folder || parent.starts_with(&format!("{folder}/"))
    }
}

fn has_tag(record: &FileRecord, wanted: &str) -> bool {
    let wanted = fold(wanted.trim_start_matches('#'));
    record.tags().iter().any(|tag| {
        let tag = fold(tag);
        tag == wanted || tag.starts_with(&format!("{wanted}/"))
    })
}

fn text_method<'a>(text: &str, name: &str, arguments: &[Value<'a>]) -> Value<'a> {
    let texts: Vec<String> = arguments.iter().map(Value::display).collect();
    let first = texts.first().map_or("", String::as_str);
    match name {
        "contains" => Value::Boolean(text.contains(first)),
        "containsAll" => Value::Boolean(texts.iter().all(|part| text.contains(part.as_str()))),
        "containsAny" => Value::Boolean(texts.iter().any(|part| text.contains(part.as_str()))),
        "startsWith" => Value::Boolean(text.starts_with(first)),
        "endsWith" => Value::Boolean(text.ends_with(first)),
        "lower" => Value::Text(text.to_lowercase()),
        "upper" => Value::Text(text.to_uppercase()),
        "trim" => Value::Text(text.trim().to_string()),
        "title" => Value::Text(title_case(text)),
        "reverse" => Value::Text(text.chars().rev().collect()),
        "repeat" => Value::Text(text.repeat(count(arguments.first()))),
        "replace" => Value::Text(text.replace(first, texts.get(1).map_or("", String::as_str))),
        "split" => Value::List(
            text.split(first)
                .map(|part| Value::Text(part.to_string()))
                .collect(),
        ),
        "slice" => {
            let characters: Vec<char> = text.chars().collect();
            let (start, end) = bounds(arguments, characters.len());
            Value::Text(characters[start..end].iter().collect())
        }
        _ => Value::Null,
    }
}

fn number_method<'a>(number: f64, name: &str, arguments: &[Value<'a>]) -> Value<'a> {
    let digits = count(arguments.first()) as i32;
    match name {
        "abs" => Value::Number(number.abs()),
        "ceil" => Value::Number(number.ceil()),
        "floor" => Value::Number(number.floor()),
        "round" => {
            let factor = 10f64.powi(digits);
            Value::Number((number * factor).round() / factor)
        }
        "toFixed" => Value::Text(format!("{number:.*}", digits.max(0) as usize)),
        _ => Value::Null,
    }
}

fn list_method<'a>(items: Vec<Value<'a>>, name: &str, arguments: &[Value<'a>]) -> Value<'a> {
    match name {
        "contains" => Value::Boolean(
            arguments
                .first()
                .is_some_and(|wanted| items.iter().any(|item| equals(item, wanted))),
        ),
        "containsAll" => Value::Boolean(
            arguments
                .iter()
                .all(|wanted| items.iter().any(|item| equals(item, wanted))),
        ),
        "containsAny" => Value::Boolean(
            arguments
                .iter()
                .any(|wanted| items.iter().any(|item| equals(item, wanted))),
        ),
        "join" => Value::Text(
            items.iter().map(Value::display).collect::<Vec<_>>().join(
                &arguments
                    .first()
                    .map_or_else(|| ",".to_string(), Value::display),
            ),
        ),
        "reverse" => Value::List(items.into_iter().rev().collect()),
        "flat" => Value::List(
            items
                .into_iter()
                .flat_map(|item| match item {
                    Value::List(inner) => inner,
                    other => vec![other],
                })
                .collect(),
        ),
        "unique" => {
            let mut unique: Vec<Value<'a>> = Vec::new();
            for item in items {
                if !unique.iter().any(|seen| equals(seen, &item)) {
                    unique.push(item);
                }
            }
            Value::List(unique)
        }
        "sort" => {
            let mut sorted = items;
            sorted.sort_by(|a, b| compare(a, b).unwrap_or(Ordering::Equal));
            Value::List(sorted)
        }
        "slice" => {
            let (start, end) = bounds(arguments, items.len());
            Value::List(items[start..end].to_vec())
        }
        _ => Value::Null,
    }
}

fn index_into<'a>(object: Value<'a>, index: &Value<'a>) -> Value<'a> {
    match (object, index) {
        (Value::List(items), Value::Number(position)) if *position >= 0.0 => items
            .into_iter()
            .nth(*position as usize)
            .unwrap_or(Value::Null),
        (object, Value::Text(key)) => field(&object, key),
        _ => Value::Null,
    }
}

fn is_empty(value: &Value<'_>) -> bool {
    match value {
        Value::Null => true,
        Value::Text(text) => text.is_empty(),
        Value::List(items) => items.is_empty(),
        Value::Object(entries) => entries.is_empty(),
        _ => false,
    }
}

fn to_number<'a>(value: &Value<'a>) -> Value<'a> {
    match value {
        Value::Number(number) => Value::Number(*number),
        Value::Boolean(flag) => Value::Number(f64::from(u8::from(*flag))),
        Value::Date(milliseconds) => Value::Number(*milliseconds as f64),
        Value::Text(text) => text.trim().parse().map_or(Value::Null, Value::Number),
        _ => Value::Null,
    }
}

fn extreme(values: Vec<Value<'_>>, maximum: bool) -> Value<'_> {
    values
        .into_iter()
        .filter(|value| !matches!(value, Value::Null))
        .reduce(|best, candidate| {
            let ordering = compare(&candidate, &best).unwrap_or(Ordering::Equal);
            if (maximum && ordering.is_gt()) || (!maximum && ordering.is_lt()) {
                candidate
            } else {
                best
            }
        })
        .unwrap_or(Value::Null)
}

fn count(value: Option<&Value<'_>>) -> usize {
    match value {
        Some(Value::Number(number)) if *number > 0.0 => *number as usize,
        _ => 0,
    }
}

fn bounds(arguments: &[Value<'_>], length: usize) -> (usize, usize) {
    let position = |value: Option<&Value<'_>>, default: usize| match value {
        Some(Value::Number(number)) if *number < 0.0 => {
            length.saturating_sub(number.abs() as usize)
        }
        Some(Value::Number(number)) => (*number as usize).min(length),
        _ => default,
    };
    let start = position(arguments.first(), 0);
    let end = position(arguments.get(1), length);
    (start, end.max(start))
}

fn title_case(text: &str) -> String {
    text.split(' ')
        .map(|word| {
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(characters).collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_number(number: f64) -> String {
    if number.fract() == 0.0 && number.abs() < 1e15 {
        format!("{}", number as i64)
    } else {
        number.to_string()
    }
}

fn date_field<'a>(milliseconds: i64, name: &str) -> Value<'a> {
    let days = milliseconds.div_euclid(86_400_000);
    let of_day = milliseconds.rem_euclid(86_400_000);
    let (year, month, day) = civil_from_days(days);
    let number = |value: i64| Value::Number(value as f64);
    match name {
        "year" => number(year),
        "month" => number(month),
        "day" => number(day),
        "hour" => number(of_day / 3_600_000),
        "minute" => number(of_day / 60_000 % 60),
        "second" => number(of_day / 1000 % 60),
        "millisecond" => number(of_day % 1000),
        _ => Value::Null,
    }
}

pub fn parse_date(text: &str) -> Option<i64> {
    let text = text.trim();
    let (date, time) = text
        .split_once(['T', ' '])
        .map_or((text, ""), |(date, time)| (date, time));
    let mut parts = date.split('-').map(str::parse::<i64>);
    let (year, month, day) = (
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
    );
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let mut clock = time
        .trim_end_matches('Z')
        .split(':')
        .filter(|part| !part.is_empty())
        .map(|part| part.split('.').next().unwrap_or("0").parse::<i64>());
    let hour = clock.next().transpose().ok()?.unwrap_or(0);
    let minute = clock.next().transpose().ok()?.unwrap_or(0);
    let second = clock.next().transpose().ok()?.unwrap_or(0);
    let days = days_from_civil(year, month, day);
    Some(((days * 24 + hour) * 60 + minute) * 60_000 + second * 1000)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}
