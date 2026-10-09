// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "ThreeDmGeometry.h"
#include <filesystem>
#include <memory>
#include <optional>
#include <string>
#include <variant>
#include <span>

namespace OpenMatrix9Gui::ThreeDm {
// Detached dimension label. The affine matrix maps text-local file units to
// world millimeters; heightMm = source text height * scaleMm. A host drawing a
// font at heightMm must divide only the matrix's 3x3 linear part by scaleMm
// (leave translation unchanged). This retains dimstyle scale/shear/reflection
// without applying file-unit conversion twice.
struct ExplodeText {
    std::string text, richText, fontFamily;
    double heightMm = 0;
    std::array<double, 16> worldTransform{}; // row-major
    std::array<double, 3> origin{}, xAxis{}, yAxis{};
};
// Current mathematical cage, not Rhino captive-object editing behavior.
struct ExplodeCage {
    std::array<int, 3> counts{}, degrees{};
    std::vector<std::array<double, 3>> points; // (u*nv+v)*nw+w, world mm
    std::vector<double> weights;
    std::array<std::vector<double>, 3> knots; // openNURBS compact knot convention
};
struct ExplodeComponent {
    std::string name, layer, sourceUuid, sourceClass, role;
    std::array<int, 3> color{180,180,180};
    bool visible = true, locked = false;
    std::variant<TopoDS_Shape, MeshData, ExplodeText> geometry;
    std::shared_ptr<const ExplodeCage> cage;
};
struct ExchangeExplode {
    std::string sourceUuid, sourceClass;
    double scaleMm = 1, tolerance = 1e-6;
    bool flattenedNestedBlocks = false;
    std::vector<ExplodeComponent> components;
};
// Read-only, all-or-error decoding. Caller verifies retained snapshot/hash before
// calling and completes decoding before changing its document. Block references
// recursively flatten embedded definitions (maximum depth 64), including nested
// references; missing/linked-only definitions and unsupported members fail.
ExchangeExplode explodeArchiveRecord(const std::filesystem::path&,
                                     const std::string& sourceUuid,
                                     double customUnitMm = 0);
// Decode the exact verified owned snapshot; no file is reopened.
ExchangeExplode explodeArchiveRecord(std::span<const unsigned char> archiveBytes,
                                     const std::string& sourceUuid,
                                     double customUnitMm = 0);
}
