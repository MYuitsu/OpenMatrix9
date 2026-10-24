// SPDX-License-Identifier: LGPL-2.1-or-later
// M03 OM9-L2-01-10 boolean / split / cap / fillet / offset / extract adapters.
//
// Every entry point takes canonical OCCT sources (or already-admitted typed data), performs one
// complete OCCT construction, and returns one bounded B-Rep snapshot. Exceptions are contained;
// no TopoDS_Shape or Handle crosses into safe Rust. Validation and result oracles stay in Rust.
#include "brep_snapshot.hxx"

#include <Bnd_Box.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepAlgoAPI_Splitter.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepOffsetAPI_MakeThickSolid.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shell.hxx>
#include <ShapeFix_Face.hxx>
#include <ShapeFix_Shell.hxx>
#include <ShapeFix_Solid.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Dir.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>

#include <cmath>
#include <cstdio>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

TopoDS_Shape read_shape(const char* source, size_t length, const char* what) {
 if (!source || length == 0) throw std::invalid_argument(std::string(what) + ": empty source");
 TopoDS_Shape shape;
 BRep_Builder builder;
 std::istringstream input(std::string(source, length));
 BRepTools::Read(shape, input, builder);
 if (shape.IsNull()) throw std::invalid_argument(std::string(what) + ": unreadable source");
 if (!BRepCheck_Analyzer(shape).IsValid())
  throw std::invalid_argument(std::string(what) + ": invalid B-Rep");
 return shape;
}

TopoDS_Edge edge_at(const TopoDS_Shape& shape, int index, const char* what) {
 if (index < 0) throw std::invalid_argument(std::string(what) + ": negative edge index");
 TopTools_IndexedMapOfShape edges;
 TopExp::MapShapes(shape, TopAbs_EDGE, edges);
 if (index >= edges.Extent())
  throw std::invalid_argument(std::string(what) + ": edge index out of range");
 return TopoDS::Edge(edges(index + 1));
}

TopoDS_Face face_at(const TopoDS_Shape& shape, int index, const char* what) {
 if (index < 0) throw std::invalid_argument(std::string(what) + ": negative face index");
 TopTools_IndexedMapOfShape faces;
 TopExp::MapShapes(shape, TopAbs_FACE, faces);
 if (index >= faces.Extent())
  throw std::invalid_argument(std::string(what) + ": face index out of range");
 return TopoDS::Face(faces(index + 1));
}

// Naked edges are edges with fewer than two face occurrences; a seam edge counts twice for its
// face. This is exact B-Rep topology, never a mesh boundary.
int naked_edge_count(const TopoDS_Shape& shape, TopTools_IndexedMapOfShape& naked) {
 TopTools_IndexedDataMapOfShapeListOfShape edge_faces;
 TopExp::MapShapesAndAncestors(shape, TopAbs_EDGE, TopAbs_FACE, edge_faces);
 int count = 0;
 for (int i = 1; i <= edge_faces.Extent(); ++i) {
  const TopoDS_Edge& edge = TopoDS::Edge(edge_faces.FindKey(i));
  if (BRep_Tool::Degenerated(edge)) continue;
  int ancestors = 0;
  for (TopTools_ListOfShape::Iterator it(edge_faces.FindFromIndex(i)); it.More(); it.Next()) {
   const TopoDS_Face& face = TopoDS::Face(it.Value());
   ancestors += BRep_Tool::IsClosed(edge, face) ? 2 : 1;
  }
  if (ancestors < 2) {
   ++count;
   naked.Add(edge);
  }
 }
 return count;
}

int emit(const TopoDS_Shape& shape, double deflection, size_t max_points, size_t max_nodes,
         size_t max_bytes, Om9BrepSink sink, void* context, char* message, int capacity) {
 return om9_emit_brep_snapshot(shape, deflection, max_points, max_nodes, max_bytes, sink, context,
                               message, capacity);
}

}  // namespace

