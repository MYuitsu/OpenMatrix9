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
#include <BRepAdaptor_Surface.hxx>
#include <BOPAlgo_MakerVolume.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_FindPlane.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <Geom_Plane.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
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
#include <gp_Cylinder.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Vec.hxx>

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <limits>
#include <stdexcept>
#include <string>
#include <utility>
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

// Query a common plane for bounded coplanar faces without transferring native handles.
extern "C" int om9_occt_coplanar_faces_plane(const char* source, size_t length, double* plane,
                                          char* message, int capacity) {
 try {
  if (!plane) throw std::invalid_argument("plane output missing");
  const TopoDS_Shape shape = read_shape(source, length, "Boss boundary");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  if (faces.IsEmpty()) throw std::invalid_argument("Boss boundary needs a face");
  if (faces.Extent() > 1) {
   TopTools_IndexedMapOfShape shells, shell_faces;
   TopExp::MapShapes(shape, TopAbs_SHELL, shells);
   if (shells.Extent() != 1) throw std::invalid_argument("Boss boundary needs one connected shell");
   TopExp::MapShapes(shells(1), TopAbs_FACE, shell_faces);
   if (shell_faces.Extent() != faces.Extent()) throw std::invalid_argument("Boss boundary has disconnected faces");
  }
  BRepAdaptor_Surface surface(TopoDS::Face(faces(1)));
  if (surface.GetType() != GeomAbs_Plane) throw std::invalid_argument("Boss boundary is not planar");
  const gp_Pln p = surface.Plane();
  const gp_Pnt origin = p.Location(); const gp_Dir normal = p.Axis().Direction();
  for (int i = 2; i <= faces.Extent(); ++i) {
   BRepAdaptor_Surface other(TopoDS::Face(faces(i)));
   if (other.GetType() != GeomAbs_Plane) throw std::invalid_argument("Boss boundary is not planar");
   const gp_Pln candidate = other.Plane();
   if (!normal.IsParallel(candidate.Axis().Direction(), 1e-9) || p.Distance(candidate.Location()) > 1e-7)
    throw std::invalid_argument("Boss boundary faces are not coplanar");
  }
  plane[0] = origin.X(); plane[1] = origin.Y(); plane[2] = origin.Z();
  plane[3] = normal.X(); plane[4] = normal.Y(); plane[5] = normal.Z();
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown plane query failure");
  return 2;
 }
}

// Build a bounded solid carrier for one exact cylindrical face. The carrier
// extends beyond the face bounds along the cylinder axis, but not radially.
extern "C" int om9_occt_boss_cylinder_carrier(const char* source, size_t length, double* axis_data, double extension,
                                               double deflection, size_t max_points, size_t max_nodes,
                                               size_t max_bytes, Om9BrepSink sink, void* context,
                                               char* message, int capacity) {
 try {
  if (!axis_data || !std::isfinite(extension) || extension < 0.) throw std::invalid_argument("invalid cylinder carrier input");
  const TopoDS_Shape shape = read_shape(source, length, "Boss cylinder boundary");
  TopTools_IndexedMapOfShape faces, solids;
  TopExp::MapShapes(shape, TopAbs_FACE, faces); TopExp::MapShapes(shape, TopAbs_SOLID, solids);
  if (faces.Extent() != 1 || !solids.IsEmpty()) throw std::invalid_argument("Boss requires one cylindrical face");
  BRepAdaptor_Surface surface(TopoDS::Face(faces(1)));
  if (surface.GetType() != GeomAbs_Cylinder) throw std::invalid_argument("Boss boundary is not cylindrical");
  const gp_Cylinder cylinder = surface.Cylinder();
  const gp_Pnt origin = cylinder.Axis().Location(); const gp_Dir axis = cylinder.Axis().Direction();
  Bnd_Box bounds; BRepBndLib::Add(shape, bounds, false);
  if (bounds.IsVoid() || bounds.IsOpen()) throw std::invalid_argument("Boss cylinder needs finite bounds");
  double xmin,ymin,zmin,xmax,ymax,zmax; bounds.Get(xmin,ymin,zmin,xmax,ymax,zmax);
  if (!std::isfinite(xmin)||!std::isfinite(ymin)||!std::isfinite(zmin)||!std::isfinite(xmax)||!std::isfinite(ymax)||!std::isfinite(zmax))
   throw std::invalid_argument("Boss cylinder bounds are not finite");
  double lo=std::numeric_limits<double>::infinity(), hi=-lo;
  for (int i=0;i<8;++i) {
   const gp_Pnt corner((i&1)?xmax:xmin,(i&2)?ymax:ymin,(i&4)?zmax:zmin);
   const double t=gp_Vec(origin,corner).Dot(gp_Vec(axis)); lo=std::min(lo,t); hi=std::max(hi,t);
  }
  const double margin=std::max(1.0,(hi-lo)*1e-6)+extension, height=hi-lo+2.*margin;
  if (!std::isfinite(lo)||!std::isfinite(hi)||!std::isfinite(height)||height<=0.)
   throw std::invalid_argument("Boss cylinder extent overflow");
  const gp_Pnt start=origin.Translated(gp_Vec(axis)*(lo-margin));
  BRepPrimAPI_MakeCylinder carrier(gp_Ax2(start,axis),cylinder.Radius(),height);
  carrier.Build();
  if (!carrier.IsDone()) throw std::runtime_error("Boss cylinder carrier failed");
  axis_data[0]=origin.X(); axis_data[1]=origin.Y(); axis_data[2]=origin.Z();
  axis_data[3]=axis.X(); axis_data[4]=axis.Y(); axis_data[5]=axis.Z(); axis_data[6]=cylinder.Radius();
  return emit(carrier.Shape(),deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message,capacity,"Unknown cylinder carrier failure"); return 2;
 }
}

