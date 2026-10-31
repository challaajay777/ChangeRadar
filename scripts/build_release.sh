#!/usr/bin/env bash
# ChangeRadar Unix/Linux/macOS Release Build Script
# Usage: ./scripts/build_release.sh

set -euo pipefail

echo "========================================"
echo "   ChangeRadar Release Packaging Tool   "
echo "========================================"

# Determine project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_ROOT}"

echo "[1/6] Verifying toolchain requirements..."
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo is required but not installed." >&2; exit 1; }
command -v rustc >/dev/null 2>&1 || { echo "Error: rustc is required but not installed." >&2; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "Error: npm is required but not installed." >&2; exit 1; }

echo "  -> rustc: $(rustc --version)"
echo "  -> node:  $(node --version)"
echo "  -> npm:   $(npm --version)"

# Prepare release directory
echo "[2/6] Preparing release directory..."
RELEASE_DIR="${PROJECT_ROOT}/dist-release"
rm -rf "${RELEASE_DIR}"
mkdir -p "${RELEASE_DIR}/bin"
mkdir -p "${RELEASE_DIR}/docs"
mkdir -p "${RELEASE_DIR}/ui"

# Build Rust binaries
echo "[3/6] Building Rust binaries in release mode..."
cargo build --release -p changeradar-cli -p changeradar-daemon

cp "${PROJECT_ROOT}/target/release/changeradar" "${RELEASE_DIR}/bin/changeradar"
cp "${PROJECT_ROOT}/target/release/changeradard" "${RELEASE_DIR}/bin/changeradard"
chmod +x "${RELEASE_DIR}/bin/changeradar"
chmod +x "${RELEASE_DIR}/bin/changeradard"
echo "  -> Copied changeradar and changeradard"

# Build Frontend UI
echo "[4/6] Building React desktop UI..."
cd "${PROJECT_ROOT}/apps/desktop"
npm run build
cd "${PROJECT_ROOT}"
cp -r "${PROJECT_ROOT}/apps/desktop/dist/"* "${RELEASE_DIR}/ui/"
echo "  -> Copied compiled UI assets to dist-release/ui"

# Copy documentation
echo "[5/6] Copying documentation and metadata..."
DOCS=("README.md" "LICENSE" "PRIVACY.md" "CLI.md" "API.md" "ARCHITECTURE.md" "SECURITY.md" "TROUBLESHOOTING.md" "CHANGELOG.md")
for doc in "${DOCS[@]}"; do
    if [ -f "${PROJECT_ROOT}/${doc}" ]; then
        cp "${PROJECT_ROOT}/${doc}" "${RELEASE_DIR}/docs/${doc}"
    fi
done
cp "${PROJECT_ROOT}/LICENSE" "${RELEASE_DIR}/LICENSE"
cp "${PROJECT_ROOT}/README.md" "${RELEASE_DIR}/README.md"

# Generate Checksums
echo "[6/6] Generating SHA-256 release checksums..."
CHECKSUM_FILE="${RELEASE_DIR}/SHA256SUMS.txt"
cd "${RELEASE_DIR}"
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum bin/changeradar bin/changeradard > "${CHECKSUM_FILE}"
elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 bin/changeradar bin/changeradard > "${CHECKSUM_FILE}"
fi

echo "========================================"
echo "   ChangeRadar Release Build Succeeded! "
echo "========================================"
echo "Artifacts location: ${RELEASE_DIR}"
echo "  - bin/changeradar  (CLI)"
echo "  - bin/changeradard (Daemon)"
echo "  - ui/              (Desktop Frontend bundle)"
echo "  - docs/            (Full documentation suite)"
echo "  - SHA256SUMS.txt   (Verification checksums)"
