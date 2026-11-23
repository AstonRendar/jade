pub mod commands;
pub mod errors;
pub mod flush;
pub mod platform;
pub mod scheduler;
pub mod view_models;
pub mod watcher;
pub mod wiring;

use std::sync::Arc;

use jade_sync::infrastructure::{HostnameDeviceInfo, KeyringCredentialStore, SystemClock};
use tauri::Manager;

use crate::commands::{app as app_commands, editor, notes, preferences, sync, AppJade};
use crate::flush::FlushCoordinator;
use crate::watcher::VaultWatcher;
use crate::wiring::Jade;

const PREFERENCES_FILE: &str = "preferences.json";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "android")]
            jade_sync::infrastructure::SystemCertificates::trust_android_store();
            let preferences_file = app.path().app_config_dir()?.join(PREFERENCES_FILE);
            let jade: Arc<AppJade> = Arc::new(Jade::new(
                preferences_file,
                KeyringCredentialStore::new(&app.config().identifier),
                SystemClock,
                HostnameDeviceInfo,
            ));
            app.manage(Arc::clone(&jade));
            let flush = Arc::new(FlushCoordinator::default());
            app.manage(Arc::clone(&flush));
            app.manage(VaultWatcher::default());
            scheduler::spawn(app.handle().clone(), jade, flush);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_commands::app_capabilities,
            notes::notes_open_vault,
            notes::notes_create_vault,
            notes::notes_list_entries,
            notes::notes_read_note,
            notes::notes_save_note,
            notes::notes_create_note,
            notes::notes_create_folder,
            notes::notes_rename_entry,
            notes::notes_delete_entry,
            notes::notes_delete_vault,
            notes::notes_search,
            notes::notes_query_base,
            notes::notes_find_backlinks,
            notes::notes_resolve_links,
            notes::notes_follow_link,
            sync::sync_synchronize,
            sync::sync_get_status,
            sync::sync_init_repository,
            sync::sync_connect_remote,
            sync::sync_clone_repository,
            sync::sync_list_note_history,
            sync::sync_read_note_revision,
            sync::sync_store_access_token,
            sync::sync_has_access_token,
            preferences::preferences_get,
            preferences::preferences_update,
            preferences::preferences_forget_vault,
            editor::editor_flush_completed,
        ])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("Jade failed to start: {error}");
        std::process::exit(1);
    }
}
