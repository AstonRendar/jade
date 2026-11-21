use super::{Timestamp, TrackedPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictCopyName(TrackedPath);

impl ConflictCopyName {
    pub fn for_conflict(
        original: &TrackedPath,
        at: &Timestamp,
        is_taken: impl Fn(&TrackedPath) -> bool,
    ) -> Self {
        let mut sequence = 1;
        loop {
            let candidate = candidate(original, at, sequence);
            if !is_taken(&candidate) {
                return Self(candidate);
            }
            sequence += 1;
        }
    }

    pub fn path(&self) -> &TrackedPath {
        &self.0
    }
}

fn candidate(original: &TrackedPath, at: &Timestamp, sequence: u32) -> TrackedPath {
    let (stem, extension) = split_extension(original.file_name());
    let suffix = conflict_suffix(at, sequence);
    let file_name = match extension {
        Some(extension) => format!("{stem} ({suffix}).{extension}"),
        None => format!("{stem} ({suffix})"),
    };
    original.sibling(&file_name)
}

fn split_extension(file_name: &str) -> (&str, Option<&str>) {
    match file_name.rfind('.') {
        Some(index) if index > 0 => (&file_name[..index], Some(&file_name[index + 1..])),
        _ => (file_name, None),
    }
}

fn conflict_suffix(at: &Timestamp, sequence: u32) -> String {
    let date = format!(
        "conflict {:04}-{:02}-{:02} {:02}{:02}",
        at.year(),
        at.month(),
        at.day(),
        at.hour(),
        at.minute()
    );
    if sequence > 1 {
        format!("{date} {sequence}")
    } else {
        date
    }
}
