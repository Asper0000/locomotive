# Build Instructions

This document explains what the automated build scripts do, and how to
build by hand if you need to debug one of them.

## Prerequisites

### Both platforms

- **Rust toolchain**, stable, 1.85 or newer (edition 2024).
  Install with [rustup](https://rustup.rs).
- **Git**, to clone the repository.

### Windows, in addition

- **Visual Studio Build Tools** with the *Desktop development with C++*
  workload. This supplies the linker and the Windows SDK. The SDK also
  provides rc.exe, which build.rs uses to embed the icon.

## Optional game names and cover art

Locomotive uses local Steam metadata and artwork caches first. To fill in
missing game names and download cover images automatically, it also needs an
internet connection and `curl`. It queries the public Steam Store App Details
endpoint first and uses the public SteamCMD app-info API as a name fallback.
For artwork, it tries portrait library covers first, including images in newer
hash-based Steam CDN folders when that path can be derived from the Store image
URL. If a portrait is not published, it falls back to official Store capsule or
header images. This is optional: the app and its build do not fail when `curl` is missing, but unknown
AppIDs may keep their fallback name and placeholder artwork. PNG and JPEG
cover images are supported. Older cached lookup failures are retried after the
artwork lookup strategy changes.

- **Windows:** modern Windows versions usually include `curl.exe`; otherwise
  install it with `winget install --id cURL.cURL -e`.
- **Debian / Ubuntu:** `sudo apt install curl`
- **Fedora:** `sudo dnf install curl`
- **Arch:** `sudo pacman -S curl`

## Application icon

Two files are needed, and both are optional but strongly recommended.

- **assets/icon.ico** - embedded into the Windows .exe by build.rs via the
  winresource crate. Without it, Explorer, the taskbar, and Alt-Tab show
  the generic Rust icon. To create it from a PNG:

      magick assets/icon.png -define icon:auto-resize=256,128,64,48,32,16 assets/icon.ico

  ImageMagick. On Linux the command is convert instead of magick.

- **assets/icon.png** - used by the Linux AppImage and by the .desktop
  file in bundle mode. 256x256 is the right size.

build.rs only runs the icon embedding when the target OS is Windows. On
Linux the step is skipped entirely. If icon.ico is missing on a Windows
build, the script prints a warning and continues; the .exe still builds.

## Build modes

### Windows

From **Command Prompt (`cmd.exe`)**:

    rem Single .exe in dist\ (default)
    scripts\BUILD_WINDOWS.bat

    rem Portable folder in dist\locomotive\, ready to wrap in an installer
    scripts\BUILD_WINDOWS.bat -Mode bundle

From **PowerShell**:

    # Single .exe in dist\ (default)
    .\scripts\BUILD_WINDOWS.ps1

    # Portable folder in dist\locomotive\, ready to wrap in an installer
    .\scripts\BUILD_WINDOWS.ps1 -Mode bundle

The **standalone** mode produces one file: dist\locomotive.exe. It runs
from anywhere, with no installation.

The project ZIP includes `Cargo.lock`. Keep this file with the source project:
Cargo normally reuses its pinned dependency versions on subsequent builds.
The Windows build script deliberately does not pass `--locked`, because Cargo
must be allowed to reconcile an outdated lockfile after a dependency or feature
change. If the lockfile is missing, Cargo generates it; if it is stale, Cargo
updates it as needed before building. Review and keep any resulting `Cargo.lock`
change with the source project so later builds use the same resolved versions.

The **bundle** mode produces dist\locomotive\ containing the .exe, the
assets\ folder, README.md, and CHANGELOG.txt. This is the input you feed
to Inno Setup, WiX, or NSIS.

## Build entry points

Use `scripts\BUILD_WINDOWS.bat` from Command Prompt, `BUILD_WINDOWS.ps1` from PowerShell, or `BUILD_LINUX.sh` on Linux for normal builds. The scripts
check the required toolchain and system libraries, then stage the release
artifacts in `dist/`. The build scripts are the supported packaging path.

## Troubleshooting

| Symptom | Cause | Fix |
| :--- | :--- | :--- |
| Typing `BUILD_WINDOWS.ps1` opens VS Code | Command Prompt treats `.ps1` as an associated file, not an executable script | Run `scripts\BUILD_WINDOWS.bat` from Command Prompt, or run `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\BUILD_WINDOWS.ps1` |
| Cargo says it cannot create or update `Cargo.lock` because `--locked` was passed | An older build script is still being used | Replace `BUILD_WINDOWS.ps1` with the current project version and run `scripts\BUILD_WINDOWS.bat`; the current script allows Cargo to create or reconcile the lockfile |
| Cargo fails to write `Cargo.lock` without mentioning `--locked` | The file or project folder may be read-only or inaccessible | Check the folder and file permissions, close programs holding the lockfile, and retry from a writable project directory |
| link.exe not found on Windows | MSVC toolchain missing or C++ workload absent | Install Visual Studio Build Tools with the Desktop development with C++ workload and a Windows SDK |
| assets/icon.ico not found warning | Icon not created yet | Run the ImageMagick command above |
| Console window opens with the .exe | windows_subsystem attribute missing from main.rs | Confirm the attribute is at the top of src/main.rs |
| cargo cannot resolve =0.14.0 | Toolchain too old | rustup update stable - edition 2024 needs 1.85+ |