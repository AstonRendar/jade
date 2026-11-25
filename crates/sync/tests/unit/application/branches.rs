use jade_shared::application::use_case::UseCase;
use jade_sync::application::list_branches::{
    BranchList, ListBranches, ListBranchesError, ListBranchesRequest,
};
use jade_sync::application::switch_branch::{SwitchBranch, SwitchBranchError, SwitchBranchRequest};
use jade_sync::domain::port::{CredentialStoreError, VersionControlError};
use jade_sync::domain::value_object::{BranchName, DeviceName};
use mockall::predicate::eq;

use crate::support::mocks::{MockCredentialStore, MockDeviceInfo, MockVersionControl};
use crate::support::mothers::{SyncMother, VAULT_DIR};

fn branch(name: &str) -> BranchName {
    BranchName::parse(name).expect("valid branch")
}

#[test]
fn lists_the_branches_after_refreshing_them_from_the_remote() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_a_remote(&mut version_control);
    version_control
        .expect_fetch_branches()
        .times(1)
        .returning(|_, _, _| Ok(()));
    given_the_branches(&mut version_control);

    // Act
    let list = ListBranches::new(version_control, given_no_token()).execute(list_request());

    // Assert
    assert_eq!(
        list,
        Ok(BranchList {
            current: branch("main"),
            branches: vec![branch("develop"), branch("main")],
        })
    );
}

#[test]
fn lists_the_known_branches_when_offline_or_without_remote() {
    // Arrange
    let mut offline = MockVersionControl::new();
    given_a_remote(&mut offline);
    offline
        .expect_fetch_branches()
        .returning(|_, _, _| Err(VersionControlError::Network("down".to_string())));
    given_the_branches(&mut offline);
    let mut local = MockVersionControl::new();
    local.expect_remote().returning(|_| Ok(None));
    given_the_branches(&mut local);

    // Act
    let results = [
        ListBranches::new(offline, given_no_token()).execute(list_request()),
        ListBranches::new(local, MockCredentialStore::new()).execute(list_request()),
    ];

    // Assert
    assert!(results.iter().all(Result::is_ok));
}

#[test]
fn reports_authentication_and_credential_failures() {
    // Arrange
    let mut rejected = MockVersionControl::new();
    given_a_remote(&mut rejected);
    rejected
        .expect_fetch_branches()
        .returning(|_, _, _| Err(VersionControlError::AuthenticationFailed));
    let mut locked = MockVersionControl::new();
    given_a_remote(&mut locked);
    let mut store = MockCredentialStore::new();
    store
        .expect_load()
        .returning(|| Err(CredentialStoreError("locked".to_string())));

    // Act
    let rejected = ListBranches::new(rejected, given_no_token()).execute(list_request());
    let locked = ListBranches::new(locked, store).execute(list_request());
    let invalid = ListBranches::new(MockVersionControl::new(), MockCredentialStore::new()).execute(
        ListBranchesRequest {
            working_directory: "relative".to_string(),
        },
    );

    // Assert
    assert_eq!(
        rejected,
        Err(ListBranchesError::VersionControl(
            VersionControlError::AuthenticationFailed
        ))
    );
    assert_eq!(
        locked,
        Err(ListBranchesError::Credentials(CredentialStoreError(
            "locked".to_string()
        )))
    );
    assert!(matches!(
        invalid,
        Err(ListBranchesError::InvalidWorkingDirectory(_))
    ));
}

