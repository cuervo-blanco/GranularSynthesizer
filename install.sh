#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Default flags
CLEAN=false
REBUILD=false

# Qt path (adjust the version if necessary)
QT_PATH=~/Qt/6.8.1/macos

# Usage information
usage() {
    echo "Usage: $0 [--clean] [--rebuild]"
    echo "  --clean    Delete previous builds and start from scratch."
    echo "  --rebuild  Rebuild everything from scratch (implies --clean)."
    echo "If neither flag is provided, the script will build normally."
    exit 1
}

# Parse command-line arguments
while [[ "$1" =~ ^- ]]; do
    case "$1" in
        --clean)
            CLEAN=true
            shift
            ;;
        --rebuild)
            REBUILD=true
            CLEAN=true  # --rebuild implies --clean
            shift
            ;;
        *)
            usage
            ;;
    esac
done

# Function to build the Rust backend for x86_64 macOS
build_backend() {
    echo "=============================="
    echo "Building Backend (x86_64)"
    echo "=============================="
    cd backend

    if [ "$CLEAN" = true ]; then
        echo "Cleaning backend build directory..."
        cargo clean
    fi

    echo "Building the backend for x86_64-apple-darwin..."
    cargo build --release --target=x86_64-apple-darwin

    echo "Backend built successfully."
    cd ..
}

# Function to build the C++ frontend using CMake and Qt
build_frontend() {
    echo "=============================="
    echo "Building Frontend (x86_64)"
    echo "=============================="
    cd frontend

    if [ "$CLEAN" = true ]; then
        echo "Cleaning frontend build directory..."
        rm -rf build
    fi

    mkdir -p build
    cd build

    echo "Configuring the frontend with CMake..."
    cmake .. -DQT_PATH="$QT_PATH"

    echo "Building the frontend..."
    cmake --build . --config Release

    echo "Frontend built successfully."
    cd ../..
}

# Function to bundle the .app using macdeployqt and create a DMG
bundle_app() {
    echo "=============================="
    echo "Bundling Application and Creating DMG"
    echo "=============================="
    
    FRONTEND_APP_PATH="frontend/build/Bundles/GranularSynth.app"
    DMG_OUTPUT_PATH="frontend/build/GranularSynth-Intel.dmg"

    if [ ! -d "$FRONTEND_APP_PATH" ]; then
        echo "Error: Application bundle not found at $FRONTEND_APP_PATH"
        exit 1
    fi

    echo "Running macdeployqt to bundle the application..."
    $QT_PATH/bin/macdeployqt "$FRONTEND_APP_PATH" -dmg

}

# Main function to orchestrate the build and packaging process
main() {
    echo "Starting the build and packaging process..."

    build_backend
    build_frontend
    bundle_app

    echo "======================================"
    echo "Build and packaging completed successfully!"
    echo "======================================"
}

# Execute the main function
main

