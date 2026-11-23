use jade_notes::domain::model::FileRecord;
use jade_notes::domain::service::{ExpressionEvaluator, Scope, Value};
use jade_notes::domain::value_object::{Data, Expression};

use crate::support::mothers::{FileRecordMother, MODIFIED_MS};

fn note() -> FileRecord {
    FileRecordMother::note(
        "Characters/Ana Torres.md",
        vec![
            ("tipo", FileRecordMother::text("personaje")),
            ("aliases", FileRecordMother::texts(&["Ana", "La Torre"])),
            ("edad", Data::Number(42.0)),
            ("activo", Data::Boolean(true)),
            (
                "mentor",
                FileRecordMother::text("[[Characters/Bruno|Bruno]]"),
            ),
            ("fecha", FileRecordMother::text("2026-01-02")),
            ("vacío", Data::Null),
            (
                "datos",
                Data::Map(vec![("nivel".to_string(), Data::Number(3.0))]),
            ),
        ],
        &["personaje", "saga/norte"],
        &["Characters/Bruno.md", "Places/Tower.md"],
    )
}

fn base_file() -> FileRecord {
    FileRecordMother::file("Index.base")
}

fn formulas() -> Vec<(String, Expression)> {
    vec![
        ("doble".to_string(), parse("edad * 2")),
        ("cuádruple".to_string(), parse("formula.doble * 2")),
        ("bucle".to_string(), parse("formula.bucle")),
    ]
}

fn parse(source: &str) -> Expression {
    Expression::parse(source).expect("valid expression")
}

fn evaluate(source: &str) -> Value<'static> {
    let file = Box::leak(Box::new(note()));
    let this = Box::leak(Box::new(base_file()));
    let formulas = Box::leak(Box::new(formulas()));
    let scope = Scope {
        file,
        this,
        formulas,
    };
    ExpressionEvaluator.evaluate(&parse(source), &scope)
}

fn text(value: &str) -> Value<'static> {
    Value::Text(value.to_string())
}

fn truthy(source: &str) -> bool {
    evaluate(source).is_truthy()
}

#[test]
fn reads_file_fields() {
    // Arrange
    let sources = [
        "file.name",
        "file.basename",
        "file.path",
        "file.folder",
        "file.ext",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            text("Ana Torres.md"),
            text("Ana Torres"),
            text("Characters/Ana Torres.md"),
            text("Characters"),
            text("md"),
        ]
    );
}

#[test]
fn reads_file_size_dates_tags_and_links() {
    // Arrange
    let sources = ["file.size", "file.mtime", "file.tags", "file.links.length"];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(values[0], Value::Number(120.0));
    assert_eq!(values[1], Value::Date(MODIFIED_MS));
    assert_eq!(
        values[2],
        Value::List(vec![text("personaje"), text("saga/norte")])
    );
    assert_eq!(values[3], Value::Number(2.0));
}

#[test]
fn reads_note_properties_bare_or_through_note() {
    // Arrange
    let sources = [
        "tipo",
        "note.tipo",
        "note.missing",
        "missing",
        "note[\"edad\"]",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            text("personaje"),
            text("personaje"),
            Value::Null,
            Value::Null,
            Value::Number(42.0)
        ]
    );
}

#[test]
fn turns_wikilink_properties_into_links() {
    // Arrange
    let source = "mentor";

    // Act
    let value = evaluate(source);

    // Assert
    assert_eq!(
        value,
        Value::Link {
            target: "Characters/Bruno".to_string(),
            label: Some("Bruno".to_string()),
        }
    );
}

#[test]
fn evaluates_folder_tag_property_and_link_checks() {
    // Arrange
    let accepted = [
        "file.inFolder(\"Characters\")",
        "file.inFolder(\"/\")",
        "file.hasTag(\"saga\")",
        "file.hasTag(\"#personaje\", \"other\")",
        "file.hasProperty(\"edad\")",
        "file.hasLink(\"Bruno\")",
        "file.hasLink(\"places/tower.md\")",
        "mentor.linksTo(\"Characters/Bruno.md\")",
    ];
    let rejected = [
        "file.inFolder(\"Char\")",
        "file.hasTag(\"sag\")",
        "file.hasProperty(\"altura\")",
        "file.hasLink(this)",
        "!file.inFolder(\"Characters\")",
    ];

    // Act
    let accepted: Vec<_> = accepted.iter().map(|source| truthy(source)).collect();
    let rejected: Vec<_> = rejected.iter().map(|source| truthy(source)).collect();

    // Assert
    assert!(accepted.iter().all(|flag| *flag), "{accepted:?}");
    assert!(rejected.iter().all(|flag| !*flag), "{rejected:?}");
}

