// SPDX-License-Identifier: LGPL-2.1-or-later
// A synchronous snapshot operation. All native objects die before returning; the Rust sink copies
// bounded immutable arrays. No OCCT object or topology index becomes a persistent document ID.
#include "brep_snapshot.hxx"

#include <BRepGProp.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepTools_ShapeSet.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <gp_GTrsf.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <GProp_GProps.hxx>
#include <ShapeFix_Shape.hxx>
#include <ShapeUpgrade_ShapeDivideContinuity.hxx>
#include <ShapeUpgrade_ShapeDivideArea.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS_Shell.hxx>
#include <TopLoc_Location.hxx>

#include <algorithm>
#include <vector>

// Descriptor visitation is opt-in and only valid during the snapshot sink.
// Subshape bytes are streamed to Rust for hashing; no native descriptor string
// or OCCT shape handle is retained by Rust after the synchronous callback.
using Om9TopologySink = int (*)(void*, int, int, uint32_t, uint32_t, int, const char*, size_t);
using Om9SignatureSink = int (*)(void*, int, size_t, const double*);
using Om9SourceHandleSink = int (*)(void*, const void*);

static void om9_emit_source_signatures(const TopoDS_Shape& shape, size_t max_nodes,
                                       Om9SignatureSink sink, void* context) {
  TopTools_IndexedMapOfShape maps[3];
  const TopAbs_ShapeEnum types[3] = {TopAbs_FACE, TopAbs_EDGE, TopAbs_VERTEX};
  size_t total = 0;
  for (int kind = 0; kind < 3; ++kind) {
   TopExp::MapShapes(shape, types[kind], maps[kind]);
   const auto count = size_t(maps[kind].Extent());
   if (count > max_nodes - total) throw std::length_error("signature topology budget");
   total += count;
  }
  if (sink(context, -1, total, nullptr)) throw std::runtime_error("signature allocation refused");
  for (int kind = 0; kind < 3; ++kind) {
   for (int i = 1; i <= maps[kind].Extent(); ++i) {
    if (sink(context, -2, 0, nullptr)) throw std::runtime_error("signature canceled");
    double values[7] = {};
    if (kind == 2) {
     const gp_Pnt point = BRep_Tool::Pnt(TopoDS::Vertex(maps[kind](i)));
     values[0] = values[3] = point.X(); values[1] = values[4] = point.Y(); values[2] = values[5] = point.Z();
    } else {
     GProp_GProps properties;
     if (kind == 0) {
      BRepGProp::SurfaceProperties(maps[kind](i), properties, 1.0e-12);
      const gp_Pnt center = properties.CentreOfMass();
      values[0] = values[3] = center.X(); values[1] = values[4] = center.Y(); values[2] = values[5] = center.Z();
     } else {
      const TopoDS_Edge edge = TopoDS::Edge(maps[kind](i));
      TopoDS_Vertex first, last;
      TopExp::Vertices(edge, first, last, true);
      if (first.IsNull() || last.IsNull()) throw std::invalid_argument("signature edge endpoints");
      const gp_Pnt a = BRep_Tool::Pnt(first), b = BRep_Tool::Pnt(last);
      values[0] = a.X(); values[1] = a.Y(); values[2] = a.Z();
      values[3] = b.X(); values[4] = b.Y(); values[5] = b.Z();
      BRepGProp::LinearProperties(edge, properties);
     }
     values[6] = properties.Mass();
    }
    if (sink(context, kind, size_t(i - 1), values)) throw std::runtime_error("signature output refused");
   }
  }
}

// Only valid while the snapshot callback owns the exact native root.
extern "C" int om9_occt_snapshot_signatures(const void* handle, size_t max_nodes,
                                            Om9SignatureSink sink, void* context) {
 try {
  if (!handle || !sink) throw std::invalid_argument("invalid signature snapshot");
  om9_emit_source_signatures(*static_cast<const TopoDS_Shape*>(handle), max_nodes, sink, context);
  return 0;
 } catch (...) { return 1; }
}

