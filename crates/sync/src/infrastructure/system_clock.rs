use chrono::{Datelike, Local, NaiveDateTime, Timelike};

use crate::domain::port::Clock;
use crate::domain::value_object::Timestamp;

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl SystemClock {
    pub fn timestamp_of(local: NaiveDateTime) -> Timestamp {
        let components = (
            u16::try_from(local.year()),
            u8::try_from(local.month()),
            u8::try_from(local.day()),
            u8::try_from(local.hour()),
            u8::try_from(local.minute()),
        );
        match components {
            (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(minute)) => {
                Timestamp::new(year, month, day, hour, minute).unwrap_or(Timestamp::unix_epoch())
            }
            _ => Timestamp::unix_epoch(),
        }
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Self::timestamp_of(Local::now().naive_local())
    }
}
