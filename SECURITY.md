# Security

The Antigravity-only fork manages the user's own Google subscription sign-ins.

| Data | Location |
|---|---|
| Saved refresh tokens | OS credential store under service `orbitswitch`, keyed by Google account ID |
| Account labels, plan and fingerprints | `accounts.json` under app ID `io.github.shengyy.orbitswitch` |
| Legacy account migration journal | Temporary `accounts-migration.json`, with identifiers and hashes only |
| Quota results and short-lived access tokens | Memory |
| Live Antigravity credentials | [Native credential storage](docs/antigravity-integration.md#signed-in-credential), written on confirmed switch |
| Diagnostic logs | OS application log directory |

Older registry entries are preserved for compatibility. Only Antigravity accounts are loaded into
the UI or used for account actions. Credentials remain in the Rust backend.

The app uses Google's OAuth, profile and Antigravity quota endpoints. Its HTTP client refuses
redirects, and the browser handles user authorization. The app has no upstream updater.

See the [architecture](docs/architecture.md) for switches and storage migration. Report security
issues through [GitHub](https://github.com/maycomayko/orbitswitch/security/advisories/new).