// The indices belong only to this source read. They are not UUIDs and must
// not be associated with an older display without a topology proof. OCCT's
// source parser and topology maps allocate outside the Rust memory budget.
extern "C" int om9_occt_source_signatures(const char* bytes, size_t length,
                                         size_t max_nodes, Om9SignatureSink sink, void* context,
                                         Om9SourceHandleSink handle_sink, void* handle_context,
                                         char* message, int capacity) {
 try {
  if (!bytes || !length || !sink) throw std::invalid_argument("invalid signature input");
  if (sink(context, -2, 0, nullptr)) throw std::runtime_error("signature canceled");
  std::istringstream input(std::string(bytes, length));
  BRep_Builder builder;
  TopoDS_Shape shape;
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull()) throw std::invalid_argument("invalid signature source");
  if (handle_sink && handle_sink(handle_context, &shape)) throw std::runtime_error("signature topology refused");
  om9_emit_source_signatures(shape, max_nodes, sink, context);
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown signature failure");
 }
 return 1;
}
namespace {
class Om9TopologyStream final : public std::streambuf {
 public:
  Om9TopologyStream(Om9TopologySink sink, void* context, size_t& remaining)
      : sink_(sink), context_(context), remaining_(remaining) {}
 protected:
  std::streamsize xsputn(const char* data, std::streamsize size) override {
   if (size < 0 || size_t(size) > remaining_) throw std::length_error("topology descriptor budget");
   if (size && sink_(context_, 1, 0, 0, 0, 0, data, size_t(size)))
    throw std::runtime_error("topology descriptor refused");
   remaining_ -= size_t(size);
   return size;
  }
  int_type overflow(int_type value) override {
   if (traits_type::eq_int_type(value, traits_type::eof())) return traits_type::not_eof(value);
   const char byte = traits_type::to_char_type(value);
   return xsputn(&byte, 1) == 1 ? value : traits_type::eof();
  }
 private:
  Om9TopologySink sink_;
  void* context_;
  size_t& remaining_;
};
}

extern "C" int om9_occt_visit_snapshot_topology(const void* handle, size_t max_nodes,
                                                size_t max_bytes, Om9TopologySink sink, void* context) {
 try {
  if (!handle || !sink) throw std::invalid_argument("invalid topology visit");
  const auto& shape = *static_cast<const TopoDS_Shape*>(handle);
  TopTools_IndexedMapOfShape maps[3];
  TopExp::MapShapes(shape, TopAbs_FACE, maps[0]);
  TopExp::MapShapes(shape, TopAbs_EDGE, maps[1]);
  TopExp::MapShapes(shape, TopAbs_VERTEX, maps[2]);
  size_t nodes = 0, remaining = max_bytes;
  const auto node = [&]() { if (nodes >= max_nodes) throw std::length_error("topology visit nodes"); ++nodes; };
  for (int kind = 0; kind < 3; ++kind) {
   for (int index = 1; index <= maps[kind].Extent(); ++index) {
    node();
    const auto& element = maps[kind](index);
    if (sink(context, 0, kind, uint32_t(index - 1), 0, int(element.Orientation()), nullptr, 0))
     throw std::runtime_error("topology descriptor refused");
    Om9TopologyStream buffer(sink, context, remaining);
    std::ostream stream(&buffer);
    stream.exceptions(std::ios::badbit | std::ios::failbit);
    BRepTools::Write(element, stream, false, false, TopTools_FormatVersion_CURRENT);
    if (sink(context, 2, kind, uint32_t(index - 1), 0, int(element.Orientation()), nullptr, 0))
     throw std::runtime_error("topology descriptor refused");
   }
  }
  for (int parent_kind = 0; parent_kind < 2; ++parent_kind) {
   const auto child_type = parent_kind == 0 ? TopAbs_EDGE : TopAbs_VERTEX;
   for (int parent = 1; parent <= maps[parent_kind].Extent(); ++parent) {
    for (TopExp_Explorer child(maps[parent_kind](parent), child_type); child.More(); child.Next()) {
     node();
     const int index = maps[parent_kind + 1].FindIndex(child.Current());
     if (!index || sink(context, 3, parent_kind, uint32_t(parent - 1), uint32_t(index - 1),
                        int(child.Current().Orientation()), nullptr, 0))
      throw std::runtime_error("topology incidence refused");
    }
   }
  }
  return 0;
 } catch (...) { return 1; }
}

