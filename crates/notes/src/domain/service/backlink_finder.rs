use super::{LinkResolver, WikiLinkParser};
use crate::domain::model::Note;
use crate::domain::value_object::NotePath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backlink {
    pub source: NotePath,
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BacklinkFinder;

impl BacklinkFinder {
    pub fn find(&self, target: &NotePath, notes: &[Note]) -> Vec<Backlink> {
        let paths: Vec<NotePath> = notes.iter().map(|note| note.path().clone()).collect();
        let resolver = LinkResolver::new(&paths);
        notes
            .iter()
            .filter(|note| note.path() != target)
            .flat_map(|note| backlinks_from(note, target, &resolver))
            .collect()
    }
}

fn backlinks_from(note: &Note, target: &NotePath, resolver: &LinkResolver) -> Vec<Backlink> {
    let lines: Vec<&str> = note.content().lines().collect();
    let mut backlinks: Vec<Backlink> = Vec::new();
    for located in WikiLinkParser.parse(note.content()) {
        let already_listed = backlinks
            .last()
            .is_some_and(|last| last.line == located.line);
        let points_to_target = resolver.resolve(located.link.target()).as_ref() == Some(target);
        if points_to_target && !already_listed {
            backlinks.push(Backlink {
                source: note.path().clone(),
                line: located.line,
                text: lines
                    .get(located.line - 1)
                    .map_or("", |line| line.trim())
                    .to_string(),
            });
        }
    }
    backlinks
}
