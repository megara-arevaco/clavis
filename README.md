<div align="center">
  <img src="packaging/clavis.svg" width="132" height="132" alt="Clavis logo">
  <h1>Clavis</h1>
  <p><strong>Your terminal shortcuts and commands — one shortcut away.</strong></p>
  <p>
    <img alt="Rust" src="https://img.shields.io/badge/Rust-native-CE7B53?logo=rust&logoColor=white">
    <img alt="GTK" src="https://img.shields.io/badge/GTK-4-78AE62?logo=gtk&logoColor=white">
    <img alt="Platform" src="https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-A8F5DF?logo=linux&logoColor=17201E">
    <img alt="License" src="https://img.shields.io/badge/License-MIT-A8F5DF">
  </p>
</div>

Clavis is a small desktop companion for remembering terminal shortcuts and everyday
commands without leaving your workflow. Open Clavis from a compositor shortcut or optionally
tap and release **Ctrl twice**. Choose a tool and search one reference or all tools in a compact panel. Click a row
to copy it; Clavis **never executes commands automatically**.

It works locally, without a Clavis account or hosted service. Reference content is
bundled with the executable; internet access is only needed when you open external
documentation, download dependencies, or fetch a release.

> [!NOTE]
> Clavis is under active development. Linux and Windows builds are unsigned development
> builds. Optional Wayland double-Ctrl detection requires explicit access to selected
> keyboards; Windows uses a system-wide keyboard hook while Clavis is running.

<p align="center">
  <img src="docs/screenshots/clavis.png" width="560" alt="Clavis showing Git command references in two columns">
</p>

## What Clavis does

- Optionally opens and hides its reference with two quick, released Ctrl taps; a compositor shortcut needs no input-device ACL.
- Displays Ghostty shortcuts and references for Git, SSH, curl, tmux, Vim, fzf,
  ripgrep, fd, jq and htop.
- Keeps commands and keyboard shortcuts clearly identified.
- Searches the selected tool or all tools; global results identify their tool and type.
- Saves favorites locally and creates/edits, imports/exports a validated personal reference catalog.
- Copies full entries without executing them or modifying your terminal input. Placeholder and high-impact warnings appear before copying.
- Focuses search when opened; Ctrl+F returns focus, Enter copies the first result, and Esc hides the panel.
- Offers normal/large text and automatically changes to one column when the window is narrow.
- Remembers your last selected tool between launches.
- Provides a tray menu to show, hide, choose a tool, or quit.
- Reads simple explicit Ghostty keybindings or falls back to bundled defaults.
- Supports multiple configured keyboards on Wayland, with reconnection attempts.
- Offers an Info screen with persistent input diagnostics, safe compositor setup, privacy notes, reference source/version/review metadata and help that does not depend on hover.
- Offers official documentation links and tool-specific usage qualifications.

The extra tools are a curated developer-oriented selection, **not a measured global
usage ranking**. Research sources and content boundaries are documented in
[RESEARCH.md](RESEARCH.md).

## Requirements

### To use the Linux application

- A Linux desktop with GTK4 and a D-Bus session.
- X11, or Wayland with a compositor shortcut (recommended) or explicitly authorized keyboard access for optional double-Ctrl detection.
- A StatusNotifier/AppIndicator host for the tray icon. Ubuntu GNOME usually has
  its AppIndicator extension enabled; other desktops may need one installed.
- For prebuilt executables, an architecture and distribution compatible with the
  build. Current release scripts produce native Linux packages, not universal
  static binaries.

The Debian package declares runtime dependencies derived from its actual executable.
The binary archive still requires system GTK4, X11/XTest and D-Bus libraries. Neither
format requires Rust, Python or Node.js to run. The referenced terminal tools do not
need to be installed just to browse their entries.

### To use the Windows application

- Windows 10 or later, x64.
- The setup executable includes GTK runtime libraries and the system tray integration.
- The global Ctrl listener runs only while Clavis is open. It checks Ctrl press/release
  state for double Ctrl and does not record or transmit ordinary keystrokes.
