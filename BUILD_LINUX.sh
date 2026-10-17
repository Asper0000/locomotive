#!/usr/bin/env bash
# BUILD_LINUX.sh
# Automated release build for Linux.
#
# Usage:
#   ./BUILD_LINUX.sh                    # AppImage in dist/
#   ./BUILD_LINUX.sh --mode standalone  # single binary in dist/
#   ./BUILD_LINUX.sh --mode bundle      # portable folder in dist/locomotive/

set -euo pipefail

step() { printf '\033[36m==> %s\033[0m\n' "$1"; }
ok()   { printf '\033[32m    %s\033[0m\n' "$1"; }
warn() { printf '\033[33m    %s\033[0m\n' "$1"; }
fail() { printf '\033[31m    %s\033[0m\n' "$1"; }

MODE="appimage"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --mode)      MODE="$2"; shift 2 ;;
        --mode=*)    MODE="${1#*=}"; shift ;;
        -h|--help)
            echo "Usage: $0 [--mode appimage|standalone|bundle]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

case "$MODE" in
    appimage|standalone|bundle) ;;
    *) fail "Invalid mode: $MODE"; echo "Use appimage, standalone, or bundle."; exit 1 ;;
esac

step "Checking optional Steam name and cover lookup"
if command -v curl >/dev/null 2>&1; then
    ok "curl found; missing game names and cover art can be resolved online"
else
    warn "curl not found; Locomotive will use only local Steam metadata and cached artwork"
    warn "Optional install commands: sudo apt install curl  |  sudo dnf install curl  |  sudo pacman -S curl"
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST="$ROOT/dist"
APPDIR="$DIST/AppDir"
BIN_NAME="locomotive"
APP_NAME="Locomotive"

# --- 1. Rust toolchain ------------------------------------------------------
step "Checking for the Rust toolchain"
if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
    fail "The Rust toolchain was not found."
    echo
    echo "Install it with rustup:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo
    exit 1
fi
ok "cargo: $(command -v cargo)"

# --- 2. System libraries ----------------------------------------------------
step "Checking for required system libraries"
if ! command -v pkg-config >/dev/null 2>&1; then
    fail "pkg-config was not found."
    echo "  Debian/Ubuntu: sudo apt install pkg-config"
    echo "  Fedora:        sudo dnf install pkgconf-pkg-config"
    echo "  Arch:          sudo pacman -S pkgconf"
    exit 1
fi

missing=()
for pkg in xkbcommon wayland-client vulkan; do
    pkg-config --exists "$pkg" 2>/dev/null || missing+=("$pkg")
done

