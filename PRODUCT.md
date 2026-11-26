# Product

OrbitSwitch is a macOS menu bar and Windows tray utility to switch between the user's own
Antigravity subscriptions and view their coding quotas. See [design](docs/design.md) and
[verified scope](docs/status.md).

- Detect the active Antigravity sign-in, and add another account through Google browser sign-in.
  Removing a saved account only removes it from OrbitSwitch.
- Switch manually. Before closing Antigravity or its related processes, show a confirmation.
  Verify the target, save the outgoing sign-in, write and read back both credential copies,
  and reopen the official desktop app.
- Restore the prior credential copies if a write or readback fails. A rollback failure leaves the
  client closed. A launch failure after a verified write reports that the account changed.
- Show reported quota windows and reset times. Preserve the last good quota as stale after a
  transient failure. Mark reauthorization when Google revokes a grant.
- Support macOS 26+ and Windows 10/11 x64. Install updates to this fork manually.

This fork only manages Antigravity. It has no model proxy, relay, account pool, automatic rotation,
or server.
