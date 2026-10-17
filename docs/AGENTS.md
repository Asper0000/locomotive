## 1. Project Identity

- **Name:** Locomotive
- **Purpose:** Windows desktop GUI file manager for `.manifest`, `.manifests`,
  and `.lua` files. It copies files into configurable local-library folders,
  groups them for browsing, and deletes only its own stored copies.
  Default destinations match the folders RustyTools reads from:
  `<Steam>\depotcache` for manifests, and RustyTools' configured
  `[lua] dir` (falling back to `<Steam>\config\stplug-in`) for Lua files.
  It must not delete installed game files or alter account entitlements.
- **Language:** Rust, edition 2024 (requires Rust >= 1.85).
- **UI Framework:** `iced`, pinned to an exact version in `Cargo.toml`.
- **Renderer:** `iced_wgpu` (Vulkan / DX12 / Metal) with the `tiny-skia`
  software fallback that Iced provides by default.
- **Targets:** Windows 10/11 (x86_64). macOS and Linux are not targets and
  must not be assumed or advertised.

### The Lua source rule

RustyTools is the process that actually loads `.lua` files into Steam.
Locomotive's job is to display exactly what RustyTools will load, plus
any installed game whose files have drifted out of that set. This is
why `paths.conf` is not authoritative for the Lua directory.

- **Primary Lua directory** = `<Steam>\rustytools.toml`'s `[lua] dir`,
  resolved by `backend/rusty_tools_config.rs`. If the key is absent,
  the primary is `<Steam>\config\stplug-in`. `backend/archive.rs` uses
  `rusty_tools_config::configured_lua_dir` for its default and
  `explicit_lua_dir` to override any `lua=` value in `paths.conf`.
- **No other directory may be treated as primary.** Files left over in
  the previous default (usually `stplug-in` after the user reconfigured
  RustyTools) do not belong to the library. `library::scan` only
  enumerates the configured directory for its normal pass.
- **Orphan exception.** If an app is installed in Steam (an
  `appmanifest_<appid>.acf` exists) and its `.lua` currently lives in
  `stplug-in` rather than the primary directory, `library::scan` creates
  an entry with `LibraryEntry::source_warning: Some(...)`. The entry's
  `.lua` is not in `files`; it is in `source_warning.current_paths`. The
  UI shows a warning card with a "Move to configured folder" button.
  `importer::fix_orphan_entry` copies the file into the configured
  directory and deletes the original; a follow-up refresh clears the
  warning. `importer::remove_entry` also deletes these orphaned `.lua`
  files when the user picks Remove instead of Fix.
- **Locomotive never executes Lua.** `backend/lua_meta.rs` reads
  `addappid(...)` lines for grouping only. `setappticket`,
  `seteticket`, `http_get` and similar functions are RustyTools'
  business and are never modelled here.
- **`paths.conf` still exists**, but only its `manifest=` entry is
  authoritative. `lua=` is a fallback used when RustyTools' config has
  no explicit `[lua] dir`.
## 2. Non-Negotiable Code Rules

1. **Modularity first.** No file may exceed ~300 lines of non-comment code.
   If a module grows past this, split it.
2. **Elm architecture, strictly.** State lives under `src/state/`. Messages
   live in `src/message.rs`. Update logic lives under `src/update/`. View
   logic lives under `src/views/`. Never mix these responsibilities.
3. **No side effects in view code.** A view function may only read state and
   return an `Element`. It must never mutate state, touch the filesystem, or
   spawn a task directly. Every side effect is a `Task` returned from
   `update`.
4. **Comments explain *why*, not *what*.** Do not write
   `// increment the counter`. Write
   `// Steam expects the manifest id as a decimal string, not hex`.
   Doc comments (`///`) are required on every public item.
5. **No `unwrap()` in production paths.** Use `?` with a real error type, or
   `expect("...")` whose message states the invariant being asserted. GUI
   code is user-facing; a panic is a crash the user watches happen.
6. **Exact dependency pinning.** Every dependency in `Cargo.toml` uses an
   exact `=x.y.z` version. Iced is pre-1.0 and does break between minors.
7. **Dark theme is the default and must stay the default.** The theme is
   built in `src/theme.rs`. Do not add a light variant unless explicitly
   asked for one.
8. **No magic colors.** Every color is a named constant in `src/theme.rs`.
   If you need a new color, add the constant there first, then use it.

## 3. Repository Layout

```text
.
├── AGENTS.md
├── README.md
├── CHANGELOG.txt
├── BUILD_INSTRUCTIONS.md
├── BUILD_WINDOWS.ps1
├── BUILD_LINUX.sh
├── Cargo.toml
├── Cargo.lock
├── assets/
└── src/
    ├── main.rs
    ├── app.rs
    ├── assets.rs
    ├── message.rs
    ├── theme.rs
    ├── state/
    │   ├── mod.rs
    │   └── library_state.rs
    ├── update/
    │   ├── mod.rs
    │   └── window.rs
    ├── views/
    │   ├── mod.rs
    │   ├── components.rs
    │   ├── titlebar.rs
    │   ├── sidebar.rs
    │   ├── dropzone.rs
    │   ├── library.rs
    │   ├── options.rs
    │   └── about.rs
    └── backend/
        ├── archive.rs
        ├── file_dialog.rs
        ├── importer.rs
        ├── keyvalues.rs
        ├── library.rs
        └── steam.rs
```

**Where new code goes:**

