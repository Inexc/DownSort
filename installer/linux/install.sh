#!/bin/bash
# ==============================================================================
# DownSort Linux Installer
# ==============================================================================
set -e

APP_NAME="downsort"
DISPLAY_NAME="DownSort"
VERSION="0.1.0"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Detect root / user install mode
if [ "$EUID" -eq 0 ]; then
    INSTALL_BIN="/usr/local/bin"
    INSTALL_APPS="/usr/share/applications"
    INSTALL_ICONS="/usr/share/icons/hicolor"
    MODE="system-wide"
else
    INSTALL_BIN="${HOME}/.local/bin"
    INSTALL_APPS="${HOME}/.local/share/applications"
    INSTALL_ICONS="${HOME}/.local/share/icons/hicolor"
    MODE="user"
fi

echo "============================================="
echo " Installing ${DISPLAY_NAME} v${VERSION} (${MODE})"
echo "============================================="

# 1. Locate binary
SRC_BIN="${ROOT_DIR}/target/release/${APP_NAME}"
if [ ! -f "${SRC_BIN}" ]; then
    # Check if a prebuilt binary exists in current script dir
    if [ -f "${SCRIPT_DIR}/${APP_NAME}" ]; then
        SRC_BIN="${SCRIPT_DIR}/${APP_NAME}"
    else
        echo "Error: Binary not found at ${SRC_BIN}"
        echo "Please build with 'cargo build --release' first or provide '${APP_NAME}'."
        exit 1
    fi
fi

# 2. Create destination directories
mkdir -p "${INSTALL_BIN}"
mkdir -p "${INSTALL_APPS}"
mkdir -p "${INSTALL_ICONS}/scalable/apps"
mkdir -p "${INSTALL_ICONS}/256x256/apps"
mkdir -p "${INSTALL_ICONS}/128x128/apps"
mkdir -p "${INSTALL_ICONS}/64x64/apps"
mkdir -p "${INSTALL_ICONS}/32x32/apps"

# 3. Copy executable
echo "Installing binary to ${INSTALL_BIN}/${APP_NAME}..."
cp -f "${SRC_BIN}" "${INSTALL_BIN}/${APP_NAME}"
chmod +x "${INSTALL_BIN}/${APP_NAME}"

# 4. Copy icons
ICON_SVG="${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-mono-f0a21c.svg"
if [ -f "${ICON_SVG}" ]; then
    cp -f "${ICON_SVG}" "${INSTALL_ICONS}/scalable/apps/${APP_NAME}.svg"
fi

ICON_256="${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-app-icon-256.png"
if [ -f "${ICON_256}" ]; then
    cp -f "${ICON_256}" "${INSTALL_ICONS}/256x256/apps/${APP_NAME}.png"
fi

ICON_128="${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-app-icon-128.png"
if [ -f "${ICON_128}" ]; then
    cp -f "${ICON_128}" "${INSTALL_ICONS}/128x128/apps/${APP_NAME}.png"
fi

ICON_64="${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-app-icon-64.png"
if [ -f "${ICON_64}" ]; then
    cp -f "${ICON_64}" "${INSTALL_ICONS}/64x64/apps/${APP_NAME}.png"
fi

ICON_32="${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-app-icon-32.png"
if [ -f "${ICON_32}" ]; then
    cp -f "${ICON_32}" "${INSTALL_ICONS}/32x32/apps/${APP_NAME}.png"
fi

# 5. Copy desktop entry
echo "Installing desktop launcher to ${INSTALL_APPS}/${APP_NAME}.desktop..."
DESKTOP_SRC="${SCRIPT_DIR}/downsort.desktop"
if [ -f "${DESKTOP_SRC}" ]; then
    cp -f "${DESKTOP_SRC}" "${INSTALL_APPS}/${APP_NAME}.desktop"
    chmod 644 "${INSTALL_APPS}/${APP_NAME}.desktop"
fi

# 6. Update system icon & desktop caches
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${INSTALL_APPS}" || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${INSTALL_ICONS}" || true
fi

echo "---------------------------------------------"
echo " ${DISPLAY_NAME} installed successfully!"
echo " Run by typing '${APP_NAME}' in terminal or launching from app menu."
echo "---------------------------------------------"
