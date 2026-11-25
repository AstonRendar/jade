use std::sync::Arc;

use tauri::{AppHandle, State};

use super::{blocking, AppJade};
use crate::errors::ErrorView;
use crate::view_models::{
    BacklinkView, BaseTableView, EntryView, FollowedLinkView, NotePropertiesView, PreferencesView,
    PropertySummaryView, PropertyView, ResolvedLinkView, SearchHitView, VaultView,
};
use crate::watcher::VaultWatcher;

#[tauri::command]
pub async fn notes_open_vault(
    app: AppHandle,
    jade: State<'_, Arc<AppJade>>,
    watcher: State<'_, VaultWatcher>,
    path: String,
) -> Result<VaultView, ErrorView> {
    let root = path.clone();
    let vault = blocking(&jade, move |jade| jade.open_vault(&path)).await?;
    watcher.watch(app, &root)?;
    Ok(vault)
}

#[tauri::command]
pub async fn notes_create_vault(
    app: AppHandle,
    jade: State<'_, Arc<AppJade>>,
    watcher: State<'_, VaultWatcher>,
    parent: String,
    name: String,
) -> Result<VaultView, ErrorView> {
    let vault = blocking(&jade, move |jade| jade.create_vault(&parent, &name)).await?;
    watcher.watch(app, &vault.path)?;
    Ok(vault)
}

#[tauri::command]
pub async fn notes_list_entries(
    jade: State<'_, Arc<AppJade>>,
) -> Result<Vec<EntryView>, ErrorView> {
    blocking(&jade, |jade| jade.list_entries()).await
}

#[tauri::command]
pub async fn notes_read_note(
    jade: State<'_, Arc<AppJade>>,
    path: String,
) -> Result<String, ErrorView> {
    blocking(&jade, move |jade| jade.read_note(&path)).await
}

#[tauri::command]
pub async fn notes_save_note(
    jade: State<'_, Arc<AppJade>>,
    path: String,
    content: String,
) -> Result<(), ErrorView> {
    blocking(&jade, move |jade| jade.save_note(&path, &content)).await
}

#[tauri::command]
pub async fn notes_create_note(
    jade: State<'_, Arc<AppJade>>,
    folder: Option<String>,
    title: String,
) -> Result<String, ErrorView> {
    blocking(&jade, move |jade| jade.create_note(folder, &title)).await
}

#[tauri::command]
pub async fn notes_create_folder(
    jade: State<'_, Arc<AppJade>>,
    path: String,
) -> Result<String, ErrorView> {
    blocking(&jade, move |jade| jade.create_folder(&path)).await
}

#[tauri::command]
pub async fn notes_rename_entry(
    jade: State<'_, Arc<AppJade>>,
    from: String,
    to: String,
) -> Result<String, ErrorView> {
    blocking(&jade, move |jade| jade.rename_entry(&from, &to)).await
}

#[tauri::command]
pub async fn notes_delete_entry(
    jade: State<'_, Arc<AppJade>>,
    path: String,
) -> Result<(), ErrorView> {
    blocking(&jade, move |jade| jade.delete_entry(&path)).await
}

#[tauri::command]
pub async fn notes_search(
    jade: State<'_, Arc<AppJade>>,
    query: String,
) -> Result<Vec<SearchHitView>, ErrorView> {
    blocking(&jade, move |jade| jade.search(&query)).await
}

#[tauri::command]
pub async fn notes_read_properties(
    jade: State<'_, Arc<AppJade>>,
    content: String,
) -> Result<NotePropertiesView, ErrorView> {
    blocking(&jade, move |jade| jade.read_properties(content)).await
}

#[tauri::command]
pub async fn notes_write_properties(
    jade: State<'_, Arc<AppJade>>,
    content: String,
    properties: Vec<PropertyView>,
) -> Result<String, ErrorView> {
    blocking(&jade, move |jade| {
        jade.write_properties(content, properties)
    })
    .await
}

#[tauri::command]
pub async fn notes_list_properties(
    jade: State<'_, Arc<AppJade>>,
) -> Result<Vec<PropertySummaryView>, ErrorView> {
    blocking(&jade, |jade| jade.list_properties()).await
}

#[tauri::command]
pub async fn notes_set_property_type(
    jade: State<'_, Arc<AppJade>>,
    name: String,
    kind: String,
) -> Result<(), ErrorView> {
    blocking(&jade, move |jade| jade.set_property_type(&name, &kind)).await
}

#[tauri::command]
pub async fn notes_query_base(
    jade: State<'_, Arc<AppJade>>,
    path: String,
    view: Option<String>,
) -> Result<BaseTableView, ErrorView> {
    blocking(&jade, move |jade| jade.query_base(&path, view)).await
}

#[tauri::command]
pub async fn notes_find_backlinks(
    jade: State<'_, Arc<AppJade>>,
    path: String,
) -> Result<Vec<BacklinkView>, ErrorView> {
    blocking(&jade, move |jade| jade.backlinks(&path)).await
}

#[tauri::command]
pub async fn notes_resolve_links(
    jade: State<'_, Arc<AppJade>>,
    targets: Vec<String>,
) -> Result<Vec<ResolvedLinkView>, ErrorView> {
    blocking(&jade, move |jade| jade.resolve_links(targets)).await
}

#[tauri::command]
pub async fn notes_follow_link(
    jade: State<'_, Arc<AppJade>>,
    target: String,
) -> Result<FollowedLinkView, ErrorView> {
    blocking(&jade, move |jade| jade.follow_link(&target)).await
}

#[tauri::command]
pub async fn notes_delete_vault(
    app: AppHandle,
    jade: State<'_, Arc<AppJade>>,
    path: String,
) -> Result<PreferencesView, ErrorView> {
    let container = super::app::app_capabilities(app)?.managed_vaults_folder;
    blocking(&jade, move |jade| {
        jade.delete_vault(&path, container.as_deref())
    })
    .await
}
