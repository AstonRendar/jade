use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{
    Commit, ErrorCode, FetchOptions, Index, IndexAddOption, Oid, PushOptions, Repository,
    RepositoryInitOptions, Signature, Sort, StatusOptions, Tree,
};

use super::git2_authentication::{network_or_backend, Git2Authentication};
use crate::domain::port::{
    Conflict, Credentials, MergeOutcome, PushOutcome, RepositoryStatus, ResolvedConflict, Revision,
    VersionControl, VersionControlError,
};
use crate::domain::service::{ConflictResolution, FileChange};
use crate::domain::value_object::{
    BranchName, CommitId, CommitMessage, RemoteUrl, TrackedPath, WorkingDirectory,
};

const ORIGIN: &str = "origin";
const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;
const FALLBACK_AUTHOR_NAME: &str = "Jade";
const FALLBACK_AUTHOR_EMAIL: &str = "jade@localhost";

#[derive(Debug, Clone, Copy, Default)]
pub struct Git2VersionControl;

struct ConflictBlobs {
    local: Option<Oid>,
    remote: Option<Oid>,
}

impl VersionControl for Git2VersionControl {
    fn is_repository(&self, dir: &WorkingDirectory) -> Result<bool, VersionControlError> {
        Ok(Repository::open(dir.as_path()).is_ok())
    }

    fn init(&self, dir: &WorkingDirectory, branch: &BranchName) -> Result<(), VersionControlError> {
        let mut options = RepositoryInitOptions::new();
        options.initial_head(branch.as_str());
        let repository = Repository::init_opts(dir.as_path(), &options).map_err(backend)?;
        configure(&repository)
    }

    fn clone_from(
        &self,
        url: &RemoteUrl,
        into: &WorkingDirectory,
        credentials: &Credentials,
    ) -> Result<(), VersionControlError> {
        let authentication = Git2Authentication::new(credentials);
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(authentication.callbacks());
        let repository = RepoBuilder::new()
            .fetch_options(fetch_options)
            .clone(url.network_url(), into.as_path())
            .map_err(|error| authentication.error(error))?;
        repository
            .remote_set_url(ORIGIN, url.as_str())
            .map_err(backend)?;
        configure(&repository)
    }

    fn current_branch(&self, dir: &WorkingDirectory) -> Result<BranchName, VersionControlError> {
        current_branch(&open(dir)?)
    }

    fn remote(&self, dir: &WorkingDirectory) -> Result<Option<RemoteUrl>, VersionControlError> {
        origin_url(&open(dir)?)
    }

    fn set_remote(
        &self,
        dir: &WorkingDirectory,
        url: &RemoteUrl,
    ) -> Result<(), VersionControlError> {
        let repository = open(dir)?;
        if repository.find_remote(ORIGIN).is_ok() {
            repository.remote_set_url(ORIGIN, url.as_str())
        } else {
            repository.remote(ORIGIN, url.as_str()).map(|_| ())
        }
        .map_err(backend)
    }

    fn commit_all(
        &self,
        dir: &WorkingDirectory,
        message: &CommitMessage,
    ) -> Result<Option<CommitId>, VersionControlError> {
        let repository = open(dir)?;
        let mut index = repository.index().map_err(backend)?;
        stage_everything(&repository, &mut index)?;
        let tree_id = index.write_tree().map_err(backend)?;
        let parent = head_commit(&repository)?;
        let unchanged = match &parent {
            Some(parent) => parent.tree_id() == tree_id,
            None => index.is_empty(),
        };
        if unchanged {
            return Ok(None);
        }
        let tree = repository.find_tree(tree_id).map_err(backend)?;
        let parents: Vec<&Commit> = parent.iter().collect();
        let author = signature(&repository)?;
        let id = repository
            .commit(
                Some("HEAD"),
                &author,
                &author,
                message.as_str(),
                &tree,
                &parents,
            )
            .map_err(backend)?;
        commit_id(id).map(Some)
    }

    fn fetch(
        &self,
        dir: &WorkingDirectory,
        url: &RemoteUrl,
        branch: &BranchName,
        credentials: &Credentials,
    ) -> Result<(), VersionControlError> {
        let repository = open(dir)?;
        let authentication = Git2Authentication::new(credentials);
        let mut options = FetchOptions::new();
        options.remote_callbacks(authentication.callbacks());
        let refspec = format!("+{}:{}", local_ref(branch), tracking_ref(branch));
        repository
            .remote_anonymous(url.network_url())
            .and_then(|mut remote| remote.fetch(&[refspec], Some(&mut options), None))
            .map_err(|error| authentication.error(error))
    }

