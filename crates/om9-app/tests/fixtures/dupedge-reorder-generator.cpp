// SPDX-License-Identifier: LGPL-2.1-or-later
// Two compounds of the same Box faces, written in opposite orders.
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Compound.hxx>
#include <vector>

int main(int argc, char** argv) {
    if (argc != 3) return 1;
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(4., 5., 6.).Shape();
    std::vector<TopoDS_Shape> faces;
    for (TopExp_Explorer it(box, TopAbs_FACE); it.More(); it.Next()) faces.push_back(it.Current());
    if (faces.size() != 6) return 2;
    BRep_Builder builder;
    TopoDS_Compound forward, reverse;
    builder.MakeCompound(forward); builder.MakeCompound(reverse);
    for (const auto& face : faces) builder.Add(forward, face);
    for (auto it = faces.rbegin(); it != faces.rend(); ++it) builder.Add(reverse, *it);
    return BRepTools::Write(forward, argv[1]) && BRepTools::Write(reverse, argv[2]) ? 0 : 3;
}
