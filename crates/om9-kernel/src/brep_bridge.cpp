// SPDX-License-Identifier: LGPL-2.1-or-later
// A synchronous snapshot operation. All native objects die before returning; the Rust sink copies
// bounded immutable arrays. No OCCT object or topology index becomes a persistent document ID.
#include "brep_snapshot.hxx"

#include <BRepGProp.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <GProp_GProps.hxx>
#include <ShapeFix_Shape.hxx>
#include <ShapeUpgrade_ShapeDivideContinuity.hxx>
#include <ShapeUpgrade_ShapeDivideArea.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS_Shell.hxx>

#include <algorithm>
#include <vector>

extern "C" int om9_occt_brep_volume(const char* source, size_t length, double* volume) {
 try {
  if (!source || !length || !volume) return 1;
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid()) return 1;
  GProp_GProps properties;
  // Rational surfaces need adaptive quadrature. The no-Eps overload uses a fixed
  // Gauss rule; its integration error must not be mistaken for a geometry defect.
  if (BRepGProp::VolumePropertiesGK(shape, properties, 1.0e-12, false, true) < 0.0)
   throw std::runtime_error("BRep volume integration failed");
  *volume = properties.Mass();
  return std::isfinite(*volume) ? 0 : 1;
 } catch (...) {
  return 2;
 }
}