    fn merge_remote(
        &self,
        dir: &WorkingDirectory,
        branch: &BranchName,
    ) -> Result<MergeOutcome, VersionControlError> {
        let repository = open(dir)?;
        let Ok(remote_ref) = repository.find_reference(&tracking_ref(branch)) else {
            return Ok(MergeOutcome::UpToDate);
        };
        let theirs = repository
            .reference_to_annotated_commit(&remote_ref)
            .map_err(backend)?;
        let (analysis, _) = repository.merge_analysis(&[&theirs]).map_err(backend)?;
        if analysis.is_up_to_date() {
            return Ok(MergeOutcome::UpToDate);
        }
        let their_commit = repository.find_commit(theirs.id()).map_err(backend)?;
        if analysis.is_fast_forward() || analysis.is_unborn() {
            return fast_forward(&repository, branch, &their_commit);
        }
        let base_tree = merge_base_tree(&repository, &their_commit)?;
        repository.merge(&[&theirs], None, None).map_err(backend)?;
        let conflicts = read_conflicts(&repository.index().map_err(backend)?)?;
        let conflicted: BTreeSet<&TrackedPath> = conflicts.iter().map(|c| &c.path).collect();
        let changed = changed_paths(&repository, Some(&base_tree), &tree_of(&their_commit)?)?
            .into_iter()
            .filter(|path| !conflicted.contains(path))
            .collect();
        Ok(MergeOutcome::Diverged { changed, conflicts })
    }

    fn existing_paths(
        &self,
        dir: &WorkingDirectory,
    ) -> Result<BTreeSet<TrackedPath>, VersionControlError> {
        let mut paths = BTreeSet::new();
        collect_files(dir.as_path(), dir.as_path(), &mut paths).map_err(io_error)?;
        Ok(paths)
    }

    fn conclude_merge(
        &self,
        dir: &WorkingDirectory,
        branch: &BranchName,
        resolutions: &[ResolvedConflict],
        message: &CommitMessage,
    ) -> Result<CommitId, VersionControlError> {
        let repository = open(dir)?;
        let mut index = repository.index().map_err(backend)?;
        let blobs = conflict_blobs(&index)?;
        for resolved in resolutions {
            apply_resolution(&repository, &mut index, dir.as_path(), resolved, &blobs)?;
        }
        if index.has_conflicts() {
            return Err(VersionControlError::Backend(
                "the merge still has unresolved conflicts".to_string(),
            ));
        }
        index.write().map_err(backend)?;
        let tree_id = index.write_tree().map_err(backend)?;
        let tree = repository.find_tree(tree_id).map_err(backend)?;
        let local = head_commit(&repository)?.ok_or_else(|| missing("local commit"))?;
        let remote_id = repository
            .refname_to_id(&tracking_ref(branch))
            .map_err(backend)?;
        let remote = repository.find_commit(remote_id).map_err(backend)?;
        let author = signature(&repository)?;
        let id = repository
            .commit(
                Some("HEAD"),
                &author,
                &author,
                message.as_str(),
                &tree,
                &[&local, &remote],
            )
            .map_err(backend)?;
        repository.cleanup_state().map_err(backend)?;
        commit_id(id)
    }

    fn push(
        &self,
        dir: &WorkingDirectory,
        url: &RemoteUrl,
        branch: &BranchName,
        credentials: &Credentials,
    ) -> Result<PushOutcome, VersionControlError> {
        let repository = open(dir)?;
        let Ok(local_id) = repository.refname_to_id(&local_ref(branch)) else {
            return Ok(PushOutcome::Pushed);
        };
        let authentication = Git2Authentication::new(credentials);
        let rejected = Cell::new(false);
        let mut callbacks = authentication.callbacks();
        callbacks.push_update_reference(|_, status| {
            rejected.set(status.is_some());
            Ok(())
        });
        let mut options = PushOptions::new();
        options.remote_callbacks(callbacks);
        let refspec = format!("{0}:{0}", local_ref(branch));
        let pushed = repository
            .remote_anonymous(url.network_url())
            .and_then(|mut remote| remote.push(&[refspec], Some(&mut options)));
        match pushed {
            Err(error) if error.code() == ErrorCode::NotFastForward => {
                return Ok(PushOutcome::RemoteMoved)
            }
            Err(error) => return Err(authentication.error(error)),
            Ok(()) if rejected.get() => return Ok(PushOutcome::RemoteMoved),
            Ok(()) => {}
        }
        repository
            .reference(&tracking_ref(branch), local_id, true, "push")
            .map_err(backend)?;
        Ok(PushOutcome::Pushed)
    }

