# AGENTS.md — Jade

Guide for any agent (or human) working on this repository. It records decisions already
made. Do not change a decision here without the maintainer's approval; if a task seems to
require it, stop and ask.

**Everything in the repository is written in English**: code, identifiers, tests, Gherkin
features, commit messages, documentation. The only non-English content is the UI
translations in `locales/`.

## 1. Product

Jade is a Markdown notes app (an Obsidian-style vault) whose storage and sync are a
**Git repository**, typically on GitHub. One codebase ships to Linux, macOS, Windows,
Android and iOS. Platform priority: **macOS desktop, Ubuntu desktop, Android**; Windows
and iOS come later.

Goals:
- Git is the core, not an add-on: automatic commit/pull/push, per-note history, safe
  conflict handling, all without the user needing to know Git.
- Full format compatibility with Obsidian vaults: a vault opens in both apps.
- UI available in English, Spanish, Simplified Chinese and Catalan.

Non-goals (do not implement unless the maintainer asks): plugin system; pull requests,
issues, Actions or any GitHub feature beyond the repository itself; real-time
collaboration; proprietary sync services.

Licence: **GPL-3.0-only**. Public repository: `AstonRendar/jade`.

## 2. Workflow

For every task, in this order:

1. **Analysis** — read the affected code and understand the context and implications
   before touching anything. If anything is ambiguous, ask.
2. **Proposal** — describe the approach (affected contexts, layers, new types, tests) and
   wait for confirmation before writing code.
3. **Tests first (TDD)** — write the tests that specify the expected behaviour and run
   them to see them fail. Unit tests for domain and application logic; Gherkin features
   for complete flows.
4. **Implementation** — the minimum code that makes the tests pass, then refactor with
   the tests green.
5. **Verification** — run the full suite (`make check`, §10) and confirm every test
   passes, including pre-existing ones, and coverage thresholds hold.
6. **Documentation** — if the change affects architecture, data flow, conventions or
   the command contract, update `AGENTS.md` and `README.md`.
7. **Commits** — small, ordered and self-contained: tests pass at every commit (e.g. value
   object, then port, then use case, then adapter, then entry point). Commit often.
   Short messages in English following Conventional Commits; anything that is code
   (types, functions, paths, keys) goes in backticks. **No authorship or co-authorship
   annotations of any kind** (no `Co-Authored-By`, no "Generated with…").

Pending work is tracked as GitHub issues, never as `TODO` comments in the code.

## 3. Stack

| Layer | Choice |
|---|---|
| Shell | Tauri 2 (desktop + mobile) |
| Core | Rust stable, edition 2021, Cargo workspace |
| Git | `git2` (libgit2), features `https`, `vendored-libgit2`, `vendored-openssl`; no SSH |
| Secrets | `keyring` (desktop, iOS); Android Keystore in phase 3 |
| File watching | `notify` |
| Unicode | `unicode-normalization` |
| Local time | `chrono` (infrastructure only: `Clock` adapter) |
| Device name | `gethostname` (infrastructure only: `DeviceInfo` adapter) |
| Native dialogs | `tauri-plugin-dialog` (folder picker) |
| Errors | `thiserror` |
| Logging | `tracing` |
| Rust tests | `cargo test`, `mockall`, `cucumber` (Gherkin), `tempfile`, `cargo-llvm-cov` |
| TS types from Rust | `ts-rs`, generated into `src/shared/bindings/` and committed |
| Frontend | React 19 + TypeScript (strict) + Vite |
| State | Zustand |
| Editor | CodeMirror 6 (`@codemirror/lang-markdown`) |
| Preview | markdown-it (`html: false`) with a custom wikilink rule |
| i18n | i18next + react-i18next |
| Styling | CSS Modules + CSS custom properties (light/dark) |
| Frontend tests | Vitest + React Testing Library + `@tauri-apps/api/mocks`, coverage v8 |
| Lint/format | rustfmt, clippy (`-D warnings`), ESLint, Prettier, `tsc --noEmit` |

Do not add dependencies outside this table without approval; propose them with a
justification.

## 4. Architecture — Domain-Driven Design

### 4.1 Bounded contexts

| Context | Responsibility |
|---|---|
| `notes` | Vault contents: notes, folders, paths, wikilinks and their resolution, backlinks, search |
| `sync` | Versioning and synchronisation: repository lifecycle, commits, remote, merge and conflict policy, note history, credentials |
| `preferences` | App preferences: language, theme, auto-sync interval, recent vaults |