// Stream the root and explicit subshape references from the very same
// snapshot. Rust owns the resulting bytes and associates UUIDs in map order.
// OCCT's temporary native allocations are not bounded by max_bytes.
extern "C" int om9_occt_write_snapshot_shape_set(const void* handle, size_t max_nodes,
                                                 size_t max_bytes, Om9TopologySink sink, void* context) {
 try {
  if (!handle || !sink) throw std::invalid_argument("invalid shape set snapshot");
  const auto& shape = *static_cast<const TopoDS_Shape*>(handle);
  if (shape.IsNull()) throw std::invalid_argument("null shape set root");
  TopTools_IndexedMapOfShape maps[3];
  const TopAbs_ShapeEnum kinds[3] = {TopAbs_FACE, TopAbs_EDGE, TopAbs_VERTEX};
  size_t total = 0;
  for (int kind = 0; kind < 3; ++kind) {
   TopExp::MapShapes(shape, kinds[kind], maps[kind]);
   const auto count = size_t(maps[kind].Extent());
   if (count > max_nodes - total) throw std::length_error("shape set topology budget");
   total += count;
  }
  if (maps[0].IsEmpty()) throw std::invalid_argument("shape set has no faces");
  BRepTools_ShapeSet shapes(false, false);
  shapes.Add(shape);
  size_t remaining = max_bytes;
  Om9TopologyStream buffer(sink, context, remaining);
  std::ostream output(&buffer);
  output.exceptions(std::ios::badbit | std::ios::failbit);
  output << "OM9-SHAPESET 1\n" << maps[0].Extent() << ' ' << maps[1].Extent()
         << ' ' << maps[2].Extent() << '\n';
  shapes.Write(output);
  shapes.Write(shape, output);
  output << '\n';
  for (int kind = 0; kind < 3; ++kind) {
   for (int index = 1; index <= maps[kind].Extent(); ++index) {
    shapes.Write(maps[kind](index), output);
    output << '\n';
   }
  }
  output.flush();
  return 0;
 } catch (...) { return 1; }
}

