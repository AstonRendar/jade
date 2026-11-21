use std::collections::BTreeSet;

use jade_shared::application::use_case::UseCase;
use jade_sync::application::synchronize_vault::{
    ConflictCopy, SyncReport, SynchronizeVault, SynchronizeVaultError, SynchronizeVaultRequest,
};
use jade_sync::domain::port::{
    Conflict, Credentials, MergeOutcome, PushOutcome, ResolvedConflict, VersionControlError,
};
use jade_sync::domain::service::{ConflictResolution, FileChange};
use jade_sync::domain::value_object::{DeviceName, TrackedPath};
use mockall::predicate::eq;
use mockall::Sequence;

use crate::support::mocks::{MockClock, MockCredentialStore, MockDeviceInfo, MockVersionControl};
use crate::support::mothers::{SyncMother, TimestampMother, TrackedPathMother};

struct Ports {
    version_control: MockVersionControl,
    credentials: MockCredentialStore,
    clock: MockClock,
    device: MockDeviceInfo,
}

#[test]
fn only_commits_when_the_vault_has_no_remote() {
    // Arrange
    let mut ports = ports();
    expect_a_sync_commit_from_laptop(&mut ports.version_control);
    given_no_remote(&mut ports.version_control);

    // Act
    let report = synchronize(ports).expect("sync succeeds");

    // Assert
    assert_eq!(
        report,
        SyncReport {
            committed: Some(SyncMother::commit_id()),
            changed_paths: vec![],
            conflict_copies: vec![],
            pushed: false,
        }
    );
}

#[test]
fn fetches_merges_and_pushes_with_the_stored_token() {
    // Arrange
    let mut ports = ports();
    given_nothing_to_commit(&mut ports.version_control);
    given_a_remote_on_main(&mut ports.version_control);
    given_a_stored_token(&mut ports.credentials);
    expect_fetch_with_token(&mut ports.version_control, 1);
    given_merge_outcomes(&mut ports.version_control, vec![MergeOutcome::UpToDate]);
    expect_push_with_token(&mut ports.version_control, vec![PushOutcome::Pushed]);

    // Act
    let report = synchronize(ports).expect("sync succeeds");

    // Assert
    assert!(report.pushed);
    assert_eq!(report.committed, None);
}

#[test]
fn reports_paths_changed_by_a_fast_forward() {
    // Arrange
    let mut ports = ports();
    given_nothing_to_commit(&mut ports.version_control);
    given_a_remote_on_main(&mut ports.version_control);
    given_no_stored_token(&mut ports.credentials);
    given_fetch_succeeds(&mut ports.version_control);
    given_merge_outcomes(
        &mut ports.version_control,
        vec![MergeOutcome::FastForwarded {
            changed: vec![TrackedPathMother::of("ideas.md")],
        }],
    );
    given_push_outcomes(&mut ports.version_control, vec![PushOutcome::Pushed]);

    // Act
    let report = synchronize(ports).expect("sync succeeds");

    // Assert
    assert_eq!(
        report.changed_paths,
        vec![TrackedPathMother::of("ideas.md")]
    );
}

