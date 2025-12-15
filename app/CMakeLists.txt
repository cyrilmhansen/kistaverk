cmake_minimum_required(VERSION 3.10.2)

# Set project name
project("kistaverk-native")

# Find JNI
find_package(JNI REQUIRED)

# Include Rust library
add_library(rust-lib SHARED IMPORTED)
set_target_properties(rust-lib PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../../rust/target/aarch64-linux-android/release/libkistaverk.so
)

# Native bridge library
add_library(native-bridge SHARED
    native-bridge.cpp
)

# Link libraries
target_link_libraries(native-bridge
    ${JNI_LIBRARIES}
    rust-lib
    log
)

# Include directories
target_include_directories(native-bridge PRIVATE
    ${JNI_INCLUDE_DIRS}
    ${CMAKE_SOURCE_DIR}/include
)