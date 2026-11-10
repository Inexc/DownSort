#!/bin/bash
# ==============================================================================
# DownSort AppImage Builder for Linux
# ==============================================================================
set -e

APP_NAME="downsort"
VERSION="0.1.0"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
APPDIR="${DIST_DIR}/AppDir"

echo "Building AppImage for DownSort v${VERSION}..."

mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"

# Copy binary
cp -f "${ROOT_DIR}/target/release/${APP_NAME}" "${APPDIR}/usr/bin/${APP_NAME}"
chmod +x "${APPDIR}/usr/bin/${APP_NAME}"

# Copy desktop and icon
cp -f "${SCRIPT_DIR}/downsort.desktop" "${APPDIR}/downsort.desktop"
cp -f "${SCRIPT_DIR}/downsort.desktop" "${APPDIR}/usr/share/applications/downsort.desktop"

ICON_SVG="${ROOT_DIR}/assets/logo.svg"
ICON_PNG="${ROOT_DIR}/assets/icon-256.png"

cp -f "${ICON_PNG}" "${APPDIR}/downsort.png"
cp -f "${ICON_PNG}" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/downsort.png"
cp -f "${ICON_SVG}" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/downsort.svg"

# Create AppRun script
cat << 'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin/:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib/:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/downsort" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

# Check if appimagetool is available
if command -v appimagetool >/dev/null 2>&1; then
    ARCH=x86_64 appimagetool "${APPDIR}" "${DIST_DIR}/DownSort-${VERSION}-x86_64.AppImage"
    echo "AppImage created at: ${DIST_DIR}/DownSort-${VERSION}-x86_64.AppImage"
else
    echo "AppDir prepared at: ${APPDIR}"
    echo "To produce .AppImage, run 'appimagetool ${APPDIR}' on a Linux host or GitHub Actions."
fi