extern "C" int om9_occt_boolean(int op, const char* source_a, size_t length_a,
                                const char* source_b, size_t length_b, double deflection,
                                size_t max_points, size_t max_nodes, size_t max_bytes,
                                Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  const TopoDS_Shape a = read_shape(source_a, length_a, "boolean argument");
  const TopoDS_Shape b = read_shape(source_b, length_b, "boolean tool");
  TopoDS_Shape result;
  switch (op) {
   case 0: {
    BRepAlgoAPI_Fuse fuse(a, b);
    fuse.Build();
    if (!fuse.IsDone() || fuse.HasErrors()) throw std::runtime_error("boolean union failed");
    result = fuse.Shape();
    break;
   }
   case 1: {
    BRepAlgoAPI_Cut cut(a, b);
    cut.Build();
    if (!cut.IsDone() || cut.HasErrors()) throw std::runtime_error("boolean difference failed");
    result = cut.Shape();
    break;
   }
   case 2: {
    BRepAlgoAPI_Common common(a, b);
    common.Build();
    if (!common.IsDone() || common.HasErrors())
     throw std::runtime_error("boolean intersection failed");
    result = common.Shape();
    break;
   }
   default:
    throw std::invalid_argument("boolean: unknown operation");
  }
  TopTools_IndexedMapOfShape faces;
  if (!result.IsNull()) TopExp::MapShapes(result, TopAbs_FACE, faces);
  // Status 3 is the typed "valid but empty" outcome (disjoint intersection); the Rust side maps it
  // to BooleanOutcome::Empty instead of inventing a zero-face shape.
  if (result.IsNull() || faces.Extent() == 0) return 3;
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message,
              capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown boolean failure");
  return 2;
 }
}

extern "C" int om9_occt_split_face(const char* source, size_t length, const double* line,
                                   size_t line_count, int* pieces, double deflection,
                                   size_t max_points, size_t max_nodes, size_t max_bytes,
                                   Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!line || line_count < 2) throw std::invalid_argument("split: trim line needs two points");
  if (!pieces) throw std::invalid_argument("split: piece marker missing");
  const TopoDS_Shape face = read_shape(source, length, "split face");
  TopTools_IndexedMapOfShape face_map;
  TopExp::MapShapes(face, TopAbs_FACE, face_map);
  if (face_map.Extent() != 1) throw std::invalid_argument("split: exactly one face is required");
  for (size_t i = 0; i < line_count * 3; ++i)
   if (!std::isfinite(line[i])) throw std::invalid_argument("split: non-finite trim point");
  BRepBuilderAPI_MakeWire wire;
  for (size_t i = 1; i < line_count; ++i) {
   const double* p = line + 3 * (i - 1);
   const double* q = line + 3 * i;
   if (std::abs(p[0] - q[0]) + std::abs(p[1] - q[1]) + std::abs(p[2] - q[2]) <= 1.0e-12)
    throw std::invalid_argument("split: trim line repeats a point");
   BRepBuilderAPI_MakeEdge edge(gp_Pnt(p[0], p[1], p[2]), gp_Pnt(q[0], q[1], q[2]));
   if (!edge.IsDone()) throw std::runtime_error("split: trim edge failed");
   wire.Add(edge.Edge());
  }
  if (!wire.IsDone()) throw std::runtime_error("split: trim wire failed");
  BRepAlgoAPI_Splitter splitter;
  NCollection_List<TopoDS_Shape> arguments;
  NCollection_List<TopoDS_Shape> tools;
  arguments.Append(face);
  tools.Append(wire.Wire());
  splitter.SetArguments(arguments);
  splitter.SetTools(tools);
  splitter.Build();
  if (!splitter.IsDone() || splitter.HasErrors()) throw std::runtime_error("split failed");
  const TopoDS_Shape result = splitter.Shape();
  TopTools_IndexedMapOfShape result_faces;
  TopExp::MapShapes(result, TopAbs_FACE, result_faces);
  *pieces = result_faces.Extent();
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message,
              capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown split failure");
  return 2;
 }
}