extern "C" int om9_occt_trim_surface_plane(const char* source, size_t length,
                                          const double* plane, const double* remove_point,
                                          double deflection, size_t max_points, size_t max_nodes,
                                          size_t max_bytes, Om9BrepSink sink, void* context,
                                          char* message, int capacity) {
 try {
  if (!plane || !remove_point) throw std::invalid_argument("plane input missing");
  const TopoDS_Shape shape = read_shape(source, length, "plane trim surface");
  const gp_Pln support(gp_Pnt(plane[0], plane[1], plane[2]), gp_Dir(plane[3], plane[4], plane[5]));
  const gp_Pnt remove(remove_point[0], remove_point[1], remove_point[2]);
  if (support.Distance(remove) <= 1e-7) throw std::invalid_argument("half-space point on plane");
  // A finite prism covering the source is used because OCCT's Boolean Cut of
  // an open shell by an infinite half-space can return both split halves.
  Bnd_Box bounds;
  BRepBndLib::Add(shape, bounds, false);
  if (bounds.IsVoid() || bounds.IsOpen()) throw std::invalid_argument("plane trim needs finite bounds");
  double xmin, ymin, zmin, xmax, ymax, zmax;
  bounds.Get(xmin, ymin, zmin, xmax, ymax, zmax);
  if (!std::isfinite(xmin) || !std::isfinite(ymin) || !std::isfinite(zmin) ||
      !std::isfinite(xmax) || !std::isfinite(ymax) || !std::isfinite(zmax))
   throw std::invalid_argument("plane trim bounds are not finite");
  const gp_Ax3 axes = support.Position();
  const gp_Dir ux = axes.XDirection(), vy = axes.YDirection(), nz = axes.Direction();
  double umin = std::numeric_limits<double>::infinity(), vmin = umin;
  double umax = -umin, vmax = -umin, max_distance = 0.0;
  for (int i = 0; i < 8; ++i) {
   const gp_Pnt corner((i & 1) ? xmax : xmin, (i & 2) ? ymax : ymin, (i & 4) ? zmax : zmin);
   const gp_Vec offset(support.Location(), corner);
   const double u = offset.Dot(gp_Vec(ux)), v = offset.Dot(gp_Vec(vy));
   umin = std::min(umin, u); umax = std::max(umax, u);
   vmin = std::min(vmin, v); vmax = std::max(vmax, v);
   max_distance = std::max(max_distance, std::abs(offset.Dot(gp_Vec(nz))));
  }
  const double margin = std::max(1.0, std::max(umax-umin, vmax-vmin)*1e-6);
  if (!std::isfinite(margin) || !std::isfinite(max_distance) ||
      !std::isfinite(umin-margin) || !std::isfinite(umax+margin) ||
      !std::isfinite(vmin-margin) || !std::isfinite(vmax+margin) || !std::isfinite(max_distance+margin))
   throw std::invalid_argument("plane trim extent overflow");
  BRepBuilderAPI_MakeFace face(support, umin-margin, umax+margin, vmin-margin, vmax+margin);
  if (!face.IsDone()) throw std::runtime_error("plane face failed");
  const double sign = gp_Vec(support.Location(), remove).Dot(gp_Vec(nz)) > 0 ? 1.0 : -1.0;
  BRepPrimAPI_MakePrism prism(face.Face(), gp_Vec(nz) * (sign * (max_distance + margin)));
  if (!prism.IsDone()) throw std::runtime_error("plane trim prism failed");
  BRepAlgoAPI_Cut cut(shape, prism.Shape());
  cut.Build();
  if (!cut.IsDone() || cut.HasErrors()) throw std::runtime_error("plane trim failed");
  const TopoDS_Shape result = cut.Shape();
  TopTools_IndexedMapOfShape faces;
  if (!result.IsNull()) TopExp::MapShapes(result, TopAbs_FACE, faces);
  if (result.IsNull() || faces.IsEmpty()) return 3;
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown plane trim failure");
  return 2;
 }
}

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

