# Docs

Document and directory map for this repository. Each document below owns its subject; `docs/` holds
only **current** facts.

## Core documents

| Document | Responsibility |
|---|---|
| [README.md](../README.md) / [README.zh-CN.md](../README.zh-CN.md) | User overview, installation and usage (kept in sync) |
| [PRODUCT.md](../PRODUCT.md) | Product rules and non-goals |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Human contributor workflow: setup, checks, issues, pull requests |
| [AGENTS.md](../AGENTS.md) | Coding-agent boundaries and routing, loaded every session |
| [CLAUDE.md](../CLAUDE.md) | Imports `AGENTS.md` and adds Claude Code note |
| [SECURITY.md](../SECURITY.md) | Data storage inventory, network destinations, vulnerability reporting |
| [CHANGELOG.md](../CHANGELOG.md) | User-visible changes per version and manual cleanup notes |
| [docs/README.md](README.md) | Document and directory map and writing rules |
| [status.md](status.md) | Platform support and feature verification scope |
| [architecture.md](architecture.md) | Code map, data flow, switch sequence and persistence |
| [antigravity-integration.md](antigravity-integration.md) | Antigravity credentials, OAuth, Cloud Code APIs, processes and verified versions |
| [design.md](design.md) | App visuals, interaction, icons and screenshots |
| [release.md](release.md) | Versioning, local builds and signing |
| [pull_request_template.md](../.github/pull_request_template.md) | Pull request submission form |
| [LICENSE](../LICENSE) | MIT license |

## Directories

| Directory | Holds |
|---|---|
| `src/` | React UI: snapshot-derived presentation, dialog state and Tauri command / plugin requests |
| `src/components/` | UI components, shared icons and colocated styles; sheets share `sheet.css` |
| `src/hooks/` | Snapshot subscription, popover auto-size and relative-time ticker |
| `src/styles/` | Design tokens and base styles |
| `src-tauri/` | Rust app and Tauri build / bundle configuration |
| `src-tauri/src/` | Backend modules ([architecture.md](architecture.md)) |
| `src-tauri/src/antigravity/` | Local Antigravity installation, credentials, OAuth client and processes |
| `src-tauri/src/google/` | Google OAuth, loopback callback and Cloud Code API clients |
| `src-tauri/src/secret_store/` | Platform credential-store primitives |
| `src-tauri/capabilities/` | Tauri permissions |
| `src-tauri/icons/` | Bundled app and tray icons |
| `assets/` | Shared visual assets for the README |
| `assets/brand/` | SVG icon sources |
| `assets/screenshots/` | Rendered screenshots for the README |
| `scripts/` | Visual asset scripts and quota behavior tests |
| `scripts/screenshot/` | Screenshot renderer and fictional demo data |
| `docs/` | Maintainer reference documents |
| `.github/` | GitHub workflows and issue / pull request templates |
| `.github/workflows/` | Continuous integration workflows |
| `.github/ISSUE_TEMPLATE/` | Bug report, feature request and issue routing templates |

A new directory requires a single responsibility and a row here in the same change.

## Writing rules

- **One owner per fact.** Other documents link to the owner.
- **Current state only.** Describe how the codebase works now; history belongs in Git and `CHANGELOG.md`.
- **Work items live in issues.** Use `deferred` for accepted limitations and `needs-decision` for open
  questions.
- **Verified, then written.** Record client and platform facts only after verifying on a real
  installation with the version; mark unchecked behavior *not verified*.
- **Language.** User and contributor docs are in English (`README.zh-CN.md` in Simplified Chinese);
  `AGENTS.md` and `CLAUDE.md` are in Chinese.
- **Names.** Lowercase kebab-case for files and directories, except conventional root files and
  language-required module names.
- **Nothing private.** Never commit tokens, real emails or machine-specific personal paths.
