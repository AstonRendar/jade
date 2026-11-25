use jade_sync::infrastructure::HostnameDeviceInfo;

#[test]
fn drops_the_local_network_suffix() {
    // Arrange
    let hostname = "Example-MacBook.local";

    // Act
    let name = HostnameDeviceInfo::device_name_of(hostname);

    // Assert
    assert_eq!(name.as_str(), "Example-MacBook");
}

#[test]
fn falls_back_to_a_generic_name_for_unusable_hostnames() {
    // Arrange
    let hostname = "   ";

    // Act
    let name = HostnameDeviceInfo::device_name_of(hostname);

    // Assert
    assert_eq!(name.as_str(), "Unknown device");
}

#[test]
fn treats_the_placeholder_hostname_as_unknown() {
    // Arrange
    let hostnames = ["localhost", "LOCALHOST", "localhost.local"];

    // Act
    let names: Vec<_> = hostnames
        .iter()
        .map(|hostname| HostnameDeviceInfo::device_name_of(hostname))
        .collect();

    // Assert
    assert!(names.iter().all(|name| name.as_str() == "Unknown device"));
}

#[test]
fn keeps_device_models_with_spaces() {
    // Arrange
    let model = "Pixel 8\n";

    // Act
    let name = HostnameDeviceInfo::device_name_of(model);

    // Assert
    assert_eq!(name.as_str(), "Pixel 8");
}
