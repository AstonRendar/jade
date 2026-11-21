use crate::domain::model::Note;
use crate::domain::value_object::{NotePath, SearchQuery};

const DEFAULT_LIMIT: usize = 300;
const TITLE_LINE: usize = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub path: NotePath,
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub struct NoteSearch {
    limit: usize,
}

impl NoteSearch {
    pub fn with_limit(limit: usize) -> Self {
        Self { limit }
    }

    pub fn search(&self, query: &SearchQuery, notes: &[Note]) -> Vec<SearchHit> {
        notes
            .iter()
            .flat_map(|note| hits_in(query, note))
            .take(self.limit)
            .collect()
    }
}

impl Default for NoteSearch {
    fn default() -> Self {
        Self::with_limit(DEFAULT_LIMIT)
    }
}

fn hits_in<'a>(query: &'a SearchQuery, note: &'a Note) -> impl Iterator<Item = SearchHit> + 'a {
    let title = query
        .matches(note.path().title())
        .then(|| hit(note, TITLE_LINE, note.path().title()));
    let lines = note
        .content()
        .lines()
        .enumerate()
        .filter(|(_, line)| query.matches(line))
        .map(|(index, line)| hit(note, index + 1, line.trim()));
    title.into_iter().chain(lines)
}

fn hit(note: &Note, line: usize, text: &str) -> SearchHit {
    SearchHit {
        path: note.path().clone(),
        line,
        text: text.to_string(),
    }
}
