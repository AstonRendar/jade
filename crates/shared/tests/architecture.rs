#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

const LAYERS: [&str; 3] = ["domain", "application", "infrastructure"];
const PURE_LAYERS: [&str; 2] = ["domain", "application"];
const ALLOWED_PURE_DEPENDENCIES: [&str; 3] = ["jade_shared", "thiserror", "unicode_normalization"];
const FORBIDDEN_STD_PATHS: [&str; 6] = [
    "std::fs",
    "std::net",
    "std::process",
    "std::env",
    "std::thread",
    "std::time::SystemTime",
];

#[derive(Debug, PartialEq)]
struct Violation {
    location: String,
    reason: String,
}

fn check_crate(crate_dir: &Path) -> Vec<Violation> {
    let src = crate_dir.join("src");
    let forbidden_dependencies = forbidden_dependencies(&crate_dir.join("Cargo.toml"));
    let mut violations = layout_violations(&src);
    for layer in PURE_LAYERS {
        for file in rust_files(&src.join(layer)) {
            violations.extend(file_violations(&file, layer, &forbidden_dependencies));
        }
    }
    violations
}

fn layout_violations(src: &Path) -> Vec<Violation> {
    let missing_layers = LAYERS
        .iter()
        .filter(|layer| !src.join(layer).is_dir())
        .map(|layer| violation(src, format!("missing layer `{layer}`")));
    let unexpected_entries = entry_names(src)
        .into_iter()
        .filter(|name| name != "lib.rs" && !LAYERS.contains(&name.as_str()))
        .map(|name| violation(src, format!("unexpected entry `{name}` in src")));
    missing_layers.chain(unexpected_entries).collect()
}

fn file_violations(file: &Path, layer: &str, forbidden_dependencies: &[String]) -> Vec<Violation> {
    let content = fs::read_to_string(file).expect("source file is readable");
    let dependency_violations = forbidden_dependencies
        .iter()
        .filter(|dependency| references(&content, &format!("{dependency}::")))
        .map(|dependency| format!("depends on `{dependency}`"));
    let path_violations = FORBIDDEN_STD_PATHS
        .iter()
        .chain(outer_layers_of(layer))
        .filter(|path| references(&content, path))
        .map(|path| format!("uses `{path}`"));
    dependency_violations
        .chain(path_violations)
        .map(|reason| violation(file, reason))
        .collect()
}

fn outer_layers_of(layer: &str) -> &'static [&'static str] {
    match layer {
        "domain" => &["crate::application", "crate::infrastructure"],
        _ => &["crate::infrastructure"],
    }
}

fn references(content: &str, path: &str) -> bool {
    content
        .match_indices(path)
        .any(|(index, _)| !is_path_character(content[..index].chars().next_back()))
}

fn is_path_character(character: Option<char>) -> bool {
    character.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ':')
}

fn forbidden_dependencies(manifest: &Path) -> Vec<String> {
    let content = fs::read_to_string(manifest).expect("manifest is readable");
    let mut section = "";
    let mut dependencies = Vec::new();
    for line in content.lines().map(str::trim) {
        if line.starts_with('[') {
            section = line;
            if let Some(name) = line.strip_prefix("[dependencies.") {
                dependencies.push(name.trim_end_matches(']').to_string());
            }
        } else if section == "[dependencies]" && line.contains('=') {
            dependencies.push(dependency_name(line));
        }
    }
    dependencies
        .into_iter()
        .map(|name| name.replace('-', "_"))
        .filter(|name| !ALLOWED_PURE_DEPENDENCIES.contains(&name.as_str()))
        .collect()
}

fn dependency_name(manifest_line: &str) -> String {
    let key = manifest_line.split('=').next().unwrap_or_default();
    key.split('.').next().unwrap_or_default().trim().to_string()
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .map(|entry| entry.expect("entry is readable").path())
        .flat_map(|path| {
            if path.is_dir() {
                rust_files(&path)
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                vec![path]
            } else {
                Vec::new()
            }
        })
        .collect()
}