- The installer offers an optional login-startup shortcut, unchecked by default.

### To develop Clavis

- A recent [Rust toolchain](https://rustup.rs/) and Cargo.
- Git, a C toolchain, pkg-config and native development libraries.
- `dpkg-dev` to build Debian packages.
- `librsvg2-bin` and `ffmpeg` only when regenerating the icon.
- On Windows, Rust's GNU x64 toolchain, GTK4 from MSYS2 UCRT64, and Inno Setup 6 to build
  the native setup executable.

On Ubuntu/Debian:

```bash
sudo apt install cargo rustc build-essential pkg-config \
  libgtk-4-dev libx11-dev libxi-dev libxtst-dev libdbus-1-dev dpkg-dev
```

On Windows, install the MSYS2 UCRT64 packages `mingw-w64-ucrt-x86_64-gtk4`,
`mingw-w64-ucrt-x86_64-toolchain`, `mingw-w64-ucrt-x86_64-pkgconf`, and
`mingw-w64-ucrt-x86_64-adwaita-icon-theme`, then install Rust's
`stable-x86_64-pc-windows-gnu` toolchain and Inno Setup 6. Build with:

```powershell
$env:MSYS2_PREFIX = 'C:\msys64\ucrt64'
$env:PATH = "$env:MSYS2_PREFIX\bin;$env:PATH"
$env:PKG_CONFIG_PATH = "$env:MSYS2_PREFIX\lib\pkgconfig"
rustup toolchain install stable-x86_64-pc-windows-gnu
cargo +stable-x86_64-pc-windows-gnu build --release --locked --target x86_64-pc-windows-gnu
./scripts/package-windows.ps1
```

The Windows setup executable bundles GTK DLLs, schemas, icons, theme, and image
loaders. The Linux `.deb` is the native installer for Debian and Ubuntu.

## Using Clavis

### Open the panel

```bash
clavis --show
```

**Recommended on Wayland (no keyboard ACL):** bind `clavis --show` in your compositor's keyboard-shortcut settings. On GNOME, add it under **Settings → Keyboard → View and Customize Shortcuts → Custom Shortcuts** (wording can vary by version). Set the command to `clavis --show` and choose a key combination. The compositor delivers this shortcut without an input-device ACL. Optional double-Ctrl monitoring is independent and continues only if separately configured.

To start silently and wait for double Ctrl:

```bash
clavis
```

Only one instance runs per desktop session. The application menu entry is **Clavis**.
Press Esc or close the panel to hide it; the listener stays running. Quit through the
tray menu, or run `pkill -x clavis`.

### Use double Ctrl

Press and release Ctrl, then press and release it again within about **380 ms**.
Either Ctrl key works. Clavis triggers on the second release, so ordinary Ctrl+C,
Ctrl+V and other chords cancel detection rather than opening the panel. Long holds
and repeated keydown events do not trigger it.

Ctrl by itself does not insert letters, and Clavis does not delay your typing.
Events still reach the focused application; that application may have its own
behavior for double Ctrl. Detection and modifier state are tracked per keyboard.
Wayland also controls window focus and placement; always-on-top is not guaranteed.

```bash
CLAVIS_TAP_DELAY=0.5 clavis
```

### Enable keyboard access on Wayland

List keyboard candidates and configure the devices you want to use:

```bash
clavis --list-keyboards
/usr/lib/clavis/enable-keyboard.sh --all
```

The second command is the setup script installed by the Debian package. For a source
or archive install, use `./enable-keyboard.sh --all` from its directory. You can also
select individual devices:

```bash
./enable-keyboard.sh /dev/input/event3 /dev/input/event15
```

The script lists the chosen devices and asks for confirmation before granting your
user read access. Restart Clavis after changing the selection. Check access with:

```bash
clavis --check-input
clavis --diagnose
```

`--check-input` reports every configured device. `--diagnose` prints connection
notices and double-Ctrl detections, not ordinary keystrokes.

> [!IMPORTANT]
> Keyboard ACLs can reset after a reboot or reconnect. Rerun setup when access is
> lost. Granting this access also lets other processes running as your user read
> those keyboards, including password keystrokes. Clavis does not save or transmit
> keys, run its interface as root, or add input-group membership.

Configured keyboards are reopened after disconnects, with name checks before reused
input paths are opened. Identically named keyboards can be ambiguous; stable device
paths are preferable. Newly added keyboards must be configured explicitly. Some
composite receiver interfaces are named Mouse while exposing keyboard capabilities.

Double Ctrl remains optional. It does require explicit access to selected keyboards; granting an ACL also makes those keystrokes readable by other processes running as your user. Clavis never stores or transmits input.

### Browse and copy references

Choose a tool, search this tool or **All tools**, and click a result to copy the complete text. The search field is focused whenever the panel opens; use **Ctrl+F** to return to it, **Enter** to copy the first matching row, and **Esc** to hide the panel. The Favorites toggle filters to saved references, and each row has a keyboard-accessible star control.

Placeholders are highlighted and curated entries with repository, file, remote-data, process, privacy or terminal-session impact carry an inline warning before copying. Warnings are attached to reviewed entries rather than inferred only from command-name heuristics. The list is conservative and not exhaustive: inspect every command and replace placeholders before running it. The entry-by-entry decisions are recorded in [`research/catalog-risk-review.json`](research/catalog-risk-review.json). Keyboard shortcuts are labeled separately from terminal commands. Open **Info** for input diagnostics, the no-ACL compositor setup, privacy explanations, tool/source/version/review details, text sizing, and import/export actions. Reference versions and review dates that are not maintained are honestly shown as **Not recorded**. The **Docs** link opens the selected tool's upstream manual.

### Personal references

Import/export is local JSON with `schema_version: 1`. The top-level object contains a `references` array. Each tool entry has `name`, `kind` (`Commands` or `Keys`), HTTPS `source`, `note`, `origin`, `version`, `reviewed`, and `entries`; each entry has `key`, `action`, `origin`, `version`, `reviewed` (an ISO date or `Not recorded`), and optional `warning`. Use `Not recorded` rather than guessing an upstream version or review date. Unknown fields, non-HTTPS sources, invalid metadata, unsupported schema versions, duplicates, and oversized catalogs are rejected. Imports are validated fully before they are added; a name collision rejects the import instead of replacing an existing reference. No imported command is executed.

Example entry shape:

```json
{
  "schema_version": 1,
  "references": [{
    "name": "My tool",
    "kind": "Commands",
    "source": "https://example.org/docs",
    "note": "Personal notes",
    "origin": "Maintainer notes",
    "version": "Not recorded",
    "reviewed": "Not recorded",
    "entries": [{
      "key": "my-tool <target>",
      "action": "Inspect a target",
      "origin": "Maintainer notes",
      "version": "Not recorded",
      "reviewed": "Not recorded",
      "warning": "Check the target before use."
    }]
  }]
}
```

The imported collection is stored in `personal-catalog.json` under Clavis' XDG configuration directory; it supplements bundled references. Use **Info → Manage personal references** to create or edit local entries without importing a file. Imports reject collisions instead of overwriting; edits require selecting an existing personal reference. Exports, including an empty catalog, are valid schema-version-1 JSON and refuse to replace an existing destination. Local settings use atomic writes and reject symlink paths. Favorites and text size are stored separately on the same machine. Clavis has no telemetry or hosted account.

Bundled references are identified as archived sources, not as the version currently installed on your computer. Only versions/dates stated in the included source snapshots are shown; unpinned revisions and Clavis review dates remain **Not recorded**. See [`research/RESEARCH.md`](research/RESEARCH.md) for source-by-source evidence.

## Development

Clone the repository and work from its root:

```bash
git clone https://github.com/megara-arevaco/clavis.git
cd clavis
cargo test --locked
cargo build --release --locked
```

Run the application:

```bash
cargo run -- --show
```

The unit suite covers double-Ctrl timing, releases, held keys, chord cancellation,
keyboard capability parsing, synthetic reconnection, entry-by-entry risk classifications
for all bundled references, source fingerprints, safe persistence, grid positions and
tray pixel layout. It never captures keyboard events.

For isolated GTK interaction checks, provide a temporary config directory and private
D-Bus session:

```bash
tmp=$(mktemp -d)
CLAVIS_TEST_CONFIG_HOME="$tmp/config" dbus-run-session -- env ./target/debug/clavis --test-mode
```

The harness refuses to start unless `CLAVIS_TEST_CONFIG_HOME` is explicitly set to a non-symlink child of the system temporary directory; it then sets `XDG_CONFIG_HOME` itself. This explicit harness disables the input listener, tray and device diagnostics. It drives synthetic global
search, favorites, language, text scale, Info/refresh, personal editor create/edit,
import/export and Ctrl+F/Esc paths. It does not inject physical key events, inspect devices/permissions or access a
keyboard device; it is not a screen-reader, hardware, tray or compositor test.

For app-only GTK rendering, `--capture` also uses a separate test application ID
and skips the tray, listeners and device diagnostics. Both isolated modes require a
temporary config path:

```bash
tmp=$(mktemp -d)
CLAVIS_TEST_CONFIG_HOME="$tmp/config" dbus-run-session -- env \
  ./target/debug/clavis --tool Git --capture /tmp/clavis-git.png --width 430
```

The PNG contains only Clavis, not the desktop or other applications. Source captures
for English/Spanish and normal/large text are in `docs/screenshots/validation/`.
Hardware input, screen-reader behavior, tray hosting and compositor focus still need
separate verification in a real desktop session.

## Building Linux executables

Build a tested release binary, Debian installer and executable archive:

```bash
./build-dist.sh
```

The current version produces:

```text
dist/clavis_0.5.0_amd64.deb
dist/clavis-0.5.0-linux-amd64.tar.gz
dist/SHA256SUMS
```

Install the Debian package on a compatible Ubuntu/Debian system:

```bash
sudo apt install ./dist/clavis_0.5.0_amd64.deb
```

Alternatively extract the archive and run its installer:

```bash
tar -xzf dist/clavis-0.5.0-linux-amd64.tar.gz
cd clavis-0.5.0-linux-amd64
./install.sh
# Or: ./install.sh --user
```

The archive includes the compiled executable, desktop icon, installer, uninstaller
and keyboard setup script. Installing it does not invoke Cargo. Building from source
with `./install.sh` does require Cargo and the native build dependencies.

The Debian installer adds an application-menu entry and silent login autostart. The
Windows installer adds a Start menu shortcut and offers optional login startup,
unchecked by default. Installers **never grant keyboard permissions automatically**.
Personal settings and Linux ACLs are retained on uninstall. Remove a Debian
installation with `sudo apt purge clavis`, or use the archive's `uninstall.sh`
(`--user` for a user installation). Stop running instances separately.

## Project structure

```text
clavis/
├── src/                   Rust application and desktop integration
│   ├── main.rs             Lifecycle and event dispatch
│   ├── input.rs            Linux Wayland/evdev and double Ctrl
│   ├── input_windows.rs    Windows global keyboard hook
│   ├── ui.rs               GTK reference grid, search, copy and captures
│   ├── tray.rs             Linux StatusNotifier icon and menu
│   ├── tray_windows.rs     Windows tray icon and menu
│   └── catalog.rs          Reference content and remembered selection
├── packaging/              Icon assets, desktop entry and Windows installer recipe
├── scripts/                Icon regeneration and Windows packaging
├── tooling/github/         Optional CI and release workflow templates
├── docs/screenshots/       Application-only README screenshot
├── catalog.json            Extra terminal-tool references
├── shortcuts.json          Bundled Ghostty Linux reference
├── install.sh              Source and executable-archive installer
├── enable-keyboard.sh      Explicit keyboard permission setup
└── build-dist.sh            Linux binary, .deb, archive and checksums
```

## Publishing a release

Update the version in `Cargo.toml`, refresh `Cargo.lock`, update the README artifact
names, run the validation commands, and commit the change. Create a matching tag:

```bash
git tag -a v0.5.0 -m "Clavis 0.5.0"
git push origin v0.5.0
```

Optional workflow templates are included under `tooling/github/`. They are **not
active GitHub Actions workflows** until copied into `.github/workflows/`, committed
and pushed. Uploading workflow files through a GitHub CLI OAuth token requires the
additional `workflow` scope:

```bash
gh auth refresh --scopes workflow
mkdir -p .github/workflows
cp tooling/github/*.yml .github/workflows/
git add .github/workflows
git commit -m "Enable CI and release workflows"
git push
```

When enabled, the release workflow verifies the tag against the package version,
builds the Linux `.deb` and binary archive, creates the Windows setup executable on a
Windows runner, and publishes all installers with `SHA256SUMS` and generated release
notes. The CI template builds and uploads artifacts without publishing a release.
Neither workflow grants keyboard access or verifies physical keyboards.

Without these workflows enabled, build the Windows setup executable on a Windows
machine with the commands above and place it under `dist/windows/` in the release
checkout. Then build and publish the release manually:

```bash
./build-dist.sh
sha256sum dist/windows/*.exe >> dist/SHA256SUMS
gh release create v0.5.0 dist/clavis_0.5.0_amd64.deb \
  dist/clavis-0.5.0-linux-amd64.tar.gz dist/SHA256SUMS dist/windows/*.exe \
  --verify-tag --generate-notes --title "Clavis 0.5.0"
```

Builds are unsigned. The `dist/` directory is intentionally ignored: executables
belong in GitHub Releases, not in source control.

## Data locations and privacy

On Linux, personal settings live under `$XDG_CONFIG_HOME/clavis` (default
`~/.config/clavis`). On Windows, they live under `%APPDATA%\Clavis`:

- `keyboards`: selected input paths and expected device names.
- `last-tool`: your last selected reference.
- `personal-catalog.json`: validated personal references, kept alongside bundled content.
- `favorites.json` and `text-size`: local UI preferences.

Ghostty's explicit config bindings are read from `$XDG_CONFIG_HOME/ghostty/config` on
Linux and `%APPDATA%\ghostty\config` on Windows.
This is a lightweight reference parser, not a full effective-keymap resolver: includes,
resets and conditional bindings are not resolved. The other bundled references are in
`catalog.json`; the UI exposes their documentation source, upstream version and review
date, using **Not recorded** where the repository has not verified that metadata.

Clavis does not inspect shell history, terminal content or foreground window titles.
It has no telemetry and does not save keystrokes. Device ACLs are managed by the
operating system; to revoke one:

```bash
sudo setfacl -x "u:$(id -u)" /dev/input/eventN
```

`CLAVIS_KEYBOARD=/dev/input/eventN` temporarily selects one device. The old
`GHOSTTY_EE_KEYBOARD`, `GHOSTTY_EE_DELAY` variables and single-keyboard configuration
remain readable for migration; they do not restore the old ee trigger.

The Debian autostart entry is `/etc/xdg/autostart/clavis.desktop`. Override it with a
same-named user entry containing `Hidden=true` to disable login startup. Archive
installations use `~/.config/autostart/clavis.desktop`.

## Regenerating the icon

The source artwork is [`packaging/clavis.svg`](packaging/clavis.svg): a mint C around
a terminal prompt. Generate the application PNG and the tray's embedded ARGB pixmap
from that single source:

```bash
sudo apt install librsvg2-bin ffmpeg
./scripts/render-icon.sh
cargo build --release --locked
```

Generated icon files carry provenance sidecars. Their renderers are development tools,
not application runtime dependencies.

## License

Clavis is available under the [MIT License](LICENSE). Third-party dependencies
retain their own licenses; their versions are pinned in `Cargo.lock`. GTK4, D-Bus
and X11 runtime libraries are supplied by the operating system.
