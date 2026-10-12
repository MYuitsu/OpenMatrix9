// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Geom2d_Curve.hxx>
#include <Geom_Curve.hxx>
#include <TopoDS_Face.hxx>
#include <QJsonObject>
#include <cstddef>
namespace OpenMatrix9Gui {
namespace ThreeDm::BasisContract { struct CurveBasisData; }
// Live immutable OCCT input; synchronous GUI-thread adapter. Rust owns the
// portable support copy and serialization. `used` accounts support JSON payloads
// across the entire common topology capture, not process-wide allocations.
QJsonObject curveSupportSnapshot(const Handle(Geom2d_Curve)&,std::size_t& used);
QJsonObject copyCurveSupport(const ThreeDm::BasisContract::CurveBasisData&,std::size_t& used);
QJsonObject curveSupportSnapshot(const Handle(Geom_Curve)&,std::size_t& used);
QJsonObject faceSupportSnapshot(const TopoDS_Face&,std::size_t& used);
}
