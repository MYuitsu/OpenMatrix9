// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "ThreeDmGeometry.h"
#include <filesystem>
#include <optional>
#include <string>
#include <span>

namespace OpenMatrix9Gui::ThreeDm {
// Owned extraction from the openNURBS adapter. Rust owns editing, bind/inverse
// solving and portable validation; no ON_* pointers cross this boundary.
struct CageDescriptor {
    std::array<int,3> counts{}, degrees{};
    std::vector<std::array<double,3>> points; // (u*nv+v)*nw+w, Euclidean world mm
    std::vector<double> weights;
    std::array<std::vector<double>,3> fullKnots; // count + degree + 1
    bool rational = false;
};
struct CageOriginalReference {
    // Rhino's m_nurbs_cage0 maps ORIGINAL world coordinates to parameters.
    // These matrices include unit conversion, and are row-major.
    std::array<double,16> worldMmToParameters{}, parametersToWorldMm{};
};
enum class CaptiveGeometryState { CurrentArchiveOutputOnly };
struct CageArchiveRecord {
    std::string sourceUuid, sourceClass;
    double scaleMm = 1, toleranceMm = 1e-6, morphToleranceMm = 0;
    CageDescriptor currentCage;
    std::optional<CageOriginalReference> originalReference;
    std::vector<std::string> captiveSourceUuids; // only explicit m_captive_id
    bool isMorphControl = false, preserveStructure = false, quickPreview = false;
    CaptiveGeometryState captiveGeometryState = CaptiveGeometryState::CurrentArchiveOutputOnly;
};
// Read-only, all-or-error. Host checks the retained archive/hash before calling,
// then resolves EVERY explicit UUID in the same import namespace before mutation.
// Morph-control serialization contains no undeformed captive snapshots. Preserve
// current captive y by solving currentCage(p)=y and freeze p; edits use editedCage(p).
// Do not evaluate editedCage(originalReference*y): that deforms current outputs twice.
// Folded/ambiguous/failed inverse solves must fail atomically in the Rust/host layer.
// Plain ON_NurbsCage has neither an original reference nor captive relationships.
// Localized, curve and surface morph controls currently fail with precise errors.
CageArchiveRecord cageArchiveRecord(const std::filesystem::path&,
                                   const std::string& sourceUuid,
                                   double customUnitMm = 0);
// Decode the exact verified owned snapshot; no file is reopened.
CageArchiveRecord cageArchiveRecord(std::span<const unsigned char> archiveBytes,
                                   const std::string& sourceUuid,
                                   double customUnitMm = 0);
}