#[test]
fn resolves_conflicts_with_the_policy_and_concludes_the_merge() {
    // Arrange
    let mut ports = ports();
    given_nothing_to_commit(&mut ports.version_control);
    given_a_remote_on_main(&mut ports.version_control);
    given_no_stored_token(&mut ports.credentials);
    given_fetch_succeeds(&mut ports.version_control);
    given_merge_outcomes(
        &mut ports.version_control,
        vec![MergeOutcome::Diverged {
            changed: vec![TrackedPathMother::of("plan.md")],
            conflicts: vec![
                conflict("ideas.md", FileChange::Modified, FileChange::Modified),
                conflict("old.md", FileChange::Modified, FileChange::Deleted),
            ],
        }],
    );
    given_existing_paths(
        &mut ports.version_control,
        &["ideas.md", "old.md", "plan.md"],
    );
    given_now(&mut ports.clock);
    expect_merge_concluded_with(
        &mut ports.version_control,
        vec![
            ResolvedConflict {
                path: TrackedPathMother::of("ideas.md"),
                resolution: ConflictResolution::KeepLocalAndCopyRemote,
                copy: Some(TrackedPathMother::of("ideas (conflict 2026-11-21 2245).md")),
            },
            ResolvedConflict {
                path: TrackedPathMother::of("old.md"),
                resolution: ConflictResolution::KeepLocal,
                copy: None,
            },
        ],
    );
    given_push_outcomes(&mut ports.version_control, vec![PushOutcome::Pushed]);

    // Act
    let report = synchronize(ports).expect("sync succeeds");

    // Assert
    assert_eq!(
        report.conflict_copies,
        vec![ConflictCopy {
            original: TrackedPathMother::of("ideas.md"),
            copy: TrackedPathMother::of("ideas (conflict 2026-11-21 2245).md"),
        }]
    );
    assert_eq!(
        report.changed_paths,
        vec![
            TrackedPathMother::of("plan.md"),
            TrackedPathMother::of("ideas (conflict 2026-11-21 2245).md"),
        ]
    );
}

#[test]
fn never_overwrites_an_existing_file_with_a_conflict_copy() {
    // Arrange
    let mut ports = ports();
    given_nothing_to_commit(&mut ports.version_control);
    given_a_remote_on_main(&mut ports.version_control);
    given_no_stored_token(&mut ports.credentials);
    given_fetch_succeeds(&mut ports.version_control);
    given_merge_outcomes(
        &mut ports.version_control,
        vec![MergeOutcome::Diverged {
            changed: vec![],
            conflicts: vec![conflict(
                "ideas.md",
                FileChange::Modified,
                FileChange::Modified,
            )],
        }],
    );
    given_existing_paths(
        &mut ports.version_control,
        &["ideas.md", "ideas (conflict 2026-11-21 2245).md"],
    );
    given_now(&mut ports.clock);
    expect_merge_concluded_with(
        &mut ports.version_control,
        vec![ResolvedConflict {
            path: TrackedPathMother::of("ideas.md"),
            resolution: ConflictResolution::KeepLocalAndCopyRemote,
            copy: Some(TrackedPathMother::of(
                "ideas (conflict 2026-11-21 2245 2).md",
            )),
        }],
    );
    given_push_outcomes(&mut ports.version_control, vec![PushOutcome::Pushed]);

    // Act
    let report = synchronize(ports).expect("sync succeeds");

    // Assert
    assert_eq!(
        report.conflict_copies[0].copy,
        TrackedPathMother::of("ideas (conflict 2026-11-21 2245 2).md")
    );
}

#[test]
fn concludes_a_divergent_merge_even_without_conflicts() {
    // Arrange
    let mut ports = ports();
    given_nothing_to_commit(&mut ports.version_control);
    given_a_remote_on_main(&mut ports.version_control);
    given_no_stored_token(&mut ports.credentials);
    given_fetch_succeeds(&mut ports.version_control);
    given_merge_outcomes(
        &mut ports.version_control,
        vec![MergeOutcome::Diverged {
            changed: vec![TrackedPathMother::of("plan.md")],
            conflicts: vec![],
        }],
    );
    expect_merge_concluded_with(&mut ports.version_control, vec![]);
    given_push_outcomes(&mut ports.version_control, vec![PushOutcome::Pushed]);

    // Act
    let report = synchronize(ports).expect("sync succeeds");

    // Assert
    assert_eq!(report.changed_paths, vec![TrackedPathMother::of("plan.md")]);
}

