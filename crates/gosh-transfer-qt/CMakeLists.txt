# SPDX-License-Identifier: AGPL-3.0
# Gosh Transfer Qt - CMake configuration

cmake_minimum_required(VERSION 3.16)
project(gosh-transfer-qt VERSION 2.0.3 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Quick QuickControls2)

# Rust library built by cxx-qt
add_library(gosh_transfer_qt_rust STATIC IMPORTED)

# Platform-specific library extension and prefix
if(WIN32)
    set(RUST_LIB_EXT ".lib")
    set(RUST_LIB_PREFIX "")
else()
    set(RUST_LIB_EXT ".a")
    set(RUST_LIB_PREFIX "lib")
endif()

# Path from workspace root (crate is in crates/gosh-transfer-qt)
set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target")

set_target_properties(gosh_transfer_qt_rust PROPERTIES
    IMPORTED_LOCATION "${RUST_TARGET_DIR}/release/${RUST_LIB_PREFIX}gosh_transfer_qt${RUST_LIB_EXT}"
)

qt_add_executable(gosh-transfer-qt
    cpp/main.cpp
    resources.qrc
)

target_link_libraries(gosh-transfer-qt PRIVATE
    Qt6::Core
    Qt6::Quick
    Qt6::QuickControls2
    gosh_transfer_qt_rust
)

install(TARGETS gosh-transfer-qt
    BUNDLE DESTINATION .
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)
