use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

const MANAGED_VAULTS_FOLDER: &str = "vaults";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CapabilitiesView {
    pub can_open_folders: bool,
    pub managed_vaults_folder: Option<String>,
}

pub fn capabilities(is_mobile: bool, data_dir: &Path) -> CapabilitiesView {
    CapabilitiesView {
        can_open_folders: !is_mobile,
        managed_vaults_folder: is_mobile
            .then(|| data_dir.join(MANAGED_VAULTS_FOLDER).display().to_string()),
    }
}
