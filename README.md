<p align="center">
  <img src="assets/icon.svg" width="160" height="160" alt="Locomotive logo">
</p>

<h1 align="center">Locomotive</h1>

<p align="center">
  <a href="#getting-started">
    <img src="src/public/view-the-installation-guide.svg" height="50" alt="View the installation guide">
  </a>
</p>

<p align="center">
  <strong>An advanced game library manager built for connected tools.</strong><br>
  Organize game-related files in one place, with an integration for <a href="https://github.com/Asper0000/rusty-tools">RustyTools</a> today and more project integrations planned for the future.
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust&logoColor=white" alt="Rust 1.85 or newer"></a>
  <a href="https://github.com/iced-rs/iced"><img src="https://img.shields.io/badge/UI-Iced%200.14-555555" alt="Iced 0.14"></a>
  <a href="https://www.microsoft.com/windows"><img src="https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-0078D6?logo=windows&logoColor=white" alt="Windows 10 and 11, 64-bit"></a>
</p>

---

## Contents

- [What is Locomotive?](#what-is-locomotive)
- [Features](#features)
- [Integrations](#integrations)
- [How it works](#how-it-works)
- [Privacy and security](#privacy-and-security)
- [Getting started](#getting-started)
- [Technical details](#technical-details)
- [License](#license)

---

## What is Locomotive?

**Locomotive** is an advanced desktop game library manager for organizing the local files and metadata used by game-related tools. Its first integration is with **RustyTools**, connecting its library to the manifest files and Lua scripts RustyTools works with while keeping everything organized by game.

Import files through the native picker or drag them into the app, configure storage locations independently, and browse a consolidated library with game names and cover art where available. Locomotive tracks the files it manages and can identify Lua files that have been left behind when RustyTools is configured to use a different directory.

RustyTools is the first integration, not the end goal. Locomotive is intended to expand to support other projects and tools over time, growing into a shared place to manage game-related libraries and their associated files.

Locomotive manages a library; it is not a game installer or launcher. It does not install or uninstall games, change account entitlements, or execute Lua scripts.

---

## Features

- **Drag and drop or browse.** Import files through the native file picker or drop them into the window.
- **Unified game library.** Keep supported game-related files organized into browsable entries grouped by game.
- **RustyTools integration.** Align the library with RustyTools' configured Lua directory and the manifest files it uses.
- **More integrations planned.** Locomotive is intended to support additional projects and tools beyond RustyTools as the project evolves.
- **Manifest and Lua management.** Manage `.manifest`, `.manifests`, and `.lua` files in one library.
- **Independent storage paths.** Choose separate destinations for manifest files and Lua scripts.
- **Automatic library scanning.** Refreshes after imports, removals, and storage changes, then groups files using app IDs and depot references.
- **Readable game cards.** Uses available local Steam metadata to show recognizable game names, with cover artwork where available.
- **Misplaced Lua detection.** Flags relevant Lua files left in `config/stplug-in` when RustyTools is configured to read from a different directory, with an option to move them to the configured folder.
- **Metadata and artwork caching.** Reuses local and cached data, with optional online lookups for missing names and covers.
- **Safer removal.** Confirms destructive actions and validates paths before deleting tracked library files.
- **Persistent settings.** Saves the configured storage locations for the current user.

---

## Integrations

### RustyTools — available now

Locomotive currently integrates with [RustyTools](https://github.com/Asper0000/rusty-tools), using its configured Lua directory as the source of truth and organizing related manifests and scripts into game entries. When files are left in RustyTools' previous default directory, Locomotive can flag them and offer to move them into the configured location.

### Other projects — planned

Locomotive is intended to grow beyond a single tool integration. Future integrations with other projects are planned, with the goal of making Locomotive a central library manager for supported game tools and their related files. These additional integrations are not available yet; the current integration and file handling are focused on RustyTools and Steam manifest/Lua workflows.

---

## How it works

Locomotive copies imported files to its configured destinations; the source files remain where they are. It then scans those destination folders and assembles the library into entries.

| File type | Default destination when Steam is detected | How it is used |
|---|---|---|
| `.manifest` / `.manifests` | `<Steam>\depotcache` | Manifest files are grouped with Lua entries when their depot IDs can be matched. Unmatched manifests can still appear as their own entries. |
| `.lua` | The `[lua] dir` in `<Steam>\rustytools.toml`; otherwise `<Steam>\config\stplug-in` | Lua files are read to identify app IDs and declared depot IDs for grouping. Locomotive does not run their code. |

If Steam is not detected, Locomotive uses a private library under the current user's application-data directory. You can change both paths independently in **Options**.

---

## Privacy and security

Locomotive is a local desktop application. Its settings and cached metadata are stored on your computer, and imported files are copied to the storage locations you configure. You do not need a Locomotive account or need to provide Steam account credentials to use the application.

When local metadata is unavailable, optional game-name and artwork lookups may send requests to public Steam services and image CDNs. These lookups require an internet connection; the core library-management workflow does not depend on them.

Locomotive does not execute imported Lua code. It reads relevant Lua declarations to identify game and depot IDs for library grouping. However, Lua files placed in a directory used by RustyTools are handled by RustyTools, not by Locomotive, so only import files from sources you trust.

---

## Getting started

Locomotive targets **Windows 10/11 (64-bit)**. To build it from a clean checkout, install the following prerequisites:

- **Rust stable 1.85 or newer**, installed through [rustup](https://rustup.rs).
- **Git**, to clone the repository.
- **Visual Studio Build Tools** with the *Desktop development with C++* workload and a Windows SDK. These provide the MSVC linker and Windows resource tools required for a Windows build.

### Build from source

Open a terminal and run:

```powershell
git clone https://github.com/Asper0000/locomotive.git
cd locomotive
cargo build --release
```

Cargo downloads the required Rust dependencies and builds the optimized executable. Once the build completes, launch Locomotive with:

```powershell
.\target\release\locomotive.exe
```

The executable is produced at `target\release\locomotive.exe`.

---

## Technical details

- **Language:** Rust, edition 2024
- **Minimum Rust version:** 1.85
- **UI framework:** [Iced 0.14](https://github.com/iced-rs/iced)
- **Target:** Windows 10/11, x86-64
- **Architecture:** Elm-style separation between state, messages, update logic, views, and backend operations

Filesystem work is isolated in `src/backend/`. View code renders the current state, while update logic dispatches operations and handles their results. The interface uses a dark, monochrome theme centralized in `src/theme.rs`.

---

## License

Locomotive is licensed under the **GNU General Public License v3.0 only (GPL-3.0-only)**.

You may use, study, modify, and redistribute Locomotive under the terms of GPLv3. If you distribute a modified version or a work based on Locomotive, you must comply with the license's applicable source-code and redistribution requirements.

The full license text is included in [`LICENSE`](LICENSE). You can also read the [official GPLv3 text](https://www.gnu.org/licenses/gpl-3.0.html).

---

<p align="center">
  <img src="src/public/built-with-deepseek.svg" alt="Built with DeepSeek">
</p>
