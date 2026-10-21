// SPDX-License-Identifier: LGPL-2.1-or-later
// Complete-operation bridge between a 3DM ON_Brep payload and an OCCT B-Rep.
// All native objects die before returning; Rust receives copied bytes only.
#include "ThreeDmGeometry.h"

#include <BRep_Builder.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepTools.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <GCPnts_QuasiUniformDeflection.hxx>
#include <Poly_Triangulation.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_FormatVersion.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Shape.hxx>
#include <Standard_Failure.hxx>

#include <algorithm>
#include <array>
#include <cstdint>
#include <cstdio>
#include <limits>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {
using BytesSink = int (*)(void*, const unsigned char*, size_t);
struct Snapshot {
  const char* source;
  size_t source_len;
  const double* points;
  size_t points_len;
  const uint32_t* triangles;
  size_t triangles_len;
  const uint32_t* segments;
  size_t segments_len;
  const uint32_t* vertices;
  size_t vertices_len;
  const uint32_t* face_edges;
  size_t face_edges_len;
  size_t faces_count;
  size_t edges_count;
};
using SnapshotSink = int (*)(void*, const Snapshot*);

void message(char* buffer, int capacity, const char* text) {
  if (buffer && capacity > 0) std::snprintf(buffer, size_t(capacity), "%s", text ? text : "");
}

void require(bool condition, const char* text) {
  if (!condition) throw std::runtime_error(text);
}

int snapshot_shape(const TopoDS_Shape& shape, double deflection, size_t max_points,
                   size_t max_nodes, size_t max_bytes, SnapshotSink sink, void* context) {
  require(!shape.IsNull() && BRepCheck_Analyzer(shape).IsValid(), "invalid OCCT BRep shape");
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
  BRepMesh_IncrementalMesh mesh(shape, deflection, false, 0.3, false);
  if (!mesh.IsDone()) throw std::runtime_error("BRep tessellation failed");
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
      triangles.push_back({base + uint32_t(a - 1), base + uint32_t(b - 1),
                           base + uint32_t(c - 1), uint32_t(f - 1)});
    }
    for (TopExp_Explorer ex(face, TopAbs_EDGE); ex.More(); ex.Next()) {
      int e = edges.FindIndex(ex.Current());
      if (!e) throw std::runtime_error("unmapped BRep edge");
      reserve(sizeof(std::array<uint32_t, 2>));
      face_edges.push_back({uint32_t(f - 1), uint32_t(e - 1)});
    }
  }
  for (int e = 1; e <= edges.Extent(); ++e) {
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
  Snapshot snapshot{text.data(), text.size(), reinterpret_cast<const double*>(points.data()),
                    points.size(), reinterpret_cast<const uint32_t*>(triangles.data()),
                    triangles.size(), reinterpret_cast<const uint32_t*>(segments.data()),
                    segments.size(), vertices.data(), vertices.size(),
                    reinterpret_cast<const uint32_t*>(face_edges.data()), face_edges.size(),
                    size_t(faces.Extent()), size_t(edges.Extent())};
  return sink(context, &snapshot);
}

std::string read_occt_source(const char* source, size_t length, size_t max_bytes) {
  require(source && length > 0 && length <= max_bytes, "BRep source budget");
  std::istringstream input(std::string(source, length));
  BRep_Builder builder;
  TopoDS_Shape shape;
  BRepTools::Read(shape, input, builder);
  require(!shape.IsNull() && BRepCheck_Analyzer(shape).IsValid(), "invalid OCCT BRep source");
  std::ostringstream archive;
  BRepTools::Write(shape, archive, false, false, TopTools_FormatVersion_CURRENT);
  std::string text = archive.str();
  require(text.size() <= max_bytes, "OCCT BRep source budget");
  return text;
}
}  // namespace