#[test]
fn commits_pending_changes_then_switches_to_the_branch() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_the_current_branch(&mut version_control, "main");
    expect_pending_changes_committed(&mut version_control);
    given_a_remote(&mut version_control);
    version_control
        .expect_fetch_branches()
        .returning(|_, _, _| Ok(()));
    version_control
        .expect_switch_branch()
        .with(eq(SyncMother::working_directory()), eq(branch("develop")))
        .times(1)
        .returning(|_, _| Ok(()));

    // Act
    let result = SwitchBranch::new(version_control, given_no_token(), given_a_device())
        .execute(switch_request("develop"));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn does_nothing_when_already_on_the_branch() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_the_current_branch(&mut version_control, "develop");

    // Act
    let result = SwitchBranch::new(
        version_control,
        MockCredentialStore::new(),
        MockDeviceInfo::new(),
    )
    .execute(switch_request("develop"));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn reports_unknown_branches_and_invalid_names() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_the_current_branch(&mut version_control, "main");
    expect_pending_changes_committed(&mut version_control);
    version_control.expect_remote().returning(|_| Ok(None));
    version_control
        .expect_switch_branch()
        .returning(|_, name| Err(VersionControlError::BranchNotFound(name.clone())));
    let mut rejected = MockVersionControl::new();
    given_the_current_branch(&mut rejected, "main");
    expect_pending_changes_committed(&mut rejected);
    given_a_remote(&mut rejected);
    rejected
        .expect_fetch_branches()
        .returning(|_, _, _| Err(VersionControlError::AuthenticationRequired));
    let mut locked = MockVersionControl::new();
    given_the_current_branch(&mut locked, "main");
    expect_pending_changes_committed(&mut locked);
    given_a_remote(&mut locked);
    let mut store = MockCredentialStore::new();
    store
        .expect_load()
        .returning(|| Err(CredentialStoreError("locked".to_string())));

    // Act
    let missing = SwitchBranch::new(
        version_control,
        MockCredentialStore::new(),
        given_a_device(),
    )
    .execute(switch_request("nope"));
    let invalid = SwitchBranch::new(
        MockVersionControl::new(),
        MockCredentialStore::new(),
        MockDeviceInfo::new(),
    )
    .execute(switch_request("bad name"));
    let rejected = SwitchBranch::new(rejected, given_no_token(), given_a_device())
        .execute(switch_request("develop"));
    let locked =
        SwitchBranch::new(locked, store, given_a_device()).execute(switch_request("develop"));
    let relative = SwitchBranch::new(
        MockVersionControl::new(),
        MockCredentialStore::new(),
        MockDeviceInfo::new(),
    )
    .execute(SwitchBranchRequest {
        working_directory: "relative".to_string(),
        branch: "develop".to_string(),
    });

    // Assert
    assert_eq!(
        missing,
        Err(SwitchBranchError::VersionControl(
            VersionControlError::BranchNotFound(branch("nope"))
        ))
    );
    assert!(matches!(invalid, Err(SwitchBranchError::InvalidBranch(_))));
    assert_eq!(
        rejected,
        Err(SwitchBranchError::VersionControl(
            VersionControlError::AuthenticationRequired
        ))
    );
    assert_eq!(
        locked,
        Err(SwitchBranchError::Credentials(CredentialStoreError(
            "locked".to_string()
        )))
    );
    assert!(matches!(
        relative,
        Err(SwitchBranchError::InvalidWorkingDirectory(_))
    ));
}

fn list_request() -> ListBranchesRequest {
    ListBranchesRequest {
        working_directory: VAULT_DIR.to_string(),
    }
}

fn switch_request(name: &str) -> SwitchBranchRequest {
    SwitchBranchRequest {
        working_directory: VAULT_DIR.to_string(),
        branch: name.to_string(),
    }
}

fn given_a_remote(version_control: &mut MockVersionControl) {
    version_control
        .expect_remote()
        .returning(|_| Ok(Some(SyncMother::remote())));
}

fn given_the_branches(version_control: &mut MockVersionControl) {
    version_control
        .expect_current_branch()
        .returning(|_| Ok(BranchName::parse("main").expect("valid")));
    version_control.expect_branches().returning(|_| {
        Ok(vec![
            BranchName::parse("develop").expect("valid"),
            BranchName::parse("main").expect("valid"),
        ])
    });
}

fn given_the_current_branch(version_control: &mut MockVersionControl, name: &'static str) {
    version_control
        .expect_current_branch()
        .returning(move |_| Ok(BranchName::parse(name).expect("valid")));
}

fn expect_pending_changes_committed(version_control: &mut MockVersionControl) {
    version_control
        .expect_commit_all()
        .times(1)
        .returning(|_, _| Ok(None));
}

fn given_no_token() -> MockCredentialStore {
    let mut store = MockCredentialStore::new();
    store.expect_load().returning(|| Ok(None));
    store
}

fn given_a_device() -> MockDeviceInfo {
    let mut device = MockDeviceInfo::new();
    device
        .expect_device_name()
        .returning(|| DeviceName::parse("laptop").expect("valid device"));
    device
}