#[test]
fn compares_values_of_every_type() {
    // Arrange
    let accepted = [
        "edad == 42",
        "edad >= 42 && edad <= 42",
        "edad > 40 && edad < 50",
        "tipo != \"lore\"",
        "\"b\" > \"A\"",
        "activo == true",
        "vacío == null",
        "aliases == [\"Ana\", \"La Torre\"]",
        "file == this || file.ext == \"md\"",
        "this == \"Index.base\"",
        "mentor == \"Characters/Bruno.md\"",
        "file.mtime > file.ctime",
        "date(\"2026-01-02\") == date(fecha)",
    ];
    let rejected = [
        "edad < null",
        "aliases == [\"Ana\"]",
        "edad == \"42\"",
        "mentor == null",
    ];

    // Act
    let accepted: Vec<_> = accepted.iter().map(|source| truthy(source)).collect();
    let rejected: Vec<_> = rejected.iter().map(|source| truthy(source)).collect();

    // Assert
    assert!(accepted.iter().all(|flag| *flag), "{accepted:?}");
    assert!(rejected.iter().all(|flag| !*flag), "{rejected:?}");
}

#[test]
fn computes_arithmetic_and_concatenation() {
    // Arrange
    let sources = [
        "edad + 1",
        "edad - 2",
        "edad / 2",
        "edad % 5",
        "-edad",
        "tipo + \" \" + edad",
        "[1] + [2]",
        "tipo - 1",
        "-tipo",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            Value::Number(43.0),
            Value::Number(40.0),
            Value::Number(21.0),
            Value::Number(2.0),
            Value::Number(-42.0),
            text("personaje 42"),
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Null,
            Value::Null,
        ]
    );
}

#[test]
fn short_circuits_boolean_operators_returning_operands() {
    // Arrange
    let sources = [
        "missing || tipo",
        "tipo || missing",
        "missing && tipo",
        "tipo && edad",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            text("personaje"),
            text("personaje"),
            Value::Null,
            Value::Number(42.0)
        ]
    );
}

#[test]
fn evaluates_formulas_and_stops_recursive_ones() {
    // Arrange
    let sources = [
        "formula.doble",
        "formula.cuádruple",
        "formula.bucle",
        "formula.none",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            Value::Number(84.0),
            Value::Number(168.0),
            Value::Null,
            Value::Null
        ]
    );
}

#[test]
fn applies_text_methods() {
    // Arrange
    let sources = [
        "tipo.contains(\"son\")",
        "tipo.containsAll(\"per\", \"aje\")",
        "tipo.containsAny(\"x\", \"aje\")",
        "tipo.startsWith(\"per\")",
        "tipo.endsWith(\"aje\")",
        "tipo.upper().lower()",
        "\"  a b \".trim().title()",
        "\"abc\".reverse()",
        "\"ab\".repeat(2)",
        "\"a-b\".replace(\"-\", \"+\")",
        "\"a,b\".split(\",\")",
        "\"abcdef\".slice(1, -2)",
        "tipo.length",
        "tipo.unknown()",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(true),
            text("personaje"),
            text("A B"),
            text("cba"),
            text("abab"),
            text("a+b"),
            Value::List(vec![text("a"), text("b")]),
            text("bcd"),
            Value::Number(9.0),
            Value::Null,
        ]
    );
}

#[test]
fn applies_number_methods() {
    // Arrange
    let sources = [
        "(-1.5).abs()",
        "1.2.ceil()",
        "1.8.floor()",
        "1.256.round(2)",
        "2.5.round()",
        "3.toFixed(2)",
        "edad.unknown()",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            Value::Number(1.5),
            Value::Number(2.0),
            Value::Number(1.0),
            Value::Number(1.26),
            Value::Number(3.0),
            text("3.00"),
            Value::Null,
        ]
    );
}

