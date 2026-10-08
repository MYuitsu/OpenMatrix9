#pragma once
#include "ThreeDmGeometry.h"
namespace OpenMatrix9Gui::ThreeDm {
TopoDS_Shape assembleClosedBrepShells(const TopoDS_Shape&,int declaredOrientation,double tolerance);
int classifiedBrepSolidOrientation(const TopoDS_Shape&,double tolerance);
}
