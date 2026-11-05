// SPDX-License-Identifier: LGPL-2.1-or-later
// Shared B-Rep snapshot emission (M03 OM9-L2-01-09 creation ops reuse the M01 L2-01-05 sink).
//
// A snapshot is one synchronous, bounded copy: every OCCT object dies before the call returns,
// the Rust sink copies plain arrays, and the canonical OCCT text stays the only topology source.
// No native handle or topology index crosses into safe Rust.
#pragma once

#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <GCPnts_QuasiUniformDeflection.hxx>
#include <Poly_Triangulation.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Standard_Failure.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <limits>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

struct Om9BrepSnapshot {
 const char* source; size_t source_len;
 const double* points; size_t points_len;
 const uint32_t* triangles; size_t triangles_len;
 const uint32_t* segments; size_t segments_len;
 const uint32_t* vertices; size_t vertices_len;
 const uint32_t* face_edges; size_t face_edges_len;
 size_t faces_count; size_t edges_count;
};
using Om9BrepSink = int (*)(void*, const Om9BrepSnapshot*);
struct Om9EdgeNodeBinding { uint32_t point; uint32_t edge; uint32_t vertex; uint32_t reserved; double parameter; };
using Om9FaceShellSink = int (*)(void*, const uint32_t*, size_t, const Om9EdgeNodeBinding*, size_t, double);

