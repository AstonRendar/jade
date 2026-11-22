use std::sync::Arc;

use tauri::State;

use super::{blocking, AppJade};
use crate::errors::ErrorView;
use crate::view_models::PreferencesView;

#[tauri::command]
pub async fn preferences_get(jade: State<'_, Arc<AppJade>>) -> Result<PreferencesView, ErrorView> {
    blocking(&jade, |jade| jade.preferences()).await
}

#[tauri::command]
pub async fn preferences_update(
    jade: State<'_, Arc<AppJade>>,
    locale: Option<String>,
    theme: String,
    auto_sync_minutes: u32,
) -> Result<PreferencesView, ErrorView> {
    blocking(&jade, move |jade| {
        jade.update_preferences(locale, &theme, auto_sync_minutes)
    })
    .await
}

#[tauri::command]
pub async fn preferences_forget_vault(
    jade: State<'_, Arc<AppJade>>,
    path: String,
) -> Result<PreferencesView, ErrorView> {
    blocking(&jade, move |jade| jade.forget_vault(&path)).await
}
