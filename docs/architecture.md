# Architecture

Code map, data flow, switch sequences and persistence mechanisms. External contracts live in
[antigravity-integration.md](antigravity-integration.md); data inventory lives in [SECURITY.md](../SECURITY.md).

OrbitSwitch is a [Tauri 2](https://tauri.app) app: Rust executes all side effects and React renders
the backend snapshot, derives presentation state and requests operations through commands and plugins.

## Data flow

```text
 tray click ─▶ shell (popover / window) ─▶ React UI
                                            │  invoke(command)        ▲ "orbitswitch://snapshot"
                                            ▼                         │
                                  commands.rs ─▶ accounts / switcher / quota / scheduler
                                                 │
                                                 ▼
                                     Antigravity / Google / vault / registry
```

`Orbit` (`state.rs`) holds the backend state and emits a full `Snapshot` (`model.rs`) on every update.
The frontend never holds tokens or cookies; it keeps dialog state and derives account ordering, quota
severity and recommendations from the snapshot.

## Backend modules (`src-tauri/src`)

| Module | Owns |
|---|---|
| `main.rs` / `lib.rs` | Entry point, plugin wiring, startup and command registration |
| `commands.rs` | Tauri command handlers |
| `state.rs` | Shared `Orbit` state, snapshot emission, operation lock, token minting |
| `model.rs` | Frontend snapshot types (`AccountView`, `ClientStatus`, `QuotaState`, `Operation`) |
| `provider.rs` | Legacy registry tags, retained for backward compatibility |
| `accounts.rs` | Antigravity browser sign-in, removal and live sign-in adoption |
| `switcher.rs` | Antigravity account switch, stop and restart |
| `transaction.rs` | Shared rollback handling for client credential writes |
| `quota.rs` | Concurrent quota and plan refresh |
| `scheduler.rs` | 5-minute sign-in sync and opt-in quota refresh |
| `autostart.rs` | Open at Login (SMAppService on macOS, Run key on Windows) |
| `registry.rs` | Account metadata and fingerprints in `accounts.json` |
| `registry_migration.rs` / `registry_migration_tests.rs` | One-time storage transfer before startup, copy journal, destination verification and resumable source cleanup |
| `vault.rs` | Antigravity refresh tokens in the OS credential store |
| `tokens.rs` | In-memory access-token cache |
| `secret_store/` | macOS Keychain (`/usr/bin/security`) and Windows Credential Manager primitives |
| `antigravity/` | Antigravity installation, `app.asar` version reader, OAuth client discovery, credential bundle, live store and processes |
| `google/` | Google OAuth (PKCE), loopback callback server and Cloud Code client |
| `shell.rs` / `tray.rs` | Popover / tray window and tray icon |
| `error.rs` | Serializable error codes localized by the frontend |
| `system_tests.rs` | Opt-in read-only checks against installed Antigravity (`--ignored`) |

## Frontend (`src`)

| Path | Owns |
|---|---|
| `main.tsx` | React root and shared stylesheets |
| `types.ts` | TypeScript mirror of `model.rs` |
| `ipc.ts` | Command and event names |
| `app.tsx` / `app.css` | Panel composition, dialog routing and account ordering |
| `menus.ts` | Native context menus and clipboard / URL actions |
| `quota.ts` | Quota severity, usable share and *Most left* recommendation |
| `client.ts` | Antigravity client labels |
| `i18n.ts` / `format.ts` | English / Simplified Chinese copy and time / percentage formatting |
| `hooks/` | Snapshot subscription, macOS popover auto-sizing and relative-time ticker |
| `components/` | UI components, icons and styles (`sheet.css` shared across sheets) |
| `styles/` | Design tokens and base styles |

## Operations and concurrency

One user operation runs at a time (`Orbit::begin` returns `Busy` otherwise): `adding`,
`removing`, `switching` (with step progress), `stopping` or `restarting`. A shared async
`sync_lock` serializes live-credential sync, quota/token refresh and switches so a background refresh
cannot race an active switch. Client installation and process checks run outside the snapshot mutex.

During Gemini browser sign-in, Rust retains the original OAuth URL with the pending cancellation
sender. The waiting sheet receives only link availability; its copy command writes the original URL
to the system clipboard in Rust. The URL is retired on callback, cancellation, timeout or operation
failure. A default-browser launch failure keeps the copy action available.

### Antigravity switch sequence

1. In **Preparing**, redeem the target account's refresh token and confirm its Google account ID.
   On `invalid_grant`, mark the target account `Reauth` and end the operation.
2. Resolve the outgoing live identity and save its refresh token to `vault.rs` when it matches a
   saved account.
3. **Closing**: request desktop exit and wait for the selected installation’s bundled processes.
   If they remain running after 8 s, ask the user to close the desktop app before changing credentials.
4. **Writing**: write keychain and `~/.gemini/jetski-standalone-oauth-token`, read back and verify;
   restore the previous credential and relaunch on failure.
5. **Launching**: reopen Antigravity and confirm the saved sign-in still matches the target. Treat
   failure to save OrbitSwitch’s local bookkeeping separately from the desktop launch.

## Persistence

- Before loading normal account state, `registry_migration.rs` consumes the previous storage
  namespace into the current one. Current metadata, grants and historical account removals take
  precedence. A journal records hashes for staged credential copies before writes, allowing retries
  to repair an incomplete copy without treating it as an existing current grant. The merged registry
  is atomically written, synced and read back before source cleanup. Cleanup interruptions resume on
  the next launch. Historical entries for other clients remain passive registry data. No old-store
  fallback remains after the transfer; official client credentials and processes are unaffected.
- `registry.rs` atomically writes `accounts.json` via a temporary file rename. The Antigravity
  refresh-token fingerprint prevents re-adopting a removed account until the native client signs
  into a different session.
- `vault.rs` stores Antigravity refresh tokens under service `orbitswitch`. On macOS,
  `secret_store/macos.rs` uses `/usr/bin/security` so items share the system tool's ACL with
  Antigravity's `go-keyring`.
