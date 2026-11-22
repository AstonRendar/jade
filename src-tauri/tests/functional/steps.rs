use cucumber::{given, then, when};
use jade_lib::view_models::ConflictCopyView;

use crate::world::AppWorld;

#[given(expr = "a folder {string} with the note {string}")]
fn a_folder_with_a_note(world: &mut AppWorld, folder: String, note: String) {
    let path = world.path(&folder);
    std::fs::create_dir_all(&path).expect("folder is created");
    std::fs::write(path.join(note), "text").expect("note is written");
}

#[given(expr = "a remote vault {string} with {string} containing {string}")]
fn a_remote_vault(world: &mut AppWorld, name: String, note: String, content: String) {
    let remote = world.path(&format!("remotes/{name}.git"));
    let mut options = git2::RepositoryInitOptions::new();
    options.bare(true).initial_head("main");
    git2::Repository::init_opts(&remote, &options).expect("remote is created");
    world.remote = Some(remote);
    let seed = world.device_folder("seed").display().to_string();
    let url = world.remote_url();
    let app = world.app("seed");
    app.open_vault(&seed).expect("seed opens");
    app.init_repository().expect("seed is a repository");
    app.connect_remote(&url).expect("seed is connected");
    app.save_note(&note, &content).expect("note is saved");
    app.synchronize().expect("seed is pushed");
}

#[given(expr = "the app on {string} cloned the remote vault")]
#[when(expr = "the app on {string} clones the remote vault")]
fn the_app_clones_the_remote(world: &mut AppWorld, device: String) {
    let url = world.remote_url();
    clone(world, &device, &url);
}

#[when(expr = "the app on {string} clones {string}")]
fn the_app_clones_an_address(world: &mut AppWorld, device: String, url: String) {
    clone(world, &device, &url);
}

#[when(expr = "the app on {string} opens the folder {string}")]
fn the_app_opens_a_folder(world: &mut AppWorld, device: String, folder: String) {
    let path = world.path(&folder).display().to_string();
    let result = world.app(&device).open_vault(&path);
    if let Some(vault) = world.record(&device, result) {
        world.opened.insert(device, vault);
    }
}

#[when(expr = "the app on {string} lists the vault entries")]
fn the_app_lists_entries(world: &mut AppWorld, device: String) {
    let result = world.app(&device).list_entries();
    world.record(&device, result);
}

#[given(expr = "the app on {string} saves {string} with {string}")]
#[when(expr = "the app on {string} saves {string} with {string}")]
fn the_app_saves_a_note(world: &mut AppWorld, device: String, path: String, content: String) {
    world
        .app(&device)
        .save_note(&path, &content)
        .expect("note is saved");
}

#[given(expr = "the app on {string} synchronizes")]
#[when(expr = "the app on {string} synchronizes")]
fn the_app_synchronizes(world: &mut AppWorld, device: String) {
    let report = world.app(&device).synchronize().expect("sync succeeds");
    world.reports.insert(device, report);
}

#[when(
    expr = "the app on {string} sets the language {string}, theme {string} and sync every {int} minutes"
)]
fn the_app_sets_preferences(
    world: &mut AppWorld,
    device: String,
    locale: String,
    theme: String,
    minutes: u32,
) {
    world
        .app(&device)
        .update_preferences(Some(locale), &theme, minutes)
        .expect("preferences are updated");
}

#[when(expr = "the app on {string} stores the token {string}")]
fn the_app_stores_a_token(world: &mut AppWorld, device: String, token: String) {
    world
        .app(&device)
        .store_access_token(&token)
        .expect("token is stored");
}

#[then(expr = "the open vault on {string} is named {string}")]
fn the_open_vault_is_named(world: &mut AppWorld, device: String, name: String) {
    let opened = world.opened.get(&device).map(|vault| vault.name.as_str());
    assert_eq!(opened, Some(name.as_str()));
}

#[then(expr = "the most recent vault on {string} is the folder {string}")]
fn the_most_recent_vault_is(world: &mut AppWorld, device: String, folder: String) {
    let expected = world.path(&folder).display().to_string();
    let preferences = world.app(&device).preferences().expect("preferences load");
    assert_eq!(preferences.recent_vaults.first(), Some(&expected));
}

#[then(expr = "the vault on {string} lists {string}")]
fn the_vault_lists(world: &mut AppWorld, device: String, path: String) {
    let entries = world
        .app(&device)
        .list_entries()
        .expect("entries are listed");
    assert!(entries.iter().any(|entry| entry.path == path));
}

#[then(expr = "the app on {string} fails with {string}")]
fn the_app_fails_with(world: &mut AppWorld, device: String, code: String) {
    let error = world.errors.get(&device).expect("an error was recorded");
    let actual = serde_json::to_value(error.code).expect("code serialises");
    assert_eq!(actual, serde_json::Value::String(code));
}

#[then(expr = "the last sync on {string} reported the conflict copy {string} of {string}")]
fn reported_conflict_copy(world: &mut AppWorld, device: String, copy: String, original: String) {
    let report = world.reports.get(&device).expect("a sync ran");
    assert_eq!(
        report.conflict_copies,
        vec![ConflictCopyView { original, copy }]
    );
}

#[then(expr = "the repository on {string} has nothing pending")]
fn nothing_pending(world: &mut AppWorld, device: String) {
    let status = world
        .app(&device)
        .repository_status()
        .expect("status is read");
    assert!(status.is_repository);
    assert_eq!(
        (status.pending_changes, status.ahead, status.behind),
        (0, 0, 0)
    );
}

