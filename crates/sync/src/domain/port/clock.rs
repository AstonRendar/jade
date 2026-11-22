use crate::domain::value_object::Timestamp;

pub trait Clock {
    fn now(&self) -> Timestamp;
}

impl<K: Clock> Clock for &K {
    fn now(&self) -> Timestamp {
        (**self).now()
    }
}