extern "C" int om9_occt_create_solid_regions(
    const char* const* sources, const size_t* lengths, size_t count, int* regions,
    double deflection, size_t max_points, size_t max_nodes, size_t max_bytes,
    Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!sources || !lengths || !regions || count == 0)
   throw std::invalid_argument("create solid: missing inputs");
  NCollection_List<TopoDS_Shape> arguments;
  for (size_t i = 0; i < count; ++i)
   arguments.Append(read_shape(sources[i], lengths[i], "create solid input"));
  BOPAlgo_MakerVolume maker;
  maker.SetArguments(arguments);
  maker.SetIntersect(true);
  maker.SetAvoidInternalShapes(true);
  maker.Perform();
  if (maker.HasErrors()) throw std::runtime_error("create solid regions failed");
  const TopoDS_Shape result = maker.Shape();
  TopTools_IndexedMapOfShape solids;
  if (!result.IsNull()) TopExp::MapShapes(result, TopAbs_SOLID, solids);
  *regions = solids.Extent();
  if (*regions == 0) return 3;
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown create solid failure");
  return 2;
 }
}

extern "C" int om9_occt_split_solid(const char* source_a, size_t length_a,
                                    const char* source_b, size_t length_b, int* pieces,
                                    double deflection, size_t max_points, size_t max_nodes,
                                    size_t max_bytes, Om9BrepSink sink, void* context,
                                    char* message, int capacity) {
 try {
  if (!pieces) throw std::invalid_argument("split solid: piece marker missing");
  const TopoDS_Shape target = read_shape(source_a, length_a, "split target");
  const TopoDS_Shape cutter = read_shape(source_b, length_b, "split cutter");
  BRepAlgoAPI_Splitter splitter;
  NCollection_List<TopoDS_Shape> arguments, tools;
  arguments.Append(target);
  tools.Append(cutter);
  splitter.SetArguments(arguments);
  splitter.SetTools(tools);
  splitter.Build();
  if (!splitter.IsDone() || splitter.HasErrors()) throw std::runtime_error("split solid failed");
  const TopoDS_Shape result = splitter.Shape();
  TopTools_IndexedMapOfShape solids;
  TopExp::MapShapes(result, TopAbs_SOLID, solids);
  *pieces = solids.Extent();
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown solid split failure");
  return 2;
 }
}

