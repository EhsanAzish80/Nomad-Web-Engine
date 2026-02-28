#!/bin/bash
#
# build_rust.sh
# Builds the Rust engine library for macOS
#

set -e

echo "Building Nomad Engine for macOS..."

# Navigate to the project root
cd "$(dirname "$0")/../.."

# Build the C API library in release mode
cargo build --release -p nomad-c-api

# Create output directory if it doesn't exist
mkdir -p "apps/nomad-browser-macos/lib"

# Copy the library
cp "target/release/libnomad_c_api.dylib" "apps/nomad-browser-macos/lib/"
cp "target/release/libnomad_c_api.a" "apps/nomad-browser-macos/lib/" || true

# Copy the header
cp "bindings/c-api/nomad_engine.h" "apps/nomad-browser-macos/NomadBrowser/"

# Update the dylib install name to use @rpath
install_name_tool -id "@rpath/libnomad_c_api.dylib" "apps/nomad-browser-macos/lib/libnomad_c_api.dylib"

echo "✓ Rust library built successfully"
echo "  Library: apps/nomad-browser-macos/lib/libnomad_c_api.dylib"
echo "  Header: apps/nomad-browser-macos/NomadBrowser/nomad_engine.h"
