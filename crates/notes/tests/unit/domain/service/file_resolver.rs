use jade_notes::domain::service::FileResolver;
use jade_notes::domain::value_object::VaultPath;

fn files(paths: &[&str]) -> Vec<VaultPath> {
    paths
        .iter()
        .map(|path| VaultPath::parse(path).expect("valid path"))
        .collect()
}

fn resolve(target: &str) -> Option<String> {
    let files = files(&[
        "Recursos/Portada.webp",
        "Recursos/Mapas/Erquia.png",
        "Archivo/Recursos/Portada.webp",
        "Personajes/Iby.md",
        "Diagrama",
    ]);
    FileResolver::new(&files)
        .resolve(target)
        .map(|path| path.as_str().to_string())
}

#[test]
fn resolves_exact_paths_names_and_partial_paths() {
    // Arrange
    let targets = [
        "Recursos/Mapas/Erquia.png",
        "erquia.PNG",
        "Mapas/Erquia.png",
        "./Recursos/Portada.webp",
        "/Diagrama",
    ];

    // Act
    let resolved: Vec<_> = targets.iter().map(|target| resolve(target)).collect();

    // Assert
    assert_eq!(
        resolved,
        vec![
            Some("Recursos/Mapas/Erquia.png".to_string()),
            Some("Recursos/Mapas/Erquia.png".to_string()),
            Some("Recursos/Mapas/Erquia.png".to_string()),
            Some("Recursos/Portada.webp".to_string()),
            Some("Diagrama".to_string()),
        ]
    );
}

#[test]
fn prefers_the_shortest_path_and_falls_back_to_notes_without_extension() {
    // Arrange
    let targets = ["Portada.webp", "Iby"];

    // Act
    let resolved: Vec<_> = targets.iter().map(|target| resolve(target)).collect();

    // Assert
    assert_eq!(
        resolved,
        vec![
            Some("Recursos/Portada.webp".to_string()),
            Some("Personajes/Iby.md".to_string())
        ]
    );
}

#[test]
fn does_not_resolve_missing_or_blank_targets() {
    // Arrange
    let targets = ["Nope.png", "  ", "Erquia.jpg", "ecursos/Portada.webp"];

    // Act
    let resolved: Vec<_> = targets.iter().map(|target| resolve(target)).collect();

    // Assert
    assert!(resolved.iter().all(Option::is_none), "{resolved:?}");
}
