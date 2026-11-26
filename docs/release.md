# Build and package

OrbitSwitch builds locally with Bun, the pinned Rust toolchain, and the Tauri platform prerequisites
listed in [CONTRIBUTING.md](../CONTRIBUTING.md). The fork uses manual distribution and does not check
the upstream release feed.

```sh
bun install --frozen-lockfile
bun tauri build --bundles app
```

On macOS the native-architecture bundle is produced under `src-tauri/target/release/bundle/macos/`.
Verify it with `codesign --verify --deep --strict` and verify the executable architecture with
`lipo -archs`. Local bundles are ad-hoc signed and are not notarized. For Windows use
`bun tauri build --bundles nsis` on Windows.

The app keeps the earlier bundle identifier and Antigravity credential store so existing saved
accounts stay available. The source version is defined in `src-tauri/Cargo.toml`.
