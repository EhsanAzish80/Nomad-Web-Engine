#!/bin/bash
#
# copy_libs.sh
# Copies and signs the Rust library into the app bundle
#

set -e

echo "Copying Nomad Engine library to app bundle..."

# Get paths
LIB_SOURCE="${PROJECT_DIR}/../lib/libnomad_c_api.dylib"
FRAMEWORKS_DIR="${BUILT_PRODUCTS_DIR}/${FRAMEWORKS_FOLDER_PATH}"

# Ensure Frameworks directory exists (create only if needed)
if [ ! -d "${FRAMEWORKS_DIR}" ]; then
    mkdir -p "${FRAMEWORKS_DIR}" || {
        echo "Warning: Could not create Frameworks directory, using MacOS instead"
        FRAMEWORKS_DIR="${BUILT_PRODUCTS_DIR}/${EXECUTABLE_FOLDER_PATH}"
    }
fi

# Copy library
if [ -f "${LIB_SOURCE}" ]; then
    cp "${LIB_SOURCE}" "${FRAMEWORKS_DIR}/"
    echo "Copied library to ${FRAMEWORKS_DIR}"
    
    # Re-sign the library with the app's identity
    if [ ! -z "${CODE_SIGN_IDENTITY}" ] && [ "${CODE_SIGN_IDENTITY}" != "-" ]; then
        echo "Signing library with identity: ${CODE_SIGN_IDENTITY}"
        codesign --force --sign "${CODE_SIGN_IDENTITY}" "${FRAMEWORKS_DIR}/libnomad_c_api.dylib"
    fi
    
    echo "✓ Library copied and signed"
else
    echo "Error: Library not found at ${LIB_SOURCE}"
    echo "Please run ./build_rust.sh first"
    exit 1
fi
