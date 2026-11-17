// SPDX-License-Identifier: LGPL-2.1-or-later
// M03 OM9-L2-01-11 B-Rep affine transform adapter.
//
// One complete operation: canonical source in, one owned transformed snapshot out. The matrix is
// already admitted in Rust (finite, non-singular). A negative determinant mirrors the body, so the
// orientation is reversed and ShapeFix repairs the shell/solid before the snapshot.
#include "brep_snapshot.hxx"

#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <ShapeFix_Shape.hxx>
#include <gp_GTrsf.hxx>
#include <gp_Trsf.hxx>

#include <cmath>
#include <sstream>
#include <stdexcept>
#include <string>

// Use the same exact OCCT round-trip for the Boolean placement gate and the adapter.
static bool exact_rigid_placement(const double* matrix, gp_Trsf& placement) {
 if (!matrix) return false;
 for (int i = 0; i < 16; ++i) if (!std::isfinite(matrix[i])) return false;
 const double det = matrix[0] * (matrix[5] * matrix[10] - matrix[6] * matrix[9]) -
                    matrix[1] * (matrix[4] * matrix[10] - matrix[6] * matrix[8]) +
                    matrix[2] * (matrix[4] * matrix[9] - matrix[5] * matrix[8]);
 if (!(det > 0.0) || std::abs(det - 1.0) > 1.0e-12) return false;
 for (int i = 0; i < 3; ++i)
  for (int j = 0; j < 3; ++j) {
   double dot = 0.0;
   for (int k = 0; k < 3; ++k) dot += matrix[k * 4 + i] * matrix[k * 4 + j];
   if (std::abs(dot - (i == j ? 1.0 : 0.0)) > 1.0e-12) return false;
  }
 try {
  placement.SetValues(matrix[0], matrix[1], matrix[2], matrix[3],
                      matrix[4], matrix[5], matrix[6], matrix[7],
                      matrix[8], matrix[9], matrix[10], matrix[11]);
  if (placement.ScaleFactor() != 1.0) return false;
  for (int row = 0; row < 3; ++row)
   for (int col = 0; col < 4; ++col)
    if (placement.Value(row + 1, col + 1) != matrix[row * 4 + col]) return false;
  return true;
 } catch (const Standard_Failure&) { return false; }
}

extern "C" int om9_occt_is_exact_rigid_transform(const double* matrix) {
 try { gp_Trsf placement; return exact_rigid_placement(matrix, placement) ? 1 : 0; }
 catch (...) { return 0; }
}

static int transform_dispatch(const char* source, size_t length, const double* matrix,
                                       double deflection, size_t max_points, size_t max_nodes,
                                       size_t max_bytes, Om9BrepSink sink, void* context,
                                       char* message, int capacity, Om9SourceSink source_sink) {
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
  // gp_Trsf may orthogonalize SetValues; use it only if its coefficients
  // round-trip exactly, so this path never changes the supplied placement.
  gp_Trsf placement;
  const bool rigid = exact_rigid_placement(matrix, placement);
  TopoDS_Shape result;
  if (rigid) {
   BRepBuilderAPI_Transform transform(shape, placement, true);
   if (!transform.IsDone()) throw std::runtime_error("transform failed");
   result = transform.Shape();
  } else {
   gp_GTrsf affine;
   for (int row = 0; row < 3; ++row)
    for (int column = 0; column < 4; ++column)
     affine.SetValue(row + 1, column + 1, matrix[row * 4 + column]);
   BRepBuilderAPI_GTransform transform(shape, affine, true);
   if (!transform.IsDone()) throw std::runtime_error("transform failed");
   result = transform.Shape();
  }
  if (det < 0) result.Reverse();
  // Heal the complete result, not just its first solid or shell. A compound may contain
  // several independent bodies (and potentially non-solid children) that all must survive.
  ShapeFix_Shape fixer(result);
  fixer.Perform();
  result = fixer.Shape();
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid())
   throw std::runtime_error("transform produced an invalid shape");
   if (source_sink) return om9_emit_source(result, max_nodes, max_bytes, source_sink, context);
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

extern "C" int om9_occt_transform_brep(const char* source, size_t length, const double* matrix,
    double deflection, size_t points, size_t nodes, size_t bytes, Om9BrepSink sink,
    void* context, char* message, int capacity) {
 return transform_dispatch(source, length, matrix, deflection, points, nodes, bytes, sink,
                           context, message, capacity, nullptr);
}
extern "C" int om9_occt_transform_source(const char* source, size_t length, const double* matrix,
    size_t nodes, size_t bytes, Om9SourceSink sink, void* context, char* message, int capacity) {
 return transform_dispatch(source, length, matrix, 0., 0, nodes, bytes, nullptr,
                           context, message, capacity, sink);
}