extern "C" int om9_occt_split_solid_piece(const char* source, size_t length, int index,
                                          double deflection, size_t max_points, size_t max_nodes,
                                          size_t max_bytes, Om9BrepSink sink, void* context,
                                          char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "split solid pieces");
  TopTools_IndexedMapOfShape solids;
  TopExp::MapShapes(shape, TopAbs_SOLID, solids);
  if (index < 0 || index >= solids.Extent()) throw std::invalid_argument("split solid piece index");
  return emit(solids(index + 1), deflection, max_points, max_nodes, max_bytes,
              sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown solid piece failure");
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
  // Partition the topology into closed chains; the order of the indexed map is arbitrary.
  std::vector<bool> used(static_cast<size_t>(naked.Extent()) + 1, false);
  std::vector<TopoDS_Face> faces;
  for (int first = 1; first <= naked.Extent(); ++first) {
   if (used[first]) continue;
   std::vector<TopoDS_Edge> chain;
   chain.push_back(TopoDS::Edge(naked(first)));
   used[first] = true;
   TopoDS_Vertex start_vertex, end_vertex;
   TopExp::Vertices(chain.front(), start_vertex, end_vertex);
   if (start_vertex.IsNull() || end_vertex.IsNull())
    throw std::runtime_error("cap: naked edge has no vertex");
   while (!end_vertex.IsSame(start_vertex)) {
    bool extended = false;
    for (int i = 1; i <= naked.Extent(); ++i) {
     if (used[i]) continue;
     const TopoDS_Edge& edge = TopoDS::Edge(naked(i));
     TopoDS_Vertex v1, v2;
     TopExp::Vertices(edge, v1, v2);
     if (v1.IsSame(end_vertex) || v2.IsSame(end_vertex)) {
      const bool reverse = v2.IsSame(end_vertex);
      chain.push_back(reverse ? TopoDS::Edge(edge.Reversed()) : edge);
      used[i] = true;
      end_vertex = reverse ? v1 : v2;
      extended = true;
      break;
     }
    }
    if (!extended) throw std::runtime_error("cap: naked loop is not closed");
   }
   BRep_Builder wire_builder;
   TopoDS_Wire wire;
   wire_builder.MakeWire(wire);
   for (const TopoDS_Edge& edge : chain) wire_builder.Add(wire, edge);
   // FindPlane examines the actual curves, including circular single-edge wires.
   BRepBuilderAPI_FindPlane finder(wire, tolerance);
   // Other openings can be non-planar; leave those boundaries unchanged.
   if (!finder.Found()) continue;
   const gp_Pln plane = finder.Plane()->Pln();
   const gp_Dir normal = plane.Axis().Direction();
   TopoDS_Face face;
   for (int flip = 0; flip < 2 && face.IsNull(); ++flip) {
    for (int reverse = 0; reverse < 2 && face.IsNull(); ++reverse) {
     TopoDS_Wire candidate_wire = wire;
     if (reverse) candidate_wire.Reverse();
     const gp_Dir direction = flip ? normal.Reversed() : normal;
     BRepBuilderAPI_MakeFace maker(gp_Pln(plane.Location(), direction), candidate_wire, true);
     if (!maker.IsDone()) continue;
     ShapeFix_Face face_fixer(maker.Face());
     face_fixer.FixOrientation();
     face_fixer.Perform();
     const TopoDS_Face candidate = face_fixer.Face();
     if (BRepCheck_Analyzer(candidate).IsValid()) face = candidate;
    }
   }
   if (face.IsNull()) throw std::runtime_error("cap: invalid planar face");
   faces.push_back(face);
  }
  if (faces.empty()) throw std::runtime_error("cap: no planar closed boundary");
  // Determine containment from exact planar face intersections, then add each
  // immediate inner wire to its containing face. Crossing regions are not holes.
  const size_t count = faces.size();
  std::vector<double> areas(count);
  std::vector<int> parent(count, -1);
  for (size_t i = 0; i < count; ++i) {
   GProp_GProps props;
   BRepGProp::SurfaceProperties(faces[i], props);
   areas[i] = props.Mass();
   if (!std::isfinite(areas[i]) || areas[i] <= tolerance * tolerance)
    throw std::runtime_error("cap: degenerate cap region");
  }
  for (size_t i = 0; i < count; ++i) {
   for (size_t j = i + 1; j < count; ++j) {
    BRepAlgoAPI_Common common(faces[i], faces[j]);
    common.Build();
    if (!common.IsDone()) throw std::runtime_error("cap: cannot compare cap regions");
    GProp_GProps props;
    BRepGProp::SurfaceProperties(common.Shape(), props);
    const double overlap = props.Mass();
    const double smaller = std::min(areas[i], areas[j]);
    const double area_tol = std::max(tolerance * tolerance, smaller * 1.0e-8);
    if (overlap <= area_tol) continue;
    if (std::abs(overlap - smaller) > area_tol || std::abs(areas[i] - areas[j]) <= area_tol)
     throw std::runtime_error("cap: intersecting or coincident boundary regions");
    const size_t child = areas[i] < areas[j] ? i : j;
    const size_t outer = child == i ? j : i;
    if (parent[child] < 0 || areas[outer] < areas[static_cast<size_t>(parent[child])])
     parent[child] = static_cast<int>(outer);
   }
  }
  std::vector<int> depth(count, 0);
  for (size_t i = 0; i < count; ++i) {
   int p = parent[i];
   while (p >= 0) {
    if (++depth[i] > static_cast<int>(count))
     throw std::runtime_error("cap: cyclic boundary containment");
    p = parent[static_cast<size_t>(p)];
   }
  }
  std::vector<TopoDS_Face> cap_faces;
  for (size_t i = 0; i < count; ++i) {
   if (depth[i] % 2 != 0) continue;
   bool has_holes = false;
   for (size_t j = 0; j < count; ++j)
    if (parent[j] == static_cast<int>(i)) has_holes = true;
   if (!has_holes) {
    cap_faces.push_back(faces[i]);
    continue;
   }
   BRepBuilderAPI_MakeFace maker(faces[i]);
   for (size_t j = 0; j < count; ++j) {
    if (parent[j] != static_cast<int>(i)) continue;
    TopExp_Explorer inner(faces[j], TopAbs_WIRE);
    if (!inner.More()) throw std::runtime_error("cap: missing inner wire");
    maker.Add(TopoDS::Wire(inner.Current()));
   }
   if (!maker.IsDone()) throw std::runtime_error("cap: cannot create face with holes");
   ShapeFix_Face face_fixer(maker.Face());
   face_fixer.FixOrientation();
   face_fixer.Perform();
   const TopoDS_Face face = face_fixer.Face();
   if (!BRepCheck_Analyzer(face).IsValid())
    throw std::runtime_error("cap: invalid face with holes");
   cap_faces.push_back(face);
  }
  BRep_Builder builder;
  TopoDS_Shell capped_shell;
  builder.MakeShell(capped_shell);
  for (TopExp_Explorer explorer(shell, TopAbs_FACE); explorer.More(); explorer.Next())
   builder.Add(capped_shell, explorer.Current());
  for (const TopoDS_Face& face : cap_faces) builder.Add(capped_shell, face);
  ShapeFix_Shell fixer(capped_shell);
  fixer.Perform();
  capped_shell = fixer.Shell();
  TopTools_IndexedMapOfShape free_edges;
  if (naked_edge_count(capped_shell, free_edges) != 0) {
   if (!BRepCheck_Analyzer(capped_shell).IsValid())
    throw std::runtime_error("cap: partially capped shell is invalid");
   return emit(capped_shell, deflection, max_points, max_nodes, max_bytes,
               sink, context, message, capacity);
  }
  BRepBuilderAPI_MakeSolid solid(capped_shell);
  if (!solid.IsDone()) throw std::runtime_error("cap: cannot construct solid");
  ShapeFix_Solid solid_fixer(solid.Solid());
  solid_fixer.Perform();
  const TopoDS_Shape fixed_shape = solid_fixer.Solid();
  if (!BRepCheck_Analyzer(fixed_shape).IsValid())
   throw std::runtime_error("cap: resulting solid is invalid");
  const TopoDS_Solid capped_solid = TopoDS::Solid(fixed_shape);
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

extern "C" int om9_occt_fillet_radii(const char* source, size_t length, const int* edges,
                                     const double* radii, size_t count, double deflection,
                                     size_t max_points, size_t max_nodes, size_t max_bytes,
                                     Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!edges || !radii || count == 0) throw std::invalid_argument("fillet: no edges");
  const TopoDS_Shape shape = read_shape(source, length, "fillet shape");
  TopTools_IndexedMapOfShape naked;
  naked_edge_count(shape, naked);
  BRepFilletAPI_MakeFillet maker(shape);
  std::vector<std::pair<int, double>> contour_radii;
  for (size_t i = 0; i < count; ++i) {
   if (!std::isfinite(radii[i]) || radii[i] <= 0)
    throw std::invalid_argument("fillet: radius must be positive");
   const TopoDS_Edge edge = edge_at(shape, edges[i], "fillet");
   if (naked.Contains(edge)) throw std::invalid_argument("fillet: edge has fewer than two adjacent faces");
   int contour = maker.Contour(edge);
   if (contour != 0) {
    auto found = std::find_if(contour_radii.begin(), contour_radii.end(),
                              [contour](const auto& item) { return item.first == contour; });
    if (found == contour_radii.end() || found->second != radii[i])
     throw std::invalid_argument("fillet: different radii on one tangent contour are unsupported");
    continue;
   }
   maker.Add(radii[i], edge);
   contour = maker.Contour(edge);
   if (contour == 0) throw std::runtime_error("fillet: cannot add edge");
   contour_radii.emplace_back(contour, radii[i]);
  }
  maker.Build();
  if (!maker.IsDone()) throw std::runtime_error("fillet failed");
  const TopoDS_Shape result = maker.Shape();
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid())
   throw std::runtime_error("fillet produced an invalid shape");
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown fillet failure");
  return 2;
 }
}

