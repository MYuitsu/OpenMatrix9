// SPDX-License-Identifier: LGPL-2.1-or-later
// Standalone diagnostic: isolate OCCT meshing from OM9's Rust snapshot copying.
// This is not linked into the product and is never run in the user's window.
#define NOMINMAX
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <psapi.h>
#include <BRep_Builder.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <Poly_Triangulation.hxx>
#include <IMeshTools_Parameters.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopTools_FormatVersion.hxx>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <iterator>
#include <sstream>

static bool sample(const char* stage) {
    PROCESS_MEMORY_COUNTERS_EX info{};
    info.cb = sizeof(info);
    if (!GetProcessMemoryInfo(GetCurrentProcess(), reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&info), sizeof(info))) return false;
    std::printf("OCCT_STAGE stage=%s private=%zu peak_pagefile=%zu working=%zu peak_working=%zu\n",
        stage, size_t(info.PrivateUsage), size_t(info.PeakPagefileUsage), size_t(info.WorkingSetSize), size_t(info.PeakWorkingSetSize));
    return true;
}

int main(int argc, char** argv) {
    if (argc < 2 || argc > 4) return 1;
    const double deflection = std::strtod(argv[1], nullptr);
    if (!(deflection > 0.)) return 1;
    try {
        const auto original = BRepPrimAPI_MakeSphere(12.).Shape();
        std::ostringstream encoded;
        BRepTools::Write(original, encoded, false, false, TopTools_FormatVersion_CURRENT);
        std::string original_source = encoded.str();
        if (argc >= 3 && std::string(argv[2]) != "-") {
            std::ifstream input_file(argv[2], std::ios::binary);
            if (!input_file) return 9;
            original_source.assign(std::istreambuf_iterator<char>(input_file), std::istreambuf_iterator<char>());
        }
        std::istringstream input(original_source);
        BRep_Builder builder;
        TopoDS_Shape shape;
        BRepTools::Read(shape, input, builder);
        if (shape.IsNull() || !sample("after_read")) return 2;
        if (!BRepCheck_Analyzer(shape).IsValid() || !sample("after_validate")) return 3;
        std::ostringstream archive;
        BRepTools::Write(shape, archive, false, false, TopTools_FormatVersion_CURRENT);
        const std::string source = archive.str();
        if (!sample("before_mesh")) return 4;
        IMeshTools_Parameters params;
        params.Deflection = deflection;
        params.Angle = 0.3;
        params.Relative = false;
        params.InParallel = false;
        const std::string algorithm = argc == 4 ? argv[3] : "default";
        if (algorithm == "watson") params.MeshAlgo = IMeshTools_MeshAlgoType_Watson;
        else if (algorithm == "delabella") params.MeshAlgo = IMeshTools_MeshAlgoType_Delabella;
        else if (algorithm != "default") return 10;
        BRepMesh_IncrementalMesh mesh(shape, params);
        if (!mesh.IsDone() || !sample("after_mesh")) return 5;
        size_t triangles = 0;
        for (TopExp_Explorer it(shape, TopAbs_FACE); it.More(); it.Next()) {
            TopLoc_Location location;
            auto poly = BRep_Tool::Triangulation(TopoDS::Face(it.Current()), location);
            if (poly.IsNull()) return 6;
            triangles += size_t(poly->NbTriangles());
        }
        std::printf("OCCT_MESH algorithm=%s deflection=%.9g triangles=%zu input_source_bytes=%zu roundtrip_source_bytes=%zu\n", algorithm.c_str(), deflection, triangles, original_source.size(), source.size());
        return triangles ? 0 : 7;
    } catch (...) {
        std::fprintf(stderr, "OCCT diagnostic failed\n");
        return 8;
    }
}
