#![allow(clippy::expect_used)]

use std::fs;
use std::path::Path;

use jade_sync::infrastructure::SystemCertificates;
use tempfile::TempDir;

#[test]
fn reads_every_certificate_of_the_first_existing_store() {
    // Arrange
    let preferred = TempDir::new().expect("temporary directory");
    let fallback = TempDir::new().expect("temporary directory");
    given_a_certificate(preferred.path(), "b.0", "second");
    given_a_certificate(preferred.path(), "a.0", "first");
    given_a_certificate(fallback.path(), "c.0", "ignored");

    // Act
    let certificates = SystemCertificates::read_from(&[preferred.path(), fallback.path()]);

    // Assert
    assert_eq!(certificates, b"first\nsecond\n");
}

#[test]
fn falls_back_to_the_next_store_when_one_is_missing() {
    // Arrange
    let fallback = TempDir::new().expect("temporary directory");
    given_a_certificate(fallback.path(), "c.0", "fallback");

    // Act
    let certificates =
        SystemCertificates::read_from(&[Path::new("/nonexistent/cacerts"), fallback.path()]);

    // Assert
    assert_eq!(certificates, b"fallback\n");
}

#[test]
fn is_empty_without_any_store() {
    // Arrange
    let missing = Path::new("/nonexistent/cacerts");

    // Act
    let certificates = SystemCertificates::read_from(&[missing]);

    // Assert
    assert!(certificates.is_empty());
}

fn given_a_certificate(store: &Path, name: &str, content: &str) {
    fs::write(store.join(name), content).expect("certificate written");
}
