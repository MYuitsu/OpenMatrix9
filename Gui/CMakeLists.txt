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

# Archive/staging adapters use the same Rust layer policy as the GUI. Propagate
# it to the standalone import worker rather than linking a second policy core.
target_link_libraries(OM9ThreeDm PUBLIC OpenMatrix9RustLib)
if(WIN32)
    target_link_libraries(OM9ThreeDm PUBLIC ws2_32 userenv ntdll bcrypt)
endif()



# ============================================================
# FreeCAD GUI module
# ============================================================

set(
    OpenMatrix9Gui_SRCS

    AppOpenMatrix9Gui.cpp
    ThreeDmPython.cpp
    LayerClipboard.cpp
    LayerClipboard.h
    LayerClipboardPython.cpp
    LayerDocumentAdapter.cpp
    LayerCollectionAdapter.cpp
    LayerCollectionAdapter.h
    LayerCollectionAbi.h
    LayerSourceArchiveAdapter.cpp
    LayerSourceArchiveAdapter.h
    LayerSourcePayloadAbi.h
    LayerDocumentAdapter.h
    LayerDocumentPython.cpp
    LayerController.cpp
    LayerController.h
    ThreeDmModeling.h
    ThreeDmHatchDialog.cpp
    ThreeDmHatchDialog.h
    ThreeDmMerge.cpp
    ThreeDmMerge.h
    CoreThreeDm.cpp
    CoreThreeDm.h

    Workbench.cpp
    Workbench.h

    Command.cpp
    SurfaceController.cpp
    SurfaceGeometry.cpp
    SurfaceLoft.cpp
    SurfaceSeams.cpp
    SurfaceRefit.cpp
    SurfaceConstraints.cpp
    EditController.cpp
    EditGeometry.cpp
    EditCurveOptions.cpp
    EditMesh.cpp
    Phase3Inputs.cpp
    EditSpecialTypes.cpp
    CurveController.cpp
    CurveController.h
    CurveGeometry.cpp
    CurveGeometry.h
    CoreRebuild.cpp
    CoreRebuild.h
    ModelingCurveEditor.cpp
    ModelingCurveEditor.h
    ModelingCurveEditorDialog.cpp
    ModelingCurveEditorDialog.h
    CurveBasisTransfer.h
    Phase2Rust.h
    CommandConsole.cpp
    CommandConsole.h
    CoreWorkspace.cpp
    CoreWorkspace.h
    CadPresentation.cpp
    CadPresentation.h
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
    SnapObjectInfo.cpp
    SnapObjectInfo.h
    SnapQueryIndex.cpp
    SnapQueryIndex.h
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
)


target_link_libraries(
    OpenMatrix9Gui

    PRIVATE

    FreeCADGui
    Part
    TKOffset
    OM9ThreeDm
    libfastsignals

    OpenMatrix9RustLib
    Qt6::Widgets
)
if(WIN32)
    target_link_libraries(OpenMatrix9Gui PRIVATE ws2_32 userenv ntdll bcrypt ole32 user32)
endif()
target_compile_features(OpenMatrix9Gui PRIVATE cxx_std_23)
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

add_executable(OM9ThreeDmImportWorker ThreeDmImportWorker.cpp)
target_link_libraries(OM9ThreeDmImportWorker PRIVATE OM9ThreeDm)
SET_BIN_DIR(OM9ThreeDmImportWorker)
set_target_properties(OM9ThreeDmImportWorker PROPERTIES DEBUG_OUTPUT_NAME OM9ThreeDmImportWorker)
add_dependencies(OpenMatrix9Gui OM9ThreeDmImportWorker)
install(TARGETS OM9ThreeDmImportWorker RUNTIME DESTINATION bin)


SET_PYTHON_PREFIX_SUFFIX(
    OpenMatrix9Gui
)


install(
    TARGETS
        OpenMatrix9Gui

    DESTINATION
        ${CMAKE_INSTALL_LIBDIR}
)
