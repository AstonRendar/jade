# Jade

**Markdown notes that live in a Git repository.**

Jade is a notes app in the spirit of Obsidian: a folder of Markdown files, linked
together with `[[wikilinks]]`. The difference is where your notes live and how they
travel between devices: every vault is a Git repository, usually on GitHub, and Jade
commits, pulls and pushes for you. You get full version history and sync across devices
without needing to know Git — and without a proprietary sync service.

> **Status: early preview.** The macOS desktop app works end to end (vaults, editor,
> links, search, history and sync). Builds for other platforms are on the way.

## Why Jade

- **Git is the core, not a plugin.** Changes are saved automatically and synchronised
  in the background. Every note has its own history, and any earlier version can be
  viewed or restored.
- **Nothing is ever lost.** If two devices edit the same note, Jade keeps both versions:
  yours stays in place and the other one is saved next to it as
  `Note (conflict 2026-11-21 2245).md`. No `<<<<<<<` markers, no rewritten history, no
  force pushes.
- **Compatible with Obsidian.** Same file format and link syntax (`[[Note]]`,
  `[[Note|alias]]`, `[[Note#Heading]]`, `![[image.png]]`). A vault opens in both apps.
- **Your storage, your rules.** Any Git host reachable over HTTPS works. Your token is
  kept in the operating system's credential store. For GitHub, a fine-grained token
  limited to your notes repository with *Contents: read and write* is enough.
- **Multilingual.** English, Spanish, Simplified Chinese and Catalan.

## Platforms

One codebase, built with [Tauri 2](https://tauri.app). Development priority:

1. macOS desktop
2. Ubuntu desktop
3. Android
4. Windows
5. iOS

## How synchronisation works

Each sync runs these steps, automatically every few minutes or on demand:

1. Save and commit local changes (`Sync from <device>`).
2. Fetch the remote branch.
3. Fast-forward, or merge when both sides changed. Conflicts are resolved by keeping
   both versions as described above.
4. Push. If another device pushed in the meantime, repeat from step 2.

A vault without a remote is still versioned locally and can be connected to one later.

## Roadmap

| Phase | Scope | State |
|---|---|---|
| 0 | Foundations: architecture, CI, synchronisation core | Done |
| 1 | MVP on macOS: vault, editor, links, search, backlinks, history, sync | Done |
| 2 | Ubuntu, link updates on rename | In progress |
| 3 | Android | Planned |
| 4 | Sign in with GitHub (per-repository access, expiring tokens) | Planned |
| 5 | Live preview, embedded images, visual conflict merge | Planned |
| 6 | Windows | Planned |
| 7 | iOS | Planned |

## Building from source

Requirements: Node 20+, Rust stable ([rustup](https://rustup.rs)), and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
git clone https://github.com/AstonRendar/jade.git
cd jade
make install   # frontend dependencies
make dev       # run the desktop app
```

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Cmd/Ctrl+O` | Open a note by name |
| `Cmd/Ctrl+Shift+F` | Search all notes |
| `Cmd/Ctrl+S` | Synchronise now |
| `Cmd/Ctrl+click` on a `[[link]]` | Follow the link (creates the note if missing) |

## Development

| Command | What it does |
|---|---|
| `make dev` | Runs the app with hot reload |
| `make test` | Unit and functional tests (Rust and frontend) |
| `make lint` | Formatting, Clippy, ESLint, type checks |
| `make coverage` | Coverage with enforced thresholds |
| `make check` | Everything CI runs |

The project follows **Domain-Driven Design** and **test-driven development**: the Rust
core is split into bounded contexts (`notes`, `sync`, `preferences`), each with
`domain`, `application` and `infrastructure` layers, and every flow is specified with
unit tests and Gherkin features. Architecture, conventions and workflow are documented
in [AGENTS.md](AGENTS.md) — read it before contributing.

## Licence

[GPL-3.0](LICENSE).
