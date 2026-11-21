use jade_sync::domain::value_object::{
    AccessToken, BranchName, CommitId, RemoteUrl, WorkingDirectory,
};

pub const VAULT_DIR: &str = "/vaults/notes";
pub const REMOTE: &str = "https://git.example.com/team/vault.git";

pub struct SyncMother;

impl SyncMother {
    pub fn working_directory() -> WorkingDirectory {
        WorkingDirectory::parse(VAULT_DIR).expect("valid working directory")
    }

    pub fn remote() -> RemoteUrl {
        RemoteUrl::parse(REMOTE).expect("valid remote")
    }

    pub fn main_branch() -> BranchName {
        BranchName::parse("main").expect("valid branch")
    }

    pub fn commit_id() -> CommitId {
        Self::commit_id_of('a')
    }

    pub fn commit_id_of(digit: char) -> CommitId {
        CommitId::parse(&digit.to_string().repeat(40)).expect("valid commit id")
    }

    pub fn token() -> AccessToken {
        AccessToken::new("token-123").expect("valid token")
    }
}
