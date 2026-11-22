use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use jade_lib::errors::ErrorView;
use jade_lib::view_models::{SyncReportView, VaultView};
use jade_lib::wiring::Jade;
use jade_sync::infrastructure::InMemoryCredentialStore;
use tempfile::TempDir;

use crate::doubles::{FixedClock, NamedDevice};

pub type TestApp = Jade<InMemoryCredentialStore, FixedClock, NamedDevice>;

#[derive(cucumber::World)]
#[world(init = Self::new)]
pub struct AppWorld {
    sandbox: TempDir,
    apps: HashMap<String, TestApp>,
    pub remote: Option<PathBuf>,
    pub opened: HashMap<String, VaultView>,
    pub errors: HashMap<String, ErrorView>,
    pub reports: HashMap<String, SyncReportView>,
}

impl fmt::Debug for AppWorld {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppWorld")
            .field("devices", &self.apps.keys().collect::<Vec<_>>())
            .field("errors", &self.errors)
            .finish()
    }
}

impl AppWorld {
    fn new() -> Self {
        Self {
            sandbox: tempfile::tempdir().expect("sandbox is created"),
            apps: HashMap::new(),
            remote: None,
            opened: HashMap::new(),
            errors: HashMap::new(),
            reports: HashMap::new(),
        }
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.sandbox.path().join(relative)
    }

    pub fn app(&mut self, device: &str) -> &TestApp {
        let preferences = self.path(&format!("config/{device}/preferences.json"));
        self.apps.entry(device.to_string()).or_insert_with(|| {
            Jade::new(
                preferences,
                InMemoryCredentialStore::default(),
                FixedClock,
                NamedDevice(device.to_string()),
            )
        })
    }

    pub fn device_folder(&self, device: &str) -> PathBuf {
        let folder = self.path(&format!("devices/{device}"));
        std::fs::create_dir_all(&folder).expect("device folder is created");
        folder
    }

    pub fn remote_url(&self) -> String {
        format!(
            "file://{}",
            self.remote.as_ref().expect("remote exists").display()
        )
    }

    pub fn record<T>(&mut self, device: &str, result: Result<T, ErrorView>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.errors.insert(device.to_string(), error);
                None
            }
        }
    }
}
