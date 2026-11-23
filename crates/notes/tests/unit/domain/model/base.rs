use jade_notes::domain::model::{Base, Filter, InvalidBaseError};
use jade_notes::domain::value_object::{Data, Expression};

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

fn expression(source: &str) -> Filter {
    Filter::Expression(Expression::parse(source).expect("valid expression"))
}

fn table_view() -> Data {
    map(vec![
        ("type", text("table")),
        ("name", text("Table")),
        (
            "filters",
            map(vec![(
                "not",
                Data::List(vec![text("file.ext == \"base\"")]),
            )]),
        ),
        ("order", Data::List(vec![text("file.name"), text("tipo")])),
        (
            "sort",
            Data::List(vec![map(vec![
                ("property", text("tipo")),
                ("direction", text("DESC")),
            ])]),
        ),
        ("limit", Data::Number(10.0)),
        (
            "columnSize",
            map(vec![
                ("file.name", Data::Number(256.0)),
                ("tipo", Data::Null),
            ]),
        ),
    ])
}

#[test]
fn reads_filters_formulas_display_names_and_views() {
    // Arrange
    let data = map(vec![
        (
            "filters",
            map(vec![(
                "and",
                Data::List(vec![
                    text("file.ext == \"md\""),
                    map(vec![("or", Data::List(vec![text("a"), text("b")]))]),
                ]),
            )]),
        ),
        ("formulas", map(vec![("twice", text("edad * 2"))])),
        (
            "properties",
            map(vec![(
                "note.tipo",
                map(vec![("displayName", text("Type"))]),
            )]),
        ),
        ("views", Data::List(vec![table_view()])),
    ]);

    // Act
    let base = Base::from_data(&data).expect("valid base");

    // Assert
    assert_eq!(
        base.filter,
        Some(Filter::All(vec![
            expression("file.ext == \"md\""),
            Filter::Any(vec![expression("a"), expression("b")]),
        ]))
    );
    assert_eq!(
        base.formulas,
        vec![(
            "twice".to_string(),
            Expression::parse("edad * 2").expect("valid")
        )]
    );
    assert_eq!(base.display_name("tipo"), Some("Type"));
    assert_eq!(base.display_name("note.tipo"), Some("Type"));
    assert_eq!(base.display_name("file.name"), None);
    let view = &base.views[0];
    assert_eq!((view.kind.as_str(), view.name.as_str()), ("table", "Table"));
    assert_eq!(
        view.filter,
        Some(Filter::None(vec![expression("file.ext == \"base\"")]))
    );
    assert_eq!(
        view.columns
            .iter()
            .map(|column| column.key.as_str())
            .collect::<Vec<_>>(),
        vec!["file.name", "tipo"]
    );
    assert_eq!(view.sort[0].column.key, "tipo");
    assert!(view.sort[0].descending);
    assert_eq!(view.limit, Some(10));
    assert_eq!(view.column_sizes, vec![("file.name".to_string(), 256)]);
}

#[test]
fn gives_an_empty_base_a_default_table_of_file_names() {
    // Arrange
    let documents = [
        Data::Null,
        map(vec![("views", Data::Null), ("filters", Data::Null)]),
    ];

    // Act
    let bases: Vec<_> = documents.iter().map(Base::from_data).collect();

    // Assert
    for base in bases {
        let base = base.expect("valid base");
        assert_eq!(base.filter, None);
        assert_eq!(base.views.len(), 1);
        assert_eq!(base.views[0].kind, "table");
        assert_eq!(base.views[0].columns[0].key, "file.name");
    }
}

#[test]
fn defaults_the_type_name_and_columns_of_a_view() {
    // Arrange
    let data = map(vec![(
        "views",
        Data::List(vec![map(vec![("limit", Data::Number(0.0))])]),
    )]);

    // Act
    let base = Base::from_data(&data).expect("valid base");

    // Assert
    let view = &base.views[0];
    assert_eq!((view.kind.as_str(), view.name.as_str()), ("table", ""));
    assert_eq!(view.columns[0].key, "file.name");
    assert_eq!(view.limit, None);
}

#[test]
fn rejects_malformed_sections_and_expressions() {
    // Arrange
    let documents = [
        text("not a map"),
        map(vec![("views", text("table"))]),
        map(vec![("filters", Data::Number(1.0))]),
        map(vec![(
            "filters",
            map(vec![("and", text("x")), ("or", text("y"))]),
        )]),
        map(vec![("filters", map(vec![("and", text("x"))]))]),
        map(vec![(
            "filters",
            map(vec![("xor", Data::List(Vec::new()))]),
        )]),
        map(vec![("filters", text("a =="))]),
        map(vec![("formulas", map(vec![("bad", text("(("))]))]),
    ];

    // Act
    let results: Vec<_> = documents.iter().map(Base::from_data).collect();

    // Assert
    assert!(results.iter().all(Result::is_err), "{results:?}");
    assert_eq!(
        results[6],
        Err(InvalidBaseError::Expression("a ==".to_string()))
    );
}
