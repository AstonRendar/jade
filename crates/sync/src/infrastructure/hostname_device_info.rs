use crate::domain::port::DeviceInfo;
use crate::domain::value_object::DeviceName;

const LOCAL_NETWORK_SUFFIX: &str = ".local";

#[derive(Debug, Clone, Copy, Default)]
pub struct HostnameDeviceInfo;

impl HostnameDeviceInfo {
    pub fn device_name_of(hostname: &str) -> DeviceName {
        let name = hostname.trim();
        let name = name.strip_suffix(LOCAL_NETWORK_SUFFIX).unwrap_or(name);
        DeviceName::parse(name).unwrap_or_else(|_| DeviceName::unknown())
    }
}

impl DeviceInfo for HostnameDeviceInfo {
    fn device_name(&self) -> DeviceName {
        Self::device_name_of(&gethostname::gethostname().to_string_lossy())
    }
}
