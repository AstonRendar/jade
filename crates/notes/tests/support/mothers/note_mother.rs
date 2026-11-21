use jade_notes::domain::value_object::NotePath;

pub struct NoteMother;

impl NoteMother {
    pub fn path(raw: &str) -> NotePath {
        NotePath::parse(raw).expect("valid note path")
    }

    pub fn paths(raws: &[&str]) -> Vec<NotePath> {
        raws.iter().map(|raw| Self::path(raw)).collect()
    }
}
