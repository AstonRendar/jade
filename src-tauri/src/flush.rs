use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use ts_rs::TS;

pub const FLUSH_REQUESTED: &str = "editor://flush-requested";
pub const FLUSH_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FlushRequestView {
    pub id: u64,
}

#[derive(Default)]
pub struct FlushCoordinator {
    next_id: AtomicU64,
    pending: Mutex<HashMap<u64, Sender<()>>>,
}

impl FlushCoordinator {
    pub fn request(&self, announce: impl FnOnce(u64), timeout: Duration) -> bool {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut pending) = self.pending.lock() {
            pending.insert(id, sender);
        }
        announce(id);
        let acknowledged = receiver.recv_timeout(timeout).is_ok();
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(&id);
        }
        acknowledged
    }

    pub fn acknowledge(&self, id: u64) {
        if let Some(sender) = self.pending.lock().ok().and_then(|mut p| p.remove(&id)) {
            let _ = sender.send(());
        }
    }
}
