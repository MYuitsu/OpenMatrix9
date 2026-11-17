// SPDX-License-Identifier: LGPL-2.1-or-later
// Independent measurement fixture: read the persisted source edge intervals.
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Edge.hxx>
#include <Geom_Curve.hxx>
#include <gp_Pnt.hxx>
#include <cmath>
#include <iomanip>
#include <iostream>

int main() {
    std::cerr << "native probe started\n";
    TopoDS_Shape shape;
    BRep_Builder builder;
    if (!BRepTools::Read(shape, "L:/om9-extractwire-domain-source.brep", builder) || shape.IsNull()) return 1;
    TopTools_IndexedMapOfShape edges;
    TopExp::MapShapes(shape, TopAbs_EDGE, edges);
    int found = 0;
    for (int i = 1; i <= edges.Extent(); ++i) {
        auto edge = TopoDS::Edge(edges(i));
        double first, last;
        TopLoc_Location location;
        auto curve = BRep_Tool::Curve(edge, location, first, last);
        if (curve.IsNull()) continue;
        auto p = curve->Value((first + last) / 2.).Transformed(location.Transformation());
        if (std::abs(std::hypot(p.X(),p.Y())-2.) < 1e-8 && p.X()>0.1 && p.Y()>0.1) {
            ++found;
            std::cout << std::setprecision(17) << "arc_z=" << p.Z() << " source_domain=[" << first << "," << last << "]\n";
        }
    }
    return found == 2 ? 0 : 2;
}