#[test]
fn fetches_and_merges_again_when_the_remote_moved_during_push() {
    // Arrange
    let mut ports = ports();
    given_nothing_to_commit(&mut ports.version_control);
    given_a_remote_on_main(&mut ports.version_control);
    given_a_stored_token(&mut ports.credentials);
    expect_fetch_with_token(&mut ports.version_control, 2);
    given_merge_outcomes(
        &mut ports.version_control,
        vec![
            MergeOutcome::UpToDate,
            MergeOutcome::FastForwarded {
                changed: vec![TrackedPathMother::of("ideas.md")],
            },
        ],
    );
    expect_push_with_token(
        &mut ports.version_control,
        vec![PushOutcome::RemoteMoved, PushOutcome::Pushed],
    );

    // Act
    let report = synchronize(ports).expect("sync succeeds");

    // Assert
    assert!(report.pushed);
    assert_eq!(
        report.changed_paths,
        vec![TrackedPathMother::of("ideas.md")]
    );
}

#[test]
fn gives_up_when_the_remote_keeps_moving() {
    // Arrange
    let mut ports = ports();
    given_nothing_to_commit(&mut ports.version_control);
    given_a_remote_on_main(&mut ports.version_control);
    given_no_stored_token(&mut ports.credentials);
    given_fetch_succeeds(&mut ports.version_control);
    given_merge_outcomes(&mut ports.version_control, vec![MergeOutcome::UpToDate; 3]);
    given_push_outcomes(
        &mut ports.version_control,
        vec![PushOutcome::RemoteMoved; 3],
    );

    // Act
    let result = synchronize(ports);

    // Assert
    assert_eq!(result, Err(SynchronizeVaultError::PushRejected));
}

#[test]
fn does_not_retry_authentication_failures() {
    // Arrange
    let mut ports = ports();
    given_nothing_to_commit(&mut ports.version_control);
    given_a_remote_on_main(&mut ports.version_control);
    given_a_stored_token(&mut ports.credentials);
    given_fetch_fails_with(
        &mut ports.version_control,
        VersionControlError::AuthenticationFailed,
    );

    // Act
    let result = synchronize(ports);

    // Assert
    assert_eq!(
        result,
        Err(SynchronizeVaultError::VersionControl(
            VersionControlError::AuthenticationFailed
        ))
    );
}

#[test]
fn rejects_a_relative_working_directory() {
    // Arrange
    let ports = ports();
    let use_case = SynchronizeVault::new(
        ports.version_control,
        ports.credentials,
        ports.clock,
        ports.device,
    );

    // Act
    let result = use_case.execute(SynchronizeVaultRequest {
        working_directory: "relative/vault".to_string(),
    });

    // Assert
    assert!(matches!(
        result,
        Err(SynchronizeVaultError::InvalidWorkingDirectory(_))
    ));
}

fn ports() -> Ports {
    let mut device = MockDeviceInfo::new();
    device
        .expect_device_name()
        .returning(|| DeviceName::parse("laptop").expect("valid"));
    Ports {
        version_control: MockVersionControl::new(),
        credentials: MockCredentialStore::new(),
        clock: MockClock::new(),
        device,
    }
}

fn synchronize(ports: Ports) -> Result<SyncReport, SynchronizeVaultError> {
    SynchronizeVault::new(
        ports.version_control,
        ports.credentials,
        ports.clock,
        ports.device,
    )
    .execute(SynchronizeVaultRequest {
        working_directory: SyncMother::working_directory()
            .as_path()
            .display()
            .to_string(),
    })
}

fn conflict(path: &str, local: FileChange, remote: FileChange) -> Conflict {
    Conflict {
        path: TrackedPathMother::of(path),
        local,
        remote,
    }
}

fn expect_a_sync_commit_from_laptop(version_control: &mut MockVersionControl) {
    version_control
        .expect_commit_all()
        .withf(|dir, message| {
            *dir == SyncMother::working_directory() && message.as_str() == "Sync from laptop"
        })
        .times(1)
        .returning(|_, _| Ok(Some(SyncMother::commit_id())));
}

