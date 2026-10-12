// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Base/Vector3D.h>
#include <TopoDS_Shape.hxx>
#include <array>
#include <optional>
#include <vector>
namespace OpenMatrix9Gui {
// Native OCCT snapshot adapter. Returns world-space poles for a finite edge or
// one rectangular face; empty means this topology requires an affine edit.
// Curves use increasing pole index, surfaces use u outer / v inner ordering.
std::vector<Base::Vector3d> cageShapeControlPoints(const TopoDS_Shape&);
// Affine edits support arbitrary valid BRep topology. Nonlinear edits preserve
// the extracted NURBS basis/weights of a single edge or rectangular face only.
// Source objects/geometry stay immutable. Failure throws before host mutation.
TopoDS_Shape cageDeformShape(const TopoDS_Shape&,
                            const std::vector<Base::Vector3d>& mappedPoles,
                            std::optional<std::array<double,16>> affine);
}
