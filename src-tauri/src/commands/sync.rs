use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use super::{blocking, AppJade};
use crate::errors::ErrorView;
use crate::flush::{FlushCoordinator, FlushRequestView, FLUSH_REQUESTED, FLUSH_TIMEOUT};
use crate::view_models::{RepositoryStatusView, RevisionView, SyncReportView, VaultView};
use crate::watcher::VaultWatcher;

pub const SYNC_STARTED: &str = "sync://started";
pub const SYNC_FINISHED: &str = "sync://finished";
pub const SYNC_FAILED: &str = "sync://failed";

pub fn synchronize_and_notify(
    app: &AppHandle,
    jade: &AppJade,
    flush: &FlushCoordinator,
) -> Result<SyncReportView, ErrorView> {
    flush.request(
        |id| {
            let _ = app.emit(FLUSH_REQUESTED, FlushRequestView { id });
        },
        FLUSH_TIMEOUT,
    );
    let _ = app.emit(SYNC_STARTED, ());
    let result = jade.synchronize();
    let _ = match &result {
        Ok(report) => app.emit(SYNC_FINISHED, report),
        Err(error) => app.emit(SYNC_FAILED, error),
    };
    result
}

#[tauri::command]
pub async fn sync_synchronize(
    app: AppHandle,
    jade: State<'_, Arc<AppJade>>,
    flush: State<'_, Arc<FlushCoordinator>>,
) -> Result<SyncReportView, ErrorView> {
    let flush = Arc::clone(&flush);
    blocking(&jade, move |jade| {
        synchronize_and_notify(&app, jade, &flush)
    })
    .await
}

#[tauri::command]
pub async fn sync_get_status(
    jade: State<'_, Arc<AppJade>>,
) -> Result<RepositoryStatusView, ErrorView> {
    blocking(&jade, |jade| jade.repository_status()).await
}

#[tauri::command]
pub async fn sync_init_repository(jade: State<'_, Arc<AppJade>>) -> Result<(), ErrorView> {
    blocking(&jade, |jade| jade.init_repository()).await
}

#[tauri::command]
pub async fn sync_connect_remote(
    jade: State<'_, Arc<AppJade>>,
    url: String,
) -> Result<(), ErrorView> {
    blocking(&jade, move |jade| jade.connect_remote(&url)).await
}

#[tauri::command]
pub async fn sync_clone_repository(
    app: AppHandle,
    jade: State<'_, Arc<AppJade>>,
    watcher: State<'_, VaultWatcher>,
    url: String,
    parent: String,
) -> Result<VaultView, ErrorView> {
    let vault = blocking(&jade, move |jade| jade.clone_repository(&url, &parent)).await?;
    watcher.watch(app, &vault.path)?;
    Ok(vault)
}

#[tauri::command]
pub async fn sync_list_note_history(
    jade: State<'_, Arc<AppJade>>,
    path: String,
) -> Result<Vec<RevisionView>, ErrorView> {
    blocking(&jade, move |jade| jade.note_history(&path)).await
}

#[tauri::command]
pub async fn sync_read_note_revision(
    jade: State<'_, Arc<AppJade>>,
    path: String,
    commit: String,
) -> Result<String, ErrorView> {
    blocking(&jade, move |jade| jade.note_revision(&path, &commit)).await
}

#[tauri::command]
pub async fn sync_store_access_token(
    jade: State<'_, Arc<AppJade>>,
    token: String,
) -> Result<(), ErrorView> {
    blocking(&jade, move |jade| jade.store_access_token(&token)).await
}

#[tauri::command]
pub async fn sync_has_access_token(jade: State<'_, Arc<AppJade>>) -> Result<bool, ErrorView> {
    blocking(&jade, |jade| jade.has_access_token()).await
}
