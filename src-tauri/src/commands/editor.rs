use std::sync::Arc;

use tauri::State;

use crate::flush::FlushCoordinator;

#[tauri::command]
pub fn editor_flush_completed(flush: State<'_, Arc<FlushCoordinator>>, id: u64) {
    flush.acknowledge(id);
}