Contexts do not import each other. Each one models what it needs with its own types
(e.g. `notes` has `VaultRoot`, `sync` has `WorkingDirectory`); the composition root
translates between them with primitives. Cross-context reactions (e.g. "notes changed →
schedule sync") are wired in the composition root, not inside a context.

### 4.2 Layers — absolute rule

Each context is one crate under `crates/` that contains **only** these three folders in
`src/`:

```
crates/notes/src/
├── domain/           # Pure business logic
├── application/      # Use cases
└── infrastructure/   # Adapters: implementations of domain ports
```

No other folder exists inside a context. If something does not fit, the design must be
reconsidered. `lib.rs` only declares the three modules.

- **domain** — entities, value objects, domain services, domain events, ports (traits)
  and domain errors. **Allowed dependencies: `std`, `thiserror`,
  `unicode-normalization`. Nothing else** — no `serde`, no `tauri`, no `git2`, no
  `std::fs`, no `std::time::SystemTime::now` (time comes through a `Clock` port).
  Derives limited to `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`.
- **application** — use cases. Each one receives a request DTO and returns a response
  DTO, and depends only on domain types and ports (generic over the port traits).
  No I/O, no framework.
- **infrastructure** — concrete implementations of domain ports: `FsNoteRepository`,
  `Git2VersionControl`, `KeyringCredentialStore`, `JsonFilePreferencesRepository`,
  `SystemClock`, `NotifyVaultWatcher`.

The **architecture test** (`crates/shared/tests/architecture.rs`) scans every crate in
the workspace and fails if `src/` contains anything other than `lib.rs` and the three
layers, if `domain/` or `application/` reference a dependency outside the allowed list
or I/O from `std` (`fs`, `net`, `process`, `env`, `thread`, `time::SystemTime`), or if a
layer reaches an outer one (`domain` → `application`/`infrastructure`, `application` →
`infrastructure`).

`crates/shared/` is **not a context**: it holds purely technical code that several
contexts need identically (e.g. the `UseCase` trait). It has the same layers, never
imports from a context and contains no business concepts.

### 4.3 Application shell (`src-tauri/`)

`src-tauri/` is the **composition root and entry point**, the equivalent of a framework
kernel plus controllers:
- builds the infrastructure adapters and injects them into the use cases;
- exposes Tauri commands — thin functions that map a serialisable input to a use-case
  request, run it (blocking work through `spawn_blocking`) and map the response or
  error to a serialisable view model (`serde` + `ts-rs` live **only** here);
- maps every application error to an `ErrorCode` for the frontend (§4.7);
- wires cross-context reactions and the auto-sync scheduler.

It contains no business logic.

### 4.4 Repository layout

```
jade/
├─ AGENTS.md  CLAUDE.md  LICENSE  README.md  Makefile
├─ Cargo.toml                    # workspace
├─ crates/
│  ├─ shared/   src/{domain,application,infrastructure}
│  ├─ notes/    src/{domain,application,infrastructure}  tests/
│  ├─ sync/     src/{domain,application,infrastructure}  tests/
│  └─ preferences/ …
├─ src-tauri/                    # composition root + Tauri commands
│  ├─ src/{main.rs, lib.rs, commands/, view_models/, errors.rs, wiring.rs}
│  ├─ tests/features/            # cross-context features (e.g. two-device sync)
│  ├─ capabilities/  icons/  tauri.conf.json
├─ src/                          # frontend, mirrors the contexts
│  ├─ notes/  sync/  preferences/
│  │   └─ {ui/, store/, gateway/}   # gateway = the only code calling invoke()/listen()
│  ├─ shared/ {ui/, bindings/, i18n/, testing/}
│  ├─ locales/{en,es,zh-CN,ca}.json
│  └─ main.tsx  App.tsx
└─ .github/workflows/
```

### 4.5 Modelling rules

- **Always-valid, immutable value objects** with private fields and named constructors
  that validate: `NotePath::parse(&str) -> Result<NotePath, InvalidNotePathError>`.
  Invalid states are unrepresentable after construction.
- **Entities and aggregates** expose behaviour, not setters. Identity is explicit.
- **Ports** are traits named after the concept with no suffix (`NoteRepository`,
  `VersionControl`, `CredentialStore`, `Clock`). Implementations carry the technology
  in their name (`FsNoteRepository`, `Git2VersionControl`).
- **Errors** are enums whose names end in `Error`, one per module that raises them.
  Domain errors describe business rules; infrastructure errors wrap the technical cause.
  When all the use cases of a context fail in the same ways, they may share one
  application error (`notes::application::error::NotesError`) instead of one per use case.
- **Secrets** (`AccessToken`) implement `Debug` redacted and are never logged.
- **Use cases** implement `shared::application::UseCase` (`type Request`,
  `type Response`, `type Error`, `fn execute`). One use case per file, named as an
  imperative (`SaveNote`, `SynchronizeVault`).
- **No comments.** Extract private functions with meaningful names instead. Doc
  comments only on public items whose contract is not obvious from the signature.
- **Folders inside layers**: `domain/{value_object, service, port}` (plus `model/` for
  entities and aggregates when a context has them); `application/` holds one module per
  use case, with its request, response and error types next to it.
- **Use-case requests carry primitives** (strings, numbers); the use case parses them
  into value objects, so validation failures are use-case errors.

### 4.6 Domain map

**`notes`**
- Value objects: `VaultRoot`, `VaultPath` (any vault-relative path: `/`-separated, NFC,
  no `..`/absolute/empty components), `NotePath` (a `VaultPath` whose file name ends in
  `.md`; title derived from the file name), `WikiLink` (target, heading, alias, embed
  flag), `SearchQuery` (Unicode- and case-insensitive matching).
- Model: `Note` (path + content).
- Domain services: `WikiLinkParser` (ignores links in fenced and inline code),
  `LinkResolver` (exact path, then name or partial path, case-insensitive, shortest path
  wins on ambiguity), `BacklinkFinder`, `NoteSearch` (title and line hits, capped).
- Ports: `NoteRepository`, `VaultWatcher`.
- Use cases: `OpenVault`, `ListVaultEntries`, `ReadNote`, `SaveNote`, `CreateNote`,
  `CreateFolder`, `RenameEntry`, `DeleteEntry`, `SearchNotes`, `FindBacklinks`,
  `ResolveLinks`, `FollowLink` (creates the note if missing).

Wikilink parsing and resolution exist **only in Rust**. The frontend asks the core
(`ResolveLinks`) instead of reimplementing them.

**`sync`**
- Value objects: `WorkingDirectory`, `TrackedPath` (repository-relative path),
  `RemoteUrl` (normalises SSH forms to HTTPS for network use; `file://` kept for local
  remotes), `BranchName`, `CommitId`, `CommitMessage` (`Sync from <device>`,
  `Merge remote changes`), `DeviceName`, `AccessToken`, `SyncInterval`, `Timestamp`
  (local wall-clock minute, supplied by `Clock`), `ConflictCopyName`
  (`Name (conflict YYYY-MM-DD HHMM).ext`, numbered ` 2`, ` 3`… when taken).
- Domain services: `ConflictResolutionPolicy` (`FileChange` × `FileChange` →
  `ConflictResolution`, table in §5), `SyncRetryPolicy`.
- Ports: `VersionControl` (with its types `MergeOutcome` — `UpToDate`,
  `FastForwarded`, `Diverged` —, `Conflict`, `ResolvedConflict`, `PushOutcome`,
  `RepositoryStatus`, `Revision`, `VersionControlError`), `CredentialStore`
  (`Credentials::Anonymous | Token`), `Clock`, `DeviceInfo`.
  The merge is two-phase: the adapter reports conflicts (`merge_remote`), the use case
  decides with the policy, the adapter applies the decisions (`conclude_merge`).
- Use cases: `InitRepository`, `CloneRepository`, `ConnectRemote`, `SynchronizeVault`
  (returns a `SyncReport`), `GetRepositoryStatus`, `ListNoteHistory`,
  `ReadNoteRevision`, `StoreAccessToken` (an empty token deletes it),
  `CheckAccessToken`.

**`preferences`**
- Value objects: `Locale` (`en`, `es`, `zh-CN`, `ca` only), `Theme`, `RecentVaults`
  (deduplicated, most recent first, max 8).
- Aggregate: `Preferences`.
- Port: `PreferencesRepository`.
- Use cases: `GetPreferences`, `UpdatePreferences`, `RememberVault`.

### 4.7 Command contract and errors

Tauri commands are named `<context>_<use_case>` in snake_case (`notes_save_note`,
`sync_synchronize_vault`). Paths crossing the boundary are vault-relative,
`/`-separated, NFC strings. Events (core → frontend): `notes://changed { paths }`,
`sync://started`, `sync://finished { report }`, `sync://failed { error }`.

Errors reach the frontend as `{ code: ErrorCode, params }`. **The core never returns
user-facing text**; the frontend maps `code` to the i18n key `errors.<code>`. Codes:
`vault_not_open`, `invalid_path`, `already_exists`, `not_found`, `not_a_repository`,
`no_remote`, `auth_required`, `auth_failed`, `network`, `file_too_large`,
`push_rejected`, `io`, `unexpected`.

## 5. Sync algorithm (`SynchronizeVault`)

Only one sync runs at a time. Note writes and sync share a lock so files are never
written during checkout or merge.

1. The frontend flushes pending saves.
2. Stage everything (respecting `.gitignore`, including deletions). If the tree differs
   from `HEAD`, commit with message `Sync from <device-name>`.
3. Fetch `refs/heads/<branch>` from `origin` into `refs/remotes/origin/<branch>`.
4. Merge: up to date → nothing; fast-forward (or unborn local branch) → move the ref and
   check out; divergent → merge, resolve every conflict with `ConflictResolutionPolicy`,
   and create a merge commit (`Merge remote changes`) with both parents.
5. Push `<branch>`. If rejected because the remote moved, repeat from step 3 (at most 3
   attempts, `SyncRetryPolicy`), then fail with `push_rejected`.
6. Report every path changed by the merge (`notes://changed`).

Error handling by type: authentication and validation errors fail immediately and are
not retried; network errors fail the current sync and the scheduler tries again at the
next interval.

Hard rules: **never** force-push, rewrite history, reset or discard user content. With no
remote, sync is a local commit only. SSH remote URLs are used over HTTPS for network
operations; the stored remote is left untouched.

**Conflict resolution policy** — no `<<<<<<<` markers are ever written into notes:

| Local | Remote | Result |
|---|---|---|
| modified | modified | local stays at the path; remote saved as a `ConflictCopyName` |
| modified | deleted | keep local |
| deleted | modified | keep remote |
| deleted | deleted | delete |

Text written into the repository (commit messages, conflict suffix) is **always English
and fixed**, regardless of UI language.

**Open note changed by sync**: if the editor has no unsaved edits, reload it. If it does,
reload from disk and save the unsaved buffer as a conflict copy. Nothing is lost.

Auto-sync every N minutes (default 5, `0` disables), plus a manual button and `Mod+S`.
Auto-save to disk 600 ms after the last keystroke.

## 6. Vault and Git details

- Hidden entries (name starting with `.`) are not listed. `.obsidian/` is never modified;
  `.jade/` is reserved for future per-vault data.
- Repositories opened or created by Jade get `core.precomposeunicode=true` and
  `core.autocrlf=false`. File contents are never rewritten.
- Files larger than 100 MB block the commit with `file_too_large` (GitHub limit).
- Commit author: the repository/global Git identity; fallback `Jade <jade@localhost>`.
- Authentication: HTTPS with a GitHub token (username `x-access-token`). Phase 1: a
  personal access token pasted in Settings; phase 2: GitHub OAuth device flow.
  Credential callbacks give up after 2 attempts to avoid libgit2 retry loops.
- Wikilink syntax supported: `[[Target]]`, `[[Target|Alias]]`, `[[Target#Heading]]`,
  `![[Embed]]`. Following a missing link creates the note at the vault root.

## 7. Testing

TDD is mandatory (§2). Two kinds of tests, both required:

| Suite | Scope | Location |
|---|---|---|
| Unit | Domain and application in isolation; ports replaced by `mockall` mocks | `crates/<ctx>/tests/unit/<layer>/…` mirroring `src/`, crate root `tests/unit/main.rs` |
| Functional | Complete flows written in Gherkin, run with `cucumber` against real adapters on temporary directories | `crates/<ctx>/tests/features/*.feature`, runner `tests/functional/main.rs` (`harness = false`); `src-tauri/tests/features/` for cross-context flows |
| Frontend | Components and stores with Vitest + RTL; the core is replaced with `mockIPC` | next to the code, `*.test.tsx` |

Functional tests never touch the network: remotes are bare repositories on disk
(`file://`), and the credential store is an in-memory adapter. Sync between devices is
tested with two clones of the same bare remote.

Required coverage:
- every value object invariant, every `ConflictResolutionPolicy` row, every use case
  (success and each error) → unit tests;
- every use case → at least one Gherkin scenario;
- locale completeness: a test fails if any locale misses a key present in `en` or has
  extra keys;
- the architecture test (§4.2).

Test files under `tests/` may start with `#![allow(clippy::expect_used)]`; production
code may not use `unwrap`/`expect` (enforced by workspace lints).

Coverage thresholds, enforced in CI: domain + application **≥ 95 %** lines; Rust overall
**≥ 85 %**; frontend **≥ 80 %**.

### Unit test conventions

1. **Arrange / Act / Assert** — every test has the three comments `// Arrange`,
   `// Act`, `// Assert` as phase separators (the only comments allowed in the codebase).
2. **Mock setup in meaningful helpers** — any mock configuration goes in a function
   starting with `given_` (stubs/state) or `expect_` (expectations). The name gives
   meaning to the test, not the mechanics:
   ```rust
   fn given_the_remote_has_new_commits(vc: &mut MockVersionControl) { … }
   fn expect_a_merge_commit_is_created(vc: &mut MockVersionControl) { … }
   ```
3. **Order inside a test file**: shared setup, `#[test]` functions, `given_*`/`expect_*`
   helpers, test data builders.
4. **Object Mothers** — factories for domain objects live in
   `crates/<ctx>/tests/support/mothers/` (e.g. `NoteMother::with_content("…")`), with
   valid defaults and overrides only for what each test cares about.
5. Test names describe behaviour: `rejects_paths_that_escape_the_vault`.

### Functional test conventions

- One `.feature` per use case or flow, named after it (`synchronize_vault.feature`).
- Steps grouped by topic in `tests/functional/steps/` (`device_steps.rs`, `remote_steps.rs`); the cucumber `World` in `tests/functional/world.rs`; port doubles that need no mocking (fixed clock, named device) in `tests/functional/doubles.rs`.
- Fictitious data only: no real people, companies or URLs. Remotes use
  `https://git.example.com/…` in parsing tests.

## 8. Internationalisation

- Locales: `en` (reference and fallback), `es`, `zh-CN`, `ca`.
- Every user-visible string goes through i18next; no hard-coded UI text.
- Keys are namespaced by context (`notes.tree.newNote`, `errors.auth_failed`).
- Language: detected from the OS, overridable in preferences. Native menus are built in
  `src-tauri` from the selected locale.
- Dates and numbers via `Intl` with the active locale. CJK uses system fonts.

## 9. Security and privacy

- Strict CSP; no remote scripts, styles or fonts.
- markdown-it with `html: false`. External links open in the system browser via
  `tauri-plugin-opener`; the WebView never navigates away.
- Tauri capabilities: the minimum needed, listed explicitly.
- The token lives only in the OS credential store; never logged, never written to
  preferences, never returned to the frontend.
- Logs (`tracing`) never include note contents or tokens; note paths only at `debug`.
- No secrets in the repository; CI signing material lives in GitHub Secrets.

## 10. Tooling and CI

`Makefile` targets: `make dev`, `make test` (all suites), `make coverage`, `make lint`,
`make check` (format + lint + tests + coverage thresholds, what CI runs).

- `main` is always releasable. Work in `feat/…`, `fix/…`, `chore/…` branches, merged via
  PR with squash.
- Semver; the version is kept in sync in `package.json`, the workspace `Cargo.toml` and
  `tauri.conf.json`.
- CI (GitHub Actions): on PR → `make check` on ubuntu and macOS (windows joins in its
  phase). On tag `v*` → `tauri-action` builds macOS and Linux into a draft release.
  Android and iOS are built locally until signing is set up.

## 11. Roadmap

Each phase ends with `make check` green and a manual smoke test by the maintainer.

**Phase 0 — Foundations.** Workspace, the three context crates with architecture tests,
`shared::UseCase`, Tauri + React scaffold, i18n with the four locales, CI, coverage,
LICENSE. `sync` domain and `Git2VersionControl` with full unit and Gherkin coverage
(commit, fetch, fast-forward, merge, every conflict case, push retry, two-device flows).

**Phase 1 — MVP on macOS.** Welcome screen (open folder, clone from GitHub, recent
vaults); file tree with context menu; CodeMirror editor with styled Markdown, auto-save,
wikilink autocomplete and Mod+click navigation; preview toggle; quick switcher
(`Mod+O`); full-text search (`Mod+Shift+F`); backlinks; per-note history with view and
restore; sync with status indicator; preferences (language, token, auto-sync, remote,
init repository); external change watcher; light/dark theme.

**Phase 2 — Ubuntu + GitHub login.** Linux build in CI, platform fixes, OAuth device
flow, renaming a note updates links to it.

**Phase 3 — Android.** Vaults in app storage (clone-based), Android Keystore, mobile
layout.

**Phase 4 — Editor.** Obsidian-style live preview, embedded images via the asset
protocol, visual conflict merge view.

**Phase 5 — Windows.** Windows build in CI and platform fixes.

**Phase 6 — iOS.** As phase 3, plus Files app integration.

## 12. Development environment

Requirements: Node ≥ 20, Rust stable (rustup) with `cargo-llvm-cov`, Xcode (macOS/iOS),
Android Studio + NDK (Android), and on Linux the Tauri system packages (webkit2gtk,
libsecret/dbus).

```
npm install
make dev       # run the desktop app
make check     # everything CI runs
```