// Decode and verify explicit references against the restored root before
// emitting a private B-rep snapshot. OCCT's internal parser allocations are
// not bounded by max_bytes: this entrypoint is not yet for untrusted files.
static int om9_read_shape_set(
    const char* bytes, size_t length, double deflection, double angular,
    size_t max_points, size_t max_nodes, size_t max_bytes,
    Om9TopologySink mapping_sink, void* mapping_context,
    Om9BrepSink snapshot_sink, void* snapshot_context, Om9SignatureSink signature_sink,
    void* signature_context, char* message, int capacity) {
 try {
  if (!bytes || !length || length > max_bytes || !mapping_sink || (!snapshot_sink && !signature_sink))
   throw std::invalid_argument("invalid shape set input");
  std::istringstream input(std::string(bytes, length));
  std::string header;
  std::getline(input, header);
  if (header != "OM9-SHAPESET 1") throw std::invalid_argument("shape set version");
  uint64_t counts[3] = {0, 0, 0};
  if (!(input >> counts[0] >> counts[1] >> counts[2]))
   throw std::invalid_argument("shape set counts");
  size_t total = 0;
  for (auto count : counts) {
   if (count > uint64_t(std::numeric_limits<int>::max()) || count > uint64_t(max_nodes - total))
    throw std::length_error("shape set topology budget");
   total += size_t(count);
  }
  if (!counts[0]) throw std::invalid_argument("shape set has no faces");
  input >> std::ws;
  BRepTools_ShapeSet shapes(false, false);
  shapes.Read(input);
  TopoDS_Shape root;
  shapes.Read(root, input);
  if (!input || root.IsNull() || !BRepCheck_Analyzer(root).IsValid())
   throw std::invalid_argument("invalid shape set root");
  TopTools_IndexedMapOfShape maps[3];
  const TopAbs_ShapeEnum kinds[3] = {TopAbs_FACE, TopAbs_EDGE, TopAbs_VERTEX};
  for (int kind = 0; kind < 3; ++kind) {
   TopExp::MapShapes(root, kinds[kind], maps[kind]);
   if (uint64_t(maps[kind].Extent()) != counts[kind])
    throw std::invalid_argument("shape set topology count mismatch");
   std::vector<unsigned char> seen(size_t(counts[kind]), 0);
   for (uint32_t old_index = 0; old_index < counts[kind]; ++old_index) {
    TopoDS_Shape reference;
    shapes.Read(reference, input);
    if (!input || reference.IsNull() || reference.ShapeType() != kinds[kind])
     throw std::invalid_argument("invalid shape set reference");
    const int new_index = maps[kind].FindIndex(reference);
    if (!new_index || seen[size_t(new_index - 1)] ||
        reference.Orientation() != maps[kind](new_index).Orientation())
     throw std::invalid_argument("ambiguous shape set reference");
    seen[size_t(new_index - 1)] = 1;
    if (mapping_sink(mapping_context, 0, kind, old_index, uint32_t(new_index - 1),
                     int(reference.Orientation()), nullptr, 0))
     throw std::runtime_error("shape set mapping refused");
   }
  }
  input >> std::ws;
  if (!input.eof()) throw std::invalid_argument("trailing shape set data");
  if (signature_sink) {
   om9_emit_source_signatures(root, max_nodes, signature_sink, signature_context);
   return 0;
  }
  std::ostringstream source_stream;
  BRepTools::Write(root, source_stream, false, false, TopTools_FormatVersion_CURRENT);
  const std::string source = source_stream.str();
  if (source.empty() || source.size() > max_bytes) throw std::length_error("shape set source budget");
  // Retain this exact root while emitting, so the mapping above and the
  // callback's face/edge/vertex indices describe the same native shapes.
  return om9_emit_brep_snapshot(root, deflection, max_points, max_nodes, max_bytes,
      snapshot_sink, snapshot_context, message, capacity, angular, nullptr, nullptr,
      false, false, source.data(), source.size());
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown shape set read failure");
  return 2;
 }
}

extern "C" int om9_occt_read_snapshot_shape_set(
    const char* bytes, size_t length, double deflection, double angular,
    size_t max_points, size_t max_nodes, size_t max_bytes,
    Om9TopologySink mapping_sink, void* mapping_context,
    Om9BrepSink snapshot_sink, void* snapshot_context, char* message, int capacity) {
 return om9_read_shape_set(bytes, length, deflection, angular, max_points, max_nodes, max_bytes,
     mapping_sink, mapping_context, snapshot_sink, snapshot_context, nullptr, nullptr, message, capacity);
}

// Only privately captured shape-set packets have an established association
// with Rust topology IDs; external packets cannot establish that association.
extern "C" int om9_occt_shape_set_signatures(
    const char* bytes, size_t length, size_t max_nodes, size_t max_bytes,
    Om9TopologySink mapping_sink, void* mapping_context,
    Om9SignatureSink signature_sink, void* signature_context, char* message, int capacity) {
 return om9_read_shape_set(bytes, length, 0., 0., 0, max_nodes, max_bytes,
     mapping_sink, mapping_context, nullptr, nullptr, signature_sink, signature_context, message, capacity);
}


