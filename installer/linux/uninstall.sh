#!/bin/bash
# ==============================================================================
# DownSort - Деинсталлятор для Linux / Linux Uninstaller
# ==============================================================================
set -e

APP_NAME="downsort"
DISPLAY_NAME="DownSort"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

if [ "$EUID" -eq 0 ]; then
    INSTALL_BIN="/usr/local/bin"
    INSTALL_APPS="/usr/share/applications"
    INSTALL_ICONS="/usr/share/icons/hicolor"
else
    INSTALL_BIN="${HOME}/.local/bin"
    INSTALL_APPS="${HOME}/.local/share/applications"
    INSTALL_ICONS="${HOME}/.local/share/icons/hicolor"
fi

echo -e "${BLUE}Удаление ${DISPLAY_NAME}...${NC}"

rm -f "${INSTALL_BIN}/${APP_NAME}"
rm -f "${INSTALL_APPS}/${APP_NAME}.desktop"
rm -f "${INSTALL_ICONS}/scalable/apps/${APP_NAME}.svg"
rm -f "${INSTALL_ICONS}/256x256/apps/${APP_NAME}.png"
rm -f "${INSTALL_ICONS}/128x128/apps/${APP_NAME}.png"
rm -f "${INSTALL_ICONS}/64x64/apps/${APP_NAME}.png"
rm -f "${INSTALL_ICONS}/32x32/apps/${APP_NAME}.png"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${INSTALL_APPS}" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${INSTALL_ICONS}" 2>/dev/null || true
fi

echo -e "${GREEN}[✓] ${DISPLAY_NAME} успешно удалён из вашей системы.${NC}"
