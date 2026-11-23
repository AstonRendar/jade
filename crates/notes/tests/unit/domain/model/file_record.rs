use crate::support::mothers::FileRecordMother;

#[test]
fn derives_name_parts_from_the_path() {
    // Arrange
    let records = [
        FileRecordMother::file("Notes/Plan.v2.md"),
        FileRecordMother::file("README"),
        FileRecordMother::file(".hidden"),
    ];

    // Act
    let parts: Vec<_> = records
        .iter()
        .map(|record| {
            (
                record.basename().to_string(),
                record.extension().to_string(),
                record.folder(),
            )
        })
        .collect();

    // Assert
    assert_eq!(
        parts,
        vec![
            ("Plan.v2".to_string(), "md".to_string(), "Notes".to_string()),
            ("README".to_string(), String::new(), String::new()),
            (".hidden".to_string(), String::new(), String::new()),
        ]
    );
}
