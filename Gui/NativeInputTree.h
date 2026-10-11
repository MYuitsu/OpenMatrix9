// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "ThreeDmGeometry.h"
#include <cstddef>
namespace OpenMatrix9Gui::ThreeDm::InputTree {
struct Summary {std::size_t nodes=0,bytes=0;};
// Borrowed live initialized immutable native owners for one synchronous call.
// No native geometry copy/evaluation/output constructor. Rust owns traversal
// limits and shared numeric basis policy. Extraction snapshots are temporary
// C++ vectors and remain separate from archive/worker/native output allocation.
Summary validate(const ON_Curve&);
Summary validate(const ON_Surface&);
Summary validate(const ON_Brep&);
Summary validate(const ON_NurbsCage&);
Summary validate(const ON_Mesh&);
Summary validate(const ON_PointCloud&,bool allowEmpty=false);
Summary validate(const Handle(Geom_Curve)&);
Summary validate(const Handle(Geom2d_Curve)&);
Summary validate(const Handle(Geom_Surface)&);
Summary validate(const TopoDS_Shape&);
}
