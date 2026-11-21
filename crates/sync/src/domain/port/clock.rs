use crate::domain::value_object::Timestamp;

pub trait Clock {
    fn now(&self) -> Timestamp;
}
