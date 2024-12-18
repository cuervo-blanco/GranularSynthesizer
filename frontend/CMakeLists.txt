cmake_minimum_required(VERSION 3.10)
project(QtRustSynth)

set(CMAKE_PREFIX_PATH "/Users/astro/Qt/6.8.1/macos")

set(CMAKE_CXX_STANDARD 17)

find_package(Qt6 REQUIRED COMPONENTS Widgets)

set(CMAKE_AUTOMOC ON)

add_executable(QtRustSynth main.cpp SynthUI.cpp)

target_include_directories(QtRustSynth PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(QtRustSynth PRIVATE Qt6::Widgets)

target_link_libraries(
    QtRustSynth PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../backend/target/release/libbackend.dylib) 


