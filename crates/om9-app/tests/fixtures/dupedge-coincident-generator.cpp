// SPDX-License-Identifier: LGPL-2.1-or-later
// Produces two independent, coincident boxes for the DupEdge ambiguity test.
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Pnt.hxx>
#include <cstdlib>

int main(int argc, char** argv) {
    if (argc < 2 || argc > 3) return 1;
    const double offset = argc == 3 ? std::strtod(argv[2], nullptr) : 0.;
    BRep_Builder builder;
    TopoDS_Compound compound;
    builder.MakeCompound(compound);
    builder.Add(compound, BRepPrimAPI_MakeBox(gp_Pnt(0., offset, 0.), 4., 5., 6.).Shape());
    builder.Add(compound, BRepPrimAPI_MakeBox(4., 5., 6.).Shape());
    return BRepTools::Write(compound, argv[1]) ? 0 : 2;
}
