use jade_notes::domain::service::JsonWriter;
use jade_notes::domain::value_object::Data;

#[test]
fn writes_indented_json_like_obsidian() {
    // Arrange
    let data = Data::Map(vec![
        (
            "types".to_string(),
            Data::Map(vec![
                ("tipo".to_string(), Data::Text("text".to_string())),
                ("año".to_string(), Data::Text("number".to_string())),
            ]),
        ),
        ("empty".to_string(), Data::Map(Vec::new())),
    ]);

    // Act
    let json = JsonWriter.write(&data);

    // Assert
    assert_eq!(
        json,
        "{\n  \"types\": {\n    \"tipo\": \"text\",\n    \"año\": \"number\"\n  },\n  \"empty\": {}\n}\n"
    );
}

#[test]
fn writes_every_value_type_and_escapes_text() {
    // Arrange
    let data = Data::List(vec![
        Data::Null,
        Data::Boolean(true),
        Data::Number(3.0),
        Data::Number(2.5),
        Data::Number(f64::NAN),
        Data::Text("quote \" slash \\ line\n\r\ttab \u{1}".to_string()),
        Data::List(Vec::new()),
    ]);

    // Act
    let json = JsonWriter.write(&data);

    // Assert
    assert_eq!(
        json,
        "[\n  null,\n  true,\n  3,\n  2.5,\n  null,\n  \"quote \\\" slash \\\\ line\\n\\r\\ttab \\u0001\",\n  []\n]\n"
    );
}
