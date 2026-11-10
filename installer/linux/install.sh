#!/bin/bash
# ==============================================================================
# DownSort - Простой установщик для Linux / Simple Linux Installer
# ==============================================================================
set -e

APP_NAME="downsort"
DISPLAY_NAME="DownSort"
VERSION="0.1.0"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo -e "${YELLOW}====================================================${NC}"
echo -e "${YELLOW}         Установка ${DISPLAY_NAME} v${VERSION} для Linux         ${NC}"
echo -e "${YELLOW}====================================================${NC}"

# Определение режима: для текущего пользователя или всей системы
if [ "$EUID" -eq 0 ]; then
    INSTALL_BIN="/usr/local/bin"
    INSTALL_APPS="/usr/share/applications"
    INSTALL_ICONS="/usr/share/icons/hicolor"
    echo -e "${BLUE}[i] Режим установки: для всех пользователей системы (root)${NC}"
else
    INSTALL_BIN="${HOME}/.local/bin"
    INSTALL_APPS="${HOME}/.local/share/applications"
    INSTALL_ICONS="${HOME}/.local/share/icons/hicolor"
    echo -e "${BLUE}[i] Режим установки: для текущего пользователя (${USER})${NC}"
fi

# 1. Поиск исполняемого файла
SRC_BIN=""
if [ -f "${ROOT_DIR}/target/release/${APP_NAME}" ]; then
    SRC_BIN="${ROOT_DIR}/target/release/${APP_NAME}"
elif [ -f "${SCRIPT_DIR}/${APP_NAME}" ]; then
    SRC_BIN="${SCRIPT_DIR}/${APP_NAME}"
elif [ -f "./${APP_NAME}" ]; then
    SRC_BIN="./${APP_NAME}"
else
    echo -e "${RED}[✕] Ошибка: исполняемый файл '${APP_NAME}' не найден!${NC}"
    echo "Сначала выполните сборку: cargo build --release"
    exit 1
fi

echo -e "${BLUE}[1/4] Копирование программы в ${INSTALL_BIN}/${APP_NAME}...${NC}"
mkdir -p "${INSTALL_BIN}"
cp -f "${SRC_BIN}" "${INSTALL_BIN}/${APP_NAME}"
chmod +x "${INSTALL_BIN}/${APP_NAME}"

echo -e "${BLUE}[2/4] Установка значков приложения...${NC}"
mkdir -p "${INSTALL_ICONS}/scalable/apps"
mkdir -p "${INSTALL_ICONS}/256x256/apps"
mkdir -p "${INSTALL_ICONS}/128x128/apps"
mkdir -p "${INSTALL_ICONS}/64x64/apps"
mkdir -p "${INSTALL_ICONS}/32x32/apps"

ICON_SVG="${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-mono-f0a21c.svg"
if [ -f "${ICON_SVG}" ]; then
    cp -f "${ICON_SVG}" "${INSTALL_ICONS}/scalable/apps/${APP_NAME}.svg"
fi

for sz in 256 128 64 32; do
    PNG_FILE="${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-app-icon-${sz}.png"
    if [ -f "${PNG_FILE}" ]; then
        cp -f "${PNG_FILE}" "${INSTALL_ICONS}/${sz}x${sz}/apps/${APP_NAME}.png"
    fi
done

echo -e "${BLUE}[3/4] Создание ярлыка в меню приложений...${NC}"
mkdir -p "${INSTALL_APPS}"
cat << 'EOF' > "${INSTALL_APPS}/${APP_NAME}.desktop"
[Desktop Entry]
Version=1.0
Type=Application
Name=DownSort
GenericName=Уборщик Загрузок
GenericName[en]=Downloads Cleaner
Comment=Умный анализатор и уборщик папки Загрузки
Comment[en]=Smart Downloads folder analyzer, cleaner and duplicate finder
Exec=downsort %F
Icon=downsort
Terminal=false
Categories=Utility;FileManager;System;Archiving;
Keywords=downloads;cleanup;duplicates;clean;storage;analyzer;загрузки;уборка;дубликаты;
StartupWMClass=downsort
EOF
chmod 644 "${INSTALL_APPS}/${APP_NAME}.desktop"

echo -e "${BLUE}[4/4] Обновление системных баз ярлыков и иконок...${NC}"
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${INSTALL_APPS}" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${INSTALL_ICONS}" 2>/dev/null || true
fi

echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN}  [✓] Программа ${DISPLAY_NAME} успешно установлена!  ${NC}"
echo -e "${GREEN}====================================================${NC}"
echo "Запуск:"
echo " 1. Из меню приложений: введите в поиске 'DownSort'"
echo " 2. Из терминала: команда 'downsort'"
if [[ ":$PATH:" != *":${INSTALL_BIN}:"* ]] && [ "$EUID" -ne 0 ]; then
    echo -e "${YELLOW}Примечание: убедитесь, что ${INSTALL_BIN} добавлен в вашу переменную PATH.${NC}"
fi
