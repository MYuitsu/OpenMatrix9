# Build the module against an existing matching FreeCAD source/build SDK.
# Does not modify/reconfigure the SDK cache or rebuild FreeCAD itself.
set(FREECAD_SOURCE_DIR "" CACHE PATH "Matching FreeCAD source checkout")
set(FREECAD_SDK_BUILD "" CACHE PATH "Existing FreeCAD build with headers/import libraries")
set(FREECAD_DEPENDENCY_PREFIX "" CACHE PATH "Matching FreeCAD dependency prefix")
set(OPENMATRIX9_RUNTIME_ROOT "" CACHE PATH "Optional separate runtime output for interactive module previews")
if(NOT OPENMATRIX9_RUNTIME_ROOT)
    set(OPENMATRIX9_RUNTIME_ROOT "${FREECAD_SDK_BUILD}")
endif()
foreach(required FREECAD_SOURCE_DIR FREECAD_SDK_BUILD FREECAD_DEPENDENCY_PREFIX)
    if(NOT IS_DIRECTORY "${${required}}")
        message(FATAL_ERROR "Provide ${required} for standalone SDK mode")
    endif()
endforeach()
add_library(Part SHARED IMPORTED GLOBAL)
set_target_properties(Part PROPERTIES IMPORTED_IMPLIB "${FREECAD_SDK_BUILD}/src/Mod/Part/App/Part.lib" IMPORTED_LOCATION "${FREECAD_SDK_BUILD}/Mod/Part/Part.pyd")
add_library(libfastsignals STATIC IMPORTED GLOBAL)
set_target_properties(libfastsignals PROPERTIES IMPORTED_LOCATION "${FREECAD_SDK_BUILD}/src/3rdParty/FastSignals/libfastsignals/libfastsignals.lib")
set(BUILD_GUI ON)
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_INSTALL_LIBDIR lib)
find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets Xml Network PrintSupport Svg OpenGLWidgets Concurrent)
set(Python3_ROOT_DIR "${FREECAD_DEPENDENCY_PREFIX}/..")
find_package(Python3 REQUIRED COMPONENTS Development)
foreach(lib Gui App Base)
    add_library(FreeCAD${lib} SHARED IMPORTED GLOBAL)
    set_target_properties(FreeCAD${lib} PROPERTIES
        IMPORTED_IMPLIB "${FREECAD_SDK_BUILD}/src/${lib}/FreeCAD${lib}.lib"
        IMPORTED_LOCATION "${FREECAD_SDK_BUILD}/bin/FreeCAD${lib}.dll")
endforeach()
target_include_directories(FreeCADGui INTERFACE
    "${FREECAD_SOURCE_DIR}/src" "${FREECAD_SDK_BUILD}" "${FREECAD_SDK_BUILD}/src"
    "${FREECAD_SOURCE_DIR}/src/3rdParty/PyCXX" "${FREECAD_SOURCE_DIR}/src/3rdParty"
    "${FREECAD_SOURCE_DIR}/src/3rdParty/FastSignals/libfastsignals/include"
    "${FREECAD_SOURCE_DIR}/src/Gui" "${FREECAD_SDK_BUILD}/src/Gui"
    "${FREECAD_SOURCE_DIR}/src/3rdParty/coin/include" "${FREECAD_SDK_BUILD}/src/3rdParty/coin/include"
    "${FREECAD_DEPENDENCY_PREFIX}/include" "${FREECAD_DEPENDENCY_PREFIX}/include/opencascade")
target_link_libraries(FreeCADGui INTERFACE FreeCADApp FreeCADBase Python3::Python
    Qt6::Core Qt6::Gui Qt6::Widgets Qt6::Xml Qt6::Network Qt6::PrintSupport Qt6::Svg Qt6::OpenGLWidgets Qt6::Concurrent)
# Viewer headers autolink the matching SDK Coin library on MSVC.
if(MSVC)
    find_library(OPENMATRIX9_COIN_LIBRARY NAMES Coin4 PATHS "${FREECAD_SDK_BUILD}/lib" NO_DEFAULT_PATH REQUIRED)
    get_filename_component(OPENMATRIX9_COIN_DIRECTORY "${OPENMATRIX9_COIN_LIBRARY}" DIRECTORY)
    target_link_directories(FreeCADGui INTERFACE "${OPENMATRIX9_COIN_DIRECTORY}")
    target_link_libraries(FreeCADGui INTERFACE "${OPENMATRIX9_COIN_LIBRARY}")
endif()
function(SET_BIN_DIR target)
    set_target_properties(${target} PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${OPENMATRIX9_RUNTIME_ROOT}/bin")
endfunction()
function(SET_PYTHON_PREFIX_SUFFIX target)
    set_target_properties(${target} PROPERTIES PREFIX "" SUFFIX ".pyd")
endfunction()
function(fc_target_copy_resource target source destination)
    foreach(file IN LISTS ARGN)
        if(IS_ABSOLUTE "${file}")
            file(RELATIVE_PATH relative "${source}" "${file}")
            set(input "${file}")
        else()
            set(relative "${file}")
            set(input "${source}/${file}")
        endif()
        get_filename_component(folder "${relative}" DIRECTORY)
        add_custom_command(TARGET ${target} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E make_directory "${OPENMATRIX9_RUNTIME_ROOT}/Mod/OpenMatrix9/${folder}"
            COMMAND ${CMAKE_COMMAND} -E copy_if_different "${input}" "${OPENMATRIX9_RUNTIME_ROOT}/Mod/OpenMatrix9/${relative}" VERBATIM)
    endforeach()
endfunction()