// Writes OCCT text, tessellation, edge polylines and the face/edge/vertex maps in one bounded
// pass. Throws std::invalid_argument / std::length_error / std::runtime_error on refusal.
inline int om9_emit_brep_snapshot(const TopoDS_Shape& shape,
                                  double deflection,
                                  size_t max_points,
                                  size_t max_nodes,
                                  size_t max_bytes,
                                  Om9BrepSink sink,
                                  void* context,
                                  char* message,
                                  int capacity,
                                  double angular_deflection = 0.3,
                                  Om9FaceShellSink shell_sink = nullptr,
                                  void* shell_context = nullptr,
                                  bool include_edge_bindings = false,
                                  bool optimize_export_sphere = false) {
 try {
  if (!sink || !std::isfinite(deflection) || deflection <= 0 ||
      !std::isfinite(angular_deflection) || angular_deflection <= 0 || angular_deflection > std::acos(-1.0))
   throw std::invalid_argument("invalid BRep snapshot input");
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
   throw std::invalid_argument("invalid BRep source shape");
  std::ostringstream archive;
  BRepTools::Write(shape, archive, false, false, TopTools_FormatVersion_CURRENT);
  std::string text = archive.str();
  if (text.size() > max_bytes) throw std::length_error("BRep source budget");
  TopTools_IndexedMapOfShape faces, edges, verts;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  TopExp::MapShapes(shape, TopAbs_EDGE, edges);
  TopExp::MapShapes(shape, TopAbs_VERTEX, verts);
  if (size_t(faces.Extent() + edges.Extent() + verts.Extent()) > max_nodes)
   throw std::length_error("BRep topology budget");
  if (faces.Extent() == 0) throw std::invalid_argument("BRep has no face");
  double max_source_tolerance = 0.0;
  if (shell_sink) {
   const auto observe_tolerance = [&](double tolerance) {
    if (!std::isfinite(tolerance) || tolerance < 0.0) throw std::invalid_argument("invalid BRep source tolerance");
    max_source_tolerance = std::max(max_source_tolerance, tolerance);
   };
   for (int i = 1; i <= faces.Extent(); ++i) observe_tolerance(BRep_Tool::Tolerance(TopoDS::Face(faces(i))));
   for (int i = 1; i <= edges.Extent(); ++i) observe_tolerance(BRep_Tool::Tolerance(TopoDS::Edge(edges(i))));
   for (int i = 1; i <= verts.Extent(); ++i) observe_tolerance(BRep_Tool::Tolerance(TopoDS::Vertex(verts(i))));
  }
  size_t group_bytes = 0;
  std::vector<uint32_t> owners;
  std::vector<Om9EdgeNodeBinding> node_edges;
  if (shell_sink) {
   TopTools_IndexedMapOfShape shells;
   TopExp::MapShapes(shape, TopAbs_SHELL, shells);
   if (size_t(shells.Extent()) > max_nodes || size_t(faces.Extent()) > max_bytes / sizeof(uint32_t))
    throw std::length_error("BRep shell group budget");
   group_bytes = size_t(faces.Extent()) * sizeof(uint32_t);
   if (group_bytes > max_bytes - text.size()) throw std::length_error("BRep shell group budget");
   owners.resize(size_t(faces.Extent()), 0);
   for (int s = 1; s <= shells.Extent(); ++s) {
    TopTools_IndexedMapOfShape shell_faces;
    TopExp::MapShapes(shells(s), TopAbs_FACE, shell_faces);
    for (int i = 1; i <= shell_faces.Extent(); ++i) {
     const int f = faces.FindIndex(shell_faces(i));
     if (!f) throw std::runtime_error("unmapped shell face");
     auto& owner = owners[size_t(f - 1)];
     if (owner != 0 && owner != uint32_t(s)) throw std::runtime_error("face belongs to multiple shells");
     owner = uint32_t(s);
    }
   }
  }
  // Restricted to a valid single-face spherical solid on the export path.
  // Keep the identical deflection and angle; other geometry uses the existing mesher.
  const bool sphere = optimize_export_sphere && shape.ShapeType() == TopAbs_SOLID && faces.Extent() == 1
      && BRepAdaptor_Surface(TopoDS::Face(faces(1))).GetType() == GeomAbs_Sphere;
  std::optional<BRepMesh_IncrementalMesh> mesh;
  if (sphere) {
   IMeshTools_Parameters parameters;
   parameters.Deflection = deflection;
   parameters.Angle = angular_deflection;
   parameters.Relative = false;
   parameters.InParallel = false;
   parameters.MeshAlgo = IMeshTools_MeshAlgoType_Delabella;
   mesh.emplace(shape, parameters);
  } else {
   mesh.emplace(shape, deflection, false, angular_deflection, false);
  }
  if (!mesh->IsDone()) throw std::runtime_error("BRep tessellation failed");
  std::vector<std::array<double, 3>> points;
  std::vector<std::array<uint32_t, 4>> triangles;
  std::vector<std::array<uint32_t, 3>> segments;
  std::vector<uint32_t> vertices;
  std::vector<std::array<uint32_t, 2>> face_edges;
  size_t used = text.size();
  auto reserve = [&](size_t bytes) {
   if (bytes > max_bytes - used) throw std::length_error("BRep snapshot bytes");
   used += bytes;
  };
  reserve(group_bytes);
  auto point = [&](const gp_Pnt& p) -> uint32_t {
   if (points.size() >= max_points || points.size() >= std::numeric_limits<uint32_t>::max())
    throw std::length_error("BRep snapshot points");
   reserve(sizeof(std::array<double, 3>));
   points.push_back({p.X(), p.Y(), p.Z()});
   return uint32_t(points.size() - 1);
  };
  for (int f = 1; f <= faces.Extent(); ++f) {
   const auto face = TopoDS::Face(faces(f));
   TopLoc_Location location;
   auto poly = BRep_Tool::Triangulation(face, location);
   if (poly.IsNull()) throw std::runtime_error("BRep face has no tessellation");
   uint32_t base = uint32_t(points.size());
   for (int i = 1; i <= poly->NbNodes(); ++i)
    point(poly->Node(i).Transformed(location.Transformation()));
   for (int i = 1; i <= poly->NbTriangles(); ++i) {
    int a, b, c;
    poly->Triangle(i).Get(a, b, c);
    if (face.Orientation() == TopAbs_REVERSED) std::swap(b, c);
    reserve(sizeof(std::array<uint32_t, 4>));
    triangles.push_back({base + uint32_t(a - 1), base + uint32_t(b - 1), base + uint32_t(c - 1),
                         uint32_t(f - 1)});
   }
   for (TopExp_Explorer ex(face, TopAbs_EDGE); ex.More(); ex.Next()) {
    int e = edges.FindIndex(ex.Current());
    if (!e) throw std::runtime_error("unmapped BRep edge");
    reserve(sizeof(std::array<uint32_t, 2>));
    face_edges.push_back({uint32_t(f - 1), uint32_t(e - 1)});
    if (shell_sink && include_edge_bindings) {
     const auto edge = TopoDS::Edge(ex.Current());
     const auto polygon = BRep_Tool::PolygonOnTriangulation(edge, poly, location);
     if (!polygon.IsNull()) {
      TopoDS_Vertex first, last;
      TopExp::Vertices(edge, first, last);
      for (int i = 1; i <= polygon->NbNodes(); ++i) {
       const int node = polygon->Node(i);
       if (node < 1 || node > poly->NbNodes()) throw std::runtime_error("invalid BRep edge mesh node");
       const double parameter = polygon->HasParameters() ? polygon->Parameter(i) : std::numeric_limits<double>::quiet_NaN();
       if (std::isinf(parameter)) throw std::runtime_error("invalid BRep edge mesh parameter");
       uint32_t vertex = std::numeric_limits<uint32_t>::max();
       for (const auto& endpoint : {first, last}) {
        if (endpoint.IsNull() || !std::isfinite(parameter)) continue;
        double endpoint_parameter = 0.;
        if (!BRep_Tool::Parameter(endpoint, edge, endpoint_parameter) || endpoint_parameter != parameter) continue;
        const int index = verts.FindIndex(endpoint);
        if (!index) throw std::runtime_error("unmapped BRep edge endpoint");
        const auto mesh_point = poly->Node(node).Transformed(location.Transformation());
        if (mesh_point.Distance(BRep_Tool::Pnt(endpoint)) > BRep_Tool::Tolerance(endpoint)) continue;
        if (vertex != std::numeric_limits<uint32_t>::max() && vertex != uint32_t(index - 1))
         throw std::runtime_error("ambiguous BRep edge endpoint");
        vertex = uint32_t(index - 1);
       }
       reserve(sizeof(Om9EdgeNodeBinding));
       node_edges.push_back({base + uint32_t(node - 1), uint32_t(e - 1), vertex, 0, parameter});
      }
     }
    }
   }
  }
  if (shell_sink && shell_sink(shell_context, owners.data(), owners.size(),
                               node_edges.data(), node_edges.size(), max_source_tolerance) != 0)
   throw std::runtime_error("Rust refused BRep shell groups and edge bindings");
  // Export snapshots already carry edge-to-triangle bindings. Their sampled
  // display polylines are unused by STL and would add both native and Rust
  // buffers; ordinary viewport snapshots retain the complete edge geometry.
  for (int e = 1; !include_edge_bindings && e <= edges.Extent(); ++e) {
   auto edge = TopoDS::Edge(edges(e));
   if (BRep_Tool::Degenerated(edge)) continue;
   BRepAdaptor_Curve curve(edge);
   GCPnts_QuasiUniformDeflection sample(curve, deflection);
   if (!sample.IsDone() || sample.NbPoints() < 2)
    throw std::runtime_error("BRep edge sampling failed");
   uint32_t previous = point(sample.Value(1));
   for (int i = 2; i <= sample.NbPoints(); ++i) {
    uint32_t next = point(sample.Value(i));
    reserve(sizeof(std::array<uint32_t, 3>));
    segments.push_back({previous, next, uint32_t(e - 1)});
    previous = next;
   }
  }
  for (int v = 1; v <= verts.Extent(); ++v) {
   reserve(sizeof(uint32_t));
   vertices.push_back(point(BRep_Tool::Pnt(TopoDS::Vertex(verts(v)))));
  }
  Om9BrepSnapshot snapshot{
      text.data(),          text.size(),
      reinterpret_cast<const double*>(points.data()), points.size(),
      reinterpret_cast<const uint32_t*>(triangles.data()), triangles.size(),
      reinterpret_cast<const uint32_t*>(segments.data()), segments.size(),
      vertices.data(),      vertices.size(),
      reinterpret_cast<const uint32_t*>(face_edges.data()), face_edges.size(),
      size_t(faces.Extent()), size_t(edges.Extent())};
  if (sink(context, &snapshot) != 0) throw std::runtime_error("Rust refused BRep snapshot");
  return 0;
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