fn entry_names(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .map(|entry| {
            entry
                .expect("entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn violation(location: &Path, reason: String) -> Violation {
    Violation {
        location: location.display().to_string(),
        reason,
    }
}

fn workspace_crates() -> Vec<PathBuf> {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut crates: Vec<PathBuf> = fs::read_dir(crates_dir)
        .expect("crates directory is readable")
        .map(|entry| entry.expect("crate entry is readable").path())
        .filter(|path| path.join("Cargo.toml").is_file())
        .collect();
    crates.sort();
    crates
}

#[test]
fn every_workspace_crate_respects_the_architecture() {
    // Arrange
    let crates = workspace_crates();

    // Act
    let violations: Vec<Violation> = crates.iter().flat_map(|dir| check_crate(dir)).collect();

    // Assert
    assert!(!crates.is_empty());
    assert_eq!(violations, Vec::new());
}

#[test]
fn accepts_a_crate_with_the_three_layers_and_allowed_dependencies() {
    // Arrange
    let fixture = given_a_crate_depending_on(&["jade-shared", "thiserror", "git2"]);
    given_a_file(
        &fixture,
        "src/domain/mod.rs",
        "use thiserror::Error;\nuse std::fmt;\n",
    );
    given_a_file(
        &fixture,
        "src/application/mod.rs",
        "use crate::domain::Note;\n",
    );
    given_a_file(
        &fixture,
        "src/infrastructure/mod.rs",
        "use git2::Repository;\nuse std::fs;\n",
    );

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(violations, Vec::new());
}

#[test]
fn rejects_an_infrastructure_dependency_in_the_domain() {
    // Arrange
    let fixture = given_a_crate_depending_on(&["git2"]);
    given_a_file(&fixture, "src/domain/mod.rs", "use git2::Repository;\n");

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(reasons(&violations), vec!["depends on `git2`"]);
}

#[test]
fn rejects_an_external_dependency_in_the_application_layer() {
    // Arrange
    let fixture = given_a_crate_depending_on(&["serde"]);
    given_a_file(
        &fixture,
        "src/application/dto.rs",
        "#[derive(serde::Serialize)]\nstruct Dto;\n",
    );

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(reasons(&violations), vec!["depends on `serde`"]);
}

#[test]
fn rejects_io_from_the_standard_library_in_pure_layers() {
    // Arrange
    let fixture = given_a_crate_depending_on(&[]);
    given_a_file(
        &fixture,
        "src/application/mod.rs",
        "fn read() { std::fs::read(\"x\"); }\n",
    );

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(reasons(&violations), vec!["uses `std::fs`"]);
}

#[test]
fn rejects_a_domain_that_reaches_outer_layers() {
    // Arrange
    let fixture = given_a_crate_depending_on(&[]);
    given_a_file(
        &fixture,
        "src/domain/mod.rs",
        "use crate::application::SaveNote;\n",
    );

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(reasons(&violations), vec!["uses `crate::application`"]);
}

#[test]
fn rejects_an_application_layer_that_reaches_infrastructure() {
    // Arrange
    let fixture = given_a_crate_depending_on(&[]);
    given_a_file(
        &fixture,
        "src/application/mod.rs",
        "use crate::infrastructure::FsNotes;\n",
    );

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(reasons(&violations), vec!["uses `crate::infrastructure`"]);
}

#[test]
fn rejects_entries_outside_the_three_layers() {
    // Arrange
    let fixture = given_a_crate_depending_on(&[]);
    given_a_file(&fixture, "src/utils/mod.rs", "");

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(
        reasons(&violations),
        vec!["unexpected entry `utils` in src"]
    );
}

#[test]
fn rejects_a_crate_missing_a_layer() {
    // Arrange
    let fixture = given_a_crate_depending_on(&[]);
    fs::remove_dir_all(fixture.path().join("src/infrastructure")).expect("layer is removable");

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(reasons(&violations), vec!["missing layer `infrastructure`"]);
}

#[test]
fn does_not_mistake_longer_identifiers_for_dependencies() {
    // Arrange
    let fixture = given_a_crate_depending_on(&["git2"]);
    given_a_file(
        &fixture,
        "src/domain/mod.rs",
        "fn f() { my_git2::call(); }\n",
    );

    // Act
    let violations = check_crate(fixture.path());

    // Assert
    assert_eq!(violations, Vec::new());
}

fn given_a_crate_depending_on(dependencies: &[&str]) -> tempfile::TempDir {
    let fixture = tempfile::tempdir().expect("temporary directory is created");
    let dependency_lines: String = dependencies
        .iter()
        .map(|name| format!("{name} = \"1\"\n"))
        .collect();
    given_a_file(
        &fixture,
        "Cargo.toml",
        &format!("[package]\nname = \"fixture\"\n\n[dependencies]\n{dependency_lines}\n[dev-dependencies]\ntempfile = \"3\"\n"),
    );
    given_a_file(
        &fixture,
        "src/lib.rs",
        "pub mod application;\npub mod domain;\npub mod infrastructure;\n",
    );
    for layer in LAYERS {
        given_a_file(&fixture, &format!("src/{layer}/mod.rs"), "");
    }
    fixture
}

fn given_a_file(fixture: &tempfile::TempDir, relative_path: &str, content: &str) {
    let path = fixture.path().join(relative_path);
    fs::create_dir_all(path.parent().expect("file has a parent")).expect("parent is created");
    fs::write(path, content).expect("file is written");
}

fn reasons(violations: &[Violation]) -> Vec<&str> {
    violations
        .iter()
        .map(|violation| violation.reason.as_str())
        .collect()
}
