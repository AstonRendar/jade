use jade_notes::domain::value_object::{Data, PropertyType};

fn text(value: &str) -> Data {
    Data::Text(value.to_string())
}

#[test]
fn reads_the_type_names_obsidian_stores() {
    // Arrange
    let names = [
        "text",
        "multitext",
        "number",
        "checkbox",
        "date",
        "datetime",
        "tags",
        "aliases",
        "select",
    ];

    // Act
    let kinds: Vec<_> = names
        .iter()
        .map(|name| PropertyType::from_name(name))
        .collect();

    // Assert
    assert_eq!(
        kinds,
        vec![
            Some(PropertyType::Text),
            Some(PropertyType::List),
            Some(PropertyType::Number),
            Some(PropertyType::Checkbox),
            Some(PropertyType::Date),
            Some(PropertyType::DateTime),
            Some(PropertyType::Tags),
            Some(PropertyType::Aliases),
            None,
        ]
    );
}

#[test]
fn infers_the_type_from_the_name_and_the_value() {
    // Arrange
    let samples = [
        ("tags", text("a")),
        ("aliases", Data::List(Vec::new())),
        ("done", Data::Boolean(true)),
        ("year", Data::Number(1024.0)),
        ("people", Data::List(vec![text("Ana")])),
        ("due", text("2026-11-23")),
        ("at", text("2026-11-23T10:30:00")),
        ("at2", text("2026-11-23 10:30")),
        ("odd", text("2026-11-23Tnope!")),
        ("year2", text("1024")),
        ("empty", Data::Null),
        ("nested", Data::Map(Vec::new())),
    ];

    // Act
    let kinds: Vec<_> = samples
        .iter()
        .map(|(key, value)| PropertyType::infer(key, value))
        .collect();

    // Assert
    assert_eq!(
        kinds,
        vec![
            Some(PropertyType::Tags),
            Some(PropertyType::Aliases),
            Some(PropertyType::Checkbox),
            Some(PropertyType::Number),
            Some(PropertyType::List),
            Some(PropertyType::Date),
            Some(PropertyType::DateTime),
            Some(PropertyType::DateTime),
            Some(PropertyType::Text),
            Some(PropertyType::Text),
            None,
            None,
        ]
    );
}

#[test]
fn names_each_type_as_obsidian_stores_it_and_keeps_tags_and_aliases_fixed() {
    // Arrange
    let names = [
        "text",
        "multitext",
        "number",
        "checkbox",
        "date",
        "datetime",
        "tags",
        "aliases",
    ];

    // Act
    let kinds: Vec<PropertyType> = names
        .iter()
        .map(|name| PropertyType::from_name(name).expect("known type"))
        .collect();

    // Assert
    assert_eq!(
        kinds.iter().map(|kind| kind.name()).collect::<Vec<_>>(),
        names
    );
    assert_eq!(kinds.iter().filter(|kind| kind.is_assignable()).count(), 6);
    assert!(!PropertyType::Tags.is_assignable() && !PropertyType::Aliases.is_assignable());
}
