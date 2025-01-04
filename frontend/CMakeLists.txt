cmake_minimum_required(VERSION 3.10)
project(GranularSynth)

set(CMAKE_CXX_STANDARD 17)

# Let CI/CD workflows set this dynamically
set(CMAKE_PREFIX_PATH "/Users/astro/Qt/6.8.1/macos")

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

set(BUNDLE_NAME "GranularSynth")
set(BUNDLE_IDENTIFIER "com.cuervoblanco.GranularSynth")
set(EXECUTABLE_NAME "GranularSynth")
set(BUNDLE_VERSION "1.0.0")
set(BUNDLE_SHORT_VERSION "1.0")
set(MINIMUM_SYSTEM_VERSION "12.7")
set(COPYRIGHT "© 2025 Cuervo Blanco")
set(ICON_FILE "")


find_package(Qt6 REQUIRED COMPONENTS Widgets Core)

find_library(RUST_LIB NAMES libbackend.a PATHS
    ${CMAKE_CURRENT_SOURCE_DIR}/../backend/target/release
    REQUIRED) 
if(NOT RUST_LIB)
    message(FATAL_ERROR "Rust library not found!")
endif()

find_library(AUDIOTOOLBOX_LIBRARY AudioToolbox)
find_library(COREAUDIO_LIBRARY CoreAudio)

qt_add_executable(GranularSynth
    resources.qrc
    main.cpp
    SynthUI.h SynthUI.cpp 
    dialog/AudioSettingsDialog.h dialog/AudioSettingsDialog.cpp 
    settings/SettingsManager.h settings/SettingsManager.cpp
)

configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/Info.plist.in
    ${CMAKE_BINARY_DIR}/Bundles/GranularSynth.app/Contents/Info.plist
    @ONLY
)

set_target_properties(GranularSynth PROPERTIES
    MACOSX_BUNDLE TRUE
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/Bundles"
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_BINARY_DIR}/Bundles/GranularSynth.app/Contents/Info.plist"
)

add_custom_command(TARGET GranularSynth POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
            "${CMAKE_BINARY_DIR}/Bundles/GranularSynth.app/Contents/MacOS/GranularSynth"
            "${CMAKE_BINARY_DIR}/bin/GranularSynth"
    COMMAND ${CMAKE_COMMAND} -E copy_directory
            ${CMAKE_SOURCE_DIR}/resources
            ${CMAKE_BINARY_DIR}/Bundles/GranularSynth.app/Contents/Resources
)

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
