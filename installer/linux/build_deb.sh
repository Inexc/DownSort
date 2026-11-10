#!/bin/bash
# ==============================================================================
# DownSort Debian Package (.deb) Builder for Linux
# ==============================================================================
set -e

APP_NAME="downsort"
VERSION="0.1.0"
ARCH="amd64"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
PKG_DIR="${DIST_DIR}/deb-pkg"

echo "Building Debian package for DownSort v${VERSION}..."

rm -rf "${PKG_DIR}"
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/share/applications"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/256x256/apps"

# Control file
cat << EOF > "${PKG_DIR}/DEBIAN/control"
Package: ${APP_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: DownSort Team <dev@downsort.local>
Description: High-performance downloads analyzer and cleaner
 DownSort scans and categorizes downloads, identifies old files
 and safely unifies duplicates with an interactive UI.
EOF

# Copy binary
cp -f "${ROOT_DIR}/target/release/${APP_NAME}" "${PKG_DIR}/usr/bin/${APP_NAME}"
chmod +x "${PKG_DIR}/usr/bin/${APP_NAME}"

# Copy desktop and icon
cp -f "${SCRIPT_DIR}/downsort.desktop" "${PKG_DIR}/usr/share/applications/downsort.desktop"
cp -f "${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-mono-f0a21c.svg" "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps/downsort.svg"
cp -f "${ROOT_DIR}/design/logos/kit/exports/downsort-symbol-app-icon-256.png" "${PKG_DIR}/usr/share/icons/hicolor/256x256/apps/downsort.png"

# Build package
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build "${PKG_DIR}" "${DIST_DIR}/downsort_${VERSION}_${ARCH}.deb"
    echo "Debian package created: ${DIST_DIR}/downsort_${VERSION}_${ARCH}.deb"
else
    echo "Package structure prepared at ${PKG_DIR}"
    echo "Run 'dpkg-deb --build ${PKG_DIR}' to generate the .deb file."
fi
