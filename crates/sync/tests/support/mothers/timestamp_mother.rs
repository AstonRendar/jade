use jade_sync::domain::value_object::Timestamp;

pub struct TimestampMother;

impl TimestampMother {
    pub fn any() -> Timestamp {
        Self::at(2026, 11, 21, 22, 45)
    }

    pub fn at(year: u16, month: u8, day: u8, hour: u8, minute: u8) -> Timestamp {
        Timestamp::new(year, month, day, hour, minute).expect("valid timestamp")
    }
}