extern "C" int om9_occt_brep_snapshot(const char* source, size_t source_len, int kind,
                                      const double* dimensions, double deflection,
                                      size_t max_points, size_t max_nodes, size_t max_bytes,
                                      Om9BrepSink sink, void* context, char* message,
                                      int capacity) {
 try {
  if (!sink || !dimensions || !std::isfinite(deflection) || deflection <= 0)
   throw std::invalid_argument("invalid BRep snapshot input");
  TopoDS_Shape shape;
  if (source_len) {
   if (!source || source_len > max_bytes) throw std::invalid_argument("BRep source budget");
   std::istringstream input(std::string(source, source_len));
   BRep_Builder builder;
   BRepTools::Read(shape, input, builder);
  } else if (kind == 0) {
   for (int i = 0; i < 3; ++i)
    if (!std::isfinite(dimensions[i]) || dimensions[i] <= 0)
     throw std::invalid_argument("invalid box dimensions");
   shape = BRepPrimAPI_MakeBox(dimensions[0], dimensions[1], dimensions[2]).Shape();
  } else if (kind == 1 && std::isfinite(dimensions[0]) && dimensions[0] > 0) {
   shape = BRepPrimAPI_MakeSphere(dimensions[0]).Shape();
  } else {
   throw std::invalid_argument("unsupported BRep primitive");
  }
  return om9_emit_brep_snapshot(shape, deflection, max_points, max_nodes, max_bytes, sink, context,
                                message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown BRep snapshot failure");
  return 2;
 }
}

// Build one exact B-Rep from a closed triangle mesh: one planar face per triangle, sewn into a
// single shell and promoted to a solid. The gem asset owns the mesh; this adapter only makes the
// same authored boundary editable as OCCT topology. An open or non-manifold mesh is refused rather
// than repaired into a fake solid.
extern "C" int om9_occt_brep_from_triangles(
    const double* points, size_t point_count, const uint32_t* triangles, size_t triangle_count,
    double tolerance, double deflection, size_t max_points, size_t max_nodes, size_t max_bytes,
    Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!sink || !points || !triangles || point_count < 4 || triangle_count < 4 ||
      !std::isfinite(tolerance) || tolerance <= 0 || !std::isfinite(deflection) || deflection <= 0)
   throw std::invalid_argument("invalid mesh BRep input");
  if (point_count > max_points / 3 || triangle_count > max_nodes)
   throw std::length_error("mesh BRep budget");
  for (size_t i = 0; i < point_count; ++i)
   if (!std::isfinite(points[i * 3]) || !std::isfinite(points[i * 3 + 1]) ||
       !std::isfinite(points[i * 3 + 2]))
    throw std::invalid_argument("mesh BRep point is not finite");
  for (size_t i = 0; i < triangle_count * 3; ++i)
   if (triangles[i] >= point_count) throw std::invalid_argument("mesh BRep corner out of range");

  BRepBuilderAPI_Sewing sewing(std::min(tolerance, 1.0e-5));
  for (size_t i = 0; i < triangle_count; ++i) {
   const uint32_t a = triangles[i * 3];
   const uint32_t b = triangles[i * 3 + 1];
   const uint32_t c = triangles[i * 3 + 2];
   if (a == b || b == c || a == c) throw std::invalid_argument("mesh BRep degenerate triangle");
   BRepBuilderAPI_MakePolygon polygon;
   polygon.Add(gp_Pnt(points[a * 3], points[a * 3 + 1], points[a * 3 + 2]));
   polygon.Add(gp_Pnt(points[b * 3], points[b * 3 + 1], points[b * 3 + 2]));
   polygon.Add(gp_Pnt(points[c * 3], points[c * 3 + 1], points[c * 3 + 2]));
   polygon.Close();
   if (!polygon.IsDone()) throw std::runtime_error("mesh BRep triangle wire failed");
   BRepBuilderAPI_MakeFace face(polygon.Wire(), Standard_True);
   if (!face.IsDone()) throw std::runtime_error("mesh BRep planar face failed");
   sewing.Add(face.Face());
  }
  sewing.Perform();
  TopoDS_Shape sewed = sewing.SewedShape();
  if (sewed.IsNull()) throw std::runtime_error("mesh BRep sewing produced no shape");

  // Exactly one closed shell: every non-degenerate edge must be shared by two faces.
  TopoDS_Shell shell;
  int shells = 0;
  for (TopExp_Explorer explorer(sewed, TopAbs_SHELL); explorer.More(); explorer.Next()) {
   shell = TopoDS::Shell(explorer.Current());
   ++shells;
  }
  if (shells == 0) throw std::runtime_error("mesh BRep has no shell");
  if (shells != 1) throw std::invalid_argument("mesh BRep has more than one shell");
  TopTools_IndexedDataMapOfShapeListOfShape edges;
  TopExp::MapShapesAndAncestors(shell, TopAbs_EDGE, TopAbs_FACE, edges);
  for (int i = 1; i <= edges.Extent(); ++i) {
   if (edges.FindFromIndex(i).Extent() != 2)
    throw std::invalid_argument("mesh BRep is not closed");
  }

  BRepBuilderAPI_MakeSolid solid(shell);
  if (!solid.IsDone()) throw std::runtime_error("mesh BRep solid failed");
  TopoDS_Shape shape = solid.Solid();
  if (!BRepCheck_Analyzer(shape).IsValid()) {
   ShapeFix_Shape fix(shape);
   fix.SetPrecision(std::min(tolerance, 1.0e-7));
   fix.SetMaxTolerance(std::min(tolerance, 1.0e-5));
   fix.Perform();
   shape = fix.Shape();
  }
  if (!BRepCheck_Analyzer(shape).IsValid())
   throw std::runtime_error("mesh BRep topology is invalid");
  GProp_GProps properties;
  BRepGProp::VolumeProperties(shape, properties);
  if (!std::isfinite(properties.Mass()) || properties.Mass() == 0)
   throw std::invalid_argument("mesh BRep has no volume");
  if (properties.Mass() < 0) shape.Reverse();
  return om9_emit_brep_snapshot(shape, deflection, max_points, max_nodes, max_bytes, sink, context,
                                message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0)
   std::snprintf(message, capacity, "Unknown mesh BRep conversion failure");
  return 2;
 }
}

