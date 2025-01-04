cmake_minimum_required(VERSION 3.10)
project(QtRustSynth)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_PREFIX_PATH "/Users/astro/Qt/6.8.1/macos")
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt6 REQUIRED COMPONENTS Widgets Core)

find_library(RUST_LIB NAMES libbackend.dylib PATHS
    ${CMAKE_CURRENT_SOURCE_DIR}/../backend/target/release
    REQUIRED) 
if(NOT RUST_LIB)
    message(FATAL_ERROR "Rust library not found!")
endif()

qt_add_executable(QtRustSynth
    resources.qrc
    main.cpp
    SynthUI.h SynthUI.cpp 
    dialog/AudioSettingsDialog.h dialog/AudioSettingsDialog.cpp 
    settings/SettingsManager.h settings/SettingsManager.cpp
)

target_link_libraries(QtRustSynth PRIVATE ${RUST_LIB})

target_include_directories(QtRustSynth PRIVATE 
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/settings
    ${CMAKE_CURRENT_SOURCE_DIR}/dialog
)
target_link_libraries(QtRustSynth PRIVATE Qt6::Widgets)
