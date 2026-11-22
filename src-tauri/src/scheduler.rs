use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tauri::AppHandle;

use crate::commands::sync::synchronize_and_notify;
use crate::commands::AppJade;

const TICK: Duration = Duration::from_secs(30);

pub fn spawn(app: AppHandle, jade: Arc<AppJade>) {
    thread::spawn(move || {
        let mut last_sync = Instant::now();
        loop {
            thread::sleep(TICK);
            if is_due(&jade, last_sync) {
                let _ = synchronize_and_notify(&app, &jade);
                last_sync = Instant::now();
            }
        }
    });
}

fn is_due(jade: &AppJade, last_sync: Instant) -> bool {
    let minutes = jade
        .preferences()
        .map(|preferences| preferences.auto_sync_minutes)
        .unwrap_or(0);
    let interval = Duration::from_secs(u64::from(minutes) * 60);
    minutes > 0
        && last_sync.elapsed() >= interval
        && jade
            .repository_status()
            .is_ok_and(|status| status.is_repository)
}
