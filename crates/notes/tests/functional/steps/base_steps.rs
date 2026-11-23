use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use jade_notes::application::error::NotesError;
use jade_notes::application::query_base::{QueryBase, QueryBaseRequest};
use jade_notes::domain::model::Cell;
use jade_notes::infrastructure::{FsNoteRepository, YamlDataFormat};
use jade_shared::application::use_case::UseCase;

use super::link_steps::table_rows;
use crate::world::NotesWorld;

#[given(expr = "the file {string} contains:")]
fn the_file_contains(world: &mut NotesWorld, path: String, step: &Step) {
    let content = step
        .docstring
        .as_deref()
        .expect("the step has a text block");
    let file = std::path::PathBuf::from(format!("{}/{path}", world.root()));
    if let Some(folder) = file.parent() {
        std::fs::create_dir_all(folder).expect("folder exists");
    }
    std::fs::write(file, content.trim_start()).expect("file is written");
}

#[when(expr = "the base {string} is queried")]
fn the_base_is_queried(world: &mut NotesWorld, path: String) {
    query(world, path, None);
}

#[when(expr = "the view {string} of the base {string} is queried")]
fn the_view_is_queried(world: &mut NotesWorld, view: String, path: String) {
    query(world, path, Some(view));
}

#[then(expr = "the base table has the columns {string}")]
fn the_columns_are(world: &mut NotesWorld, columns: String) {
    let table = &world.base.as_ref().expect("the base was queried").table;
    let actual: Vec<&str> = table
        .columns
        .iter()
        .map(|column| column.display_name.as_deref().unwrap_or(&column.key))
        .collect();
    assert_eq!(actual.join(", "), columns);
}

#[then("the base rows are:")]
fn the_rows_are(world: &mut NotesWorld, step: &Step) {
    let table = &world.base.as_ref().expect("the base was queried").table;
    let actual: Vec<Vec<String>> = table
        .rows
        .iter()
        .map(|row| {
            let cells: Vec<String> = row.cells.iter().map(text).collect();
            vec![
                row.path.as_str().to_string(),
                cells.join(" | ").trim_end().to_string(),
            ]
        })
        .collect();
    let expected: Vec<Vec<String>> = table_rows(step)
        .into_iter()
        .map(|row| vec![row[0].clone(), row[1].trim_end().to_string()])
        .collect();
    assert_eq!(actual, expected);
}

#[then("the operation fails because the base is invalid")]
fn fails_because_the_base_is_invalid(world: &mut NotesWorld) {
    assert!(matches!(world.last_error, Some(NotesError::InvalidBase(_))));
}

fn query(world: &mut NotesWorld, path: String, view: Option<String>) {
    let result = QueryBase::new(FsNoteRepository, YamlDataFormat).execute(QueryBaseRequest {
        vault_root: world.root(),
        path,
        view,
    });
    world.base = world.record(result);
}

fn text(cell: &Cell) -> String {
    match cell {
        Cell::Null => String::new(),
        Cell::Text(text) => text.clone(),
        Cell::List(items) => items.iter().map(text).collect::<Vec<_>>().join(", "),
        Cell::Link { target, .. } => target.clone(),
        other => format!("{other:?}"),
    }
}
