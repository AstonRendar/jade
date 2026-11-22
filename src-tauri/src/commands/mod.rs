pub mod editor;
pub mod notes;
pub mod preferences;
pub mod sync;

use std::sync::Arc;

use jade_sync::infrastructure::{HostnameDeviceInfo, KeyringCredentialStore, SystemClock};

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
