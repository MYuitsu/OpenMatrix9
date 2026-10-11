// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Base/Vector3D.h>
#include <QPoint>
#include <array>
#include <cstdint>
#include <optional>
#include <string>

namespace Gui { class View3DInventor; }
class SoSeparator;

namespace OpenMatrix9Gui {
enum class GumballMeshComponentKind { WholeMesh, Vertex, Edge };

// Owned GUI-thread observations. Native Mesh point/facet IDs are zero based
// and meaningful only while the source topology stamp remains unchanged.
// Edge IDs and highlight endpoints are always ordered lower -> higher ID.
struct GumballMeshComponentPick {
    std::string documentName;
    std::string objectName;
    std::string subelement; // OM9MeshVertex:N / OM9MeshEdge:A:B; empty = whole Mesh.
    GumballMeshComponentKind kind=GumballMeshComponentKind::WholeMesh;
    std::uint32_t first=0;
    std::uint32_t second=0;
    std::uint32_t facet=0;
    std::array<Base::Vector3d,2> worldHighlightPoints{Base::Vector3d(),Base::Vector3d()};
    unsigned highlightPointCount=0;
};

// Query the nearest visible native Mesh triangle, with its actual SoFaceDetail.
// A vertex within logicalPickRadius takes priority over an edge. Otherwise the
// triangle interior returns a whole-owner selection. No selection/model writes.
std::optional<GumballMeshComponentPick> pickGumballMeshComponent(
    Gui::View3DInventor* view, const QPoint& logicalPosition, double logicalPickRadius);

// Replace this caller-owned rendering node. nullptr / whole Mesh clears it.
// Unpickable annotation geometry is excluded from fitting/bounding boxes.
void updateGumballMeshComponentHighlight(
    SoSeparator* root, const GumballMeshComponentPick* pick, double devicePixelRatio=1.0);
}
