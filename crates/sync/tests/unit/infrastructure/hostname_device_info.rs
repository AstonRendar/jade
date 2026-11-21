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
