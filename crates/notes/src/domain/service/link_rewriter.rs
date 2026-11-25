use std::ops::Range;

use super::{LinkResolver, WikiLinkParser};
use crate::domain::value_object::NotePath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteMove {
    pub from: NotePath,
    pub to: NotePath,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LinkRewriter;

impl LinkRewriter {
    pub fn rewrite(
        &self,
        content: &str,
        moves: &[NoteMove],
        before: &[NotePath],
    ) -> Option<String> {
        let after = LinkResolver::new(&paths_after(before, moves));
        let before = LinkResolver::new(before);
        let edits: Vec<(Range<usize>, String)> = WikiLinkParser
            .parse(content)
            .into_iter()
            .filter_map(|located| {
                let target = located.link.target();
                let moved = moved_note(target, moves, &before)?;
                if after.resolve(target).as_ref() == Some(moved) {
                    return None;
                }
                Some((located.target_span, link_text(target, moved, &after)))
            })
            .collect();
        if edits.is_empty() {
            return None;
        }
        let mut rewritten = content.to_string();
        for (span, text) in edits.into_iter().rev() {
            rewritten.replace_range(span, &text);
        }
        Some(rewritten)
    }
}

fn moved_note<'m>(
    target: &str,
    moves: &'m [NoteMove],
    before: &LinkResolver,
) -> Option<&'m NotePath> {
    let resolved = before.resolve(target)?;
    moves
        .iter()
        .find(|note_move| note_move.from == resolved)
        .map(|note_move| &note_move.to)
}

fn paths_after(before: &[NotePath], moves: &[NoteMove]) -> Vec<NotePath> {
    before
        .iter()
        .map(|path| {
            moves
                .iter()
                .find(|note_move| &note_move.from == path)
                .map_or_else(|| path.clone(), |note_move| note_move.to.clone())
        })
        .collect()
}

fn link_text(original: &str, moved: &NotePath, after: &LinkResolver) -> String {
    let title = moved.title();
    let title_is_enough = after.resolve(title).as_ref() == Some(moved);
    if original.contains('/') || !title_is_enough {
        moved.without_extension().to_string()
    } else {
        title.to_string()
    }
}