extern "C" int om9_occt_brep_from_3dm_snapshot(
    const unsigned char* payload,
    size_t length,
    double tolerance,
    double deflection,
    size_t max_points,
    size_t max_nodes,
    size_t max_bytes,
    SnapshotSink sink,
    void* context,
    char* error,
    int capacity) {
  try {
    require(payload && length > 0 && length <= max_bytes, "3DM BRep payload budget");
    require(sink != nullptr, "3DM BRep snapshot sink");
    require(std::isfinite(tolerance) && tolerance > 0, "3DM BRep tolerance");
    require(std::isfinite(deflection) && deflection > 0, "3DM BRep deflection");
    ON_Read3dmBufferArchive archive(length, const_cast<unsigned char*>(payload), true, 50,
                                    ON::Version());
    ON_Object* object = nullptr;
    require(archive.ReadObject(&object) == 1 && object, "3DM BRep payload read");
    std::unique_ptr<ON_Object> owner(object);
    const auto* brep = ON_Brep::Cast(object);
    require(brep && brep->IsValid(), "3DM BRep payload type/validity");
    TopoDS_Shape shape = OpenMatrix9Gui::ThreeDm::importBrep(*brep, tolerance);
    if (snapshot_shape(shape, deflection, max_points, max_nodes, max_bytes, sink, context) != 0) {
      message(error, capacity, "Rust refused converted OCCT BRep snapshot");
      return 1;
    }
    return 0;
  } catch (const Standard_Failure& failure) {
    message(error, capacity, failure.what());
    return 2;
  } catch (const std::exception& failure) {
    message(error, capacity, failure.what());
    return 1;
  } catch (...) {
    message(error, capacity, "unknown 3DM BRep snapshot failure");
    return 2;
  }
}

extern "C" int om9_occt_brep_from_3dm(
    const unsigned char* payload,
    size_t length,
    double tolerance,
    size_t max_bytes,
    BytesSink sink,
    void* context,
    char* error,
    int capacity) {
  try {
    require(payload && length > 0 && length <= max_bytes, "3DM BRep payload budget");
    require(sink != nullptr, "3DM BRep sink");
    require(std::isfinite(tolerance) && tolerance > 0, "3DM BRep tolerance");
    ON_Read3dmBufferArchive archive(length, const_cast<unsigned char*>(payload), true, 50, ON::Version());
    ON_Object* object = nullptr;
    require(archive.ReadObject(&object) == 1 && object, "3DM BRep payload read");
    std::unique_ptr<ON_Object> owner(object);
    const auto* brep = ON_Brep::Cast(object);
    require(brep && brep->IsValid(), "3DM BRep payload type/validity");
    TopoDS_Shape shape = OpenMatrix9Gui::ThreeDm::importBrep(*brep, tolerance);
    require(!shape.IsNull() && BRepCheck_Analyzer(shape).IsValid(), "converted OCCT BRep validity");
    std::ostringstream output;
    BRepTools::Write(shape, output, false, false, TopTools_FormatVersion_CURRENT);
    const std::string text = output.str();
    require(!text.empty() && text.size() <= max_bytes, "converted OCCT BRep budget");
    if (sink(context, reinterpret_cast<const unsigned char*>(text.data()), text.size()) != 0) {
      message(error, capacity, "Rust refused converted OCCT BRep");
      return 1;
    }
    return 0;
  } catch (const Standard_Failure& failure) {
    message(error, capacity, failure.what());
    return 2;
  } catch (const std::exception& failure) {
    message(error, capacity, failure.what());
    return 1;
  } catch (...) {
    message(error, capacity, "unknown 3DM BRep conversion failure");
    return 2;
  }
}

extern "C" int om9_occt_brep_to_3dm(
    const char* source,
    size_t length,
    double tolerance,
    size_t max_bytes,
    unsigned* crc,
    BytesSink sink,
    void* context,
    char* error,
    int capacity) {
  try {
    require(sink != nullptr && crc != nullptr, "BRep output target");
    require(std::isfinite(tolerance) && tolerance > 0, "BRep export tolerance");
    std::istringstream input(std::string(source, length));
    BRep_Builder builder;
    TopoDS_Shape shape;
    BRepTools::Read(shape, input, builder);
    require(!shape.IsNull() && BRepCheck_Analyzer(shape).IsValid(), "invalid OCCT BRep for 3DM export");
    auto brep = OpenMatrix9Gui::ThreeDm::exportBrep(shape, tolerance);
    require(brep && brep->IsValid(), "exported ON_Brep validity");
    ON_Write3dmBufferArchive archive(0, max_bytes, 50, ON::Version());
    require(archive.WriteObject(brep.get()), "ON_Brep payload write");
    const size_t bytes = archive.SizeOfArchive();
    require(bytes > 0 && bytes <= max_bytes, "ON_Brep payload budget");
    *crc = brep->DataCRC(0);
    if (sink(context, static_cast<const unsigned char*>(archive.Buffer()), bytes) != 0) {
      message(error, capacity, "Rust refused ON_Brep payload");
      return 1;
    }
    return 0;
  } catch (const Standard_Failure& failure) {
    message(error, capacity, failure.what());
    return 2;
  } catch (const std::exception& failure) {
    message(error, capacity, failure.what());
    return 1;
  } catch (...) {
    message(error, capacity, "unknown ON_Brep export failure");
    return 2;
  }
}