fn given_nothing_to_commit(version_control: &mut MockVersionControl) {
    version_control
        .expect_commit_all()
        .returning(|_, _| Ok(None));
}

fn given_no_remote(version_control: &mut MockVersionControl) {
    version_control.expect_remote().returning(|_| Ok(None));
}

fn given_a_remote_on_main(version_control: &mut MockVersionControl) {
    version_control
        .expect_remote()
        .returning(|_| Ok(Some(SyncMother::remote())));
    version_control
        .expect_current_branch()
        .returning(|_| Ok(SyncMother::main_branch()));
}

fn given_a_stored_token(credentials: &mut MockCredentialStore) {
    credentials
        .expect_load()
        .returning(|| Ok(Some(SyncMother::token())));
}

fn given_no_stored_token(credentials: &mut MockCredentialStore) {
    credentials.expect_load().returning(|| Ok(None));
}

fn given_fetch_succeeds(version_control: &mut MockVersionControl) {
    version_control
        .expect_fetch()
        .returning(|_, _, _, _| Ok(()));
}

fn given_fetch_fails_with(version_control: &mut MockVersionControl, error: VersionControlError) {
    version_control
        .expect_fetch()
        .times(1)
        .return_once(move |_, _, _, _| Err(error));
}

fn expect_fetch_with_token(version_control: &mut MockVersionControl, times: usize) {
    version_control
        .expect_fetch()
        .with(
            eq(SyncMother::working_directory()),
            eq(SyncMother::remote()),
            eq(SyncMother::main_branch()),
            eq(Credentials::Token(SyncMother::token())),
        )
        .times(times)
        .returning(|_, _, _, _| Ok(()));
}

fn given_merge_outcomes(version_control: &mut MockVersionControl, outcomes: Vec<MergeOutcome>) {
    let mut sequence = Sequence::new();
    for outcome in outcomes {
        version_control
            .expect_merge_remote()
            .times(1)
            .in_sequence(&mut sequence)
            .return_once(move |_, _| Ok(outcome));
    }
}

fn given_existing_paths(version_control: &mut MockVersionControl, paths: &[&str]) {
    let existing: BTreeSet<TrackedPath> = paths.iter().map(|p| TrackedPathMother::of(p)).collect();
    version_control
        .expect_existing_paths()
        .return_once(move |_| Ok(existing));
}

fn given_now(clock: &mut MockClock) {
    clock.expect_now().returning(TimestampMother::any);
}

fn expect_merge_concluded_with(
    version_control: &mut MockVersionControl,
    expected: Vec<ResolvedConflict>,
) {
    version_control
        .expect_conclude_merge()
        .withf(move |_, branch, resolutions, message| {
            *branch == SyncMother::main_branch()
                && resolutions == expected.as_slice()
                && message.as_str() == "Merge remote changes"
        })
        .times(1)
        .returning(|_, _, _, _| Ok(SyncMother::commit_id_of('b')));
}

fn given_push_outcomes(version_control: &mut MockVersionControl, outcomes: Vec<PushOutcome>) {
    let mut sequence = Sequence::new();
    for outcome in outcomes {
        version_control
            .expect_push()
            .times(1)
            .in_sequence(&mut sequence)
            .return_once(move |_, _, _, _| Ok(outcome));
    }
}

fn expect_push_with_token(version_control: &mut MockVersionControl, outcomes: Vec<PushOutcome>) {
    let mut sequence = Sequence::new();
    for outcome in outcomes {
        version_control
            .expect_push()
            .with(
                eq(SyncMother::working_directory()),
                eq(SyncMother::remote()),
                eq(SyncMother::main_branch()),
                eq(Credentials::Token(SyncMother::token())),
            )
            .times(1)
            .in_sequence(&mut sequence)
            .return_once(move |_, _, _, _| Ok(outcome));
    }
}
