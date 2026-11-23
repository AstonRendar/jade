use jade_notes::domain::model::{Base, BaseTable, Cell, FileRecord, TableColumn};
use jade_notes::domain::service::BaseQuery;
use jade_notes::domain::value_object::Data;

use crate::support::mothers::FileRecordMother;

fn text(value: &str) -> Data {
    Data::Text(value.to_string())
}

fn map(entries: Vec<(&str, Data)>) -> Data {
    Data::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn records() -> Vec<FileRecord> {
    vec![
        FileRecordMother::note(
            "Lore/Tower.md",
            vec![("tipo", text("lore")), ("orden", Data::Number(2.0))],
            &[],
            &[],
        ),
        FileRecordMother::note(
            "Characters/Ana.md",
            vec![
                ("tipo", text("personaje")),
                ("orden", Data::Number(1.0)),
                (
                    "aliases",
                    Data::List(vec![text("A"), Data::Number(3.0), Data::Boolean(true)]),
                ),
                ("mentor", text("[[Bruno]]")),
                ("datos", map(vec![("n", Data::Number(1.0))])),
            ],
            &[],
            &[],
        ),
        FileRecordMother::note(
            "Characters/Bruno.md",
            vec![("tipo", text("personaje"))],
            &[],
            &[],
        ),
        FileRecordMother::note(
            "_Workshop/Draft.md",
            vec![("tipo", text("taller"))],
            &[],
            &[],
        ),
        FileRecordMother::file("Index.base"),
    ]
}

fn run(base: Data) -> BaseTable {
    let base = Base::from_data(&base).expect("valid base");
    let records = records();
    let this = records[4].clone();
    BaseQuery.run(&base, &base.views[0], &records, &this)
}

fn paths(table: &BaseTable) -> Vec<&str> {
    table.rows.iter().map(|row| row.path.as_str()).collect()
}

#[test]
fn keeps_the_files_that_match_the_global_and_view_filters() {
    // Arrange
    let base = map(vec![
        (
            "filters",
            map(vec![(
                "and",
                Data::List(vec![
                    text("file.ext == \"md\""),
                    text("!file.inFolder(\"_Workshop\")"),
                ]),
            )]),
        ),
        (
            "views",
            Data::List(vec![map(vec![(
                "filters",
                map(vec![("not", Data::List(vec![text("tipo == \"lore\"")]))]),
            )])]),
        ),
    ]);

    // Act
    let table = run(base);

    // Assert
    assert_eq!(
        paths(&table),
        vec!["Characters/Ana.md", "Characters/Bruno.md"]
    );
    assert_eq!(table.total, 2);
}

#[test]
fn sorts_by_the_sort_keys_with_empty_values_last_and_applies_the_limit() {
    // Arrange
    let base = map(vec![(
        "views",
        Data::List(vec![map(vec![
            (
                "sort",
                Data::List(vec![
                    map(vec![("property", text("tipo")), ("direction", text("ASC"))]),
                    map(vec![
                        ("property", text("orden")),
                        ("direction", text("DESC")),
                    ]),
                ]),
            ),
            ("limit", Data::Number(4.0)),
        ])]),
    )]);

    // Act
    let table = run(base);

    // Assert
    assert_eq!(
        paths(&table),
        vec![
            "Lore/Tower.md",
            "Characters/Ana.md",
            "Characters/Bruno.md",
            "_Workshop/Draft.md"
        ]
    );
    assert_eq!(table.total, 5);
}

#[test]
fn puts_files_without_the_sort_value_last_in_both_directions() {
    // Arrange
    let sorted_by = |direction: &str| {
        map(vec![(
            "views",
            Data::List(vec![map(vec![(
                "sort",
                Data::List(vec![map(vec![
                    ("property", text("orden")),
                    ("direction", text(direction)),
                ])]),
            )])]),
        )])
    };

    // Act
    let ascending = run(sorted_by("ASC"));
    let descending = run(sorted_by("DESC"));

    // Assert
    assert_eq!(
        &paths(&ascending)[..2],
        ["Characters/Ana.md", "Lore/Tower.md"]
    );
    assert_eq!(
        &paths(&descending)[..2],
        ["Lore/Tower.md", "Characters/Ana.md"]
    );
    assert_eq!(paths(&ascending)[2..], paths(&descending)[2..]);
}

#[test]
fn builds_columns_with_display_names_widths_and_typed_cells() {
    // Arrange
    let base = map(vec![
        ("filters", text("file.name == \"Ana.md\"")),
        (
            "properties",
            map(vec![("tipo", map(vec![("displayName", text("Type"))]))]),
        ),
        (
            "views",
            Data::List(vec![map(vec![
                (
                    "order",
                    Data::List(vec![
                        text("file.name"),
                        text("tipo"),
                        text("aliases"),
                        text("mentor"),
                        text("orden"),
                        text("file.mtime"),
                        text("file"),
                        text("datos"),
                        text("missing"),
                        text("orden > 0"),
                    ]),
                ),
                ("columnSize", map(vec![("tipo", Data::Number(120.0))])),
            ])]),
        ),
    ]);

    // Act
    let table = run(base);

    // Assert
    assert_eq!(
        table.columns[..2],
        [
            TableColumn {
                key: "file.name".to_string(),
                display_name: None,
                width: None
            },
            TableColumn {
                key: "tipo".to_string(),
                display_name: Some("Type".to_string()),
                width: Some(120)
            },
        ]
    );
    let cells = &table.rows[0].cells;
    assert_eq!(cells[0], Cell::Text("Ana.md".to_string()));
    assert_eq!(cells[1], Cell::Text("personaje".to_string()));
    assert_eq!(
        cells[2],
        Cell::List(vec![
            Cell::Text("A".to_string()),
            Cell::Number(3.0),
            Cell::Boolean(true)
        ])
    );
    assert_eq!(
        cells[3],
        Cell::Link {
            target: "Bruno".to_string(),
            label: None
        }
    );
    assert_eq!(cells[4], Cell::Number(1.0));
    assert!(matches!(cells[5], Cell::Date(_)));
    assert!(matches!(&cells[6], Cell::File(path) if path.as_str() == "Characters/Ana.md"));
    assert_eq!(cells[7], Cell::Text("n: 1".to_string()));
    assert_eq!(cells[8], Cell::Null);
    assert_eq!(cells[9], Cell::Boolean(true));
}