    fn status(&self, dir: &WorkingDirectory) -> Result<RepositoryStatus, VersionControlError> {
        let repository = open(dir)?;
        let branch = current_branch(&repository)?;
        let mut options = StatusOptions::new();
        options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);
        let pending_changes = repository
            .statuses(Some(&mut options))
            .map_err(backend)?
            .len();
        let (ahead, behind) = ahead_behind(&repository, &branch)?;
        Ok(RepositoryStatus {
            remote: origin_url(&repository)?,
            branch,
            pending_changes,
            ahead,
            behind,
        })
    }

    fn history(
        &self,
        dir: &WorkingDirectory,
        path: &TrackedPath,
        limit: usize,
    ) -> Result<Vec<Revision>, VersionControlError> {
        let repository = open(dir)?;
        let Some(head) = head_commit(&repository)? else {
            return Ok(Vec::new());
        };
        let mut walk = repository.revwalk().map_err(backend)?;
        walk.push(head.id()).map_err(backend)?;
        walk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)
            .map_err(backend)?;
        let mut revisions = Vec::new();
        for id in walk {
            let commit = repository
                .find_commit(id.map_err(backend)?)
                .map_err(backend)?;
            if touches(&commit, path)? {
                revisions.push(revision_of(&commit)?);
            }
            if revisions.len() == limit {
                break;
            }
        }
        Ok(revisions)
    }

    fn read_at(
        &self,
        dir: &WorkingDirectory,
        path: &TrackedPath,
        commit: &CommitId,
    ) -> Result<Vec<u8>, VersionControlError> {
        let repository = open(dir)?;
        let id = Oid::from_str(commit.as_str()).map_err(backend)?;
        let tree = tree_of(&repository.find_commit(id).map_err(backend)?)?;
        let entry = tree
            .get_path(Path::new(path.as_str()))
            .map_err(|_| VersionControlError::PathNotFound(path.clone()))?;
        let blob = repository.find_blob(entry.id()).map_err(backend)?;
        Ok(blob.content().to_vec())
    }
}

fn open(dir: &WorkingDirectory) -> Result<Repository, VersionControlError> {
    let repository = Repository::open(dir.as_path()).map_err(|error| match error.code() {
        ErrorCode::NotFound => VersionControlError::NotARepository,
        _ => backend(error),
    })?;
    configure(&repository)?;
    Ok(repository)
}

fn configure(repository: &Repository) -> Result<(), VersionControlError> {
    let mut config = repository.config().map_err(backend)?;
    for (key, value) in [("core.precomposeunicode", true), ("core.autocrlf", false)] {
        if config.get_bool(key).ok() != Some(value) {
            config.set_bool(key, value).map_err(backend)?;
        }
    }
    Ok(())
}

fn current_branch(repository: &Repository) -> Result<BranchName, VersionControlError> {
    let head = repository.find_reference("HEAD").map_err(backend)?;
    let target = head
        .symbolic_target()
        .map_err(backend)?
        .ok_or_else(|| missing("branch: HEAD is detached"))?;
    let name = target.strip_prefix("refs/heads/").unwrap_or(target);
    BranchName::parse(name).map_err(domain_error)
}

fn origin_url(repository: &Repository) -> Result<Option<RemoteUrl>, VersionControlError> {
    match repository.find_remote(ORIGIN) {
        Ok(remote) => {
            let url = remote.url().map_err(backend)?;
            RemoteUrl::parse(url).map(Some).map_err(domain_error)
        }
        Err(error) if error.code() == ErrorCode::NotFound => Ok(None),
        Err(error) => Err(backend(error)),
    }
}

fn stage_everything(repository: &Repository, index: &mut Index) -> Result<(), VersionControlError> {
    let workdir = workdir(repository)?;
    let oversized: RefCell<Option<PathBuf>> = RefCell::new(None);
    let added = {
        let mut reject_oversized = |path: &Path, _: &[u8]| -> i32 {
            if is_oversized(&workdir.join(path)) {
                *oversized.borrow_mut() = Some(path.to_path_buf());
                -1
            } else {
                0
            }
        };
        index.add_all(["*"], IndexAddOption::DEFAULT, Some(&mut reject_oversized))
    };
    if let Some(path) = oversized.into_inner() {
        return Err(VersionControlError::FileTooLarge(tracked(&path)?));
    }
    added.map_err(backend)?;
    index.update_all(["*"], None).map_err(backend)?;
    index.write().map_err(backend)
}