// Copy native topological vertex positions; no display triangulation is used.
extern "C" int om9_occt_brep_vertices(const char* source, size_t length, double* points,
                                       size_t expected, char* message, int capacity) {
 try {
  if (!source || !length || (expected && !points)) throw std::invalid_argument("vertices: missing input");
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid()) throw std::invalid_argument("vertices: invalid B-Rep");
  TopTools_IndexedMapOfShape vertices;
  TopExp::MapShapes(shape, TopAbs_VERTEX, vertices);
  if (size_t(vertices.Extent()) != expected) throw std::invalid_argument("vertices: topology count mismatch");
  for (int i = 1; i <= vertices.Extent(); ++i) {
   const gp_Pnt point = BRep_Tool::Pnt(TopoDS::Vertex(vertices(i)));
   points[size_t(i - 1) * 3] = point.X(); points[size_t(i - 1) * 3 + 1] = point.Y(); points[size_t(i - 1) * 3 + 2] = point.Z();
  }
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown BRep vertices failure"); return 2;
 }
}

// World bounds from source surfaces, without mesh generation or a snapshot.
extern "C" int om9_occt_brep_world_bounds(const char* source, size_t length,
                                           const double* matrix, double* bounds,
                                           char* message, int capacity) {
 try {
  if (!source || !length || !matrix || !bounds) throw std::invalid_argument("bounds: missing input");
  for (int i = 0; i < 16; ++i) if (!std::isfinite(matrix[i])) throw std::invalid_argument("bounds: non-finite transform");
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid()) throw std::invalid_argument("bounds: invalid B-Rep");
  bool identity = true;
  for (int r = 0; r < 4; ++r) for (int c = 0; c < 4; ++c)
   identity = identity && matrix[4 * r + c] == (r == c ? 1.0 : 0.0);
  bool translation = true;
  for (int r = 0; r < 3; ++r) for (int c = 0; c < 3; ++c)
   translation = translation && matrix[4 * r + c] == (r == c ? 1.0 : 0.0);
  if (!identity && translation) {
   // A GTransform converts analytic surfaces to splines even for translation;
   // applying a location preserves the exact underlying surface and its bounds.
   gp_Trsf offset;
   offset.SetTranslation(gp_Vec(matrix[3], matrix[7], matrix[11]));
   shape.Move(TopLoc_Location(offset));
  } else if (!identity) {
   gp_GTrsf affine;
   for (int r = 0; r < 3; ++r) for (int c = 0; c < 4; ++c)
    affine.SetValue(r + 1, c + 1, matrix[4 * r + c]);
   BRepBuilderAPI_GTransform transformed(shape, affine, true);
   if (!transformed.IsDone() || transformed.Shape().IsNull()) throw std::runtime_error("bounds: transform failed");
   shape = transformed.Shape();
  }
  Bnd_Box box;
  BRepBndLib::AddOptimal(shape, box, false, false);
  if (box.IsVoid() || box.IsOpen()) throw std::runtime_error("bounds: empty or unbounded B-Rep");
  box.Get(bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], bounds[5]);
  for (int i = 0; i < 6; ++i) if (!std::isfinite(bounds[i])) throw std::runtime_error("bounds: non-finite result");
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown BRep bounds failure"); return 2;
 }
}

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

// Inspect the canonical source without meshing. Counts are snapshot-local and
// cannot establish any mapping to persisted UUIDs.
extern "C" int om9_occt_brep_inspect_source(const char* source, size_t length,
                                             uint64_t* counts, char* message, int capacity) {
 try {
  if (!source || !length || !counts) throw std::invalid_argument("invalid BRep source");
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
   throw std::invalid_argument("invalid OCCT BRep source");
  const TopAbs_ShapeEnum kinds[3] = {TopAbs_FACE, TopAbs_EDGE, TopAbs_VERTEX};
  for (int i = 0; i < 3; ++i) {
   TopTools_IndexedMapOfShape map;
   TopExp::MapShapes(shape, kinds[i], map);
   counts[i] = static_cast<uint64_t>(map.Extent());
  }
  if (!counts[0]) throw std::invalid_argument("BRep source has no faces");
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown BRep inspection failure");
  return 2;
 }
}

extern "C" int om9_occt_brep_snapshot(const char* source, size_t source_len, int kind,
                                      const double* dimensions, double deflection, double angular_deflection,
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
                                message, capacity, angular_deflection, nullptr, nullptr, false, false,
                                source_len ? source : nullptr, source_len);
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

