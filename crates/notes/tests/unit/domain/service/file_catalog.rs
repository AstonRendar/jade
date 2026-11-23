use jade_notes::domain::service::FileCatalog;
use jade_notes::domain::value_object::Data;

use crate::support::mothers::{FileRecordMother, NoteMother};

#[test]
fn collects_properties_tags_and_link_targets_of_a_note() {
    // Arrange
    let frontmatter = Data::Map(vec![
        ("tipo".to_string(), Data::Text("lore".to_string())),
        (
            "tags".to_string(),
            FileRecordMother::texts(&["#moc", "saga"]),
        ),
    ]);
    let content = "---\ntipo: lore\n---\nSee [[Tower]] and [[Missing]] #saga #extra\n";

    // Act
    let record = FileCatalog.note(
        NoteMother::vault_path("Lore.md"),
        FileRecordMother::metadata(),
        Some(&frontmatter),
        content,
    );

    // Assert
    assert_eq!(
        record.property("tipo"),
        Some(&Data::Text("lore".to_string()))
    );
    assert_eq!(record.tags(), ["moc", "saga", "extra"]);
    assert_eq!(record.links(), ["Tower", "Missing"]);
}

#[test]
fn reads_tags_written_as_text_and_tolerates_missing_frontmatter() {
    // Arrange
    let frontmatter = Data::Map(vec![(
        "tags".to_string(),
        Data::Text("a, #b c".to_string()),
    )]);

    // Act
    let tagged = FileCatalog.note(
        NoteMother::vault_path("a.md"),
        FileRecordMother::metadata(),
        Some(&frontmatter),
        "",
    );
    let bare = FileCatalog.note(
        NoteMother::vault_path("b.md"),
        FileRecordMother::metadata(),
        None,
        "text",
    );
    let file = FileCatalog.file(
        NoteMother::vault_path("image.png"),
        FileRecordMother::metadata(),
    );

    // Assert
    assert_eq!(tagged.tags(), ["a", "b", "c"]);
    assert!(bare.properties().is_empty() && bare.tags().is_empty());
    assert!(file.properties().is_empty() && file.links().is_empty());
}