fn is_oversized(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.len() > MAX_FILE_SIZE_BYTES)
}

fn fast_forward(
    repository: &Repository,
    branch: &BranchName,
    target: &Commit,
) -> Result<MergeOutcome, VersionControlError> {
    let before = head_commit(repository)?
        .map(|commit| tree_of(&commit))
        .transpose()?;
    repository
        .reference(&local_ref(branch), target.id(), true, "fast-forward")
        .map_err(backend)?;
    repository.set_head(&local_ref(branch)).map_err(backend)?;
    repository
        .checkout_head(Some(CheckoutBuilder::new().force()))
        .map_err(backend)?;
    let changed = changed_paths(repository, before.as_ref(), &tree_of(target)?)?;
    Ok(MergeOutcome::FastForwarded { changed })
}

fn merge_base_tree<'r>(
    repository: &'r Repository,
    their_commit: &Commit,
) -> Result<Tree<'r>, VersionControlError> {
    let local = head_commit(repository)?.ok_or_else(|| missing("local commit"))?;
    let base = repository
        .merge_base(local.id(), their_commit.id())
        .map_err(backend)?;
    tree_of(&repository.find_commit(base).map_err(backend)?)
}

fn read_conflicts(index: &Index) -> Result<Vec<Conflict>, VersionControlError> {
    let mut conflicts = Vec::new();
    for conflict in index.conflicts().map_err(backend)? {
        let conflict = conflict.map_err(backend)?;
        let Some(entry) = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
        else {
            continue;
        };
        conflicts.push(Conflict {
            path: tracked_bytes(&entry.path)?,
            local: change_of(conflict.our.is_some()),
            remote: change_of(conflict.their.is_some()),
        });
    }
    Ok(conflicts)
}

fn change_of(present: bool) -> FileChange {
    if present {
        FileChange::Modified
    } else {
        FileChange::Deleted
    }
}

fn conflict_blobs(
    index: &Index,
) -> Result<HashMap<TrackedPath, ConflictBlobs>, VersionControlError> {
    let mut blobs = HashMap::new();
    for conflict in index.conflicts().map_err(backend)? {
        let conflict = conflict.map_err(backend)?;
        let Some(entry) = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
        else {
            continue;
        };
        blobs.insert(
            tracked_bytes(&entry.path)?,
            ConflictBlobs {
                local: conflict.our.as_ref().map(|entry| entry.id),
                remote: conflict.their.as_ref().map(|entry| entry.id),
            },
        );
    }
    Ok(blobs)
}

fn apply_resolution(
    repository: &Repository,
    index: &mut Index,
    workdir: &Path,
    resolved: &ResolvedConflict,
    blobs: &HashMap<TrackedPath, ConflictBlobs>,
) -> Result<(), VersionControlError> {
    let conflict = blobs
        .get(&resolved.path)
        .ok_or_else(|| missing("conflict for a resolved path"))?;
    match resolved.resolution {
        ConflictResolution::KeepLocal => {
            restore(repository, index, workdir, &resolved.path, conflict.local)
        }
        ConflictResolution::KeepRemote => {
            restore(repository, index, workdir, &resolved.path, conflict.remote)
        }
        ConflictResolution::KeepLocalAndCopyRemote => {
            let copy = resolved
                .copy
                .as_ref()
                .ok_or_else(|| missing("conflict copy path"))?;
            restore(repository, index, workdir, &resolved.path, conflict.local)?;
            restore(repository, index, workdir, copy, conflict.remote)
        }
        ConflictResolution::Delete => remove(index, workdir, &resolved.path),
    }
}

fn restore(
    repository: &Repository,
    index: &mut Index,
    workdir: &Path,
    path: &TrackedPath,
    blob: Option<Oid>,
) -> Result<(), VersionControlError> {
    let Some(blob) = blob else {
        return remove(index, workdir, path);
    };
    let content = repository.find_blob(blob).map_err(backend)?;
    let file = workdir.join(path.as_str());
    if let Some(folder) = file.parent() {
        fs::create_dir_all(folder).map_err(io_error)?;
    }
    fs::write(&file, content.content()).map_err(io_error)?;
    index.add_path(Path::new(path.as_str())).map_err(backend)
}

fn remove(
    index: &mut Index,
    workdir: &Path,
    path: &TrackedPath,
) -> Result<(), VersionControlError> {
    match fs::remove_file(workdir.join(path.as_str())) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(io_error(error)),
        _ => {}
    }
    index.remove_path(Path::new(path.as_str())).map_err(backend)
}