- A new user-facing screen → a new file in `src/views/` plus a `View` variant.
- A new window action → `src/update/window.rs`.
- A new color or metric → `src/theme.rs`.
- Anything that touches the filesystem → `src/backend/`. `update` owns the
  action and returns any resulting `Task`; views must never call `std::fs`.
- Steam metadata access is read-only and only used to improve game names and
  grouping. Storage validation must reject folders inside Steam installations.
  Do not add code that writes to Steam's active folders, changes entitlements,
  or launches uninstallers as part of library removal.

## 4. The Theme Contract

The theme is built once in `theme::app_theme()`. If you change a color you
must (a) change it in `src/theme.rs` only, (b) update the table below, and
(c) eyeball the result on both Windows and Linux.

| Token             | Value     | Usage                                        |
|-------------------|-----------|----------------------------------------------|
| `BG_BASE`         | `#0E0E0E` | Window background, deepest layer             |
| `BG_SURFACE`      | `#141414` | Sidebar, cards, dropzone                     |
| `BG_ELEVATED`     | `#1A1A1A` | Titlebar, active nav item                    |
| `BG_HOVER`        | `#202020` | Hover states                                 |
| `MODAL_BACKDROP`  | `rgba(0, 0, 0, 0.82)` | Dimmed background behind guide dialogs     |
| `BORDER`          | `#252525` | Card and dropzone borders                    |
| `BORDER_SUBTLE`   | `#1C1C1C` | Hairline dividers                            |
| `TEXT_PRIMARY`    | `#DDDDDD` | Headings, primary labels                     |
| `TEXT_SECONDARY`  | `#888888` | Descriptions, metadata                       |
| `TEXT_MUTED`      | `#555555` | Footnotes, counts, disabled text             |
| `ACCENT`          | `#DDDDDD` | Focus / drag-over highlight (neutral)        |
| `SUCCESS`         | `#388E3C` | Affirmative states (a toggle that is on)     |
| `DANGER`          | `#B43E35` | Destructive hover only                       |

The palette is deliberately **monochrome**. The accent is near-white, not a
brand hue. Red appears *only* on hover of a destructive control, so the UI
reads as pure dark gray at rest. Do not introduce a blue, purple, teal, or
green anywhere.

## 5. Iced 0.14 API Traps

These are version-specific and will silently break copy-pasted code:

1. `iced::application(boot, update, view)` — **no title argument**. Set the
   title with `.title("...")` on the returned builder.
2. Window controls take a `window::Id`. This app uses the built-in
   `window::Id::MAIN` constant rather than tracking the id in state.
3. File drops arrive as `window::Event::FileDropped(PathBuf)` from the
   `iced::window::events()` subscription. This requires
   `drag_and_drop: true` in the window settings.
4. **Wayland does not implement file drop.** The dropzone will not receive
   files under a Wayland session. This is upstream, not a bug here. State it
   plainly in any user-facing docs.
5. Styling is done with closures: `.style(|theme, status| Style { .. })`.
   There is no `StyleSheet` trait to implement in this version.

## 6. Build System Contract

- `scripts/BUILD_WINDOWS.ps1` is **the** build entry point.
  Documentation must not tell users to run `cargo build` directly.
- Both scripts must be idempotent. Running either twice must not fail.
- Both scripts must probe for their required tooling and, when something is
  missing, print the exact install command for that platform and exit
  non-zero. They must never fail silently.
- Windows output: `dist/locomotive.exe`.
- Adding a tool dependency means adding a probe for it **and** documenting
  it in `BUILD_INSTRUCTIONS.md`.

## 9. Mandatory Change Logging

Every code change must be recorded in `CHANGELOG.txt` at the repository
root before the change is considered complete.

- Entries are in reverse-chronological order, newest first.
- Each entry has a date, a short title, and a bullet list.
- Each bullet states **what** changed and **why**, unless the reason is
  genuinely obvious from the what. "Fixed the build" is not acceptable.
  "Removed the 'palette' feature because it does not exist on iced 0.14.0
  and Cargo refused to resolve the dependency" is.
- Do not delete or rewrite old entries. Append corrections as new entries.
- The log is plain text on purpose, so that any editor, grep, or language
  model can read it without a parser.

## 10. Output Format Preference

When you produce code changes, prefer the following formats in order.
Choose the first one the environment allows.

1. **A `.zip` archive** containing the full changed files, if file uploads
   are available. This is the most reliable and the easiest to review.

2. **A self-applying patch script.** On Windows, a `.bat` file. On Linux,
   a `.sh` file. The script must write every changed file verbatim and be
   safe to run twice. Use an embedded PowerShell payload with literal
   here-strings inside the `.bat`, because batch cannot contain arbitrary
   text. Escape nothing in the payload; here-strings do not interpolate.

3. **Individual file blocks**, clearly labelled with their target path, as
   a last resort.

Never produce a partial patch that assumes the reader will fill in the
gaps. Every output must be complete and runnable as-is.

## 11. Branding Assets

There is exactly one branding source: `assets/icon.png`.

- It is the titlebar brand mark, loaded by `components::brand_icon`.
- It is the window icon, loaded by `app::window_icon`.
- On Windows, `assets/icon.ico` is derived from it once and embedded into
  the `.exe` by `build.rs`. The `.ico` is a build artifact; regenerate it
  whenever the PNG changes.
- There is no separate SVG logo. Do not reintroduce one; keeping a single
  source prevents the two from drifting apart.
- Both the window icon and the brand mark are gated on the `has_icon_png`
  cfg that `build.rs` emits. Never call `include_bytes!` on these paths
  without that guard, or a fresh checkout will fail to compile.