extern "C" int om9_occt_oriented_edge_endpoints(const char* source, size_t length, int index,
                                                 double* points, char* message, int capacity) {
 try {
  if (!points) throw std::invalid_argument("edge endpoints: missing output");
  const TopoDS_Shape shape = read_shape(source, length, "edge endpoints");
  const TopoDS_Edge edge = edge_at(shape, index, "edge endpoints");
  TopoDS_Vertex first, last;
  TopExp::Vertices(edge, first, last, true);
  if (first.IsNull() || last.IsNull())
   throw std::invalid_argument("edge endpoints: missing vertex");
  if (first.IsSame(last)) return 3; // A closed edge has one topological endpoint.
  const gp_Pnt a = BRep_Tool::Pnt(first);
  const gp_Pnt b = BRep_Tool::Pnt(last);
  points[0] = a.X(); points[1] = a.Y(); points[2] = a.Z();
  points[3] = b.X(); points[4] = b.Y(); points[5] = b.Z();
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown edge endpoints failure");
  return 2;
 }
}

extern "C" int om9_occt_edge_is_line(const char* source, size_t length, int index,
                                       int* is_line, char* message, int capacity) {
 try {
  if (!is_line) throw std::invalid_argument("edge type: missing output");
  const TopoDS_Shape shape = read_shape(source, length, "edge type");
  const TopoDS_Edge edge = edge_at(shape, index, "edge type");
  *is_line = BRepAdaptor_Curve(edge).GetType() == GeomAbs_Line ? 1 : 0;
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown edge type failure");
  return 2;
 }
}

