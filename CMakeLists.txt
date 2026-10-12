# SPDX-License-Identifier: LGPL-2.1-or-later
cmake_minimum_required(VERSION 3.24)
if(CMAKE_SOURCE_DIR STREQUAL CMAKE_CURRENT_SOURCE_DIR)
    project(OpenMatrix9Native LANGUAGES CXX)
    include(cmake/StandaloneSDK.cmake)
endif()

if(BUILD_GUI)
    add_subdirectory(Gui)
endif()

set(OpenMatrix9_Scripts
    Init.py
    ThreeDm.py
    ThreeDmModeling.py
    ThreeDmClipboard.py
    ThreeDmArchiveState.py
    ThreeDmStorage.py
    ThreeDmTextDot.py
    ThreeDmPointCloud.py
    ThreeDmHatch.py
    ThreeDmNativeFields.py
    ThreeDmMigration.py
    ThreeDmUpgrade.py
)

if(BUILD_GUI)
    list(APPEND OpenMatrix9_Scripts
        InitGui.py
    )
endif()

add_custom_target(
    OpenMatrix9Scripts ALL
    SOURCES
        ${OpenMatrix9_Scripts}
)

fc_target_copy_resource(
    OpenMatrix9Scripts
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_BINARY_DIR}/Mod/OpenMatrix9
    ${OpenMatrix9_Scripts}
)
file(GLOB_RECURSE OpenMatrix9_Resources CONFIGURE_DEPENDS RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}" "${CMAKE_CURRENT_SOURCE_DIR}/Resources/*")
fc_target_copy_resource(OpenMatrix9Scripts "${CMAKE_CURRENT_SOURCE_DIR}" "${CMAKE_BINARY_DIR}/Mod/OpenMatrix9" ${OpenMatrix9_Resources})

# Khi chỉ build OpenMatrix9Gui thì vẫn copy InitGui.py
if(BUILD_GUI)
    add_dependencies(
        OpenMatrix9Gui
        OpenMatrix9Scripts
    )
endif()

install(
    FILES
        ${OpenMatrix9_Scripts}
    DESTINATION
        Mod/OpenMatrix9
)
install(DIRECTORY Resources DESTINATION Mod/OpenMatrix9)
