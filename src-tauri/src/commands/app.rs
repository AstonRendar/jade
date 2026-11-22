use tauri::{AppHandle, Manager};

use crate::errors::ErrorView;
use crate::platform::{capabilities, CapabilitiesView};

#[tauri::command]
pub fn app_capabilities(app: AppHandle) -> Result<CapabilitiesView, ErrorView> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| ErrorView::unexpected(error.to_string()))?;
    Ok(capabilities(cfg!(mobile), &data_dir))
}