// Extract a bounded circular edge without retaining native handles beyond this call.
extern "C" int om9_occt_circular_edge(const char* source, size_t length, int index,
                                        double* output, char* message, int capacity) {
 try {
  if (!output) throw std::invalid_argument("circular edge: missing output");
  const TopoDS_Shape shape = read_shape(source, length, "circular edge");
  const TopoDS_Edge edge = edge_at(shape, index, "circular edge");
  BRepAdaptor_Curve adaptor(edge);
  if (adaptor.GetType() != GeomAbs_Circle) throw std::invalid_argument("edge is not circular");
  const gp_Circ circle = adaptor.Circle();
  const gp_Pnt center = circle.Location();
  const gp_Dir x = circle.Position().XDirection(), y = circle.Position().YDirection();
  const double first = adaptor.FirstParameter(), last = adaptor.LastParameter();
  if (!std::isfinite(first) || !std::isfinite(last) || first >= last)
   throw std::invalid_argument("circular edge: invalid domain");
  output[0] = center.X(); output[1] = center.Y(); output[2] = center.Z();
  output[3] = x.X(); output[4] = x.Y(); output[5] = x.Z();
  output[6] = y.X(); output[7] = y.Y(); output[8] = y.Z();
  output[9] = circle.Radius(); output[10] = first; output[11] = last;
  output[12] = edge.Orientation() == TopAbs_REVERSED ? 1.0 : 0.0;
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown circular edge failure"); return 2;
 }
}

extern "C" int om9_occt_fillet_project_edge(const char* source, size_t length, int index,
                                             const double* point, double* output, char* message, int capacity) {
 try {
  if (!point || !output || !std::isfinite(point[0]) || !std::isfinite(point[1]) || !std::isfinite(point[2]))
   throw std::invalid_argument("fillet edge projection: invalid point");
  const TopoDS_Shape shape = read_shape(source, length, "fillet edge projection");
  const TopoDS_Edge edge = edge_at(shape, index, "fillet edge projection");
  TopoDS_Vertex first_vertex, last_vertex;
  TopExp::Vertices(edge, first_vertex, last_vertex, true);
  if (first_vertex.IsNull() || last_vertex.IsNull() || first_vertex.IsSame(last_vertex))
   throw std::invalid_argument("fillet edge projection: open edge required");
  double first, last;
  const auto curve = BRep_Tool::Curve(edge, first, last);
  if (curve.IsNull() || !std::isfinite(first) || !std::isfinite(last) || first >= last)
   throw std::invalid_argument("fillet edge projection: invalid curve");
  GeomAPI_ProjectPointOnCurve projection(gp_Pnt(point[0], point[1], point[2]), curve, first, last);
  if (projection.NbPoints() == 0) throw std::runtime_error("fillet edge projection failed");
  const double parameter = projection.LowerDistanceParameter();
  BRepAdaptor_Curve adaptor(edge);
  const double total = GCPnts_AbscissaPoint::Length(adaptor, first, last, 1e-9);
  const double partial = GCPnts_AbscissaPoint::Length(adaptor, first, parameter, 1e-9);
  if (!std::isfinite(total) || !std::isfinite(partial) || total <= 1e-12)
   throw std::runtime_error("fillet edge projection: invalid length");
  const double fraction = edge.Orientation() == TopAbs_REVERSED ? 1.0 - partial / total : partial / total;
  const gp_Pnt projected = projection.NearestPoint();
  output[0] = fraction; output[1] = projected.X(); output[2] = projected.Y(); output[3] = projected.Z();
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown fillet projection failure"); return 2;
 }
}

extern "C" int om9_occt_fillet_shared_endpoints(const char* source, size_t length,
                                                int anchor_index, int side, const int* candidates,
                                                size_t count, unsigned char* masks, char* message, int capacity) {
 try {
  if (!candidates || !masks || count == 0 || (side != 0 && side != 1)) throw std::invalid_argument("fillet shared endpoint: invalid argument");
  const TopoDS_Shape shape = read_shape(source, length, "fillet shared endpoint");
  TopTools_IndexedMapOfShape edge_map;
  TopExp::MapShapes(shape, TopAbs_EDGE, edge_map);
  if (anchor_index < 0 || anchor_index >= edge_map.Extent()) throw std::invalid_argument("fillet shared endpoint: invalid anchor");
  const TopoDS_Edge anchor = TopoDS::Edge(edge_map(anchor_index + 1));
  TopoDS_Vertex a, b;
  TopExp::Vertices(anchor, a, b, true);
  if (a.IsNull() || b.IsNull())
   throw std::invalid_argument("fillet shared endpoint: missing vertex");
  BRepFilletAPI_MakeFillet maker(shape);
  maker.Add(anchor);
  const int contour = maker.Contour(anchor);
  if (contour == 0) throw std::runtime_error("fillet shared endpoint: cannot add edge");
  const TopoDS_Vertex& target = side == 0 ? a : b;
  for (size_t i = 0; i < count; ++i) {
   if (candidates[i] < 0 || candidates[i] >= edge_map.Extent()) throw std::invalid_argument("fillet shared endpoint: invalid candidate");
   const TopoDS_Edge candidate = TopoDS::Edge(edge_map(candidates[i] + 1));
   TopoDS_Vertex c, d;
   TopExp::Vertices(candidate, c, d, true);
   if (c.IsNull() || d.IsNull()) throw std::invalid_argument("fillet shared endpoint: missing vertex");
   masks[i] = 0;
   if (maker.Contour(candidate) == contour) {
    if (target.IsSame(c)) masks[i] |= 1;
    if (target.IsSame(d)) masks[i] |= 2;
   }
  }
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown shared endpoint failure"); return 2;
 }
}