// Generic measured report used by the creation-op acceptance oracle: exact volume/area, topology
// counts, closedness (every edge has two face ancestors) and the optimal bounding box. Nothing is
// derived from the display mesh.
extern "C" int om9_occt_brep_measure(const char* source, size_t length, double* volume,
                                     double* area, long long* faces, long long* edges,
                                     long long* vertices, long long* solids, long long* shells,
                                     long long* naked_edges, int* closed, double* bounds,
                                     char* message, int capacity) {
 try {
  if (!source || !length || !volume || !area || !faces || !edges || !vertices || !solids ||
      !shells || !naked_edges || !closed || !bounds)
   throw std::invalid_argument("invalid BRep measure output");
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
   throw std::invalid_argument("invalid measured BRep");
  TopTools_IndexedMapOfShape face_map, edge_map, vertex_map, solid_map, shell_map;
  TopExp::MapShapes(shape, TopAbs_FACE, face_map);
  TopExp::MapShapes(shape, TopAbs_EDGE, edge_map);
  TopExp::MapShapes(shape, TopAbs_VERTEX, vertex_map);
  TopExp::MapShapes(shape, TopAbs_SOLID, solid_map);
  TopExp::MapShapes(shape, TopAbs_SHELL, shell_map);
  *faces = face_map.Extent();
  *edges = edge_map.Extent();
  *vertices = vertex_map.Extent();
  *solids = solid_map.Extent();
  *shells = shell_map.Extent();
  GProp_GProps surface;
  BRepGProp::SurfaceProperties(shape, surface, 1.0e-12);
  *area = std::fabs(surface.Mass());
  // Adaptive surface integration can miss knot boundaries or underresolve a
  // smooth extrusion. Subdivide temporary integration shapes; do not replace
  // the returned B-Rep or alter its topology.
  for (int i = 1; i <= face_map.Extent(); ++i) {
   const TopoDS_Face face = TopoDS::Face(face_map(i));
   BRepAdaptor_Surface adaptor(face);
   if (adaptor.GetType() != GeomAbs_SurfaceOfExtrusion) continue;
   GProp_GProps before, after;
   BRepGProp::SurfaceProperties(face, before, 1.0e-12);
   TopoDS_Shape integration_shape = face;
   if (adaptor.UContinuity() == GeomAbs_C0 || adaptor.UContinuity() == GeomAbs_C1) {
    ShapeUpgrade_ShapeDivideContinuity split(face);
    split.SetSurfaceCriterion(GeomAbs_C2);
    if (!split.Perform() || split.Result().IsNull() || !BRepCheck_Analyzer(split.Result()).IsValid())
     throw std::runtime_error("surface integration continuity split failed");
    integration_shape = split.Result();
   }
   ShapeUpgrade_ShapeDivideArea divide(integration_shape);
   divide.SetSplittingByNumber(true);
   divide.SetNumbersUVSplits(4, 1);
   TopoDS_Shape pieces;
   if (divide.Perform() && !divide.Result().IsNull() && BRepCheck_Analyzer(divide.Result()).IsValid())
    pieces = divide.Result();
   else {
    ShapeUpgrade_ShapeDivideArea retry(integration_shape);
    retry.SetSplittingByNumber(true);
    retry.SetNumbersUVSplits(2, 1);
    if (!retry.Perform() || retry.Result().IsNull() || !BRepCheck_Analyzer(retry.Result()).IsValid())
     throw std::runtime_error("surface integration subdivision failed");
    pieces = retry.Result();
   }
   BRepGProp::SurfaceProperties(pieces, after, 1.0e-12);
   *area += std::fabs(after.Mass()) - std::fabs(before.Mass());
  }
  if (solid_map.Extent() > 0) {
   GProp_GProps solid;
   if (BRepGProp::VolumePropertiesGK(shape, solid, 1.0e-12, false, true) < 0.0)
    throw std::runtime_error("BRep volume integration failed");
   *volume = std::fabs(solid.Mass());
  } else {
   *volume = 0.0;
  }
  // Closedness counts face ancestry per edge occurrence: a seam edge belongs twice to its face,
  // so a cylinder or a sphere is closed while a real free boundary keeps the shape open.
  TopTools_IndexedDataMapOfShapeListOfShape edge_faces;
  TopExp::MapShapesAndAncestors(shape, TopAbs_EDGE, TopAbs_FACE, edge_faces);
  int open_edges = 0;
  for (int i = 1; i <= edge_faces.Extent(); ++i) {
   const TopoDS_Edge& edge = TopoDS::Edge(edge_faces.FindKey(i));
   if (BRep_Tool::Degenerated(edge)) continue;
   int ancestors = 0;
   for (TopTools_ListOfShape::Iterator it(edge_faces.FindFromIndex(i)); it.More(); it.Next()) {
    const TopoDS_Face& face = TopoDS::Face(it.Value());
    ancestors += BRep_Tool::IsClosed(edge, face) ? 2 : 1;
   }
   if (ancestors < 2) ++open_edges;
  }
  *naked_edges = open_edges;
  *closed = open_edges == 0 ? 1 : 0;
  Bnd_Box box;
  BRepBndLib::AddOptimal(shape, box, false, false);
  if (box.IsVoid()) throw std::invalid_argument("empty BRep bounds");
  box.Get(bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], bounds[5]);
  if (!std::isfinite(*volume) || !std::isfinite(*area)) return 1;
  for (int i = 0; i < 6; ++i)
   if (!std::isfinite(bounds[i])) return 1;
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown BRep measure failure");
  return 2;
 }
}