#[test]
fn applies_list_methods() {
    // Arrange
    let sources = [
        "aliases.contains(\"Ana\")",
        "aliases.containsAll(\"Ana\", \"La Torre\")",
        "aliases.containsAny(\"x\", \"Ana\")",
        "aliases.join(\" / \")",
        "aliases.join()",
        "[3, 1, 2].sort()",
        "[1, 1, 2].unique()",
        "[[1], 2].flat()",
        "[1, 2, 3].reverse().slice(1)",
        "[1, 2, 3].filter(value > 1)",
        "[1, 2, 3].map(value * index)",
        "[1, 2, 3][1]",
        "[1][5]",
        "aliases.unknown()",
        "[1].map()",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    let numbers = |items: &[f64]| Value::List(items.iter().map(|n| Value::Number(*n)).collect());
    assert_eq!(
        values,
        vec![
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(true),
            text("Ana / La Torre"),
            text("Ana,La Torre"),
            numbers(&[1.0, 2.0, 3.0]),
            numbers(&[1.0, 2.0]),
            numbers(&[1.0, 2.0]),
            numbers(&[2.0, 1.0]),
            numbers(&[2.0, 3.0]),
            numbers(&[0.0, 2.0, 6.0]),
            Value::Number(2.0),
            Value::Null,
            Value::Null,
            Value::Null,
        ]
    );
}

#[test]
fn applies_methods_available_on_any_value() {
    // Arrange
    let sources = [
        "vacío.isEmpty()",
        "\"\".isEmpty()",
        "[].isEmpty()",
        "datos.isEmpty()",
        "edad.isEmpty()",
        "edad.isTruthy()",
        "edad.toString()",
        "aliases.toString()",
        "datos.toString()",
        "activo.toString()",
        "vacío.toString()",
        "1.5.toString()",
        "mentor.toString()",
        "file.toString()",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Boolean(true),
            text("42"),
            text("Ana, La Torre"),
            text("nivel: 3"),
            text("true"),
            text(""),
            text("1.5"),
            text("Bruno"),
            text("Ana Torres"),
        ]
    );
}

#[test]
fn reads_objects_and_dates() {
    // Arrange
    let sources = [
        "datos.nivel",
        "datos.keys()",
        "datos.values()",
        "datos.unknown()",
        "note.edad",
        "file.properties.tipo",
        "date(\"2026-03-04 05:06:07\").year",
        "date(\"2026-03-04T05:06:07Z\").month",
        "date(\"2026-03-04\").day",
        "date(\"2026-03-04 05:06:07\").hour",
        "date(\"2026-03-04 05:06:07\").minute",
        "date(\"2026-03-04 05:06:07\").second",
        "date(\"2026-03-04\").millisecond",
        "date(\"2026-03-04\").week",
        "date(\"2026-13-04\")",
        "date(\"nope\")",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            Value::Number(3.0),
            Value::List(vec![text("nivel")]),
            Value::List(vec![Value::Number(3.0)]),
            Value::Null,
            Value::Number(42.0),
            text("personaje"),
            Value::Number(2026.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
            Value::Number(6.0),
            Value::Number(7.0),
            Value::Number(0.0),
            Value::Null,
            Value::Null,
            Value::Null,
        ]
    );
}

#[test]
fn applies_global_functions() {
    // Arrange
    let sources = [
        "if(activo, \"sí\", \"no\")",
        "if(vacío, \"sí\", \"no\")",
        "if(vacío, \"sí\")",
        "list(tipo)",
        "list(aliases).length",
        "list(vacío)",
        "number(\" 7 \")",
        "number(activo)",
        "number(\"x\")",
        "number(aliases)",
        "max(1, 5, null, 3)",
        "min(4, 2, 8)",
        "link(\"Places/Tower\", \"Tower\")",
        "link()",
        "file.asLink(\"Ana\")",
        "unknown()",
        "(1)()",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            text("sí"),
            text("no"),
            Value::Null,
            Value::List(vec![text("personaje")]),
            Value::Number(2.0),
            Value::List(Vec::new()),
            Value::Number(7.0),
            Value::Number(1.0),
            Value::Null,
            Value::Null,
            Value::Number(5.0),
            Value::Number(2.0),
            Value::Link {
                target: "Places/Tower".to_string(),
                label: Some("Tower".to_string()),
            },
            Value::Null,
            Value::Link {
                target: "Characters/Ana Torres.md".to_string(),
                label: Some("Ana".to_string()),
            },
            Value::Null,
            Value::Null,
        ]
    );
}

#[test]
fn exposes_the_file_itself_and_its_date_as_numbers() {
    // Arrange
    let sources = [
        "file.file.name",
        "number(file.mtime)",
        "file.unknown",
        "file.unknown()",
    ];

    // Act
    let values: Vec<_> = sources.iter().map(|source| evaluate(source)).collect();

    // Assert
    assert_eq!(
        values,
        vec![
            text("Ana Torres.md"),
            Value::Number(MODIFIED_MS as f64),
            Value::Null,
            Value::Null,
        ]
    );
}