extern "C" int om9_occt_fillet_end_radii(const char* source, size_t length, const int* edges,
                                         const double* starts, const double* ends, size_t count,
                                         const int* control_edges, const double* fractions,
                                         const double* control_radii, size_t control_count,
                                         double deflection, size_t max_points, size_t max_nodes,
                                         size_t max_bytes, Om9BrepSink sink, void* context,
                                         char* message, int capacity) {
 try {
  if (!edges || !starts || !ends || count == 0 || (control_count && (!control_edges || !fractions || !control_radii))) throw std::invalid_argument("fillet: no edges or controls");
  const TopoDS_Shape shape = read_shape(source, length, "fillet shape");
  TopTools_IndexedMapOfShape naked;
  naked_edge_count(shape, naked);
  BRepFilletAPI_MakeFillet maker(shape);
  struct FilletInput { TopoDS_Edge edge; int index; double start, end; int contour; };
  std::vector<FilletInput> selected;
  selected.reserve(count);
  for (size_t i = 0; i < count; ++i) {
   if (!std::isfinite(starts[i]) || !std::isfinite(ends[i]) || starts[i] <= 0 || ends[i] <= 0)
    throw std::invalid_argument("fillet: radii must be positive");
   const TopoDS_Edge edge = edge_at(shape, edges[i], "fillet");
   if (naked.Contains(edge)) throw std::invalid_argument("fillet: edge has fewer than two adjacent faces");
   int contour = maker.Contour(edge);
   if (contour == 0) { maker.Add(edge); contour = maker.Contour(edge); }
   if (contour == 0) throw std::runtime_error("fillet: cannot add edge");
   selected.push_back({edge, edges[i], starts[i], ends[i], contour});
  }
  for (size_t i = 0; i < control_count; ++i) {
   if (!std::isfinite(fractions[i]) || fractions[i] <= 0 || fractions[i] >= 1 ||
       !std::isfinite(control_radii[i]) || control_radii[i] <= 0 ||
       std::none_of(selected.begin(), selected.end(), [&](const auto& item) { return item.index == control_edges[i]; }))
    throw std::invalid_argument("fillet: invalid radius control");
  }
  for (int contour = 1; contour <= maker.NbContours(); ++contour) {
   std::vector<const FilletInput*> members;
   for (const auto& item : selected) if (item.contour == contour) members.push_back(&item);
   if (members.empty()) throw std::runtime_error("fillet: empty contour");
   const double radius = members.front()->start;
   const bool has_controls = std::any_of(members.begin(), members.end(), [&](const auto* item) {
    for (size_t i = 0; i < control_count; ++i) if (control_edges[i] == item->index) return true;
    return false;
   });
   const bool uniform = !has_controls && std::all_of(members.begin(), members.end(), [radius](const auto* item) {
    return item->start == radius && item->end == radius;
   });
   if (uniform) { maker.SetRadius(radius, contour, 1); continue; }
   if (maker.Closed(contour) || static_cast<int>(members.size()) != maker.NbEdges(contour))
    throw std::invalid_argument("fillet: variable radius requires every edge of an open contour");
   std::vector<std::pair<double, double>> law;
   for (const auto* item : members) {
    TopoDS_Vertex first, last;
    TopExp::Vertices(item->edge, first, last, true);
    if (first.IsNull() || last.IsNull() || first.IsSame(last))
     throw std::invalid_argument("fillet: variable radius requires distinct edge endpoints");
    for (const auto& point : {std::make_pair(first, item->start), std::make_pair(last, item->end)}) {
     const double u = maker.RelativeAbscissa(contour, point.first);
     if (!std::isfinite(u) || u < -1e-9 || u > 1.0 + 1e-9)
      throw std::runtime_error("fillet: cannot map endpoint to contour");
     law.emplace_back(std::clamp(u, 0.0, 1.0), point.second);
    }
    const double u0 = maker.RelativeAbscissa(contour, first);
    const double u1 = maker.RelativeAbscissa(contour, last);
    for (size_t i = 0; i < control_count; ++i) {
     if (control_edges[i] == item->index)
      law.emplace_back(std::clamp(u0 + fractions[i] * (u1 - u0), 0.0, 1.0), control_radii[i]);
    }
   }
   std::sort(law.begin(), law.end());
   std::vector<std::pair<double, double>> unique;
   for (const auto& point : law) {
    if (!unique.empty() && std::abs(point.first - unique.back().first) < 1e-9) {
     if (std::abs(point.second - unique.back().second) > 1e-10 * std::max({1.0, point.second, unique.back().second}))
      throw std::invalid_argument("fillet: inconsistent radii at a shared contour vertex");
    } else unique.push_back(point);
   }
   if (unique.size() < 2 || unique.front().first > 1e-9 || unique.back().first < 1.0 - 1e-9)
    throw std::invalid_argument("fillet: incomplete contour radius law");
   if (unique.size() == 2) {
    maker.SetRadius(unique.front().second, unique.back().second, contour, 1);
   } else {
    if (unique.size() > static_cast<size_t>(std::numeric_limits<int>::max()))
     throw std::length_error("fillet: too many radius controls");
    NCollection_Array1<gp_Pnt2d> points(1, static_cast<int>(unique.size()));
    for (size_t i = 0; i < unique.size(); ++i) points.SetValue(static_cast<int>(i + 1), gp_Pnt2d(unique[i].first, unique[i].second));
    maker.SetRadius(points, contour, 1);
   }
  }
  maker.Build();
  if (!maker.IsDone()) throw std::runtime_error("fillet failed");
  const TopoDS_Shape result = maker.Shape();
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid())
   throw std::runtime_error("fillet produced an invalid shape");
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown variable fillet failure");
  return 2;
 }
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

