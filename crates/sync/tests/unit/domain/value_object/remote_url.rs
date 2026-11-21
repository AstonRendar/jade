use jade_sync::domain::value_object::RemoteUrl;

#[test]
fn uses_https_urls_as_they_are() {
    // Arrange
    let raw = "https://git.example.com/team/vault.git";

    // Act
    let url = RemoteUrl::parse(raw).expect("valid");

    // Assert
    assert_eq!(url.network_url(), raw);
}

#[test]
fn converts_scp_like_ssh_urls_to_https() {
    // Arrange
    let raw = "git@git.example.com:team/vault.git";

    // Act
    let url = RemoteUrl::parse(raw).expect("valid");

    // Assert
    assert_eq!(url.network_url(), "https://git.example.com/team/vault.git");
}

#[test]
fn converts_ssh_scheme_urls_to_https_dropping_user_and_port() {
    // Arrange
    let raw = "ssh://git@git.example.com:2222/team/vault.git";

    // Act
    let url = RemoteUrl::parse(raw).expect("valid");

    // Assert
    assert_eq!(url.network_url(), "https://git.example.com/team/vault.git");
}

#[test]
fn uses_local_file_urls_as_they_are() {
    // Arrange
    let raw = "file:///tmp/remotes/vault.git";

    // Act
    let url = RemoteUrl::parse(raw).expect("valid");

    // Assert
    assert_eq!(url.network_url(), raw);
}

#[test]
fn keeps_the_original_form() {
    // Arrange
    let raw = "git@git.example.com:team/vault.git";

    // Act
    let url = RemoteUrl::parse(raw).expect("valid");

    // Assert
    assert_eq!(url.as_str(), raw);
}

#[test]
fn rejects_urls_it_cannot_reach_over_https() {
    // Arrange
    let raws = [
        "",
        "not a url",
        "ftp://git.example.com/vault.git",
        "https://",
        "https://git.example.com",
        "git@git.example.com:",
        "ssh://git.example.com",
        "file://",
    ];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| RemoteUrl::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}
