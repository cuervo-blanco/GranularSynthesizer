#!/bin/bash

CLEAN=false
REBUILD=false

usage() {
    echo "Usage: $0 [--clean] [--rebuild]"
    echo "--clean    Delete previous builds and start from scratch."
    echo "--rebuild  Rebuild everything from scratch (clean backend and frontend)."
    echo "If neither flag is provided, the script will build normally."
    exit 1
}

while [[ "$1" =~ ^- ]]; do
    case "$1" in
        --clean)
            CLEAN=true
            shift
            ;;
        --rebuild)
            REBUILD=true
            CLEAN=true  # If --rebuild is specified, we also clean
            shift
            ;;
        *)
            usage
            ;;
    esac
done

build_backend() {
    echo "Building backend..."
    cd backend || exit 1
    
    if [ "$CLEAN" = true ]; then
        echo "Cleaning backend build..."
        cargo clean
    fi

    echo "Building for Intel macOS..."
    cargo build --release --target=x86_64-apple-darwin || exit 1

    echo "Building for Apple Silicon macOS..."
    cargo build --release --target=aarch64-apple-darwin || exit 1

    echo "Merging into universal binary..."
    mkdir -p target/universal/release
    lipo -create \
        target/x86_64-apple-darwin/release/libbackend.a \
        target/aarch64-apple-darwin/release/libbackend.a \
        -output target/universal/release/libbackend.a || exit 1

    echo "Universal binary created at target/universal/release/libbackend.a"

    cd - || exit 1
}

build_frontend() {
    echo "Building frontend..."
    cd frontend || exit 1
    
    if [ "$CLEAN" = true ]; then
        echo "Cleaning frontend build..."
        rm -rf build/
        mkdir build
    fi

    echo "Running cmake and building frontend..."

    if [ ! -d "build" ]; then
        echo "Directory 'build' does not exist. Creating it now..."
        mkdir -p build
    else
        echo "Directory 'build' already exists."
    fi

    cd build || exit 1

    if [ "$REBUILD" = true ]; then
        cmake ..
        cmake --build .
    else
        cmake ..
        cmake --build .
    fi

    cd - || exit 1
}

# Main installation process
main() {
    build_backend
    build_frontend
    echo "Build completed successfully!"
}

# Call the main function
main

