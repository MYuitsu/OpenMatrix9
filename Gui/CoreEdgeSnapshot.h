// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS_Shape.hxx>
#include <QJsonObject>
#include <cstddef>
namespace OpenMatrix9Gui {
using TopologyShapeMap=NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>;
// Caller guards live immutable source on GUI thread. No native handles escape.
QJsonObject edgeGeometrySnapshot(const TopoDS_Shape&,const TopologyShapeMap&,const TopologyShapeMap&,std::size_t&);
}