if [ ${#missing[@]} -gt 0 ]; then
    fail "Missing development packages: ${missing[*]}"
    echo
    echo "  Debian / Ubuntu:"
    echo "    sudo apt install libxkbcommon-dev libwayland-dev libvulkan-dev \\"
    echo "                     libssl-dev pkg-config"
    echo "  Fedora:"
    echo "    sudo dnf install libxkbcommon-devel wayland-devel vulkan-loader-devel \\"
    echo "                     openssl-devel pkgconf-pkg-config"
    echo "  Arch:"
    echo "    sudo pacman -S libxkbcommon wayland vulkan-icd-loader openssl pkgconf"
    exit 1
fi
ok "All required dev libraries found."

# --- 3. Build ---------------------------------------------------------------
step "Building release binary (first run downloads crates)"
cd "$ROOT"
cargo build --release
ok "Build finished."

mkdir -p "$DIST"
# Clear stale payloads left by older releases that bundled an injected hook.
for stale in "$DIST/dwmapi.dll" "$DIST/winhttp.dll" "$DIST/locomotive_hook.dll"; do
    if [ -f "$stale" ]; then
        rm -f "$stale"
        warn "Removed stale $(basename "$stale") from dist/"
    fi
done
if [ -d "$DIST/steamhook" ]; then
    rm -rf "$DIST/steamhook"
    warn "Removed stale steamhook directory from dist/"
fi
BIN_SRC="$ROOT/target/release/$BIN_NAME"

# --- 4a. Standalone ---------------------------------------------------------
if [ "$MODE" = "standalone" ]; then
    step "Staging standalone binary"
    cp "$BIN_SRC" "$DIST/$BIN_NAME"
    chmod +x "$DIST/$BIN_NAME"
    ok "Wrote $DIST/$BIN_NAME"
    echo
    echo "Build complete."
    exit 0
fi

# --- 4b. Bundle -------------------------------------------------------------
if [ "$MODE" = "bundle" ]; then
    step "Staging bundle"

    BUNDLE="$DIST/locomotive"
    rm -rf "$BUNDLE"
    mkdir -p "$BUNDLE/assets"

    cp "$BIN_SRC" "$BUNDLE/$BIN_NAME"
    chmod +x "$BUNDLE/$BIN_NAME"

    # Only user-facing docs. CHANGELOG.txt is a developer artifact and
    # stays in the source tree.
    for doc in README.md LICENSE; do
        [ -f "$ROOT/$doc" ] && cp "$ROOT/$doc" "$BUNDLE/"
    done

    for asset in icon.png; do
        [ -f "$ROOT/assets/$asset" ] && cp "$ROOT/assets/$asset" "$BUNDLE/assets/"
    done

    cat > "$BUNDLE/$BIN_NAME.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Steam depot manifest and Lua file manager
Exec=$BIN_NAME
Icon=$BIN_NAME
Categories=Utility;Game;
Terminal=false
EOF

    cat > "$BUNDLE/install.sh" <<'INSTALL'
#!/usr/bin/env bash
# Installs Locomotive for the current user only. No root required.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

install -m 755 "$HERE/locomotive" "$BIN_DIR/locomotive"
[ -f "$HERE/assets/icon.png" ] && \
    install -m 644 "$HERE/assets/icon.png" "$ICON_DIR/locomotive.png"
install -m 644 "$HERE/locomotive.desktop" "$APP_DIR/locomotive.desktop"

echo "Installed. Make sure $BIN_DIR is on your PATH."
INSTALL
    chmod +x "$BUNDLE/install.sh"

    ok "Wrote bundle to $BUNDLE"
    echo
    echo "Build complete."
    echo "Run $BUNDLE/install.sh to install for the current user."
    exit 0
fi

# --- 4c. AppImage -----------------------------------------------------------
step "Staging AppDir"

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/applications"

cp "$BIN_SRC" "$APPDIR/usr/bin/$BIN_NAME"
chmod +x "$APPDIR/usr/bin/$BIN_NAME"

if [ -f "$ROOT/assets/icon.png" ]; then
    cp "$ROOT/assets/icon.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$BIN_NAME.png"
    cp "$ROOT/assets/icon.png" "$APPDIR/$BIN_NAME.png"
    ok "Icon installed."
else
    warn "assets/icon.png is missing; the AppImage will use a generic icon."
fi

cat > "$APPDIR/$BIN_NAME.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Steam depot manifest and Lua file manager
Exec=$BIN_NAME
Icon=$BIN_NAME
Categories=Utility;Game;
Terminal=false
EOF

cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/locomotive" "$@"
EOF
chmod +x "$APPDIR/AppRun"

ok "AppDir staged at $APPDIR"

step "Checking for appimagetool"
APPIMAGETOOL="$DIST/appimagetool-x86_64.AppImage"
if [ ! -x "$APPIMAGETOOL" ]; then
    warn "Not found. Downloading the continuous build."
    curl -L -o "$APPIMAGETOOL" \
        "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$APPIMAGETOOL"
    ok "Downloaded appimagetool."
fi

step "Building AppImage"
OUT="$DIST/${BIN_NAME}-x86_64.AppImage"
rm -f "$OUT"
ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGETOOL" "$APPDIR" "$OUT"

ok "Wrote $OUT"
echo
echo "Build complete."