use crate::domain::value_object::VaultPath;

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Null,
    Boolean(bool),
    Number(f64),
    Text(String),
    Date(i64),
    List(Vec<Cell>),
    Link {
        target: String,
        label: Option<String>,
    },
    File(VaultPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableColumn {
    pub key: String,
    pub display_name: Option<String>,
    pub width: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub path: VaultPath,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseTable {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub total: usize,
}
