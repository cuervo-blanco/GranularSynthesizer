cmake_minimum_required(VERSION 3.10)
project(GranularSynth)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Finding the Qt path
set(QT_BASE_DIR "$ENV{HOME}/Qt")
file(GLOB QT_DIRS "${QT_BASE_DIR}/*/macos")
list(SORT QT_DIRS)
list(GET QT_DIRS -1 QT_LATEST_DIR)
if(QT_LATEST_DIR)
    set(CMAKE_PREFIX_PATH "${QT_LATEST_DIR}" CACHE PATH "Path to Qt libraries" FORCE)
    message(STATUS "Using Qt directory: ${QT_LATEST_DIR}")
else()
    message(FATAL_ERROR "No Qt installation found in ${QT_BASE_DIR}")
endif()

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

set(BUNDLE_NAME "GranularSynth")
set(BUNDLE_IDENTIFIER "com.cuervoblanco.GranularSynth")
set(EXECUTABLE_NAME "GranularSynth")
set(BUNDLE_VERSION "1.0.0")
set(BUNDLE_SHORT_VERSION "1.0")
set(MINIMUM_SYSTEM_VERSION "12.0")
set(COPYRIGHT "© 2025 Cuervo Blanco")
set(ICON_FILE "")


set(CMAKE_OSX_DEPLOYMENT_TARGET "12.0")
set(CMAKE_OSX_ARCHITECTURES "x86_64;arm64")

find_package(Qt6 REQUIRED COMPONENTS Widgets Core)

# Check whether the processor is set to arm64 or x86_64
if(CMAKE_SYSTEM_PROCESSOR MATCHES "arm64")
    message(STATUS "Building for ARM architecture")
    set(RUST_LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../backend/target/aarch64-apple-darwin/release")
    set(SYSTEM_ARCH "ARM64")
elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "x86_64")
    message(STATUS "Building for Intel architecture")
    set(RUST_LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../backend/target/x86_64-apple-darwin/release")
    set(SYSTEM_ARCH "x8664")
    # Put code here
else()
    message(FATAL_ERROR "Unsupported architecture: ${CMAKE_SYSTEM_PROCESSOR}")
endif()

find_library(RUST_LIB NAMES libbackend.a PATHS ${RUST_LIB_PATH} REQUIRED)
if(NOT RUST_LIB)
    message(FATAL_ERROR "Rust library not found in ${RUST_LIB_PATH}!")
endif()

# Specific MacOS libraries
find_library(AUDIOTOOLBOX_LIBRARY AudioToolbox)
find_library(COREAUDIO_LIBRARY CoreAudio)

# Setting Qt stuff
message(STATUS "Setting up for Qt")
qt_standard_project_setup()
qt_add_executable(GranularSynth
    resources.qrc
    main.cpp
    SynthUI.h SynthUI.cpp 
    dialog/AudioSettingsDialog.h dialog/AudioSettingsDialog.cpp 
    settings/SettingsManager.h settings/SettingsManager.cpp
)
#Creating the configure file
message(STATUS "Configuring Info.plist")
configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/Info.plist.in
    ${CMAKE_BINARY_DIR}/Bundles/GranularSynth.app/Contents/Info.plist
    @ONLY
)

message(STATUS "Setting target properties")
set_target_properties(GranularSynth PROPERTIES
    MACOSX_BUNDLE TRUE
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/Bundles"
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_BINARY_DIR}/Bundles/GranularSynth.app/Contents/Info.plist"
)

message(STATUS "Targeting")
target_link_libraries(GranularSynth PRIVATE
    ${AUDIOTOOLBOX_LIBRARY}
    ${COREAUDIO_LIBRARY}
    ${RUST_LIB}
    Qt6::Widgets
)

target_include_directories(GranularSynth PRIVATE 
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/settings
    ${CMAKE_CURRENT_SOURCE_DIR}/dialog
)

message(STATUS "Building")
add_custom_command(TARGET GranularSynth POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
    "${CMAKE_BINARY_DIR}/Bundles/GranularSynth_${SYSTEM_ARCH}.app/Contents/MacOS/GranularSynth"
    "${CMAKE_BINARY_DIR}/bin/GranularSynth"
    COMMAND ${CMAKE_COMMAND} -E copy_directory
    "${CMAKE_SOURCE_DIR}/resources"
    "${CMAKE_BINARY_DIR}/Bundles/GranularSynth_${SYSTEM_ARCH}.app/Contents/Resources"
)

message(STATUS "Finished")
