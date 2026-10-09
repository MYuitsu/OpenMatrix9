# SPDX-License-Identifier: LGPL-2.1-or-later

find_program(
    CARGO_EXECUTABLE
    cargo
    REQUIRED
)
include(../cmake/OpenNURBS.cmake)


# ============================================================
# Rust
# ============================================================

set(
    OPENMATRIX9_RUST_DIR
    "${CMAKE_CURRENT_SOURCE_DIR}/../rust"
)

set(
    OPENMATRIX9_RUST_TARGET_DIR
    "${CMAKE_CURRENT_BINARY_DIR}/rust-target"
)


if(WIN32)

    set(
        OPENMATRIX9_RUST_LIB
        "${OPENMATRIX9_RUST_TARGET_DIR}/release/openmatrix9_rust.lib"
    )

else()

    set(
        OPENMATRIX9_RUST_LIB
        "${OPENMATRIX9_RUST_TARGET_DIR}/release/libopenmatrix9_rust.a"
    )

endif()


file(
    GLOB_RECURSE
    OPENMATRIX9_RUST_SOURCES
    CONFIGURE_DEPENDS
    "${OPENMATRIX9_RUST_DIR}/src/*.rs"
)


add_custom_command(

    OUTPUT
        "${OPENMATRIX9_RUST_LIB}"

    COMMAND
        ${CMAKE_COMMAND}
        -E
        env
        "CARGO_TARGET_DIR=${OPENMATRIX9_RUST_TARGET_DIR}"
        ${CARGO_EXECUTABLE}
        build
        --release
        --manifest-path
        "${OPENMATRIX9_RUST_DIR}/Cargo.toml"

    DEPENDS
        "${OPENMATRIX9_RUST_DIR}/Cargo.toml"
        "${CMAKE_CURRENT_SOURCE_DIR}/../Resources/menu/MainMenu.ini"
        "${CMAKE_CURRENT_SOURCE_DIR}/../Resources/menu/CommandNames.txt"
        ${OPENMATRIX9_RUST_SOURCES}

    WORKING_DIRECTORY
        "${OPENMATRIX9_RUST_DIR}"

    COMMENT
        "Building OpenMatrix9 Rust core"

    VERBATIM
)


add_custom_target(
    OpenMatrix9Rust
    DEPENDS
        "${OPENMATRIX9_RUST_LIB}"
)


add_library(
    OpenMatrix9RustLib
    STATIC
    IMPORTED
    GLOBAL
)



set_target_properties(
    OpenMatrix9RustLib
    PROPERTIES

    IMPORTED_LOCATION
        "${OPENMATRIX9_RUST_LIB}"
)


add_dependencies(
    OpenMatrix9RustLib
    OpenMatrix9Rust
)



# ============================================================
# FreeCAD GUI module
# ============================================================

set(
    OpenMatrix9Gui_SRCS
    CoreUnits.cpp
    CoreUnits.h
    CoreCPlanes.cpp
    CoreCPlanes.h

    AppOpenMatrix9Gui.cpp
    ThreeDmPython.cpp
    CoreThreeDm.cpp
    CoreThreeDm.h

    Workbench.cpp
    Workbench.h

    Command.cpp
    CurveController.cpp
    CurveController.h
    CurveCircle.cpp
    CurveCircle.h
    CurveEllipse.cpp
    CurveEllipse.h
    CircleHistory.cpp
    CircleHistory.h
    CircleHistoryAdapter.cpp
    CircleHistoryAdapter.h
    CircleTangent.cpp
    CircleTangent.h
    CoreLayers.cpp
    CoreLayers.h
    SurfaceController.cpp
    EditController.cpp
    EditController.h
    EditGeometry.cpp
    EditCurveOptions.cpp
    EditMesh.cpp
    EditSpecialTypes.cpp
    EditSpecialTypes.h
    ExplodedText.cpp
    ExplodedText.h
    HistoryController.cpp
    HistoryController.h
    CageController.cpp
    CageController.h
    HistoryFeature.cpp
    HistoryFeature.h
    BuilderHistory.cpp
    BuilderHistory.h
    CageFeature.cpp
    CageFeature.h
    CageGeometry.cpp
    CageGeometry.h
    CageRust.h
    RetainedArchive.cpp
    RetainedArchive.h
    EditGeometry.h
    SurfaceController.h
    SolidController.cpp
    SolidController.h
    SolidReferences.cpp
    SolidReferences.h
    SurfaceGeometry.cpp
    SurfaceGeometry.h
    SurfaceLoft.cpp
    SurfaceLoft.h
    SurfaceSeams.cpp
    SurfaceSeams.h
    SurfaceRefit.cpp
    SurfaceRefit.h
    SurfaceConstraints.cpp
    SurfaceConstraints.h
    SurfaceHistory.cpp
    SurfaceHistory.h
    CurveGeometry.cpp
    CurveGeometry.h
    CoreRebuild.cpp
    CoreRebuild.h
    CommandConsole.cpp
    CommandConsole.h
    CoreWorkspace.cpp
    CoreWorkspace.h
    CoreViewControls.cpp
    CoreViewControls.h
    CoreNotes.cpp
    CoreNotes.h
    CorePictureFrame.cpp
    CorePictureFrame.h
    CoreDistance.cpp
    CoreDistance.h
    CoreSnapGeometry.cpp
    CoreSnapGeometry.h
    CoreSnaps.cpp
    CoreSnaps.h
    CoreKeyboard.cpp
    CoreKeyboard.h
    CoreMouse.cpp
    CoreMouse.h
    CameraState.h

    RustBridge.h
    NativeCommands.h
    MatrixSidebar.h
    MatrixSidebar.cpp
)