extern "C" int om9_occt_remove_faces(const char* source, size_t length, const int* indices,
                                     size_t count, double deflection, size_t max_points,
                                     size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
                                     void* context, char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "remove faces");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  if (!indices || count == 0 || count >= static_cast<size_t>(faces.Extent()))
   throw std::invalid_argument("remove faces: invalid count");
  std::vector<bool> removed(static_cast<size_t>(faces.Extent()), false);
  for (size_t i = 0; i < count; ++i) {
   if (indices[i] < 0 || indices[i] >= faces.Extent() || removed[indices[i]])
    throw std::invalid_argument("remove faces: invalid or duplicate index");
   removed[indices[i]] = true;
  }
  BRep_Builder builder;
  TopoDS_Shell shell;
  builder.MakeShell(shell);
  for (int i = 1; i <= faces.Extent(); ++i)
   if (!removed[static_cast<size_t>(i - 1)]) builder.Add(shell, faces(i));
  if (!BRepCheck_Analyzer(shell).IsValid())
   throw std::runtime_error("remove faces: invalid remainder topology");
  return emit(shell, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown remove-faces failure");
  return 2;
 }
}

extern "C" int om9_occt_sew(const char* source_a, size_t length_a, const char* source_b,
                            size_t length_b, int reverse_b, double tolerance, double deflection, size_t max_points,
                            size_t max_nodes, size_t max_bytes, Om9BrepSink sink, void* context,
                            char* message, int capacity) {
 try {
  if (!std::isfinite(tolerance) || tolerance <= 0)
   throw std::invalid_argument("sew: tolerance must be positive");
  const TopoDS_Shape a = read_shape(source_a, length_a, "sew argument");
  TopoDS_Shape b = read_shape(source_b, length_b, "sew tool");
  if (reverse_b != 0) b.Reverse();
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

extern "C" int om9_occt_shell(const char* source, size_t length, const size_t* face_indices, size_t face_count, double thickness,
                              double deflection, size_t max_points, size_t max_nodes,
                              size_t max_bytes, Om9BrepSink sink, void* context, char* message,
                              int capacity) {
 try {
  if (!source || !length || !face_indices || !face_count || !std::isfinite(thickness) || thickness <= 0)
   throw std::invalid_argument("invalid shell input");
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
   throw std::invalid_argument("invalid shell source");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  if (face_count >= size_t(faces.Extent())) throw std::invalid_argument("shell requires a retained face");
  TopTools_ListOfShape closing;
  for (size_t i = 0; i < face_count; ++i) {
   if (face_indices[i] >= size_t(faces.Extent())) throw std::invalid_argument("shell face index");
   closing.Append(faces.FindKey(int(face_indices[i]) + 1));
  }
  BRepOffsetAPI_MakeThickSolid thick;
  thick.MakeThickSolidByJoin(shape, closing, -thickness, 1e-6, BRepOffset_Skin, false, false,
                             GeomAbs_Intersection, true);
  thick.Build();
  if (!thick.IsDone()) throw std::runtime_error("shell construction failed");
  if (thick.Shape().IsNull() || !BRepCheck_Analyzer(thick.Shape()).IsValid())
   throw std::runtime_error("shell construction produced invalid geometry");
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
