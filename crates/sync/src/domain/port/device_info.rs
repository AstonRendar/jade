use crate::domain::value_object::DeviceName;

pub trait DeviceInfo {
    fn device_name(&self) -> DeviceName;
}

impl<D: DeviceInfo> DeviceInfo for &D {
    fn device_name(&self) -> DeviceName {
        (**self).device_name()
    }
}
