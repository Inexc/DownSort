#!/bin/bash
# ==============================================================================
# DownSort Linux Uninstaller
# ==============================================================================
set -e

APP_NAME="downsort"
DISPLAY_NAME="DownSort"

if [ "$EUID" -eq 0 ]; then
    INSTALL_BIN="/usr/local/bin"
    INSTALL_APPS="/usr/share/applications"
    INSTALL_ICONS="/usr/share/icons/hicolor"
else
    INSTALL_BIN="${HOME}/.local/bin"
    INSTALL_APPS="${HOME}/.local/share/applications"
    INSTALL_ICONS="${HOME}/.local/share/icons/hicolor"
fi

echo "Removing ${DISPLAY_NAME}..."

rm -f "${INSTALL_BIN}/${APP_NAME}"
rm -f "${INSTALL_APPS}/${APP_NAME}.desktop"
rm -f "${INSTALL_ICONS}/scalable/apps/${APP_NAME}.svg"
rm -f "${INSTALL_ICONS}/256x256/apps/${APP_NAME}.png"
rm -f "${INSTALL_ICONS}/128x128/apps/${APP_NAME}.png"
rm -f "${INSTALL_ICONS}/64x64/apps/${APP_NAME}.png"
rm -f "${INSTALL_ICONS}/32x32/apps/${APP_NAME}.png"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${INSTALL_APPS}" || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${INSTALL_ICONS}" || true
fi

echo "${DISPLAY_NAME} has been completely uninstalled."