fn ahead_behind(
    repository: &Repository,
    branch: &BranchName,
) -> Result<(usize, usize), VersionControlError> {
    let local = repository.refname_to_id(&local_ref(branch));
    let tracking = repository.refname_to_id(&tracking_ref(branch));
    match (local, tracking) {
        (Ok(local), Ok(tracking)) => repository
            .graph_ahead_behind(local, tracking)
            .map_err(backend),
        _ => Ok((0, 0)),
    }
}

fn touches(commit: &Commit, path: &TrackedPath) -> Result<bool, VersionControlError> {
    let current = entry_id(&tree_of(commit)?, path);
    let previous = match commit.parent_count() {
        0 => None,
        _ => entry_id(&tree_of(&commit.parent(0).map_err(backend)?)?, path),
    };
    Ok(current != previous)
}

fn entry_id(tree: &Tree, path: &TrackedPath) -> Option<Oid> {
    tree.get_path(Path::new(path.as_str()))
        .ok()
        .map(|entry| entry.id())
}

fn revision_of(commit: &Commit) -> Result<Revision, VersionControlError> {
    let author = commit.author();
    Ok(Revision {
        id: commit_id(commit.id())?,
        summary: commit
            .summary()
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        author: author.name().unwrap_or_default().to_string(),
        epoch_seconds: commit.time().seconds(),
    })
}

fn changed_paths(
    repository: &Repository,
    old: Option<&Tree>,
    new: &Tree,
) -> Result<Vec<TrackedPath>, VersionControlError> {
    let diff = repository
        .diff_tree_to_tree(old, Some(new), None)
        .map_err(backend)?;
    diff.deltas()
        .filter_map(|delta| delta.new_file().path().or(delta.old_file().path()))
        .map(tracked)
        .collect()
}

fn collect_files(root: &Path, folder: &Path, paths: &mut BTreeSet<TrackedPath>) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if path.file_name().is_some_and(|name| name == ".git") {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, paths)?;
        } else if let Ok(relative) = path.strip_prefix(root) {
            if let Ok(tracked) = tracked(relative) {
                paths.insert(tracked);
            }
        }
    }
    Ok(())
}

fn head_commit(repository: &Repository) -> Result<Option<Commit<'_>>, VersionControlError> {
    match repository.head() {
        Ok(head) => head.peel_to_commit().map(Some).map_err(backend),
        Err(error) if matches!(error.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => {
            Ok(None)
        }
        Err(error) => Err(backend(error)),
    }
}

fn tree_of<'r>(commit: &Commit<'r>) -> Result<Tree<'r>, VersionControlError> {
    commit.tree().map_err(backend)
}

fn signature(repository: &Repository) -> Result<Signature<'static>, VersionControlError> {
    repository
        .signature()
        .or_else(|_| Signature::now(FALLBACK_AUTHOR_NAME, FALLBACK_AUTHOR_EMAIL))
        .map_err(backend)
}

fn workdir(repository: &Repository) -> Result<PathBuf, VersionControlError> {
    repository
        .workdir()
        .map(Path::to_path_buf)
        .ok_or_else(|| missing("working directory"))
}

fn local_ref(branch: &BranchName) -> String {
    format!("refs/heads/{}", branch.as_str())
}

fn tracking_ref(branch: &BranchName) -> String {
    format!("refs/remotes/{ORIGIN}/{}", branch.as_str())
}

fn commit_id(id: Oid) -> Result<CommitId, VersionControlError> {
    CommitId::parse(&id.to_string()).map_err(domain_error)
}

fn tracked(path: &Path) -> Result<TrackedPath, VersionControlError> {
    let joined: Vec<String> = path
        .iter()
        .map(|component| component.to_string_lossy().into_owned())
        .collect();
    TrackedPath::parse(&joined.join("/")).map_err(domain_error)
}

fn tracked_bytes(path: &[u8]) -> Result<TrackedPath, VersionControlError> {
    TrackedPath::parse(&String::from_utf8_lossy(path)).map_err(domain_error)
}

fn backend(error: git2::Error) -> VersionControlError {
    network_or_backend(error)
}

fn io_error(error: io::Error) -> VersionControlError {
    VersionControlError::Backend(error.to_string())
}

fn domain_error(error: impl std::error::Error) -> VersionControlError {
    VersionControlError::Backend(error.to_string())
}

fn missing(what: &str) -> VersionControlError {
    VersionControlError::Backend(format!("missing {what}"))
}
