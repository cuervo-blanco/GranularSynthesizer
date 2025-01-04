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

    echo "Running cargo build --release for backend..."
    cargo build --release

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
        mkdir -p frontend/build
    else
        echo "Directory 'build' already exists."
    fi

    cd build || exit 1

    if [ "$REBUILD" = true ]; then
        cmake ..
        cmake --build .
    else
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