extern "C" int om9_occt_cap(const char* source, size_t length, double tolerance, double deflection,
                            size_t max_points, size_t max_nodes, size_t max_bytes,
                            Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!std::isfinite(tolerance) || tolerance <= 0)
   throw std::invalid_argument("cap: tolerance must be positive");
  const TopoDS_Shape shell = read_shape(source, length, "cap shell");
  TopTools_IndexedMapOfShape naked;
  const int naked_count = naked_edge_count(shell, naked);
  if (naked_count == 0) throw std::invalid_argument("cap: the shape has no naked edge");
  // Order the naked edges into one closed chain; the map order is arbitrary, so a naive MakeWire
  // can stop early with an open wire.
  std::vector<TopoDS_Edge> chain;
  std::vector<bool> used(static_cast<size_t>(naked.Extent()) + 1, false);
  chain.push_back(TopoDS::Edge(naked(1)));
  used[1] = true;
  TopoDS_Vertex start_vertex, end_vertex;
  TopExp::Vertices(chain.front(), start_vertex, end_vertex);
  if (start_vertex.IsNull() || end_vertex.IsNull())
   throw std::runtime_error("cap: naked edge has no vertex");
  while (chain.size() < static_cast<size_t>(naked.Extent())) {
   bool extended = false;
   for (int i = 1; i <= naked.Extent(); ++i) {
    if (used[i]) continue;
    const TopoDS_Edge& edge = TopoDS::Edge(naked(i));
    TopoDS_Vertex v1, v2;
    TopExp::Vertices(edge, v1, v2);
    if (v1.IsSame(end_vertex)) {
     chain.push_back(edge);
     used[i] = true;
     end_vertex = v2;
     extended = true;
     break;
    }
    if (v2.IsSame(end_vertex)) {
     chain.push_back(TopoDS::Edge(edge.Reversed()));
     used[i] = true;
     end_vertex = v1;
     extended = true;
     break;
    }
   }
   if (!extended) throw std::runtime_error("cap: naked edges do not form one wire");
  }
  if (!end_vertex.IsSame(start_vertex))
   throw std::runtime_error("cap: naked loop is not closed");
  BRep_Builder wire_builder;
  TopoDS_Wire loop_wire;
  wire_builder.MakeWire(loop_wire);
  for (const TopoDS_Edge& edge : chain) wire_builder.Add(loop_wire, edge);
  // Admit the loop as planar: three vertices define the plane and every vertex must lie on it.
  std::vector<gp_Pnt> loop;
  for (TopExp_Explorer explorer(loop_wire, TopAbs_VERTEX); explorer.More(); explorer.Next())
   loop.push_back(BRep_Tool::Pnt(TopoDS::Vertex(explorer.Current())));
  if (loop.size() < 3) throw std::runtime_error("cap: naked loop has too few vertices");
  gp_Pln plane;
  bool found = false;
  for (size_t i = 1; i + 1 < loop.size() && !found; ++i) {
   for (size_t j = i + 1; j < loop.size() && !found; ++j) {
    const gp_Vec a(loop[0], loop[i]);
    const gp_Vec b(loop[0], loop[j]);
    const gp_Vec normal = a.Crossed(b);
    if (normal.Magnitude() > 1.0e-9) {
     plane = gp_Pln(loop[0], gp_Dir(normal));
     found = true;
    }
   }
  }
  if (!found) throw std::runtime_error("cap: naked loop is degenerate");
  const double planarity = std::max(tolerance, 1.0e-7);
  for (const gp_Pnt& point : loop)
   if (plane.Distance(point) > planarity) throw std::runtime_error("cap: naked loop is not planar");
  // Try the two plane orientations and the two wire directions; the first combination whose face
  // and assembled solid pass BRepCheck is the cap. This avoids guessing the boundary direction.
  const gp_Dir base_normal = plane.Axis().Direction();
  bool assembled = false;
  TopoDS_Solid capped_solid;
  for (int flip = 0; flip < 2 && !assembled; ++flip) {
   for (int reverse = 0; reverse < 2 && !assembled; ++reverse) {
    const gp_Dir normal = flip ? gp_Dir(-base_normal.X(), -base_normal.Y(), -base_normal.Z())
                               : base_normal;
    TopoDS_Wire wire_candidate = loop_wire;
    if (reverse) wire_candidate.Reverse();
    BRepBuilderAPI_MakeFace candidate(gp_Pln(loop[0], normal), wire_candidate, true);
    if (!candidate.IsDone()) continue;
    ShapeFix_Face face_fixer(candidate.Face());
    face_fixer.FixOrientation();
    face_fixer.Perform();
    const TopoDS_Face candidate_face = face_fixer.Face();
    if (!BRepCheck_Analyzer(candidate_face).IsValid()) continue;
    BRep_Builder builder;
    TopoDS_Shell capped_shell;
    builder.MakeShell(capped_shell);
    for (TopExp_Explorer explorer(shell, TopAbs_FACE); explorer.More(); explorer.Next())
     builder.Add(capped_shell, explorer.Current());
    builder.Add(capped_shell, candidate_face);
    ShapeFix_Shell fixer(capped_shell);
    fixer.Perform();
    capped_shell = fixer.Shell();
    TopTools_IndexedMapOfShape free_edges;
    if (naked_edge_count(capped_shell, free_edges) != 0) continue;
    BRepBuilderAPI_MakeSolid solid(capped_shell);
    if (!solid.IsDone()) continue;
    ShapeFix_Solid solid_fixer(solid.Solid());
    solid_fixer.Perform();
    const TopoDS_Shape fixed_shape = solid_fixer.Solid();
    if (!BRepCheck_Analyzer(fixed_shape).IsValid()) continue;
    capped_solid = TopoDS::Solid(fixed_shape);
    assembled = true;
   }
  }
  if (!assembled) throw std::runtime_error("cap: cannot orient and close the cap");
  return emit(capped_solid, deflection, max_points, max_nodes, max_bytes, sink, context, message,
              capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown cap failure");
  return 2;
 }
}

