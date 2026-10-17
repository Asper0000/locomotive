# Locomotive

Locomotive is a cross-platform desktop manager for local collections of Steam
depot manifest files (`.manifest` / `.manifests`) and Lua files (`.lua`). It
provides import staging, drag-and-drop, independently configurable local
storage folders, automatic library scanning, grouping by inferred game name,
and per-entry removal.

Locomotive copies selected files into the configured storage folders. When
Steam is detected, the default destinations are Steam's `config/stplug-in`
and `depotcache` folders; you can choose different folders in Options. The
app does not install or uninstall games or change account entitlements.
Removing an entry deletes only the matching manifest and Lua files in the
configured folders; it does not remove installed game binaries.

## Features

- Choose multiple files through the native file picker or drag files into the window.
- Supports `.manifest`, `.manifests`, and `.lua` extensions.
- Copies imports into separate manifest and Lua storage folders; originals remain in place.
- Prevents different files with the same name from overwriting each other.
- Refreshes the library when opened and after imports, removals, or storage changes.
- Groups files by game ID and uses local Steam metadata to show recognizable game names.
- Shows compact Steam cover art cards with collapsed file lists and supports PNG artwork.
- Caches cover art and game names; missing metadata can be resolved through the Steam Store App Details API, the public SteamCMD app-info API, and Steam image CDNs when `curl` and an internet connection are available.
- Automatically dismisses operation notices; routine library refreshes stay silent.
- Confirms before deleting a library entry and validates every path before removal.
- Persists storage paths per user.

## Stack

- Rust, edition 2024
- [Iced](https://github.com/iced-rs/iced) 0.14
- Windows 10/11 and Linux

## Building

See `BUILD_INSTRUCTIONS.md` for prerequisites and packaging modes.

**Windows (Command Prompt or PowerShell)**

From Command Prompt (`cmd.exe`), run the wrapper so PowerShell is launched
explicitly:

```bat
BUILD_WINDOWS.bat
```

From PowerShell, run the script directly:

```powershell
.\BUILD_WINDOWS.ps1
```

**Linux**

```bash
./BUILD_LINUX.sh
```

The scripts check their prerequisites and place release output in `dist/`. `BUILD_WINDOWS.bat` is the Command Prompt entry point; it invokes the PowerShell script with a process-scoped execution-policy override and propagates the build exit code.

## Storage and privacy

When a Steam installation is detected, the default destinations are
`<Steam>/config/stplug-in` for Lua files and `<Steam>/depotcache` for manifest
files. Without a detected Steam installation, Locomotive uses a private library
under the current user's application-data directory. The two destinations can
be changed independently in Options. Saving a path does not move files that
are already stored there; it applies to future imports.

Locomotive copies selected files to the configured destinations and removes
only the matching manifest and Lua files represented by a library entry. It
reads local Steam metadata to improve grouping and names. If
a game name or cover is missing from local metadata, the app can query the
public Steam Store and image CDN by AppID, with the public SteamCMD app-info API as a name fallback, then cache the result in the user's
Locomotive data directory. This lookup does not require a Steam sign-in. If
`curl` or internet access is unavailable, the app falls back to local metadata
and any previously cached names or covers.

File drag-and-drop is not implemented by Iced on Wayland. On Linux, use an X11
session if drag-and-drop is unavailable; the native file picker remains an
alternative.

## Project structure

The application follows an Elm-style split: state, messages, update logic,
backend operations, and view code each have dedicated modules. Filesystem
operations are isolated in `src/backend/`; view functions only render state.

## Design

Dark-only, monochrome, and built around the existing UI. Colors and spacing
are centralized in `src/theme.rs`.

## License

To be determined.
