use crate::domain::port::DeviceInfo;
use crate::domain::value_object::DeviceName;

const LOCAL_NETWORK_SUFFIX: &str = ".local";
const PLACEHOLDER_HOSTNAME: &str = "localhost";

#[derive(Debug, Clone, Copy, Default)]
pub struct HostnameDeviceInfo;

impl HostnameDeviceInfo {
    pub fn device_name_of(hostname: &str) -> DeviceName {
        let name = hostname.trim();
        let name = name.strip_suffix(LOCAL_NETWORK_SUFFIX).unwrap_or(name);
        if name.eq_ignore_ascii_case(PLACEHOLDER_HOSTNAME) {
            return DeviceName::unknown();
        }
        DeviceName::parse(name).unwrap_or_else(|_| DeviceName::unknown())
    }
}

impl DeviceInfo for HostnameDeviceInfo {
    fn device_name(&self) -> DeviceName {
        Self::device_name_of(&system_device_name())
    }
}

#[cfg(target_os = "android")]
fn system_device_name() -> String {
    std::process::Command::new("getprop")
        .arg("ro.product.model")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default()
}

#[cfg(not(target_os = "android"))]
fn system_device_name() -> String {
    gethostname::gethostname().to_string_lossy().into_owned()
}