// Export-only face ownership, emitted from the same parsed shape as the mesh snapshot.
extern "C" int om9_occt_brep_snapshot_grouped(const char* source, size_t source_len,
                                               double deflection, double angular_deflection,
                                               size_t max_points, size_t max_nodes, size_t max_bytes,
                                               Om9BrepSink sink, void* context,
                                               Om9FaceShellSink shell_sink, void* shell_context,
                                               int include_edge_bindings,
                                               int optimize_export_sphere,
                                               char* message, int capacity) {
 try {
  if (!source || !source_len || source_len > max_bytes || !sink || !shell_sink)
   throw std::invalid_argument("invalid grouped BRep snapshot input");
  std::istringstream input(std::string(source, source_len));
  BRep_Builder builder;
  TopoDS_Shape shape;
  BRepTools::Read(shape, input, builder);
  return om9_emit_brep_snapshot(shape, deflection, max_points, max_nodes, max_bytes, sink, context,
                                message, capacity, angular_deflection, shell_sink, shell_context,
                                include_edge_bindings != 0, optimize_export_sphere != 0, source, source_len);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown grouped BRep snapshot failure");
  return 2;
 }
}

// Build one exact B-Rep from a closed triangle mesh: one planar face per triangle, sewn into a
// single shell and promoted to a solid. The gem asset owns the mesh; this adapter only makes the
// same authored boundary editable as OCCT topology. An open or non-manifold mesh is refused rather
// than repaired into a fake solid.
extern "C" int om9_occt_brep_from_facets(
    const double* points, size_t point_count, const uint32_t* indices, size_t index_count,
    const size_t* face_sizes, size_t face_count,
    double tolerance, double deflection, size_t max_points, size_t max_nodes, size_t max_bytes,
    Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!sink || !points || !indices || point_count < 4 || face_count < 4 ||
      !std::isfinite(tolerance) || tolerance <= 0 || !std::isfinite(deflection) || deflection <= 0)
   throw std::invalid_argument("invalid mesh BRep input");
  if (point_count > max_points / 3 || face_count > max_nodes || index_count > max_bytes / sizeof(uint32_t))
   throw std::length_error("mesh BRep budget");
  for (size_t i = 0; i < point_count; ++i)
   if (!std::isfinite(points[i * 3]) || !std::isfinite(points[i * 3 + 1]) ||
       !std::isfinite(points[i * 3 + 2]))
    throw std::invalid_argument("mesh BRep point is not finite");
  for (size_t i = 0; i < index_count; ++i)
   if (indices[i] >= point_count) throw std::invalid_argument("mesh BRep corner out of range");

  BRepBuilderAPI_Sewing sewing(std::min(tolerance, 1.0e-5));
  size_t offset = 0;
  for (size_t i = 0; i < face_count; ++i) {
   const size_t size = face_sizes ? face_sizes[i] : 3;
   if (size < 3 || size > 32 || size > index_count - offset)
    throw std::invalid_argument("mesh BRep facet corner count");
   BRepBuilderAPI_MakePolygon polygon;
   const uint32_t* face_indices = indices + offset;
   for (size_t j = 0; j < size; ++j) {
    const uint32_t index = face_indices[j];
    if (std::find(face_indices, face_indices + j, index) != face_indices + j)
     throw std::invalid_argument("mesh BRep degenerate facet");
    const size_t vertex = size_t(index) * 3;
    polygon.Add(gp_Pnt(points[vertex], points[vertex + 1], points[vertex + 2]));
   }
   offset += size;
   polygon.Close();
   if (!polygon.IsDone()) throw std::runtime_error("mesh BRep facet wire failed");
   BRepBuilderAPI_MakeFace face(polygon.Wire(), Standard_True);
   if (!face.IsDone()) throw std::runtime_error("mesh BRep planar face failed");
   sewing.Add(face.Face());
  }
  if (offset != index_count) throw std::invalid_argument("mesh BRep unused facet indices");
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