add_library(
    OpenMatrix9Gui
    SHARED

    ${OpenMatrix9Gui_SRCS}
)


add_dependencies(
    OpenMatrix9Gui
    OpenMatrix9Rust
)

if(WIN32)
    target_compile_definitions(OpenMatrix9Gui PRIVATE NOMINMAX)
endif()


target_include_directories(
    OpenMatrix9Gui

    PRIVATE

    ${CMAKE_BINARY_DIR}

    ${CMAKE_SOURCE_DIR}/src

    ${CMAKE_BINARY_DIR}/src

    ${CMAKE_CURRENT_BINARY_DIR}
    ${OpenCASCADE_INCLUDE_DIR}
)


target_link_libraries(
    OpenMatrix9Gui

    PRIVATE

    FreeCADGui
    OM9ThreeDm
    libfastsignals

    OpenMatrix9RustLib
    Qt6::Widgets
    TKOffset
)
if(TARGET Part)
    target_link_libraries(OpenMatrix9Gui PRIVATE Part)
else()
    find_library(OM9_PART_LIBRARY NAMES Part
        HINTS "${FREECAD_SDK_BUILD}/src/Mod/Part/App" REQUIRED)
    target_link_libraries(OpenMatrix9Gui PRIVATE "${OM9_PART_LIBRARY}")
endif()
if(WIN32)
    target_link_libraries(OpenMatrix9Gui PRIVATE ws2_32 userenv ntdll bcrypt)
endif()
target_compile_features(OpenMatrix9Gui PRIVATE cxx_std_23)
foreach(om9_occ_lib TKGeomAlgo TKGeomBase TKG2d TKG3d TKMath TKernel)
    find_library(OM9_${om9_occ_lib}_LIBRARY NAMES ${om9_occ_lib}
        HINTS "${FREECAD_DEPENDENCY_PREFIX}/lib" "${OpenCASCADE_LIBRARY_DIR}" ${OpenCASCADE_LIBRARY_DIRS} REQUIRED)
    target_link_libraries(OpenMatrix9Gui PRIVATE "${OM9_${om9_occ_lib}_LIBRARY}")
endforeach()
if(MSVC)
    target_compile_options(OpenMatrix9Gui PRIVATE /utf-8 /FIFCGlobal.h /FIFCConfig.h)
endif()


if(MSVC)

    target_compile_options(
        OpenMatrix9Gui
        PRIVATE

        /wd4251
        /wd4273
    )

endif()


SET_BIN_DIR(
    OpenMatrix9Gui
)

# Optional isolated native output for validation while an SDK instance is open.
set(OPENMATRIX9_RUNTIME_OUTPUT_DIR "" CACHE PATH "Isolated OpenMatrix9Gui runtime output directory")
if(OPENMATRIX9_RUNTIME_OUTPUT_DIR)
    set_target_properties(OpenMatrix9Gui PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${OPENMATRIX9_RUNTIME_OUTPUT_DIR}")
endif()


SET_PYTHON_PREFIX_SUFFIX(
    OpenMatrix9Gui
)


install(
    TARGETS
        OpenMatrix9Gui

    DESTINATION
        ${CMAKE_INSTALL_LIBDIR}
)
