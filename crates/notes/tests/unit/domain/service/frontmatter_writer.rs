use jade_notes::domain::service::FrontmatterWriter;
use jade_notes::domain::value_object::Data;

fn text(value: &str) -> Data {
    Data::Text(value.to_string())
}

fn properties(entries: Vec<(&str, Data)>) -> Vec<(String, Data)> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

#[test]
fn replaces_the_frontmatter_keeping_the_body() {
    // Arrange
    let content = "---\nold: 1\n---\n# Body\n";
    let new = properties(vec![
        ("tipo", text("personaje")),
        ("edad", Data::Number(42.0)),
        ("peso", Data::Number(70.5)),
        ("activo", Data::Boolean(false)),
        ("aliases", Data::List(vec![text("Ana"), text("La Torre")])),
        ("vacío", Data::Null),
        ("sin", Data::List(Vec::new())),
    ]);

    // Act
    let written = FrontmatterWriter.write(content, &new);

    // Assert
    assert_eq!(
        written,
        "---\ntipo: personaje\nedad: 42\npeso: 70.5\nactivo: false\naliases:\n  - Ana\n  - La Torre\nvacío:\nsin: []\n---\n# Body\n"
    );
}

#[test]
fn adds_a_frontmatter_to_a_note_without_one_and_removes_an_empty_one() {
    // Arrange
    let plain = "# Body\n";
    let with = "---\na: 1\n---\n# Body\n";

    // Act
    let added = FrontmatterWriter.write(plain, &properties(vec![("a", text("b"))]));
    let removed = FrontmatterWriter.write(with, &[]);

    // Assert
    assert_eq!(added, "---\na: b\n---\n# Body\n");
    assert_eq!(removed, "# Body\n");
}

#[test]
fn quotes_text_that_yaml_would_read_differently() {
    // Arrange
    let values = [
        "[[Bruno]]",
        "",
        " padded",
        "a: b",
        "a #b",
        "ends:",
        "true",
        "null",
        "1024",
        "#tag",
        "line\nbreak",
        "say \"hi\"",
        "C:\\path",
        "plain text",
    ];

    // Act
    let written: Vec<String> = values
        .iter()
        .map(|value| {
            FrontmatterWriter
                .write("", &properties(vec![("k", text(value))]))
                .lines()
                .nth(1)
                .unwrap_or_default()
                .to_string()
        })
        .collect();

    // Assert
    assert_eq!(
        written,
        vec![
            "k: \"[[Bruno]]\"",
            "k: \"\"",
            "k: \" padded\"",
            "k: \"a: b\"",
            "k: \"a #b\"",
            "k: \"ends:\"",
            "k: \"true\"",
            "k: \"null\"",
            "k: \"1024\"",
            "k: \"#tag\"",
            "k: \"line\\nbreak\"",
            "k: say \"hi\"",
            "k: C:\\path",
            "k: plain text",
        ]
    );
}

#[test]
fn writes_nested_maps_and_lists() {
    // Arrange
    let nested = properties(vec![
        (
            "datos",
            Data::Map(vec![
                ("nivel".to_string(), Data::Number(3.0)),
                ("vacío".to_string(), Data::Map(Vec::new())),
            ]),
        ),
        (
            "matriz",
            Data::List(vec![Data::List(vec![Data::Number(1.0)])]),
        ),
        ("dos puntos:", Data::Boolean(true)),
    ]);

    // Act
    let written = FrontmatterWriter.write("", &nested);

    // Assert
    assert_eq!(
        written,
        "---\ndatos:\n  nivel: 3\n  vacío: {}\nmatriz:\n  -\n    - 1\n\"dos puntos:\": true\n---\n"
    );
}
