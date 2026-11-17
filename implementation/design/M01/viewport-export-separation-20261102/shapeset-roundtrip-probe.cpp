// SPDX-License-Identifier: LGPL-2.1-or-later
// Local OCCT 8 compatibility probe. Not part of the OM9 runtime.
#include <BRepTools.hxx>
#include <BRepTools_ShapeSet.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRep_Builder.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS_Compound.hxx>
#include <algorithm>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

static std::string source(const TopoDS_Shape& s) {
 std::ostringstream out;
 BRepTools::Write(s, out, false, false, TopTools_FormatVersion_CURRENT);
 return out.str();
}
static std::string normalized(const TopoDS_Shape& shape) {
 std::istringstream in(source(shape)); TopoDS_Shape restored; BRep_Builder builder;
 BRepTools::Read(restored, in, builder);
 if (restored.IsNull()) throw std::runtime_error("invalid subshape");
 return source(restored);
}
static void probe(const char* label, const TopoDS_Shape& root) {
 const TopAbs_ShapeEnum kinds[] = {TopAbs_FACE, TopAbs_EDGE, TopAbs_VERTEX};
 TopTools_IndexedMapOfShape old[3], restored[3];
 for (int k=0;k<3;k++) TopExp::MapShapes(root, kinds[k], old[k]);
 BRepTools_ShapeSet write(false, false);
 write.Add(root);
 std::ostringstream data;
 write.Write(data);
 write.Write(root, data); data << '\n';
 // Reverse references to exercise explicit association, not iteration order.
 for (int k=0;k<3;k++) for (int i=old[k].Extent();i>=1;i--) {
  write.Write(old[k](i), data); data << '\n';
 }
 std::istringstream in(data.str());
 BRepTools_ShapeSet read(false, false);
 read.Read(in);
 TopoDS_Shape restored_root;
 read.Read(restored_root, in);
 if (restored_root.IsNull() || !BRepCheck_Analyzer(restored_root).IsValid())
  throw std::runtime_error("invalid restored root");
 for (int k=0;k<3;k++) {
  TopExp::MapShapes(restored_root, kinds[k], restored[k]);
  if (old[k].Extent()!=restored[k].Extent()) throw std::runtime_error("count differs");
  std::vector<bool> seen(restored[k].Extent(), false);
  for (int i=old[k].Extent();i>=1;i--) {
   TopoDS_Shape ref;
   read.Read(ref,in);
   int found=restored[k].FindIndex(ref);
   if (!found || seen[found-1] || ref.ShapeType()!=kinds[k])
    throw std::runtime_error("unmapped or duplicate topology reference");
   seen[found-1]=true;
   if (ref.Orientation()!=old[k](i).Orientation() || normalized(ref)!=normalized(old[k](i))) {
    std::cerr << label << " kind=" << k << " ref=" << i << " orientation="
              << (ref.Orientation()==old[k](i).Orientation()) << " normalized="
              << (normalized(ref)==normalized(old[k](i))) << "\n";
    throw std::runtime_error("orientation or normalized geometry differs");
   }
  }
 }
 std::cout << label << " faces=" << old[0].Extent() << " edges=" << old[1].Extent()
           << " vertices=" << old[2].Extent() << " bytes=" << data.str().size() << "\n";
}
int main() {
 try {
  probe("box",BRepPrimAPI_MakeBox(4.,5.,6.).Shape());
  probe("sphere",BRepPrimAPI_MakeSphere(3.).Shape());
  BRep_Builder builder; TopoDS_Compound compound; builder.MakeCompound(compound);
  builder.Add(compound,BRepPrimAPI_MakeBox(4.,5.,6.).Shape());
  builder.Add(compound,BRepPrimAPI_MakeBox(4.,5.,6.).Shape());
  probe("coincident-boxes",compound);
  return 0;
 } catch (const std::exception& error) { std::cerr << error.what() << "\n"; return 1; }
}
