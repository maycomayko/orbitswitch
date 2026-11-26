# Design

Visual presentation, interaction, icons and screenshots for the app. Product rules live in
[PRODUCT.md](../PRODUCT.md); build and signing live in [release.md](release.md).

## Surfaces

- **macOS (26+)**: menu bar popover with no Dock icon. It fits the account list or open sheet (up to
  680 pt) and dismisses on focus loss. The WebView is mounted inside the native Liquid Glass container via
  `window-vibrancy`'s `content_view` API with 18 pt corner clipping and no extra system window shadow.
- **Windows**: compact tray window that hides to the tray on close.
- Both platforms share the React UI with `platform-macos` / `platform-windows` root classes. Left
  click toggles the window; right click opens the native tray menu.

## Interaction

- Confirmations and progress slide up as in-surface sheets (`Enter` confirms,
  `Escape` cancels). Confirmations list the affected processes.
- Long-running operations show backend step progress; success closes after 1.6 s and errors remain
  until dismissed.
- Browser sign-in shows a cancellable waiting sheet until authentication completes. The sheet also
  offers **Copy sign-in link** for a private window or another browser on the same computer, with
  a disabled action during preparation and inline copy success/failure feedback.
- Account actions (`Sign In Again…`, `Copy Email`, `Remove Account…`) open in a native context menu
  on right click or `⋯`. App actions live in the header's `⋯` menu.
- **Show plans and quota** is unchecked by default. Enabling it opens a risk confirmation with
  **Cancel** initially focused. Selecting the checked item turns off future checks.

## Account card

- Avatar (profile photo or initial), display name and email. The plan badge appears when plans
  and quota are enabled.
- `Current` marks the active account. When quota is enabled, `Most left` marks the recommended switch target.
- Quota rows render each reported group and window meter with remaining percentage. Meters turn amber
  below 35 % and red below 10 %, with a low-quota recovery footnote and reset time on hover.
- Hover or keyboard focus highlights switchable cards; clicking or activating the account identity
  opens the switch confirmation.

## Visual tokens and icons

- `src/styles/tokens.css` defines all light/dark surface, text, mineral-green accent (`#476f5b` /
  `#82aa98`) and semantic status tokens. Typography uses the system font at 13 px (11–11.5 px
  secondary) with tabular numerals.
- `assets/brand/app-icon.svg` owns the three-segment rotary selector app icon;
  `assets/brand/tray-template.svg` owns the monochrome tray template mark.

Regenerate bundled icons after editing the SVGs:

```bash
rsvg-convert -w 1024 -h 1024 assets/brand/app-icon.svg -o /tmp/orbitswitch-icon.png
bun tauri icon /tmp/orbitswitch-icon.png -o src-tauri/icons
rm -rf src-tauri/icons/android src-tauri/icons/ios src-tauri/icons/Square*Logo.png src-tauri/icons/StoreLogo.png src-tauri/icons/64x64.png
rsvg-convert -h 36 assets/brand/tray-template.svg -o src-tauri/icons/tray-template.png
```

## Screenshots

Render README screenshots from the real UI with fictional accounts:

```bash
bun scripts/screenshot/render.ts --readme   # requires Google Chrome and ImageMagick
```

Writes the English and Chinese README images in light and dark mode. Omit `--readme` to also
render the switch and sign-in sheets.
