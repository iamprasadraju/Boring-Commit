#!/bin/sh
# Boring Commit installer for macOS and Linux.
#
#   curl -fsSL https://raw.githubusercontent.com/iamprasadraju/Boring-Commit/main/scripts/install.sh | sh
#
# Env overrides:
#   BCOMMIT_VERSION    release tag, e.g. v0.1.0 (default: latest release)
#   BCOMMIT_BASE_URL   download base URL (default: GitHub releases)
#   BCOMMIT_INSTALL_DIR
#                      install prefix holding bin/ (default: /usr/local)
#   BCOMMIT_NO_SUDO=1  never use sudo (prefix must be writable)
#
# Wrap everything in main() so a truncated partial download piped
# into sh never executes half a script.
main() {

set -eu

REPO="iamprasadraju/Boring-Commit"
DEFAULT_BASE_URL="https://github.com/${REPO}/releases"

red="$( (/usr/bin/tput bold || :; /usr/bin/tput setaf 1 || :) 2>&-)"
plain="$( (/usr/bin/tput sgr0 || :) 2>&-)"
status() { echo ">>> $*" >&2; }
error() { echo "${red}ERROR:${plain} $*" >&2; exit 1; }

TEMP_DIR=$(mktemp -d)
cleanup() { rm -rf "$TEMP_DIR"; }
trap cleanup EXIT

available() { command -v "$1" >/dev/null 2>&1; }

# --- 1. Detect OS and architecture -----------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) error "Unsupported architecture: $(uname -m). Prebuilt binaries exist for x86_64 and arm64 only. Fallback: cargo install bcommit" ;;
esac

case "$OS" in
    Darwin)
        [ "$ARCH" = "arm64" ] || error "Only Apple Silicon macs have prebuilt binaries (found $ARCH). Fallback: cargo install bcommit"
        ASSET="bcommit-macos-aarch64.tar.gz"
        ;;
    Linux)
        [ "$ARCH" = "x86_64" ] || error "Only x86_64 Linux has prebuilt binaries (found $ARCH). Fallback: cargo install bcommit"
        ASSET="bcommit-linux-x86_64.tar.gz"
        ;;
    *) error "This script installs on macOS and Linux only (found $OS). Windows: irm https://raw.githubusercontent.com/${REPO}/main/scripts/install.ps1 | iex" ;;
esac

NEEDS=""
for TOOL in curl tar awk; do
    available "$TOOL" || NEEDS="$NEEDS $TOOL"
done
if command -v sha256sum >/dev/null 2>&1; then
    SHA256="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
    SHA256="shasum -a 256"
else
    NEEDS="$NEEDS sha256sum-or-shasum"
fi
[ -z "$NEEDS" ] || error "Missing required tools:$NEEDS"

# --- 2. Resolve version ------------------------------------------------------
BASE_URL="${BCOMMIT_BASE_URL:-$DEFAULT_BASE_URL}"
if [ -n "${BCOMMIT_VERSION:-}" ]; then
    case "$BCOMMIT_VERSION" in
        v*) TAG="$BCOMMIT_VERSION" ;;
        *) TAG="v$BCOMMIT_VERSION" ;;
    esac
elif [ "$BASE_URL" = "$DEFAULT_BASE_URL" ]; then
    status "Resolving latest release..."
    EFFECTIVE_URL="$(curl -fsSL -o /dev/null -w '%{url_effective}' "$BASE_URL/latest")" \
        || error "Could not reach $BASE_URL/latest. Check your network."
    TAG="$(basename "$EFFECTIVE_URL")"
    [ -n "$TAG" ] && [ "$TAG" != "latest" ] || error "Could not determine latest release tag."
else
    TAG="" # custom mirror: download bare asset filenames, no version
fi
if [ -n "$TAG" ]; then
    status "Installing bcommit $TAG for $OS/$ARCH..."
else
    status "Installing bcommit for $OS/$ARCH..."
fi

# --- 3. Download + verify checksum -------------------------------------------
if [ "$BASE_URL" = "$DEFAULT_BASE_URL" ]; then
    DL_BASE="$BASE_URL/download/$TAG"
elif [ -n "$TAG" ]; then
    DL_BASE="$BASE_URL/download/$TAG" # GitHub-layout mirror
else
    DL_BASE="$BASE_URL" # flat mirror serving bare asset filenames
fi

status "Downloading $ASSET..."
curl --fail --show-error --location --progress-bar \
    -o "$TEMP_DIR/$ASSET" "$DL_BASE/$ASSET" \
    || error "Download failed: $DL_BASE/$ASSET"

status "Verifying checksum..."
curl --fail --silent --show-error --location \
    -o "$TEMP_DIR/$ASSET.sha256" "$DL_BASE/$ASSET.sha256" \
    || error "Checksum file missing: $DL_BASE/$ASSET.sha256"
EXPECTED="$(awk '{print $1}' "$TEMP_DIR/$ASSET.sha256")"
ACTUAL="$($SHA256 "$TEMP_DIR/$ASSET" | awk '{print $1}')"
[ "$EXPECTED" = "$ACTUAL" ] || error "Checksum mismatch for $ASSET. Aborting before install."

# --- 4. Install system-wide ---------------------------------------------------
PREFIX="${BCOMMIT_INSTALL_DIR:-/usr/local}"
BINDIR="$PREFIX/bin"

SUDO=""
mkdir -p "$BINDIR" 2>/dev/null || true
if [ ! -w "$BINDIR" ]; then
    if [ -n "${BCOMMIT_NO_SUDO:-}" ]; then
        error "$BINDIR is not writable and BCOMMIT_NO_SUDO is set. Re-run with sudo or set BCOMMIT_INSTALL_DIR to a writable prefix."
    fi
    available sudo || error "Need write access to $BINDIR. Re-run with sudo or set BCOMMIT_INSTALL_DIR to a writable prefix."
    SUDO="sudo"
    $SUDO mkdir -p "$BINDIR"
fi

status "Extracting..."
tar -xzf "$TEMP_DIR/$ASSET" -C "$TEMP_DIR" bcommit \
    || error "Archive did not contain a top-level bcommit binary."

status "Installing to $BINDIR/bcommit..."
$SUDO cp "$TEMP_DIR/bcommit" "$BINDIR/bcommit"
$SUDO chmod 755 "$BINDIR/bcommit"

case ":$PATH:" in
    *":$BINDIR:"*) ;;
    *) status "NOTE: $BINDIR is not on your PATH. Add this line to your shell profile:"; echo "  export PATH=\"$BINDIR:\$PATH\"" >&2 ;;
esac

# --- 5. Smoke test ------------------------------------------------------------
"$BINDIR/bcommit" --version || error "Installed binary failed to run."
status "Install complete. Next step: bcommit config"

}

main
