pub mod app;
pub mod editor;
pub mod notes;
pub mod preferences;
pub mod sync;

use std::sync::Arc;

use jade_sync::infrastructure::{HostnameDeviceInfo, KeyringCredentialStore, SystemClock};

use tauri::{AppHandle, Manager};

use crate::errors::ErrorView;
use crate::wiring::Jade;

pub type AppJade = Jade<KeyringCredentialStore, SystemClock, HostnameDeviceInfo>;

pub async fn blocking<T, F>(jade: &Arc<AppJade>, task: F) -> Result<T, ErrorView>
where
    T: Send + 'static,
    F: FnOnce(&AppJade) -> Result<T, ErrorView> + Send + 'static,
{
    let jade = Arc::clone(jade);
    tauri::async_runtime::spawn_blocking(move || task(&jade))
        .await
        .map_err(|error| ErrorView::unexpected(error.to_string()))?
}

pub fn expose_vault_files(app: &AppHandle, vault: &str) -> Result<(), ErrorView> {
    app.asset_protocol_scope()
        .allow_directory(vault, true)
        .map_err(|error| ErrorView::unexpected(error.to_string()))
}

pub fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = text.get(index + 1..index + 3)?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}
