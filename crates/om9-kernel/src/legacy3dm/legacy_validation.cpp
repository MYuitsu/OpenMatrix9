// SPDX-License-Identifier: LGPL-2.1-or-later
// Minimal standalone validation hooks for the ported ThreeDm conversion code.
// The full host build delegates these to FEBE/Rust-owned numeric policy; the
// standalone kernel only needs explicit validity/bounds checks here.
#include "ThreeDmGeometry.h"
#include "NativeInputTree.h"
#include "NurbsBasisAdapter.h"

#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <TopoDS_Shape.hxx>

namespace OpenMatrix9Gui::ThreeDm::InputTree {
Summary validate(const ON_Curve& value) {
  if (!value.IsValid()) throw ExchangeError("invalid native curve");
  return {};
}
Summary validate(const ON_Surface& value) {
  if (!value.IsValid()) throw ExchangeError("invalid native surface");
  return {};
}
Summary validate(const ON_Brep& value) {
  if (!value.IsValid()) throw ExchangeError("invalid native BRep");
  return {};
}
Summary validate(const ON_NurbsCage& value) {
  if (!value.IsValid()) throw ExchangeError("invalid native NURBS cage");
  return {};
}
Summary validate(const ON_Mesh& value) {
  if (!value.IsValid()) throw ExchangeError("invalid native mesh");
  return {};
}
Summary validate(const ON_PointCloud& value, bool allowEmpty) {
  if (!value.IsValid() || (!allowEmpty && value.PointCount() == 0))
    throw ExchangeError("invalid native point cloud");
  return {};
}
Summary validate(const Handle(Geom_Curve)& value) {
  if (value.IsNull()) throw ExchangeError("null OCCT curve");
  return {};
}
Summary validate(const Handle(Geom2d_Curve)& value) {
  if (value.IsNull()) throw ExchangeError("null OCCT 2D curve");
  return {};
}
Summary validate(const Handle(Geom_Surface)& value) {
  if (value.IsNull()) throw ExchangeError("null OCCT surface");
  return {};
}
Summary validate(const TopoDS_Shape& value) {
  if (value.IsNull()) throw ExchangeError("null OCCT shape");
  return {};
}
}  // namespace OpenMatrix9Gui::ThreeDm::InputTree

namespace OpenMatrix9Gui::ThreeDm::BasisContract {
static void finite_curve(const ON_NurbsCurve& value) {
  if (!value.IsValid() || value.Degree() < 1 || value.CVCount() <= value.Degree())
    throw ExchangeError("invalid native NURBS curve basis");
}
static void finite_surface(const ON_NurbsSurface& value) {
  if (!value.IsValid() || value.Degree(0) < 1 || value.Degree(1) < 1 ||
      value.CVCount(0) <= value.Degree(0) || value.CVCount(1) <= value.Degree(1))
    throw ExchangeError("invalid native NURBS surface basis");
}
CurveBasisData checkedBasis(const ON_NurbsCurve& value) {
  finite_curve(value);
  return {};
}
SurfaceBasisData checkedBasis(const ON_NurbsSurface& value) {
  finite_surface(value);
  return {};
}
CurveBasisData checkedBasis(const Handle(Geom_BSplineCurve)& value) {
  if (value.IsNull() || value->Degree() < 1 || value->NbPoles() <= value->Degree())
    throw ExchangeError("invalid OCCT NURBS curve basis");
  return {};
}
CurveBasisData checkedBasis(const Handle(Geom2d_BSplineCurve)& value) {
  if (value.IsNull() || value->Degree() < 1 || value->NbPoles() <= value->Degree())
    throw ExchangeError("invalid OCCT 2D NURBS curve basis");
  return {};
}
SurfaceBasisData checkedBasis(const Handle(Geom_BSplineSurface)& value) {
  if (value.IsNull() || value->UDegree() < 1 || value->VDegree() < 1 ||
      value->NbUPoles() <= value->UDegree() || value->NbVPoles() <= value->VDegree())
    throw ExchangeError("invalid OCCT NURBS surface basis");
  return {};
}
CurveBasisData checkedBasis(const Handle(Geom_BezierCurve)& value) {
  if (value.IsNull() || value->Degree() < 1) throw ExchangeError("invalid OCCT Bezier curve");
  return {};
}
CurveBasisData checkedBasis(const Handle(Geom2d_BezierCurve)& value) {
  if (value.IsNull() || value->Degree() < 1) throw ExchangeError("invalid OCCT 2D Bezier curve");
  return {};
}
SurfaceBasisData checkedBasis(const Handle(Geom_BezierSurface)& value) {
  if (value.IsNull() || value->UDegree() < 1 || value->VDegree() < 1)
    throw ExchangeError("invalid OCCT Bezier surface");
  return {};
}
CurveBasisData checkedBasis(const ON_PolylineCurve& value) {
  if (!value.IsValid() || value.PointCount() < 2)
    throw ExchangeError("invalid native polyline curve");
  return {};
}
}  // namespace OpenMatrix9Gui::ThreeDm::BasisContract
