use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use jade_shared::application::use_case::UseCase;
use jade_sync::application::get_repository_status::RepositoryState;
use jade_sync::application::init_repository::InitRepositoryError;
use jade_sync::application::synchronize_vault::{
    SyncReport, SynchronizeVault, SynchronizeVaultRequest,
};
use jade_sync::domain::port::Revision;
use jade_sync::infrastructure::{Git2VersionControl, InMemoryCredentialStore};
use tempfile::TempDir;

use crate::doubles::{FixedClock, NamedDevice};

#[derive(Debug, cucumber::World)]
#[world(init = Self::new)]
pub struct SyncWorld {
    sandbox: TempDir,
    pub remote: Option<PathBuf>,
    pub last_report: Option<SyncReport>,
    pub pushed_heads: Vec<git2::Oid>,
    pub status: Option<RepositoryState>,
    pub history: Vec<Revision>,
    pub init_result: Option<Result<(), InitRepositoryError>>,
    devices: HashMap<String, PathBuf>,
}

impl SyncWorld {
    fn new() -> Self {
        Self {
            sandbox: tempfile::tempdir().expect("sandbox is created"),
            remote: None,
            last_report: None,
            pushed_heads: Vec::new(),
            status: None,
            history: Vec::new(),
            init_result: None,
            devices: HashMap::new(),
        }
    }

    pub fn sandbox_path(&self, name: &str) -> PathBuf {
        self.sandbox.path().join(name)
    }

    pub fn add_device(&mut self, name: &str) -> String {
        let dir = self.sandbox_path(&format!("devices/{name}"));
        fs::create_dir_all(&dir).expect("device folder is created");
        self.devices.insert(name.to_string(), dir.clone());
        dir.display().to_string()
    }

    pub fn device_dir(&self, name: &str) -> &PathBuf {
        self.devices.get(name).expect("device exists")
    }

    pub fn device_working_directory(&self, name: &str) -> String {
        self.device_dir(name).display().to_string()
    }

    pub fn remote_dir(&self) -> &PathBuf {
        self.remote.as_ref().expect("remote exists")
    }

    pub fn remote_url(&self) -> String {
        format!("file://{}", self.remote_dir().display())
    }

    pub fn synchronize(&mut self, device: &str) -> SyncReport {
        let use_case = SynchronizeVault::new(
            Git2VersionControl,
            InMemoryCredentialStore::default(),
            FixedClock,
            NamedDevice(device.to_string()),
        );
        let report = use_case
            .execute(SynchronizeVaultRequest {
                working_directory: self.device_working_directory(device),
            })
            .expect("sync succeeds");
        if report.pushed {
            self.pushed_heads.push(self.remote_head().id);
        }
        report
    }

    pub fn remote_head(&self) -> RemoteHead {
        let repository = git2::Repository::open_bare(self.remote_dir()).expect("remote opens");
        let id = repository
            .refname_to_id("refs/heads/main")
            .expect("remote has main");
        let commit = repository.find_commit(id).expect("head commit exists");
        RemoteHead {
            id,
            summary: commit
                .summary()
                .ok()
                .flatten()
                .unwrap_or_default()
                .to_string(),
            parent_count: commit.parent_count(),
        }
    }

    pub fn remote_descends_from(&self, ancestor: git2::Oid) -> bool {
        let repository = git2::Repository::open_bare(self.remote_dir()).expect("remote opens");
        let head = self.remote_head().id;
        head == ancestor
            || repository
                .graph_descendant_of(head, ancestor)
                .expect("ancestry is computed")
    }
}

pub struct RemoteHead {
    pub id: git2::Oid,
    pub summary: String,
    pub parent_count: usize,
}
