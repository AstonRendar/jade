use jade_sync::domain::port::{Clock, DeviceInfo};
use jade_sync::domain::value_object::{DeviceName, Timestamp};

pub struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp::new(2026, 11, 21, 22, 45).expect("valid timestamp")
    }
}

pub struct NamedDevice(pub String);

impl DeviceInfo for NamedDevice {
    fn device_name(&self) -> DeviceName {
        DeviceName::parse(&self.0).expect("valid device name")
    }
}
