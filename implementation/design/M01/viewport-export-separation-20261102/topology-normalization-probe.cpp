#include <BRep_Builder.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepTools.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
#include <set>
#include <sstream>
#include <string>

static std::string save(const TopoDS_Shape& shape) {
 std::ostringstream stream;
 BRepTools::Write(shape, stream, false, false, TopTools_FormatVersion_CURRENT);
 return stream.str();
}
static std::string normalized_subshape(const TopoDS_Shape& shape) {
 auto text = save(shape);
 std::istringstream stream(text); TopoDS_Shape value; BRep_Builder builder;
 BRepTools::Read(value, stream, builder);
 if (value.IsNull()) throw std::runtime_error("invalid subshape roundtrip");
 return save(value);
}

static void probe(const char* name, const TopoDS_Shape& original) {
 auto canonical = save(original);
 std::istringstream stream(canonical);
 TopoDS_Shape decoded; BRep_Builder builder;
 BRepTools::Read(decoded, stream, builder);
 if (decoded.IsNull() || !BRepCheck_Analyzer(decoded).IsValid()) throw std::runtime_error("invalid source roundtrip");
 auto normalized = save(decoded);
 std::istringstream again_stream(normalized);
 TopoDS_Shape again; BRep_Builder again_builder;
 BRepTools::Read(again, again_stream, again_builder);
 if (again.IsNull() || !BRepCheck_Analyzer(again).IsValid()) throw std::runtime_error("invalid second roundtrip");
 std::cout << name << " canonical_equal=" << (canonical == normalized) << " normalized_equal=" << (normalized == save(again));
 for (auto type : {TopAbs_FACE, TopAbs_EDGE, TopAbs_VERTEX}) {
  TopTools_IndexedMapOfShape first, second, third;
  TopExp::MapShapes(original, type, first);
  TopExp::MapShapes(decoded, type, second);
  TopExp::MapShapes(again, type, third);
  bool ordered_equal = first.Extent() == second.Extent();
  bool normalized_ordered = second.Extent() == third.Extent();
  bool subshape_ordered = first.Extent() == second.Extent();
  std::set<std::string> fingerprints, normalized_fingerprints;
  for (int i = 1; i <= first.Extent(); ++i) {
   auto data = save(first(i));
   fingerprints.insert(data);
   if (i > second.Extent() || data != save(second(i))) ordered_equal = false;
   auto normalized_data = normalized_subshape(first(i));
   normalized_fingerprints.insert(normalized_data);
   if (i > second.Extent() || normalized_data != normalized_subshape(second(i))) subshape_ordered = false;
  }
  for (int i = 1; i <= second.Extent(); ++i) {
   if (i > third.Extent() || save(second(i)) != save(third(i))) normalized_ordered = false;
  }
  std::cout << " " << (type == TopAbs_FACE ? "faces" : type == TopAbs_EDGE ? "edges" : "vertices")
            << "=" << first.Extent() << ",unique=" << fingerprints.size() << ",ordered=" << ordered_equal << ",normalized_ordered=" << normalized_ordered << ",subshape_unique=" << normalized_fingerprints.size() << ",subshape_ordered=" << subshape_ordered;
 }
 std::cout << "\n";
}
int main() {
 try {
  probe("box", BRepPrimAPI_MakeBox(10, 20, 30).Shape());
  probe("sphere", BRepPrimAPI_MakeSphere(5).Shape());
  BRep_Builder builder; TopoDS_Compound duplicate;
  builder.MakeCompound(duplicate);
  builder.Add(duplicate, BRepPrimAPI_MakeBox(10, 20, 30).Shape());
  builder.Add(duplicate, BRepPrimAPI_MakeBox(10, 20, 30).Shape());
  probe("coincident_boxes", duplicate);
 } catch (const std::exception& e) { std::cerr << e.what() << "\n"; return 1; }
}
