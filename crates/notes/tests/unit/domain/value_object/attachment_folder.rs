use jade_notes::domain::value_object::{AttachmentFolder, VaultPath};

fn folder_for(setting: Option<&str>, note: &str) -> Option<String> {
    AttachmentFolder::from_setting(setting)
        .folder_for(&VaultPath::parse(note).expect("valid note"))
        .map(|folder| folder.as_str().to_string())
}

#[test]
fn places_attachments_where_obsidian_does() {
    // Arrange
    let cases = [
        (None, "Lore/Torre.md"),
        (Some("/"), "Lore/Torre.md"),
        (Some(""), "Lore/Torre.md"),
        (Some("./"), "Lore/Torre.md"),
        (Some("."), "Torre.md"),
        (Some("./adjuntos"), "Lore/Torre.md"),
        (Some("./adjuntos/"), "Torre.md"),
        (Some("Recursos"), "Lore/Torre.md"),
        (Some("/Recursos/Imágenes/"), "Torre.md"),
        (Some("../fuera"), "Torre.md"),
        (Some("./.."), "Lore/Torre.md"),
    ];

    // Act
    let folders: Vec<_> = cases
        .iter()
        .map(|(setting, note)| folder_for(*setting, note))
        .collect();

    // Assert
    assert_eq!(
        folders,
        vec![
            None,
            None,
            None,
            Some("Lore".to_string()),
            None,
            Some("Lore/adjuntos".to_string()),
            Some("adjuntos".to_string()),
            Some("Recursos".to_string()),
            Some("Recursos/Imágenes".to_string()),
            None,
            Some("Lore".to_string()),
        ]
    );
}
