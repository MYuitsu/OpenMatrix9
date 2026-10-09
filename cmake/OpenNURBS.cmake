include(FetchContent)
enable_language(C)
FetchContent_Declare(om9_opennurbs
    GIT_REPOSITORY https://github.com/mcneel/opennurbs.git
    GIT_TAG eb92af3ba1806b0a34a99aba0d3bda83e3d46083
    SOURCE_SUBDIR om9-fetch-only
)
FetchContent_MakeAvailable(om9_opennurbs)
if(NOT TARGET opennurbsStatic)
    add_subdirectory("${om9_opennurbs_SOURCE_DIR}" "${om9_opennurbs_BINARY_DIR}" EXCLUDE_FROM_ALL)
endif()
set_target_properties(opennurbsStatic PROPERTIES MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>DLL")
include(${CMAKE_CURRENT_LIST_DIR}/PolyEdgeArchive.cmake)
om9_repair_polyedge_read(opennurbsStatic "${om9_opennurbs_SOURCE_DIR}")
include(${CMAKE_CURRENT_LIST_DIR}/CurveOnSurfaceArchive.cmake)
om9_repair_curve_on_surface_read(opennurbsStatic "${om9_opennurbs_SOURCE_DIR}" "3cf041dd2b9ad4c4e97cfc132c3640b9a41c5a7f85a1c30352d2a7c3c7128d76")
include(${CMAKE_CURRENT_LIST_DIR}/RdkDocumentArchive.cmake)
om9_repair_rdk_document_write(opennurbsStatic "${om9_opennurbs_SOURCE_DIR}")
find_package(OpenCASCADE REQUIRED COMPONENTS FoundationClasses ModelingData ModelingAlgorithms)
find_package(Qt6 REQUIRED COMPONENTS Core)
add_library(OM9ThreeDm STATIC
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmStaging.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmImportJobs.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmCurves.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmSurfaces.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmMeshes.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmBrep.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmArchive.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmExplode.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmCage.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmInventory.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmTransforms.cpp ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmPointCloud.cpp ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmHatch.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmCurveOnSurface.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmCurveOnSurfaceSchema.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmNativeReferences.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmTrimMapping.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmClassRegistry.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmSolidShells.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmHatchLoops.cpp)
target_sources(OM9ThreeDm PRIVATE ${CMAKE_CURRENT_LIST_DIR}/../Gui/ThreeDmHatchLoopEdit.cpp)
target_include_directories(OM9ThreeDm PUBLIC ${CMAKE_CURRENT_LIST_DIR}/../Gui ${OpenCASCADE_INCLUDE_DIR})
target_link_libraries(OM9ThreeDm PUBLIC opennurbsStatic TKernel TKMath TKG2d TKG3d TKGeomBase TKBRep TKGeomAlgo TKTopAlgo TKPrim TKShHealing)
target_compile_features(OM9ThreeDm PUBLIC cxx_std_23)
target_link_libraries(OM9ThreeDm PUBLIC Qt6::Core)
if(MSVC)
    target_compile_options(OM9ThreeDm PRIVATE /utf-8 /bigobj)
    target_compile_definitions(OM9ThreeDm PUBLIC NOMINMAX)
endif()

add_executable(OM9ThreeDmExplodeTest EXCLUDE_FROM_ALL ${CMAKE_CURRENT_LIST_DIR}/../tests/native/three_dm_explode.cpp)
target_link_libraries(OM9ThreeDmExplodeTest PRIVATE OM9ThreeDm)
add_executable(OM9ThreeDmCageTest EXCLUDE_FROM_ALL ${CMAKE_CURRENT_LIST_DIR}/../tests/native/three_dm_cage.cpp)
target_link_libraries(OM9ThreeDmCageTest PRIVATE OM9ThreeDm)
add_executable(OM9CageGeometryTest EXCLUDE_FROM_ALL
    ${CMAKE_CURRENT_LIST_DIR}/../tests/native/cage_geometry.cpp
    ${CMAKE_CURRENT_LIST_DIR}/../Gui/CageGeometry.cpp)
target_include_directories(OM9CageGeometryTest PRIVATE "${CMAKE_SOURCE_DIR}/../../src" "${CMAKE_BINARY_DIR}" "${FREECAD_SDK_BUILD}")
target_link_libraries(OM9CageGeometryTest PRIVATE OM9ThreeDm FreeCADBase)
if(MSVC)
    target_compile_options(OM9CageGeometryTest PRIVATE /utf-8 /FIFCGlobal.h /FIFCConfig.h)
endif()