static int fillet_or_chamfer(bool chamfer, const char* source, size_t length, const int* edges,
                             size_t edge_count, double value, double deflection,
                             size_t max_points, size_t max_nodes, size_t max_bytes,
                             Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!edges || edge_count == 0) throw std::invalid_argument("edge edit: no edge selected");
  if (!std::isfinite(value) || value <= 0)
   throw std::invalid_argument("edge edit: value must be positive");
  const TopoDS_Shape shape = read_shape(source, length, "edge edit solid");
  TopTools_IndexedMapOfShape solids;
  TopExp::MapShapes(shape, TopAbs_SOLID, solids);
  if (solids.Extent() == 0) throw std::invalid_argument("edge edit: a solid is required");
  TopoDS_Shape result;
  if (chamfer) {
   BRepFilletAPI_MakeChamfer maker(shape);
   for (size_t i = 0; i < edge_count; ++i)
    maker.Add(value, edge_at(shape, edges[i], "chamfer"));
   maker.Build();
   if (!maker.IsDone()) throw std::runtime_error("chamfer failed");
   result = maker.Shape();
  } else {
   BRepFilletAPI_MakeFillet maker(shape);
   for (size_t i = 0; i < edge_count; ++i)
    maker.Add(value, edge_at(shape, edges[i], "fillet"));
   maker.Build();
   if (!maker.IsDone()) throw std::runtime_error("fillet failed");
   result = maker.Shape();
  }
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid())
   throw std::runtime_error("edge edit produced an invalid shape");
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message,
              capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown edge edit failure");
  return 2;
 }
}

extern "C" int om9_occt_fillet(const char* source, size_t length, const int* edges,
                               size_t edge_count, double radius, double deflection,
                               size_t max_points, size_t max_nodes, size_t max_bytes,
                               Om9BrepSink sink, void* context, char* message, int capacity) {
 return fillet_or_chamfer(false, source, length, edges, edge_count, radius, deflection, max_points,
                          max_nodes, max_bytes, sink, context, message, capacity);
}

extern "C" int om9_occt_chamfer(const char* source, size_t length, const int* edges,
                                size_t edge_count, double distance, double deflection,
                                size_t max_points, size_t max_nodes, size_t max_bytes,
                                Om9BrepSink sink, void* context, char* message, int capacity) {
 return fillet_or_chamfer(true, source, length, edges, edge_count, distance, deflection,
                          max_points, max_nodes, max_bytes, sink, context, message, capacity);
}

