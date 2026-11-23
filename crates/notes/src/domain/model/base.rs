use crate::domain::value_object::{Data, Expression, InvalidExpressionError};

const DEFAULT_VIEW_KIND: &str = "table";
const DEFAULT_COLUMN: &str = "file.name";

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    All(Vec<Filter>),
    Any(Vec<Filter>),
    None(Vec<Filter>),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub key: String,
    pub expression: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortKey {
    pub column: Column,
    pub descending: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseView {
    pub kind: String,
    pub name: String,
    pub filter: Option<Filter>,
    pub columns: Vec<Column>,
    pub sort: Vec<SortKey>,
    pub limit: Option<usize>,
    pub column_sizes: Vec<(String, u32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Base {
    pub filter: Option<Filter>,
    pub formulas: Vec<(String, Expression)>,
    pub display_names: Vec<(String, String)>,
    pub views: Vec<BaseView>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBaseError {
    #[error("invalid expression `{0}`")]
    Expression(String),
    #[error("invalid section `{0}`")]
    Section(String),
}

impl From<InvalidExpressionError> for InvalidBaseError {
    fn from(error: InvalidExpressionError) -> Self {
        Self::Expression(error.0)
    }
}

impl Base {
    pub fn from_data(data: &Data) -> Result<Self, InvalidBaseError> {
        if !matches!(data, Data::Map(_) | Data::Null) {
            return Err(InvalidBaseError::Section("base".to_string()));
        }
        let views = match data.get("views") {
            Some(Data::List(views)) => views.iter().map(view).collect::<Result<_, _>>()?,
            Some(Data::Null) | None => Vec::new(),
            Some(_) => return Err(InvalidBaseError::Section("views".to_string())),
        };
        Ok(Self {
            filter: present(data.get("filters")).map(filter).transpose()?,
            formulas: formulas(data.get("formulas"))?,
            display_names: display_names(data.get("properties")),
            views: if views.is_empty() {
                vec![default_view()?]
            } else {
                views
            },
        })
    }

    pub fn display_name(&self, key: &str) -> Option<&str> {
        let bare = key.strip_prefix("note.").unwrap_or(key);
        self.display_names
            .iter()
            .find(|(name, _)| name.strip_prefix("note.").unwrap_or(name) == bare)
            .map(|(_, label)| label.as_str())
    }
}

fn default_view() -> Result<BaseView, InvalidBaseError> {
    Ok(BaseView {
        kind: DEFAULT_VIEW_KIND.to_string(),
        name: String::new(),
        filter: None,
        columns: vec![column(DEFAULT_COLUMN)?],
        sort: Vec::new(),
        limit: None,
        column_sizes: Vec::new(),
    })
}

fn view(data: &Data) -> Result<BaseView, InvalidBaseError> {
    let text = |key: &str| data.get(key).and_then(Data::as_text).map(str::to_string);
    let columns = match data.get("order").and_then(Data::as_list) {
        Some(keys) if !keys.is_empty() => keys
            .iter()
            .filter_map(Data::as_text)
            .map(column)
            .collect::<Result<_, _>>()?,
        _ => vec![column(DEFAULT_COLUMN)?],
    };
    Ok(BaseView {
        kind: text("type").unwrap_or_else(|| DEFAULT_VIEW_KIND.to_string()),
        name: text("name").unwrap_or_default(),
        filter: present(data.get("filters")).map(filter).transpose()?,
        columns,
        sort: sort_keys(data.get("sort"))?,
        limit: data
            .get("limit")
            .and_then(Data::as_number)
            .filter(|limit| *limit >= 1.0)
            .map(|limit| limit as usize),
        column_sizes: data
            .get("columnSize")
            .and_then(Data::as_map)
            .unwrap_or_default()
            .iter()
            .filter_map(|(key, width)| {
                width
                    .as_number()
                    .filter(|width| *width > 0.0)
                    .map(|width| (key.clone(), width as u32))
            })
            .collect(),
    })
}

fn column(key: &str) -> Result<Column, InvalidBaseError> {
    Ok(Column {
        key: key.to_string(),
        expression: Expression::parse(key)?,
    })
}

fn sort_keys(data: Option<&Data>) -> Result<Vec<SortKey>, InvalidBaseError> {
    data.and_then(Data::as_list)
        .unwrap_or_default()
        .iter()
        .filter_map(|entry| {
            let property = entry.get("property").and_then(Data::as_text)?;
            let descending = entry
                .get("direction")
                .and_then(Data::as_text)
                .is_some_and(|direction| direction.eq_ignore_ascii_case("desc"));
            Some(column(property).map(|column| SortKey { column, descending }))
        })
        .collect()
}

fn filter(data: &Data) -> Result<Filter, InvalidBaseError> {
    match data {
        Data::Text(source) => Ok(Filter::Expression(Expression::parse(source)?)),
        Data::Map(entries) => {
            let [(operator, operands)] = entries.as_slice() else {
                return Err(InvalidBaseError::Section("filters".to_string()));
            };
            let operands = operands
                .as_list()
                .ok_or_else(|| InvalidBaseError::Section(operator.clone()))?
                .iter()
                .map(filter)
                .collect::<Result<_, _>>()?;
            match operator.as_str() {
                "and" => Ok(Filter::All(operands)),
                "or" => Ok(Filter::Any(operands)),
                "not" => Ok(Filter::None(operands)),
                other => Err(InvalidBaseError::Section(other.to_string())),
            }
        }
        _ => Err(InvalidBaseError::Section("filters".to_string())),
    }
}

fn formulas(data: Option<&Data>) -> Result<Vec<(String, Expression)>, InvalidBaseError> {
    data.and_then(Data::as_map)
        .unwrap_or_default()
        .iter()
        .map(|(name, source)| {
            let source = source.as_text().unwrap_or_default();
            Ok((name.clone(), Expression::parse(source)?))
        })
        .collect()
}

fn display_names(data: Option<&Data>) -> Vec<(String, String)> {
    data.and_then(Data::as_map)
        .unwrap_or_default()
        .iter()
        .filter_map(|(key, settings)| {
            settings
                .get("displayName")
                .and_then(Data::as_text)
                .map(|label| (key.clone(), label.to_string()))
        })
        .collect()
}

fn present(data: Option<&Data>) -> Option<&Data> {
    data.filter(|data| !matches!(data, Data::Null))
}
