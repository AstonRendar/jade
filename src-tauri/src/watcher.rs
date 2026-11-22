use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use crate::errors::ErrorView;
use crate::view_models::ChangedPathsView;

pub const NOTES_CHANGED: &str = "notes://changed";
const QUIET_PERIOD: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct VaultWatcher {
    active: Mutex<Option<RecommendedWatcher>>,
}

impl VaultWatcher {
    pub fn watch(&self, app: AppHandle, root: &str) -> Result<(), ErrorView> {
        let (sender, receiver) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(sender)
            .map_err(|e| ErrorView::unexpected(e.to_string()))?;
        watcher
            .watch(Path::new(root), RecursiveMode::Recursive)
            .map_err(|e| ErrorView::unexpected(e.to_string()))?;
        let root = PathBuf::from(root);
        thread::spawn(move || forward_changes(&app, &root, &receiver));
        *self
            .active
            .lock()
            .map_err(|_| ErrorView::unexpected("lock poisoned"))? = Some(watcher);
        Ok(())
    }
}

fn forward_changes(app: &AppHandle, root: &Path, receiver: &Receiver<notify::Result<Event>>) {
    while let Ok(first) = receiver.recv() {
        let mut changed = BTreeSet::new();
        collect(root, first, &mut changed);
        loop {
            match receiver.recv_timeout(QUIET_PERIOD) {
                Ok(event) => collect(root, event, &mut changed),
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        if !changed.is_empty() {
            let _ = app.emit(
                NOTES_CHANGED,
                ChangedPathsView {
                    paths: changed.into_iter().collect(),
                },
            );
        }
    }
}

fn collect(root: &Path, event: notify::Result<Event>, changed: &mut BTreeSet<String>) {
    let Ok(event) = event else {
        return;
    };
    changed.extend(
        event
            .paths
            .iter()
            .filter_map(|path| visible_relative(root, path)),
    );
}

pub fn visible_relative(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let segments: Vec<String> = relative
        .iter()
        .map(|segment| segment.to_string_lossy().into_owned())
        .collect();
    let hidden = segments.iter().any(|segment| segment.starts_with('.'));
    (!segments.is_empty() && !hidden).then(|| segments.join("/"))
}
