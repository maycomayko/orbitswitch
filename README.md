# OrbitSwitch for Antigravity

[简体中文](README.zh-CN.md) · [Compatibility and verification](docs/status.md)

A menu bar and tray utility for switching between your own Antigravity subscriptions. See the
current account, remaining quota, and reset times. The app manages Antigravity Desktop.

<picture><source media="(prefers-color-scheme: dark)" srcset="assets/screenshots/hero-dark.png"><img src="assets/screenshots/hero-light.png" width="640" alt="Antigravity account list in OrbitSwitch"></picture>

## Use

1. Launch OrbitSwitch from the menu bar or system tray. It detects the native Antigravity sign-in.
2. Use **Add account** to authorize the second Google account in your browser.
3. Select an account and confirm to switch. OrbitSwitch verifies the target, closes the affected
   processes, writes and checks the new credentials, then reopens Antigravity Desktop.

Quotas come from your Antigravity plan. Shared model pools reported by Antigravity can include
Claude and GPT models. OrbitSwitch never contacts those standalone services.

Saved Antigravity accounts from earlier builds are preserved. Install a newer fork build manually
to upgrade. See [build instructions](docs/release.md) and [credential storage](SECURITY.md).

Use **⋯ → Network settings** to follow your system network or set a local HTTP proxy port.
The connection test checks Google before saving. The setting only applies to OrbitSwitch's
requests, and takes effect immediately.

MIT. Based on [upstream OrbitSwitch](https://github.com/shengyy/orbitswitch). Unaffiliated with Google.
