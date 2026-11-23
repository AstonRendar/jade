use std::cmp::Ordering;

use super::expression_evaluator::{compare, ExpressionEvaluator, Scope, Value};
use crate::domain::model::{
    Base, BaseTable, BaseView, Cell, FileRecord, Filter, TableColumn, TableRow,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct BaseQuery;

impl BaseQuery {
    pub fn run(
        &self,
        base: &Base,
        view: &BaseView,
        records: &[FileRecord],
        this: &FileRecord,
    ) -> BaseTable {
        let mut rows: Vec<(Vec<Value<'_>>, TableRow)> = records
            .iter()
            .filter_map(|record| {
                let scope = Scope {
                    file: record,
                    this,
                    formulas: &base.formulas,
                };
                let included = [&base.filter, &view.filter]
                    .into_iter()
                    .flatten()
                    .all(|filter| matches(filter, &scope));
                included.then(|| row(view, &scope))
            })
            .collect();
        rows.sort_by(|(a, _), (b, _)| by_sort_keys(view, a, b));
        let total = rows.len();
        rows.truncate(view.limit.unwrap_or(total));
        BaseTable {
            columns: view
                .columns
                .iter()
                .map(|column| TableColumn {
                    key: column.key.clone(),
                    display_name: base.display_name(&column.key).map(str::to_string),
                    width: view
                        .column_sizes
                        .iter()
                        .find(|(key, _)| *key == column.key)
                        .map(|(_, width)| *width),
                })
                .collect(),
            rows: rows.into_iter().map(|(_, row)| row).collect(),
            total,
        }
    }
}

fn row<'a>(view: &BaseView, scope: &Scope<'a>) -> (Vec<Value<'a>>, TableRow) {
    let sort_values = view
        .sort
        .iter()
        .map(|key| ExpressionEvaluator.evaluate(&key.column.expression, scope))
        .collect();
    let cells = view
        .columns
        .iter()
        .map(|column| cell(ExpressionEvaluator.evaluate(&column.expression, scope)))
        .collect();
    (
        sort_values,
        TableRow {
            path: scope.file.path().clone(),
            cells,
        },
    )
}

fn by_sort_keys(view: &BaseView, a: &[Value<'_>], b: &[Value<'_>]) -> Ordering {
    view.sort
        .iter()
        .zip(a.iter().zip(b))
        .map(|(key, (a, b))| {
            let ordering = match (a, b) {
                (Value::Null, Value::Null) => Ordering::Equal,
                (Value::Null, _) => return Ordering::Greater,
                (_, Value::Null) => return Ordering::Less,
                _ => compare(a, b).unwrap_or(Ordering::Equal),
            };
            if key.descending {
                ordering.reverse()
            } else {
                ordering
            }
        })
        .find(|ordering| ordering.is_ne())
        .unwrap_or(Ordering::Equal)
}

fn matches(filter: &Filter, scope: &Scope<'_>) -> bool {
    match filter {
        Filter::All(filters) => filters.iter().all(|filter| matches(filter, scope)),
        Filter::Any(filters) => filters.iter().any(|filter| matches(filter, scope)),
        Filter::None(filters) => !filters.iter().any(|filter| matches(filter, scope)),
        Filter::Expression(expression) => {
            ExpressionEvaluator.evaluate(expression, scope).is_truthy()
        }
    }
}

fn cell(value: Value<'_>) -> Cell {
    match value {
        Value::Null => Cell::Null,
        Value::Boolean(flag) => Cell::Boolean(flag),
        Value::Number(number) => Cell::Number(number),
        Value::Text(text) => Cell::Text(text),
        Value::Date(milliseconds) => Cell::Date(milliseconds),
        Value::List(items) => Cell::List(items.into_iter().map(cell).collect()),
        Value::Link { target, label } => Cell::Link { target, label },
        Value::File(record) => Cell::File(record.path().clone()),
        object @ Value::Object(_) => Cell::Text(object.display()),
    }
}
