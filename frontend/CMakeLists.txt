cmake_minimum_required(VERSION 3.10)
project(GranularSynth)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_OSX_DEPLOYMENT_TARGET "12.0")
# FOR THE MOMENT ONLY x86_64
set(CMAKE_OSX_ARCHITECTURES "x86_64")

set(BUNDLE_NAME "GranularSynth")
set(BUNDLE_IDENTIFIER "com.cuervoblanco.GranularSynth")
set(EXECUTABLE_NAME "GranularSynth")
set(BUNDLE_VERSION "1.0.1")
set(BUNDLE_SHORT_VERSION "1.0")
set(MINIMUM_SYSTEM_VERSION "12.0")
set(COPYRIGHT "© 2025 Cuervo Blanco")
set(ICON_FILE "")

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

set(RUST_LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../backend/target/x86_64-apple-darwin/release")

set(QT_PATH "" CACHE PATH "Path to Qt installation provided by the user")
if(NOT QT_PATH)
    message(FATAL_ERROR "QT_PATH is not set. Please provide the Qt installation path using -DQT_PATH=<path>.")
endif()
if(NOT EXISTS "${QT_PATH}")
    message(FATAL_ERROR "The provided QT_PATH '${QT_PATH}' does not exist or is invalid.")
endif()

set(CMAKE_PREFIX_PATH "${QT_PATH}")

find_package(Qt6 REQUIRED COMPONENTS Widgets Core)

find_library(RUST_LIB NAMES libbackend.a PATHS ${RUST_LIB_PATH} REQUIRED)
if(NOT RUST_LIB)
    message(FATAL_ERROR "Rust library not found in ${RUST_LIB_PATH}!")
endif()

# Specific MacOS libraries
find_library(AUDIOTOOLBOX_LIBRARY AudioToolbox)
find_library(COREAUDIO_LIBRARY CoreAudio)

# Setting Qt stuff
message(STATUS "Setting up for Qt")
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
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/Bundles/"
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_BINARY_DIR}/Bundles/GranularSynth.app/Contents/Info.plist"
)

message(STATUS "Setting up target link libraries")
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

message(STATUS "Copying directories")
add_custom_command(TARGET GranularSynth POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
    "${CMAKE_SOURCE_DIR}/resources"
    "${CMAKE_BINARY_DIR}/Bundles/GranularSynth.app/Contents/Resources"
)

message(STATUS "Finished")
