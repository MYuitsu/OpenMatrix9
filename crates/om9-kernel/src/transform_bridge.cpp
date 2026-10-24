// SPDX-License-Identifier: LGPL-2.1-or-later
// M03 OM9-L2-01-11 B-Rep affine transform adapter.
//
// One complete operation: canonical source in, one owned transformed snapshot out. The matrix is
// already admitted in Rust (finite, non-singular). A negative determinant mirrors the body, so the
// orientation is reversed and ShapeFix repairs the shell/solid before the snapshot.
#include "brep_snapshot.hxx"

#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <ShapeFix_Shell.hxx>
#include <ShapeFix_Solid.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <gp_GTrsf.hxx>

#include <cmath>
#include <sstream>
#include <stdexcept>
#include <string>

extern "C" int om9_occt_transform_brep(const char* source, size_t length, const double* matrix,
                                       double deflection, size_t max_points, size_t max_nodes,
                                       size_t max_bytes, Om9BrepSink sink, void* context,
                                       char* message, int capacity) {
 try {
  if (!source || length == 0) throw std::invalid_argument("transform: empty source");
  if (!matrix) throw std::invalid_argument("transform: missing matrix");
  for (int i = 0; i < 16; ++i)
   if (!std::isfinite(matrix[i])) throw std::invalid_argument("transform: non-finite matrix");
  const double det = matrix[0] * (matrix[5] * matrix[10] - matrix[6] * matrix[9]) -
                     matrix[1] * (matrix[4] * matrix[10] - matrix[6] * matrix[8]) +
                     matrix[2] * (matrix[4] * matrix[9] - matrix[5] * matrix[8]);
  if (!std::isfinite(det) || std::abs(det) <= 1.0e-15)
   throw std::invalid_argument("transform: singular matrix");
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
   throw std::invalid_argument("transform: invalid B-Rep source");
  gp_GTrsf affine;
  for (int row = 0; row < 3; ++row)
   for (int column = 0; column < 4; ++column)
    affine.SetValue(row + 1, column + 1, matrix[row * 4 + column]);
  BRepBuilderAPI_GTransform transform(shape, affine, true);
  if (!transform.IsDone()) throw std::runtime_error("transform failed");
  TopoDS_Shape result = transform.Shape();
  if (det < 0) result.Reverse();
  TopTools_IndexedMapOfShape solids;
  TopExp::MapShapes(result, TopAbs_SOLID, solids);
  if (solids.Extent() > 0) {
   ShapeFix_Solid fixer(TopoDS::Solid(solids(1)));
   fixer.Perform();
   result = fixer.Solid();
  } else {
   TopTools_IndexedMapOfShape shells;
   TopExp::MapShapes(result, TopAbs_SHELL, shells);
   if (shells.Extent() > 0) {
    ShapeFix_Shell fixer(TopoDS::Shell(shells(1)));
    fixer.Perform();
    result = fixer.Shell();
   }
  }
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid())
   throw std::runtime_error("transform produced an invalid shape");
  return om9_emit_brep_snapshot(result, deflection, max_points, max_nodes, max_bytes, sink,
                                context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown transform failure");
  return 2;
 }
}
