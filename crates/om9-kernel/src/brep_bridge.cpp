// SPDX-License-Identifier: LGPL-2.1-or-later
// A synchronous snapshot operation. All native objects die before returning; the Rust sink copies
// bounded immutable arrays. No OCCT object or topology index becomes a persistent document ID.
#include "brep_snapshot.hxx"

#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <GProp_GProps.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_ListOfShape.hxx>

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
