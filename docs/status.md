# Status

Platform support and feature verification scope. The external client contract lives in [antigravity-integration.md](antigravity-integration.md).
Update this page in the same PR whenever verification scope changes.

## Platforms

| | macOS 26+ | Windows 10/11 x64 |
|---|---|---|
| Build target and CI configuration | Yes | Yes |
| Installer | Native architecture `.app`, ad-hoc signed | NSIS `-setup.exe`, unsigned |
| Verified on a real installation | Yes (macOS 27) | **Not verified** ([#3](https://github.com/shengyy/orbitswitch/issues/3)) |

## Installed-client verification (macOS)

- **OrbitSwitch storage upgrade** (0.2.1 → 0.2.2, macOS 27):
  - Combined earlier and current data into 14 saved accounts across all three providers, copied
    missing Antigravity grants, retained the current grant, and consumed the old registry and vault
    entries after verification.
  - A second launch retained the identical registry and did not repeat the transfer. Official-client
    credential files and the running desktop-client processes remained unchanged.
- **Antigravity** (2.19.1, 2.22.0, 2.23.1, 2.26.0 on macOS 27):
  - OAuth client discovery, browser sign-in, adopting the active IDE sign-in, and repeated confirmed
    switches with Antigravity reopening on the target account.
  - Installed-version `User-Agent` discovery from `Resources/app.asar`, returning both Gemini and
    Claude/GPT quota groups across Pro and Ultra accounts.
  - Read-only quota and plan probes on 2.34.0 (macOS 27.2.1) returned a shared Gemini group and
    a separate Claude/GPT group, each with a five-hour and weekly window, for the tested Pro
    and Ultra accounts.
  - On 2.32.2 (macOS 27.2.1), the identity resolver used Google UserInfo for a live credential
    without usable ID-token claims. The same access token returned the account's plan and quota.
    Switching such a session in the installed app remains **not verified**.
  - On 2.35.2 (macOS 27.2.1), a read-only probe resolved the bundled version and decoded the
    native personal sign-in, which can include an account email without a raw ID token.
  - On that installation, the user added a second Google account through OrbitSwitch. The
    existing Free account remained current, and the added Pro account showed five-hour and
    weekly windows for both the Gemini and shared quota pools.
  - Switching between these accounts and reopening projects, conversations, and authorized
    tools remain **not verified**.
  - The fork skips quota queries for confirmed Free accounts while continuing to refresh their
    plans. Network settings can use system networking or an optional local HTTP proxy.
  - Native menus, clipboard copy and Open at Login through SMAppService.
  - The original OAuth request reaches Google's sign-in form in a fresh Chrome 156.0.8078.48
    incognito context on macOS 27.0.1. Completing private-window account enrollment remains
    **not verified**.
## Synthetic and loopback verification

- **Antigravity quota**: the loopback check uses the current installed hub identity. Synthetic
  cases cover reset times, distinguishing empty groups from exhausted windows, and preserving
  additional reported groups outside account rankings.
- **Antigravity identity**: synthetic coverage verifies missing or unreadable ID tokens, the
  ID-token fallback, rejection of an unresolved identity, retrying a failed fingerprint write,
  writing the verified email without an ID token, and restoring different keyring and fallback
  states when readback fails.
- **Account storage upgrades**: merging earlier and current registries, retaining current grants and
  desktop bindings, honoring explicit native-account removals, original Google-only metadata,
  malformed/unsupported source rejection, missing grants, failed token writes/readback, failed
  registry publication, credential-read failure, source grant rotation, interrupted cleanup, and
  repeated startup without account resurrection. Linked source/destination directories are rejected
  before any transfer or cleanup.
- **Antigravity browser enrollment**: the original authorization URL preserves callback and state;
  loopback callbacks require matching state and no browser cookies. The waiting sheet's copy action
  has preparation, success and retry states. Real private-window Google sign-in remains **not verified**.
## Known limitations

Accepted limitations are tracked under
[`deferred`](https://github.com/shengyy/orbitswitch/issues?q=is%3Aissue+is%3Aopen+label%3Adeferred).
