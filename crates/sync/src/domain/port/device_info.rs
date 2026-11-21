use crate::domain::value_object::DeviceName;

pub trait DeviceInfo {
    fn device_name(&self) -> DeviceName;
}
