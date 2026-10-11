# Installed FreeCAD 1.1.4 SDK: exact source headers and installed DLL exports.
# Never configure or build a FreeCAD root target.
set(BUILD_GUI ON)
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_INSTALL_LIBDIR lib)
set(HAVE_Q_DISABLE_COPY_MOVE 1)
configure_file("${FREECAD_SOURCE_DIR}/src/QtCore.h.cmake" "${FREECAD_SDK_BUILD}/src/QtCore.h")
find_package(Qt6 6.8 REQUIRED COMPONENTS Core Gui Widgets Xml Network PrintSupport Svg OpenGLWidgets Concurrent)
set(Python3_ROOT_DIR "${FREECAD_DEPENDENCY_PREFIX}/..")
find_package(Python3 3.11 REQUIRED COMPONENTS Development)
foreach(lib Gui App Base)
    add_library(FreeCAD${lib} SHARED IMPORTED GLOBAL)
    set_target_properties(FreeCAD${lib} PROPERTIES
        IMPORTED_IMPLIB "${FREECAD_SDK_BUILD}/src/${lib}/FreeCAD${lib}.lib"
        IMPORTED_LOCATION "${OPENMATRIX9_STOCK_HOST}/bin/FreeCAD${lib}.dll")
endforeach()
add_library(Part SHARED IMPORTED GLOBAL)
set_target_properties(Part PROPERTIES
    IMPORTED_IMPLIB "${FREECAD_SDK_BUILD}/src/Mod/Part/App/Part.lib"
    IMPORTED_LOCATION "${OPENMATRIX9_STOCK_HOST}/lib/Part.pyd")
# FreeCAD 1.1 uses Boost signals, so no FastSignals binary may be linked.
add_library(libfastsignals INTERFACE)
target_compile_definitions(FreeCADGui INTERFACE PY_SSIZE_T_CLEAN)
target_include_directories(FreeCADGui INTERFACE
    "${CMAKE_CURRENT_LIST_DIR}/../compat/freecad11"
    "${FREECAD_SOURCE_DIR}/src" "${FREECAD_SDK_BUILD}" "${FREECAD_SDK_BUILD}/src"
    "${FREECAD_SOURCE_DIR}/src/3rdParty/PyCXX" "${FREECAD_SOURCE_DIR}/src/3rdParty"
    "${FREECAD_SOURCE_DIR}/src/Gui" "${FREECAD_SDK_BUILD}/src/Gui"
    "${FREECAD_DEPENDENCY_PREFIX}/include" "${FREECAD_DEPENDENCY_PREFIX}/include/opencascade")
target_link_libraries(FreeCADGui INTERFACE FreeCADApp FreeCADBase Python3::Python
    Qt6::Core Qt6::Gui Qt6::Widgets Qt6::Xml Qt6::Network Qt6::PrintSupport Qt6::Svg Qt6::OpenGLWidgets Qt6::Concurrent)
target_link_directories(FreeCADGui INTERFACE "${FREECAD_DEPENDENCY_PREFIX}/lib")
target_link_libraries(FreeCADGui INTERFACE "${FREECAD_DEPENDENCY_PREFIX}/lib/Coin4.lib")
target_link_libraries(FreeCADGui INTERFACE "${FREECAD_DEPENDENCY_PREFIX}/lib/fmt.lib")
function(SET_BIN_DIR target)
    set_target_properties(${target} PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${OPENMATRIX9_RUNTIME_ROOT}/bin")
endfunction()
function(SET_PYTHON_PREFIX_SUFFIX target)
    set_target_properties(${target} PROPERTIES PREFIX "" SUFFIX ".pyd")
endfunction()
function(fc_target_copy_resource target source destination)
    foreach(file IN LISTS ARGN)
        get_filename_component(folder "${file}" DIRECTORY)
        add_custom_command(TARGET ${target} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E make_directory "${OPENMATRIX9_RUNTIME_ROOT}/Mod/OpenMatrix9/${folder}"
            COMMAND ${CMAKE_COMMAND} -E copy_if_different "${source}/${file}" "${OPENMATRIX9_RUNTIME_ROOT}/Mod/OpenMatrix9/${file}" VERBATIM)
    endforeach()
endfunction()