extern "C" int om9_occt_thicken(const char* source, size_t length, double thickness,
                                double deflection, size_t max_points, size_t max_nodes,
                                size_t max_bytes, Om9BrepSink sink, void* context, char* message,
                                int capacity) {
 try {
  if (!std::isfinite(thickness) || thickness == 0)
   throw std::invalid_argument("thicken: thickness must be finite and non-zero");
  const TopoDS_Shape shape = read_shape(source, length, "thicken shape");
  BRepOffsetAPI_MakeThickSolid thicken;
  thicken.MakeThickSolidBySimple(shape, thickness);
  thicken.Build();
  if (!thicken.IsDone()) throw std::runtime_error("thicken failed");
  return emit(thicken.Shape(), deflection, max_points, max_nodes, max_bytes, sink, context, message,
              capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown thicken failure");
  return 2;
 }
}

extern "C" int om9_occt_offset_shape(const char* source, size_t length, double distance,
                                     double deflection, size_t max_points, size_t max_nodes,
                                     size_t max_bytes, Om9BrepSink sink, void* context,
                                     char* message, int capacity) {
 try {
  if (!std::isfinite(distance) || distance == 0)
   throw std::invalid_argument("offset: distance must be finite and non-zero");
  const TopoDS_Shape shape = read_shape(source, length, "offset shape");
  BRepOffsetAPI_MakeOffsetShape offset;
  offset.PerformBySimple(shape, distance);
  if (!offset.IsDone()) throw std::runtime_error("offset failed");
  return emit(offset.Shape(), deflection, max_points, max_nodes, max_bytes, sink, context, message,
              capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown offset failure");
  return 2;
 }
}

extern "C" int om9_occt_extract_face(const char* source, size_t length, int face_index,
                                     double deflection, size_t max_points, size_t max_nodes,
                                     size_t max_bytes, Om9BrepSink sink, void* context,
                                     char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "extract face");
  const TopoDS_Face face = face_at(shape, face_index, "extract face");
  return emit(face, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown extract failure");
  return 2;
 }
}

extern "C" int om9_occt_remove_face(const char* source, size_t length, int face_index,
                                    double deflection, size_t max_points, size_t max_nodes,
                                    size_t max_bytes, Om9BrepSink sink, void* context,
                                    char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "remove face");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  if (face_index < 0 || face_index >= faces.Extent())
   throw std::invalid_argument("remove face: face index out of range");
  BRep_Builder builder;
  TopoDS_Shell shell;
  builder.MakeShell(shell);
  for (int i = 1; i <= faces.Extent(); ++i) {
   if (i == face_index + 1) continue;
   builder.Add(shell, faces(i));
  }
  if (shell.IsNull()) throw std::runtime_error("remove face produced no shell");
  return emit(shell, deflection, max_points, max_nodes, max_bytes, sink, context, message,
              capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown remove-face failure");
  return 2;
 }
}

extern "C" int om9_occt_sew(const char* source_a, size_t length_a, const char* source_b,
                            size_t length_b, double tolerance, double deflection, size_t max_points,
                            size_t max_nodes, size_t max_bytes, Om9BrepSink sink, void* context,
                            char* message, int capacity) {
 try {
  if (!std::isfinite(tolerance) || tolerance <= 0)
   throw std::invalid_argument("sew: tolerance must be positive");
  const TopoDS_Shape a = read_shape(source_a, length_a, "sew argument");
  const TopoDS_Shape b = read_shape(source_b, length_b, "sew tool");
  BRepBuilderAPI_Sewing sewing(tolerance);
  sewing.Add(a);
  sewing.Add(b);
  sewing.Perform();
  const TopoDS_Shape sewed = sewing.SewedShape();
  if (sewed.IsNull()) throw std::runtime_error("sew produced no shape");
  // A closed sewn shell becomes an exact solid; an open join stays a shell.
  TopTools_IndexedMapOfShape shells;
  TopExp::MapShapes(sewed, TopAbs_SHELL, shells);
  TopTools_IndexedMapOfShape free_edges;
  const bool closed = naked_edge_count(sewed, free_edges) == 0;
  if (closed && shells.Extent() == 1) {
   BRepBuilderAPI_MakeSolid solid(TopoDS::Shell(shells(1)));
   if (!solid.IsDone()) throw std::runtime_error("sew: closed shell cannot become a solid");
   return emit(solid.Solid(), deflection, max_points, max_nodes, max_bytes, sink, context, message,
               capacity);
  }
  return emit(sewed, deflection, max_points, max_nodes, max_bytes, sink, context, message,
              capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown sew failure");
  return 2;
 }
}

extern "C" int om9_occt_shell(const char* source, size_t length, size_t face_index, double thickness,
                              double deflection, size_t max_points, size_t max_nodes,
                              size_t max_bytes, Om9BrepSink sink, void* context, char* message,
                              int capacity) {
 try {
  if (!source || !length || !std::isfinite(thickness) || thickness <= 0)
   throw std::invalid_argument("invalid shell input");
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
   throw std::invalid_argument("invalid shell source");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  if (face_index >= size_t(faces.Extent())) throw std::invalid_argument("shell face index");
  TopTools_ListOfShape closing;
  closing.Append(faces.FindKey(int(face_index) + 1));
  BRepOffsetAPI_MakeThickSolid thick;
  thick.MakeThickSolidByJoin(shape, closing, -thickness, 1e-6, BRepOffset_Skin, false, false,
                             GeomAbs_Intersection, true);
  thick.Build();
  if (!thick.IsDone()) throw std::runtime_error("shell construction failed");
  return om9_emit_brep_snapshot(thick.Shape(), deflection, max_points, max_nodes, max_bytes, sink,
                                context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown shell failure");
  return 2;
 }
}