#[then(
    expr = "the preferences on {string} show language {string}, theme {string} and sync every {int} minutes"
)]
fn preferences_show(
    world: &mut AppWorld,
    device: String,
    locale: String,
    theme: String,
    minutes: u32,
) {
    let preferences = world.app(&device).preferences().expect("preferences load");
    assert_eq!(preferences.locale, Some(locale));
    assert_eq!(preferences.theme, theme);
    assert_eq!(preferences.auto_sync_minutes, minutes);
}

#[then(expr = "the app on {string} has a token")]
fn the_app_has_a_token(world: &mut AppWorld, device: String) {
    assert_eq!(world.app(&device).has_access_token(), Ok(true));
}

fn clone(world: &mut AppWorld, device: &str, url: &str) {
    let parent = world.device_folder(device).display().to_string();
    let result = world.app(device).clone_repository(url, &parent);
    if let Some(vault) = world.record(device, result) {
        world.opened.insert(device.to_string(), vault);
    }
}

#[given(expr = "the app on {string} opened the folder {string}")]
fn the_app_opened_a_folder(world: &mut AppWorld, device: String, folder: String) {
    let path = world.path(&folder).display().to_string();
    world.app(&device).open_vault(&path).expect("vault opens");
}

#[when(expr = "the app on {string} creates the note {string} in the root")]
fn the_app_creates_a_note(world: &mut AppWorld, device: String, title: String) {
    world
        .app(&device)
        .create_note(None, &title)
        .expect("note is created");
}

#[when(expr = "the app on {string} creates the folder {string}")]
fn the_app_creates_a_folder(world: &mut AppWorld, device: String, path: String) {
    world
        .app(&device)
        .create_folder(&path)
        .expect("folder is created");
}

#[when(expr = "the app on {string} renames {string} to {string}")]
fn the_app_renames(world: &mut AppWorld, device: String, from: String, to: String) {
    world
        .app(&device)
        .rename_entry(&from, &to)
        .expect("entry is renamed");
}

#[when(expr = "the app on {string} follows the link {string}")]
fn the_app_follows_a_link(world: &mut AppWorld, device: String, target: String) {
    world
        .app(&device)
        .follow_link(&target)
        .expect("link is followed");
}

#[when(expr = "the app on {string} deletes {string}")]
fn the_app_deletes(world: &mut AppWorld, device: String, path: String) {
    world
        .app(&device)
        .delete_entry(&path)
        .expect("entry is deleted");
}

#[when(expr = "the app on {string} initialises a repository")]
fn the_app_initialises(world: &mut AppWorld, device: String) {
    world
        .app(&device)
        .init_repository()
        .expect("repository is initialised");
}

#[when(expr = "the app on {string} forgets the folder {string}")]
fn the_app_forgets(world: &mut AppWorld, device: String, folder: String) {
    let path = world.path(&folder).display().to_string();
    world
        .app(&device)
        .forget_vault(&path)
        .expect("vault is forgotten");
}

#[then(expr = "{string} on {string} reads {string}")]
fn a_note_reads(world: &mut AppWorld, path: String, device: String, content: String) {
    assert_eq!(world.app(&device).read_note(&path), Ok(content));
}

#[then(expr = "searching {string} on {string} finds {string}")]
fn searching_finds(world: &mut AppWorld, query: String, device: String, path: String) {
    let hits = world.app(&device).search(&query).expect("search runs");
    assert!(hits.iter().any(|hit| hit.path == path));
}

#[then(expr = "the backlinks of {string} on {string} come from {string}")]
fn backlinks_come_from(world: &mut AppWorld, path: String, device: String, source: String) {
    let backlinks = world.app(&device).backlinks(&path).expect("backlinks load");
    let sources: Vec<&str> = backlinks.iter().map(|b| b.source.as_str()).collect();
    assert_eq!(sources, vec![source.as_str()]);
}

#[then(expr = "the link {string} on {string} resolves to {string}")]
fn the_link_resolves(world: &mut AppWorld, target: String, device: String, path: String) {
    let resolved = world
        .app(&device)
        .resolve_links(vec![target])
        .expect("links resolve");
    assert_eq!(resolved[0].path.as_deref(), Some(path.as_str()));
}

#[then(expr = "the vault on {string} does not list {string}")]
fn the_vault_does_not_list(world: &mut AppWorld, device: String, path: String) {
    let entries = world
        .app(&device)
        .list_entries()
        .expect("entries are listed");
    assert!(entries.iter().all(|entry| entry.path != path));
}

#[then(expr = "the history of {string} on {string} has {int} revision(s)")]
fn history_has(world: &mut AppWorld, path: String, device: String, count: usize) {
    let history = world
        .app(&device)
        .note_history(&path)
        .expect("history loads");
    assert_eq!(history.len(), count);
}

#[then(expr = "the first revision of {string} on {string} reads {string}")]
fn first_revision_reads(world: &mut AppWorld, path: String, device: String, content: String) {
    let app = world.app(&device);
    let history = app.note_history(&path).expect("history loads");
    let oldest = history.last().expect("history is not empty");
    assert_eq!(app.note_revision(&path, &oldest.id), Ok(content));
}

#[then(expr = "{string} remembers no vaults")]
fn remembers_no_vaults(world: &mut AppWorld, device: String) {
    let preferences = world.app(&device).preferences().expect("preferences load");
    assert!(preferences.recent_vaults.is_empty());
}
