use jade_sync::domain::value_object::TrackedPath;

pub struct TrackedPathMother;

impl TrackedPathMother {
    pub fn of(path: &str) -> TrackedPath {
        TrackedPath::parse(path).expect("valid tracked path")
    }
}
