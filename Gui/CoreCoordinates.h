// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once

#include <Base/Placement.h>
#include <Base/Vector3D.h>
#include <array>
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

class QString;
namespace App { class Document; class DocumentObject; }
namespace Gui { class View3DInventor; }

namespace OpenMatrix9Gui {
enum class CoordinateExpected : std::uint32_t { Point = 0, PointOrLength = 1 };
enum class CoordinateSource : std::uint32_t {
    Typed = 0, WorldPick = 1, WorldSnap = 2, OriginShortcut = 3
};
enum class CoordinateValueKind : std::uint32_t { Point = 0, Length = 1 };

// Owned command-start snapshot. The frame stays latched after viewport/CPlane
// navigation; logicalViewSlot identifies a document slot, not a unique native view.
struct CoordinateContext {
    std::uint64_t documentOwner = 0;
    std::uint64_t frameRevision = 0;
    unsigned logicalViewSlot = 0;
    std::string documentUid;
    std::string unitContext;
    std::array<double, 12> frame{}; // origin, X, Y, Z in world coordinates
    Base::Placement plane;
};

// All values and provenance are owned. The request fields permit Rust to
// reconstruct/validate combined reports without a second native JSON policy.
struct CoordinateInput {
    CoordinateValueKind kind = CoordinateValueKind::Point;
    Base::Vector3d point;
    double lengthMm = 0;
    std::string provenance;
    CoordinateContext context;
    CoordinateExpected expected = CoordinateExpected::Point;
    CoordinateSource source = CoordinateSource::Typed;
    std::string original;
    std::optional<Base::Vector3d> previous;
};

// GUI-thread only. Capture/verify do not write document properties or metadata.
// Verification rejects another document lifetime or changed units, but deliberately
// does not compare the current CPlane/revision with the command-start frame.
CoordinateContext captureCoordinateContext(App::Document&, Gui::View3DInventor*);
void verifyCoordinateContext(const App::Document&, const CoordinateContext&);

// Stateless Rust resolution. Call verifyCoordinateContext before accepting an
// input and again before mutation. Only advance previous after consumer acceptance.
CoordinateInput parseCoordinateInput(
    const CoordinateContext&, const QString&,
    CoordinateExpected = CoordinateExpected::Point,
    const std::optional<Base::Vector3d>& previous = std::nullopt);
CoordinateInput worldCoordinateInput(
    const CoordinateContext&, const Base::Vector3d&,
    CoordinateSource = CoordinateSource::WorldPick,
    const std::optional<Base::Vector3d>& previous = std::nullopt);

// Rust serializes the complete versioned JSON inputs array (maximum 64 inputs).
std::string coordinateInputReport(const std::vector<CoordinateInput>&);

// Property-only write of OM9CoordinateReport. Caller owns the transaction and
// rollback; this helper neither opens/commits a transaction nor recomputes geometry.
void persistCoordinateProvenance(App::DocumentObject&, const std::vector<CoordinateInput>&);
}
