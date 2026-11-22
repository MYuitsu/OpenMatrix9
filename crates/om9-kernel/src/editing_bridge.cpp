// SPDX-License-Identifier: LGPL-2.1-or-later
// M03 OM9-L2-01-10 boolean / split / cap / fillet / offset / extract adapters.
//
// Every entry point takes canonical OCCT sources (or already-admitted typed data), performs one
// complete OCCT construction, and returns one bounded B-Rep snapshot. Exceptions are contained;
// no TopoDS_Shape or Handle crosses into safe Rust. Validation and result oracles stay in Rust.
#include "brep_snapshot.hxx"
#include "axis_box_certificate.hxx"
#include <cstdint>

#include <Bnd_Box.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Check.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepAlgoAPI_Splitter.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepExtrema_ExtPF.hxx>
#include <BOPAlgo_MakerVolume.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_FindPlane.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <Geom_Plane.hxx>
#include <GeomLib_IsPlanarSurface.hxx>
#include <IntCurvesFace_ShapeIntersector.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_Surface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <GeomConvert.hxx>
#include <GeomTools.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepOffsetAPI_MakeThickSolid.hxx>
#include <BRepTools.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
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
#include <TopoDS_Compound.hxx>
#include <TopoDS_Shell.hxx>
#include <ShapeFix_Face.hxx>
#include <ShapeFix_Shell.hxx>
#include <ShapeFix_Solid.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Dir.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Circ.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Vec.hxx>

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdio>
#include <iomanip>
#include <limits>
#include <sstream>
#include <streambuf>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace {

// A point inside the argument and either inside a tool (Intersection) or outside
// every tool (Difference) disproves an empty result. No witness is inconclusive.
bool has_nonempty_witness(const TopoDS_Shape& argument, const TopoDS_Shape& tool, bool intersection) {
 TopTools_IndexedMapOfShape arguments, tools;
 TopExp::MapShapes(argument, TopAbs_SOLID, arguments);
 TopExp::MapShapes(tool, TopAbs_SOLID, tools);
 // Bound the number of classifier calls; native cost per call depends on geometry.
 if (arguments.IsEmpty() || tools.IsEmpty() || arguments.Extent() > 8 || tools.Extent() > 8) return false;
 Bnd_Box bounds;
 BRepBndLib::AddOptimal(argument, bounds, false, true);
 if (bounds.IsVoid() || bounds.IsOpen()) return false;
 double xmin, ymin, zmin, xmax, ymax, zmax;
 bounds.Get(xmin, ymin, zmin, xmax, ymax, zmax);
 for (double x : {0.25, 0.5, 0.75}) for (double y : {0.25, 0.5, 0.75}) for (double z : {0.25, 0.5, 0.75}) {
  const gp_Pnt p(xmin + x * (xmax - xmin), ymin + y * (ymax - ymin), zmin + z * (zmax - zmin));
  bool inside = false;
  for (int i = 1; i <= arguments.Extent() && !inside; ++i) {
   BRepClass3d_SolidClassifier classifier(arguments(i), p, 1e-7);
   inside = classifier.State() == TopAbs_IN;
  }
  if (!inside) continue;
  bool matches = !intersection;
  for (int i = 1; i <= tools.Extent(); ++i) {
   BRepClass3d_SolidClassifier classifier(tools(i), p, 1e-7);
   if (intersection && classifier.State() == TopAbs_IN) { matches = true; break; }
   if (!intersection && classifier.State() != TopAbs_OUT) { matches = false; break; }
  }
  if (matches) return true;
 }
 return false;
}

// A point inside both inputs and outside every result solid disproves a complete
// intersection. This bounded sampling is inconclusive when no witness is found.
bool intersection_has_missing_witness(const TopoDS_Shape& a, const TopoDS_Shape& b, const TopoDS_Shape& result) {
 TopTools_IndexedMapOfShape left, right, output;
 TopExp::MapShapes(a, TopAbs_SOLID, left); TopExp::MapShapes(b, TopAbs_SOLID, right);
 TopExp::MapShapes(result, TopAbs_SOLID, output);
 if (left.IsEmpty() || right.IsEmpty() || output.IsEmpty() || left.Extent()>8 || right.Extent()>8 || output.Extent()>8) return false;
 Bnd_Box ba, bb; BRepBndLib::AddOptimal(a, ba, false, true); BRepBndLib::AddOptimal(b, bb, false, true);
 if (ba.IsVoid() || bb.IsVoid() || ba.IsOpen() || bb.IsOpen()) return false;
 double amin[3], amax[3], bmin[3], bmax[3];
 ba.Get(amin[0],amin[1],amin[2],amax[0],amax[1],amax[2]);
 bb.Get(bmin[0],bmin[1],bmin[2],bmax[0],bmax[1],bmax[2]);
 double low[3], span[3];
 for (int i=0;i<3;++i) {
  low[i]=std::max(amin[i],bmin[i]); span[i]=std::min(amax[i],bmax[i])-low[i];
  if (!std::isfinite(low[i]) || !std::isfinite(span[i]) || span[i]<=0.) return false;
 }
 auto inside=[](const TopTools_IndexedMapOfShape& solids, const gp_Pnt& p) {
  for (int i=1;i<=solids.Extent();++i) {
   BRepClass3d_SolidClassifier classifier(solids(i),p,1e-7);
   if (classifier.State()==TopAbs_IN) return true;
  }
  return false;
 };
 for (int axis=0;axis<3;++axis) for (double near : {1./16.,15./16.})
  for (double first : {3./16.,13./16.}) for (double second : {3./16.,13./16.}) {
   double fraction[3]; fraction[axis]=near; fraction[(axis+1)%3]=first; fraction[(axis+2)%3]=second;
   const gp_Pnt p(low[0]+span[0]*fraction[0],low[1]+span[1]*fraction[1],low[2]+span[2]*fraction[2]);
   if (!inside(left,p) || !inside(right,p)) continue;
   bool outside=true;
   for (int i=1;i<=output.Extent() && outside;++i) {
    BRepClass3d_SolidClassifier classifier(output(i),p,1e-7);
    outside=classifier.State()==TopAbs_OUT;
   }
   if (outside) return true;
  }
 return false;
}

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

static int boolean_dispatch(int op, int require_solids, const char* source_a, size_t length_a,
                                const char* source_b, size_t length_b, double deflection,
                                size_t max_points, size_t max_nodes, size_t max_bytes,
                                Om9BrepSink sink, void* context, char* message, int capacity,
                                Om9SourceSink source_sink) {
 try {
  TopoDS_Shape a = read_shape(source_a, length_a, "boolean argument");
  TopoDS_Shape b = read_shape(source_b, length_b, "boolean tool");
  if (require_solids) {
   TopTools_IndexedMapOfShape solids_a, solids_b;
   TopExp::MapShapes(a, TopAbs_SOLID, solids_a);
   TopExp::MapShapes(b, TopAbs_SOLID, solids_b);
   if (solids_a.IsEmpty() || solids_b.IsEmpty())
    throw std::invalid_argument("boolean requires two closed solids");
  }
  if ((require_solids > 1 && op == 2) || (require_solids > 2 && op == 0)) {
   // The certified operand is temporary; all source-based witness checks below
   // still read and classify the original immutable inputs.
   TopoDS_Shape replacement=om9_axis_box::make_operand(a);
   if (!replacement.IsNull()) a=replacement;
   replacement=om9_axis_box::make_operand(b);
   if (!replacement.IsNull()) b=replacement;
  }
  TopTools_ListOfShape arguments, tools;
  arguments.Append(a);
  tools.Append(b);
  TopoDS_Shape result;
  switch (op) {
   case 0: {
    BRepAlgoAPI_Fuse fuse;
    fuse.SetArguments(arguments);
    fuse.SetTools(tools);
    fuse.SetUseOBB(true);
    fuse.Build();
    if (!fuse.IsDone() || fuse.HasErrors()) throw std::runtime_error("boolean union failed");
    result = fuse.Shape();
    break;
   }
   case 1: {
    BRepAlgoAPI_Cut cut;
    cut.SetArguments(arguments);
    cut.SetTools(tools);
    cut.SetUseOBB(true);
    cut.Build();
    if (!cut.IsDone() || cut.HasErrors()) throw std::runtime_error("boolean difference failed");
    result = cut.Shape();
    break;
   }
   case 2: {
    BRepAlgoAPI_Common common;
    common.SetArguments(arguments);
    common.SetTools(tools);
    common.SetUseOBB(true);
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
  if (result.IsNull() || faces.Extent() == 0) {
   if (require_solids > 1 && op == 1 && has_nonempty_witness(a, b, false))
    throw std::runtime_error("boolean difference returned empty despite an interior witness");
   if (require_solids > 1 && op == 2) {
    const TopoDS_Shape original_a=read_shape(source_a,length_a,"intersection original argument");
    const TopoDS_Shape original_b=read_shape(source_b,length_b,"intersection original tool");
    if (has_nonempty_witness(original_a, original_b, true))
     throw std::runtime_error("boolean intersection returned empty despite an interior witness");
   }
   return 3;
  }
  // Value 2 requests the conservative publication check for user Boolean commands;
  // value 1 retains the existing behavior for construction operations such as Pipe.
  if (require_solids > 1) {
   BRepAlgoAPI_Check check(result, true, true);
   if (!check.IsValid()) {
    bool aborted = false;
    bool small_edge = false;
    for (NCollection_List<BOPAlgo_CheckResult>::Iterator it(check.Result()); it.More(); it.Next()) {
     aborted |= it.Value().GetCheckStatus() == BOPAlgo_OperationAborted;
     small_edge |= it.Value().GetCheckStatus() == BOPAlgo_TooSmallEdge;
    }
    if (aborted) throw std::runtime_error("boolean result failed OCCT validity checks: self-interference check aborted");
    if (small_edge) throw std::runtime_error("boolean result failed OCCT validity checks: small edge detected");
    throw std::runtime_error("boolean result failed OCCT validity checks");
   }
   if (op == 2) {
    // OCCT Boolean may alter its in-memory operands: witnesses must classify the
    // canonical inputs, re-read from the still-live immutable source buffers.
    const TopoDS_Shape original_a=read_shape(source_a,length_a,"intersection original argument");
    const TopoDS_Shape original_b=read_shape(source_b,length_b,"intersection original tool");
    if (has_nonempty_witness(result, original_a, false) || has_nonempty_witness(result, original_b, false))
     throw std::runtime_error("boolean intersection result extends outside an input solid");
    if (intersection_has_missing_witness(original_a, original_b, result))
     throw std::runtime_error("boolean intersection result misses an interior witness");
   }
  }
   if (source_sink) return om9_emit_source(result, max_nodes, max_bytes, source_sink, context);
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

extern "C" int om9_occt_boolean(int op, int require_solids, const char* a, size_t alen,
    const char* b, size_t blen, double deflection, size_t points, size_t nodes, size_t bytes,
    Om9BrepSink sink, void* context, char* message, int capacity) {
 return boolean_dispatch(op, require_solids, a, alen, b, blen, deflection, points, nodes, bytes,
                         sink, context, message, capacity, nullptr);
}
extern "C" int om9_occt_boolean_source(int certified, const char* a, size_t alen,
    const char* b, size_t blen, size_t nodes, size_t bytes, Om9SourceSink sink,
    void* context, char* message, int capacity) {
 return boolean_dispatch(0, certified ? 3 : 2, a, alen, b, blen, 0., 0, nodes, bytes,
                         nullptr, context, message, capacity, sink);
}

// Status 4 means boxes overlap or cannot be bounded: use the general tool-union path.
extern "C" int om9_occt_cut_disjoint_tools(
    const char* source, size_t source_len, const char* const* tool_sources,
    const size_t* lengths, size_t count, double deflection, size_t max_points,
    size_t max_nodes, size_t max_bytes, Om9BrepSink sink, void* context,
    char* message, int capacity) {
 try {
  if (!tool_sources || !lengths || count == 0 || count > 128 || count > max_nodes)
   throw std::invalid_argument("invalid boolean tools");
  const TopoDS_Shape target = read_shape(source, source_len, "boolean argument");
  TopTools_IndexedMapOfShape solids;
  TopExp::MapShapes(target, TopAbs_SOLID, solids);
  if (solids.IsEmpty()) throw std::invalid_argument("boolean requires solid target");
  TopTools_ListOfShape arguments, tools;
  arguments.Append(target);
  std::vector<Bnd_Box> bounds;
  bounds.reserve(count);
  for (size_t i = 0; i < count; ++i) {
   const TopoDS_Shape tool = read_shape(tool_sources[i], lengths[i], "boolean tool");
   TopTools_IndexedMapOfShape tool_solids;
   TopExp::MapShapes(tool, TopAbs_SOLID, tool_solids);
   if (tool_solids.IsEmpty()) throw std::invalid_argument("boolean requires solid tool");
   Bnd_Box box;
   BRepBndLib::AddOptimal(tool, box, false, true);
   if (box.IsVoid() || box.IsOpen()) return 4;
   for (const Bnd_Box& existing : bounds) if (!box.IsOut(existing)) return 4;
   bounds.push_back(box);
   tools.Append(tool);
  }
  BRepAlgoAPI_Cut cut;
  cut.SetArguments(arguments);
  cut.SetTools(tools);
  cut.SetUseOBB(true);
  cut.Build();
  if (!cut.IsDone() || cut.HasErrors()) throw std::runtime_error("boolean difference failed");
  const TopoDS_Shape result = cut.Shape();
  TopTools_IndexedMapOfShape faces;
  if (!result.IsNull()) TopExp::MapShapes(result, TopAbs_FACE, faces);
  if (result.IsNull() || faces.IsEmpty()) {
   // The witness helper also caps the total number of solids it classifies.
   if (count <= 8) {
    BRep_Builder builder;
    TopoDS_Compound tool_group;
    builder.MakeCompound(tool_group);
    for (TopTools_ListOfShape::Iterator it(tools); it.More(); it.Next()) builder.Add(tool_group, it.Value());
    if (has_nonempty_witness(target, tool_group, false))
     throw std::runtime_error("boolean difference returned empty despite an interior witness");
   }
   return 3;
  }
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown multi-tool boolean failure"); return 2;
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
   // Traverse the edges as oriented on the shell. A reversed edge starts at its last parameter.
   TopExp::Vertices(chain.front(), start_vertex, end_vertex, true);
   if (start_vertex.IsNull() || end_vertex.IsNull())
    throw std::runtime_error("cap: naked edge has no vertex");
   while (!end_vertex.IsSame(start_vertex)) {
    bool extended = false;
    for (int i = 1; i <= naked.Extent(); ++i) {
     if (used[i]) continue;
     const TopoDS_Edge& edge = TopoDS::Edge(naked(i));
     TopoDS_Vertex v1, v2;
     TopExp::Vertices(edge, v1, v2, true);
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

// Transfer exact curves from a caller-owned edge map. The sink borrows the
// arrays only for this call; no native handle escapes.
static void emit_edge_curves(const TopTools_IndexedMapOfShape& edges, const int* indices, size_t count,
    size_t max_points, size_t max_nodes, size_t max_bytes, int max_degree,
    int (*sink)(void*, int, const double*, const double*, size_t, const double*, size_t),
    void* context) {
  if (!sink || !count || count > max_nodes || max_degree <= 0 || (!indices && count != size_t(edges.Extent()))) throw std::invalid_argument("edge curve: invalid output");
  for (size_t item = 0; item < count; ++item) {
  const int index = indices ? indices[item] : static_cast<int>(item);
  if (index < 0 || index >= edges.Extent()) throw std::invalid_argument("edge curve: index out of range");
  const TopoDS_Edge edge = TopoDS::Edge(edges(index + 1));
  if (BRep_Tool::Degenerated(edge)) throw std::invalid_argument("edge curve: degenerate edge");
  double first, last;
  TopLoc_Location location;
  auto curve = BRep_Tool::Curve(edge, location, first, last);
  if (curve.IsNull() || !std::isfinite(first) || !std::isfinite(last) || first >= last)
   throw std::invalid_argument("edge curve: invalid source or domain");
  curve = Handle(Geom_Curve)::DownCast(curve->Transformed(location.Transformation()));
  auto spline = Handle(Geom_BSplineCurve)::DownCast(curve);
  Handle(Geom_BSplineCurve) result;
  if (!spline.IsNull()) {
   result = Handle(Geom_BSplineCurve)::DownCast(spline->Copy());
   result->Segment(first, last);
  } else {
   result = GeomConvert::CurveToBSplineCurve(new Geom_TrimmedCurve(curve, first, last));
  }
  if (result.IsNull()) throw std::runtime_error("edge curve: conversion failed");
  if (result->IsPeriodic()) result->SetNotPeriodic();
  if (edge.Orientation() == TopAbs_REVERSED) result->Reverse();
  const int poles = result->NbPoles(), degree = result->Degree();
  if (poles <= 0 || degree <= 0 || degree > max_degree || size_t(poles) > max_points)
   throw std::length_error("edge curve: pole or degree budget");
  size_t knots = 0;
  for (int i = 1; i <= result->NbKnots(); ++i) {
   const int multiplicity = result->Multiplicity(i);
   if (multiplicity <= 0 || size_t(multiplicity) > max_nodes - knots)
    throw std::length_error("edge curve: knot budget");
   knots += size_t(multiplicity);
  }
  if (knots != size_t(poles) + size_t(degree) + 1 || size_t(poles) > max_bytes / (4 * sizeof(double)) ||
      knots > (max_bytes - size_t(poles) * 4 * sizeof(double)) / sizeof(double))
   throw std::length_error("edge curve: output budget");
  std::vector<double> points(size_t(poles) * 3), weights(static_cast<size_t>(poles)), expanded;
  expanded.reserve(knots);
  for (int i = 1; i <= poles; ++i) {
   const gp_Pnt p = result->Pole(i);
   points[size_t(i - 1) * 3] = p.X(); points[size_t(i - 1) * 3 + 1] = p.Y(); points[size_t(i - 1) * 3 + 2] = p.Z();
   weights[size_t(i - 1)] = result->Weight(i);
  }
  for (int i = 1; i <= result->NbKnots(); ++i)
   for (int j = 0; j < result->Multiplicity(i); ++j) expanded.push_back(result->Knot(i));
  if (sink(context, degree, points.data(), weights.data(), size_t(poles), expanded.data(), expanded.size()))
   throw std::runtime_error("edge curve: output refused");
  }
}

class FaceWitnessMiss : public std::invalid_argument {
 public:
  using std::invalid_argument::invalid_argument;
};

int resolve_source_face(const TopTools_IndexedMapOfShape& faces, const double* triangle,
                        double tolerance, double vertex_tolerance, size_t max_nodes);

// Resolve a picked face from its source witness, never a persisted display index.
static TopoDS_Face isocurve_single_face(const char* source, size_t length, size_t max_nodes,
                                        const double* triangle, double witness_deflection, double tolerance) {
 const TopoDS_Shape shape = read_shape(source, length, "isocurve");
 TopTools_IndexedMapOfShape faces, solids, edges;
 TopExp::MapShapes(shape, TopAbs_FACE, faces);
 TopExp::MapShapes(shape, TopAbs_SOLID, solids);
 TopExp::MapShapes(shape, TopAbs_EDGE, edges);
 if (size_t(edges.Extent())>max_nodes)
  throw std::invalid_argument("isocurve requires one open B-rep face within the topology limit");
 if (triangle) {
  const int index=resolve_source_face(faces,triangle,witness_deflection+tolerance,tolerance,max_nodes);
  return TopoDS::Face(faces(index+1));
 }
 if (faces.Extent()!=1 || !solids.IsEmpty())
  throw std::invalid_argument("isocurve requires one open B-rep face within the topology limit");
 return TopoDS::Face(faces(1));
}

extern "C" int om9_occt_isocurve_face_bounds(const char* source, size_t length, size_t max_nodes,
 const double* triangle, double witness_deflection, double tolerance, double* bounds, char* message, int capacity) {
 try {
  if (!bounds) throw std::invalid_argument("isocurve: missing bounds");
  const TopoDS_Face face=isocurve_single_face(source,length,max_nodes,triangle,witness_deflection,tolerance);
  BRepTools::UVBounds(face,bounds[0],bounds[1],bounds[2],bounds[3]);
  if (!std::isfinite(bounds[0]) || !std::isfinite(bounds[1]) || !std::isfinite(bounds[2]) ||
      !std::isfinite(bounds[3]) || bounds[0]>=bounds[1] || bounds[2]>=bounds[3])
   throw std::invalid_argument("isocurve: invalid face bounds");
  return 0;
 } catch (const Standard_Failure& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) { if (message && capacity>0) std::snprintf(message,capacity,"Unknown isocurve failure"); return 2; }
}

// Resolve the exact face in this native read. No display triangle or face
// index is carried across native topology maps.
static TopoDS_Face isocurve_ray_face(const char* source, size_t length, size_t max_nodes,
 const double* origin, const double* direction, const double* reference, double tolerance, double max_distance, double* resolved,
 const double* witness, double witness_deflection, double witness_tolerance) {
  if (!source || !length || !origin || !direction || !reference || !std::isfinite(tolerance) || tolerance<=0. ||
      !std::isfinite(max_distance) || max_distance<=0.) throw std::invalid_argument("isocurve ray: invalid input");
  for (int i=0;i<3;++i) if (!std::isfinite(origin[i]) || !std::isfinite(direction[i]) || !std::isfinite(reference[i]))
   throw std::invalid_argument("isocurve ray: non-finite input");
  const gp_Vec vector(direction[0],direction[1],direction[2]);
  if (vector.SquareMagnitude()<=1e-24) throw std::invalid_argument("isocurve ray: zero direction");
  const TopoDS_Shape shape=read_shape(source,length,"isocurve ray");
  TopTools_IndexedMapOfShape faces,edges;
  TopExp::MapShapes(shape,TopAbs_FACE,faces); TopExp::MapShapes(shape,TopAbs_EDGE,edges);
  if (faces.IsEmpty() || size_t(faces.Extent())>max_nodes || size_t(edges.Extent())>max_nodes)
   throw std::length_error("isocurve ray: topology budget");
  IntCurvesFace_ShapeIntersector intersections; intersections.Load(shape,tolerance);
  intersections.Perform(gp_Lin(gp_Pnt(origin[0],origin[1],origin[2]),gp_Dir(vector)),0.,RealLast());
  if (!intersections.IsDone()) throw std::runtime_error("isocurve ray: intersection failed");
  if (size_t(intersections.NbPnt())>max_nodes) throw std::length_error("isocurve ray: hit budget");
  int witness_index=-1;
  if (witness) {
   if (!std::isfinite(witness_deflection) || witness_deflection<0. || !std::isfinite(witness_tolerance) || witness_tolerance<=0. ||
       !std::isfinite(witness_deflection+witness_tolerance))
    throw std::invalid_argument("isocurve ray: invalid face witness");
   witness_index=resolve_source_face(faces,witness,witness_deflection+witness_tolerance,witness_tolerance,max_nodes);
  }
  const gp_Pnt ref(reference[0],reference[1],reference[2]); int best=0; double best_distance=RealLast();
  for (int i=1;i<=intersections.NbPnt();++i) {
   const double distance=ref.Distance(intersections.Pnt(i));
   if (!std::isfinite(distance)) throw std::invalid_argument("isocurve ray: invalid distance");
   if (distance<best_distance) {best=i;best_distance=distance;}
  }
  if (!best || best_distance>max_distance) throw FaceWitnessMiss("isocurve ray: no matching face");
  if (witness_index>=0 && !intersections.Face(best).IsSame(faces(witness_index+1)))
   throw std::invalid_argument("isocurve ray: selected face mismatch");
  for (int i=1;i<=intersections.NbPnt();++i) if (i!=best) {
   const double distance=ref.Distance(intersections.Pnt(i));
   if (std::abs(distance-best_distance)<=tolerance && !intersections.Face(i).IsSame(intersections.Face(best)))
    throw std::invalid_argument("isocurve ray: ambiguous faces");
  }
  if (resolved) {const gp_Pnt& p=intersections.Pnt(best);resolved[0]=p.X();resolved[1]=p.Y();resolved[2]=p.Z();}
  return intersections.Face(best);
}

extern "C" int om9_occt_isocurve_ray_face_bounds(const char* source, size_t length, size_t max_nodes,
 const double* origin, const double* direction, const double* reference, double tolerance, double max_distance,
 const double* witness, double witness_deflection, double witness_tolerance,
 double* bounds, double* resolved, char* message, int capacity) {
 try {
  if (!bounds) throw std::invalid_argument("isocurve ray: missing bounds");
  const TopoDS_Face face=isocurve_ray_face(source,length,max_nodes,origin,direction,reference,tolerance,max_distance,resolved,witness,witness_deflection,witness_tolerance);
  BRepTools::UVBounds(face,bounds[0],bounds[1],bounds[2],bounds[3]);
  if (!std::isfinite(bounds[0]) || !std::isfinite(bounds[1]) || !std::isfinite(bounds[2]) || !std::isfinite(bounds[3]) ||
      bounds[0]>=bounds[1] || bounds[2]>=bounds[3]) throw std::invalid_argument("isocurve ray: invalid face bounds");
  return 0;
 } catch (const FaceWitnessMiss& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 3;
 } catch (const Standard_Failure& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) { if (message && capacity>0) std::snprintf(message,capacity,"Unknown isocurve ray failure"); return 2; }
}

// Snapshot the trimmed face resolved by the same exact viewport ray as ExtractIsocurve.
extern "C" int om9_occt_extract_ray_face(const char* source, size_t length,
 const double* origin, const double* direction, const double* reference, double tolerance,
 double max_distance, double* resolved, double deflection, size_t max_points, size_t max_nodes,
 size_t max_bytes, Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  const auto face=isocurve_ray_face(source,length,max_nodes,origin,direction,reference,tolerance,
                                    max_distance,resolved,nullptr,0.,0.);
  return om9_emit_brep_snapshot(face,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 } catch (const Standard_Failure& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.GetMessageString()); return 2;
 } catch (const std::exception& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) { if (message && capacity>0) std::snprintf(message,capacity,"Boundary face extraction failed"); return 2; }
}

static void emit_face_isocurve(const TopoDS_Face& face, int along_u, double value,
 size_t max_points, size_t max_nodes, size_t max_bytes, int max_degree,
 int (*prepare)(void*,size_t),
 int (*sink)(void*,int,const double*,const double*,size_t,const double*,size_t),
 void* context) {
  if (!prepare || !sink || !std::isfinite(value) || (along_u!=0 && along_u!=1))
   throw std::invalid_argument("isocurve: invalid arguments");
  double u0,u1,v0,v1; BRepTools::UVBounds(face,u0,u1,v0,v1);
  if (!std::isfinite(u0) || !std::isfinite(u1) || !std::isfinite(v0) || !std::isfinite(v1) || u0>=u1 || v0>=v1 ||
      (along_u ? value<=v0 || value>=v1 : value<=u0 || value>=u1))
   throw std::invalid_argument("isocurve: parameter outside face interior");
  const Handle(Geom_Surface) surface=BRep_Tool::Surface(face);
  if (surface.IsNull()) throw std::invalid_argument("isocurve: missing surface");
  const Handle(Geom_Curve) curve=along_u ? surface->VIso(value) : surface->UIso(value);
  if (curve.IsNull()) throw std::invalid_argument("isocurve: missing curve");
  BRepBuilderAPI_MakeEdge make_edge(curve,along_u?u0:v0,along_u?u1:v1);
  if (!make_edge.IsDone()) throw std::runtime_error("isocurve: edge creation failed");
  BRepAlgoAPI_Common common(make_edge.Edge(),face);
  if (!common.IsDone() || common.HasErrors()) throw std::runtime_error("isocurve: face clipping failed");
  TopTools_IndexedMapOfShape edges; TopExp::MapShapes(common.Shape(),TopAbs_EDGE,edges);
  if (edges.IsEmpty() || size_t(edges.Extent())>max_nodes)
   throw std::invalid_argument("isocurve: no admissible clipped curves");
  if (prepare(context,size_t(edges.Extent()))) throw std::runtime_error("isocurve: output refused");
  emit_edge_curves(edges,nullptr,size_t(edges.Extent()),max_points,max_nodes,max_bytes,max_degree,sink,context);
}

// Gather default-density interior isocurves in one native source read. No
// face indices or topology IDs escape this operation; callers must retain the
// immutable source and validate its publication context separately.
extern "C" int om9_occt_default_face_isocurves(const char* source, size_t length,
 size_t max_points, size_t max_nodes, size_t max_bytes, int max_degree,
 int (*checkpoint)(void*),
 int (*prepare)(void*,size_t),
 int (*sink)(void*,int,const double*,const double*,size_t,const double*,size_t),
 void* context, char* message, int capacity) {
 try {
  if (!source || !length || !checkpoint || !prepare || !sink || !max_nodes)
   throw std::invalid_argument("default isocurves: invalid input");
  const TopoDS_Shape shape=read_shape(source,length,"default isocurves");
  TopTools_IndexedMapOfShape faces, curves;
  TopExp::MapShapes(shape,TopAbs_FACE,faces);
  if (faces.IsEmpty() || size_t(faces.Extent())>max_nodes)
   throw std::length_error("default isocurves: face budget");
  size_t requests=0;
  for (int i=1;i<=faces.Extent();++i) {
   if (checkpoint(context)) throw std::runtime_error("default isocurves: canceled or refused");
   const TopoDS_Face face=TopoDS::Face(faces(i));
   double u0,u1,v0,v1; BRepTools::UVBounds(face,u0,u1,v0,v1);
   if (!std::isfinite(u0) || !std::isfinite(u1) || !std::isfinite(v0) || !std::isfinite(v1) || u0>=u1 || v0>=v1)
    throw std::invalid_argument("default isocurves: invalid face bounds");
   const Handle(Geom_Surface) surface=BRep_Tool::Surface(face);
   if (surface.IsNull()) throw std::invalid_argument("default isocurves: missing surface");
   const Handle(Geom_BSplineSurface) spline=Handle(Geom_BSplineSurface)::DownCast(surface);
   for (int axis=0;axis<2;++axis) {
    const double lo=axis==0?u0:v0, hi=axis==0?u1:v1;
    std::vector<double> parameters;
    if (!spline.IsNull()) {
     const int count=axis==0?spline->NbUKnots():spline->NbVKnots();
     if (count<0 || size_t(count)>max_nodes) throw std::length_error("default isocurves: knot budget");
     for (int k=1;k<=count;++k) {
      const double value=axis==0?spline->UKnot(k):spline->VKnot(k);
      if (value>lo && value<hi && (parameters.empty() || parameters.back()!=value)) parameters.push_back(value);
     }
    }
    if (parameters.empty()) {
     const double mid=lo/2.+hi/2.;
     if (!(mid>lo && mid<hi)) throw std::invalid_argument("default isocurves: midpoint precision");
     parameters.push_back(mid);
    }
    if (parameters.size()>max_nodes-requests) throw std::length_error("default isocurves: request budget");
    requests+=parameters.size();
    for (double value:parameters) {
     if (checkpoint(context)) throw std::runtime_error("default isocurves: canceled or refused");
     const Handle(Geom_Curve) curve=axis==0?surface->UIso(value):surface->VIso(value);
     if (curve.IsNull()) throw std::invalid_argument("default isocurves: missing curve");
     BRepBuilderAPI_MakeEdge edge(curve,axis==0?v0:u0,axis==0?v1:u1);
     if (!edge.IsDone()) throw std::runtime_error("default isocurves: edge creation failed");
     BRepAlgoAPI_Common common(edge.Edge(),face);
     if (!common.IsDone() || common.HasErrors()) throw std::runtime_error("default isocurves: face clipping failed");
     if (checkpoint(context)) throw std::runtime_error("default isocurves: canceled or refused");
     TopExp::MapShapes(common.Shape(),TopAbs_EDGE,curves);
     if (size_t(curves.Extent())>max_nodes) throw std::length_error("default isocurves: output budget");
    }
   }
  }
  if (curves.IsEmpty()) throw std::invalid_argument("default isocurves: no clipped curves");
  if (prepare(context,size_t(curves.Extent()))) throw std::runtime_error("default isocurves: output refused");
  emit_edge_curves(curves,nullptr,size_t(curves.Extent()),max_points,max_nodes,max_bytes,max_degree,sink,context);
  return 0;
 } catch (const Standard_Failure& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) { if (message && capacity>0) std::snprintf(message,capacity,"Unknown default isocurve failure"); return 2; }
}

extern "C" int om9_occt_face_isocurve(const char* source, size_t length, int along_u, double value,
 const double* triangle, double witness_deflection, double tolerance,
 size_t max_points, size_t max_nodes, size_t max_bytes, int max_degree,
 int (*prepare)(void*,size_t),
 int (*sink)(void*,int,const double*,const double*,size_t,const double*,size_t),
 void* context, char* message, int capacity) {
 try {
  const TopoDS_Face face=isocurve_single_face(source,length,max_nodes,triangle,witness_deflection,tolerance);
  emit_face_isocurve(face,along_u,value,max_points,max_nodes,max_bytes,max_degree,prepare,sink,context);
  return 0;
 } catch (const Standard_Failure& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) { if (message && capacity>0) std::snprintf(message,capacity,"Unknown isocurve failure"); return 2; }
}

extern "C" int om9_occt_ray_face_isocurve(const char* source, size_t length, int along_u, double value,
 const double* origin, const double* direction, const double* reference, double tolerance, double max_distance,
 const double* witness, double witness_deflection, double witness_tolerance,
 size_t max_points, size_t max_nodes, size_t max_bytes, int max_degree,
 int (*prepare)(void*,size_t),
 int (*sink)(void*,int,const double*,const double*,size_t,const double*,size_t),
 void* context, char* message, int capacity) {
 try {
  const TopoDS_Face face=isocurve_ray_face(source,length,max_nodes,origin,direction,reference,tolerance,max_distance,nullptr,witness,witness_deflection,witness_tolerance);
  emit_face_isocurve(face,along_u,value,max_points,max_nodes,max_bytes,max_degree,prepare,sink,context);
  return 0;
 } catch (const Standard_Failure& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) { if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) { if (message && capacity>0) std::snprintf(message,capacity,"Unknown isocurve failure"); return 2; }
}

extern "C" int om9_occt_edges_nurbs(const char* source, size_t length, const int* indices, size_t count,
    size_t max_points, size_t max_nodes, size_t max_bytes, int max_degree,
    int (*sink)(void*, int, const double*, const double*, size_t, const double*, size_t),
    void* context, char* message, int capacity) {
 try {
  if (!sink || !indices || !count || count > max_nodes || max_degree <= 0)
   throw std::invalid_argument("edge curve: invalid output");
  const TopoDS_Shape shape = read_shape(source, length, "edge curve");
  TopTools_IndexedMapOfShape edges;
  TopExp::MapShapes(shape, TopAbs_EDGE, edges);
  emit_edge_curves(edges, indices, count, max_points, max_nodes, max_bytes, max_degree, sink, context);
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown edge curve failure"); return 2;
 }
}

// Count and emit from the same source read and edge map; no persisted index is trusted.
extern "C" int om9_occt_all_edges_nurbs(const char* source, size_t length, size_t max_edges, int boundary_only,
    size_t max_points, size_t max_nodes, size_t max_bytes, int max_degree,
    int (*prepare)(void*, size_t),
    int (*sink)(void*, int, const double*, const double*, size_t, const double*, size_t),
    void* context, char* message, int capacity) {
 try {
  if (!prepare || !sink) throw std::invalid_argument("edge curve: missing callback");
  const TopoDS_Shape shape = read_shape(source, length, "edge curve");
  TopTools_IndexedMapOfShape edges;
  TopExp::MapShapes(shape, TopAbs_EDGE, edges);
  const size_t total = size_t(edges.Extent());
  if (total > max_edges) throw std::length_error("edge curve: edge count budget");
  if (!total) {
   if (boundary_only == 2) return 0;
   throw std::length_error("edge curve: edge count budget");
  }
  std::vector<int> indices;
  if (total > max_bytes / (2 * sizeof(int))) throw std::length_error("edge curve: index budget");
  indices.reserve(total);
  if (boundary_only == 1) {
   std::vector<int> uses(total, 0);
   for (TopExp_Explorer faces(shape, TopAbs_FACE); faces.More(); faces.Next()) {
    for (TopExp_Explorer face_edges(faces.Current(), TopAbs_EDGE); face_edges.More(); face_edges.Next()) {
     const int index = edges.FindIndex(face_edges.Current());
     if (index <= 0) throw std::runtime_error("edge curve: boundary map mismatch");
     if (uses[size_t(index - 1)] < 2) ++uses[size_t(index - 1)];
    }
   }
   for (size_t i = 0; i < total; ++i) if (uses[i] == 1) indices.push_back(static_cast<int>(i));
  } else {
   for (size_t i = 0; i < total; ++i) {
    const TopoDS_Edge edge = TopoDS::Edge(edges(static_cast<int>(i + 1)));
    if (BRep_Tool::Degenerated(edge)) {
     TopoDS_Vertex first, last;
     TopExp::Vertices(edge, first, last, true);
     double u0, u1; TopLoc_Location loc;
     if (first.IsNull() || last.IsNull() || !first.IsSame(last) || !BRep_Tool::Curve(edge, loc, u0, u1).IsNull())
      throw std::invalid_argument("edge curve: inconsistent degenerate edge");
     continue;
    }
    indices.push_back(static_cast<int>(i));
   }
  }
  const size_t count = indices.size();
  if (!count) {
   if (boundary_only == 2) return 0;
   throw std::invalid_argument(boundary_only == 1 ? "edge curve: no boundary edges" : "edge curve: no nondegenerate edges");
  }
  if (prepare(context, count)) throw std::runtime_error("edge curve: output refused");
  emit_edge_curves(edges, indices.data(), count, max_points, max_nodes, max_bytes, max_degree, sink, context);
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown edge curve failure"); return 2;
 }
}

// Bound a chord's distance to a complete circle by the maximum endpoint error
// plus the sagitta of the chord between projected endpoints. Open/trimmed arcs
// retain the general subdivision check below.
static bool full_circle_chord_within(const TopoDS_Edge& edge, double first, double last,
 const gp_Pnt (&projected)[2], const double (&errors)[2], double tolerance) {
 const BRepAdaptor_Curve adaptor(edge);
 if (adaptor.GetType() != GeomAbs_Circle || last - first < 2. * std::acos(-1.)) return false;
 const gp_Circ circle = adaptor.Circle();
 const double radius = circle.Radius();
 const double half = 0.5 * projected[0].Distance(projected[1]);
 if (!std::isfinite(radius) || !std::isfinite(half) || radius <= 0. || half >= radius) return false;
 const double radicand = (radius - half) * (radius + half);
 if (!std::isfinite(radicand) || radicand <= 0.) return false;
 const double sagitta = half * (half / (radius + std::sqrt(radicand)));
 const gp_Pnt center = circle.Location();
 const double scale = 1. + std::abs(center.X()) + std::abs(center.Y()) + std::abs(center.Z()) + radius + half;
 const double margin = 128. * std::numeric_limits<double>::epsilon() * scale;
 const double bound = std::max(errors[0], errors[1]) + sagitta + margin;
 return std::isfinite(bound) && bound < tolerance;
}

// Select by a local-space point and emit from the same native edge map. The
// caller supplies a bounded tolerance and must validate the viewport pick.
int resolve_source_edge(const TopTools_IndexedMapOfShape& edges, const double* point, double tolerance, const double* segment, double segment_tolerance, size_t max_nodes, gp_Pnt& selected_point) {
  if (!point || !std::isfinite(tolerance) || tolerance <= 0 || !std::all_of(point, point + 3, [](double v) { return std::isfinite(v); }))
   throw std::invalid_argument("source edge pick: invalid input");
  if (segment) {
   if (!std::isfinite(segment_tolerance) || segment_tolerance <= 0 || !std::all_of(segment, segment + 6, [](double v) { return std::isfinite(v); }))
    throw std::invalid_argument("source edge pick: invalid segment");
   const gp_Pnt a(segment[0], segment[1], segment[2]), b(segment[3], segment[4], segment[5]);
   if (a.Distance(b) <= segment_tolerance) throw std::invalid_argument("source edge pick: segment too short");
  }
  const gp_Pnt target(point[0], point[1], point[2]);
  int selected = -1;
  size_t remaining_segment_checks = std::min<size_t>(max_nodes, 4096);
  for (int i = 1; i <= edges.Extent(); ++i) {
   const TopoDS_Edge edge = TopoDS::Edge(edges(i));
   if (BRep_Tool::Degenerated(edge)) {
    TopoDS_Vertex first_vertex, last_vertex;
    TopExp::Vertices(edge, first_vertex, last_vertex, true);
    if (first_vertex.IsNull()) throw std::invalid_argument("source edge pick: degenerate edge without vertex");
    if (target.Distance(BRep_Tool::Pnt(first_vertex)) <= tolerance)
     throw std::invalid_argument("source edge pick: degenerate edge at pick");
    continue;
   }
   double first, last;
   TopLoc_Location location;
   auto curve = BRep_Tool::Curve(edge, location, first, last);
   if (curve.IsNull() || !std::isfinite(first) || !std::isfinite(last) || first >= last)
    throw std::invalid_argument("source edge pick: invalid curve");
   curve = Handle(Geom_Curve)::DownCast(curve->Transformed(location.Transformation()));
   GeomAPI_ProjectPointOnCurve projection(target, curve, first, last);
   gp_Pnt nearest = curve->Value(first);
   double distance = target.Distance(nearest);
   const gp_Pnt endpoint = curve->Value(last);
   if (target.Distance(endpoint) < distance) { nearest = endpoint; distance = target.Distance(endpoint); }
   if (projection.NbPoints() > 0 && projection.LowerDistance() < distance) {
    nearest = projection.NearestPoint(); distance = projection.LowerDistance();
   }
   if (!std::isfinite(distance)) throw std::runtime_error("source edge pick: invalid distance");
   if (distance <= tolerance) {
     if (segment) {
      bool matched = true;
      double endpoint_distances[2] = {0., 0.};
       gp_Pnt projected[2];
      for (int end = 0; end < 2; ++end) {
       const gp_Pnt sample(segment[3 * end], segment[3 * end + 1], segment[3 * end + 2]);
       GeomAPI_ProjectPointOnCurve test(sample, curve, first, last);
        projected[end] = curve->Value(first);
        double sample_distance = sample.Distance(projected[end]);
        const gp_Pnt final_point = curve->Value(last);
        if (sample.Distance(final_point) < sample_distance) { projected[end] = final_point; sample_distance = sample.Distance(final_point); }
        if (test.NbPoints() > 0 && test.LowerDistance() < sample_distance) {
         projected[end] = test.NearestPoint(); sample_distance = test.LowerDistance();
        }
       if (!std::isfinite(sample_distance)) throw std::runtime_error("source edge pick: invalid segment distance");
        if (sample_distance > std::min(segment_tolerance, tolerance)) { matched = false; break; }
        endpoint_distances[end] = sample_distance;
      }
      if (!matched) continue;
      // Distance to a set is 1-Lipschitz. Bound each subsegment using its
      // endpoint distances and length, subdividing only when needed. A line
      // segment's distance to a bounded line edge is convex, so its endpoint
      // checks already bound the whole segment. OCCT projection and floating
      // point remain numerical rather than interval-arithmetic guarantees.
      if (BRepAdaptor_Curve(edge).GetType() != GeomAbs_Line &&
          !(max_nodes >= 4096 && full_circle_chord_within(edge, first, last, projected, endpoint_distances, tolerance))) {
       struct SegmentInterval { double lo, hi, dlo, dhi; };
       std::array<SegmentInterval, 64> stack;
       size_t size = 1;
       stack[0] = {0., 1., endpoint_distances[0], endpoint_distances[1]};
       const gp_Pnt start(segment[0], segment[1], segment[2]);
       const gp_Pnt end(segment[3], segment[4], segment[5]);
       const double length = start.Distance(end);
        if (!std::isfinite(length)) throw std::runtime_error("source edge pick: invalid segment length");
       while (size) {
        const SegmentInterval interval = stack[--size];
        if (std::max(interval.dlo, interval.dhi) + 0.5 * length * (interval.hi - interval.lo) <= tolerance) continue;
        if (remaining_segment_checks == 0 || size + 2 > stack.size())
         throw std::length_error("source edge pick: segment verification budget");
        --remaining_segment_checks;
        const double t = 0.5 * (interval.lo + interval.hi);
        if (t == interval.lo || t == interval.hi)
         throw std::runtime_error("source edge pick: segment precision limit");
        const gp_Pnt sample(segment[0] + t * (segment[3] - segment[0]),
                            segment[1] + t * (segment[4] - segment[1]),
                            segment[2] + t * (segment[5] - segment[2]));
        GeomAPI_ProjectPointOnCurve test(sample, curve, first, last);
        double d = std::min(sample.Distance(curve->Value(first)), sample.Distance(curve->Value(last)));
        if (test.NbPoints() > 0) d = std::min(d, test.LowerDistance());
        if (!std::isfinite(d)) throw std::runtime_error("source edge pick: invalid segment distance");
        if (d > tolerance) { matched = false; break; }
        stack[size++] = {interval.lo, t, interval.dlo, d};
        stack[size++] = {t, interval.hi, d, interval.dhi};
       }
      }
      if (!matched) continue;
     }
    if (selected >= 0) throw std::invalid_argument("source edge pick: ambiguous edges");
    selected = i - 1;
    selected_point = nearest;
   }
  }
  if (selected < 0) throw std::invalid_argument("source edge pick: no edge within tolerance");
  return selected;
}

extern "C" int om9_occt_verified_edge_endpoints(const char* source, size_t length, const double* segment,
    double tolerance, double segment_tolerance, size_t max_nodes, double* points, char* message, int capacity) {
 try {
  if (!segment || !points || !std::isfinite(tolerance) || tolerance <= 0 || !std::isfinite(segment_tolerance) || segment_tolerance <= 0 ||
      !std::all_of(segment, segment + 6, [](double v) { return std::isfinite(v); }))
   throw std::invalid_argument("fillet: invalid display segment");
  const TopoDS_Shape shape = read_shape(source, length, "fillet edge");
  TopTools_IndexedMapOfShape edges;
  TopExp::MapShapes(shape, TopAbs_EDGE, edges);
  if (edges.Extent() == 0 || size_t(edges.Extent()) > max_nodes) throw std::length_error("fillet: source edge budget");
  const double midpoint[3] = {(segment[0]+segment[3])*0.5, (segment[1]+segment[4])*0.5, (segment[2]+segment[5])*0.5};
  gp_Pnt resolved;
  const int index = resolve_source_edge(edges, midpoint, tolerance, segment, segment_tolerance, max_nodes, resolved);
  const TopoDS_Edge edge = TopoDS::Edge(edges(index + 1));
  TopoDS_Vertex first, last;
  TopExp::Vertices(edge, first, last, true);
  if (first.IsNull() || last.IsNull()) throw std::invalid_argument("fillet: missing edge vertex");
  if (first.IsSame(last)) return 3;
  const gp_Pnt a = BRep_Tool::Pnt(first), b = BRep_Tool::Pnt(last);
  points[0] = a.X(); points[1] = a.Y(); points[2] = a.Z();
  points[3] = b.X(); points[4] = b.Y(); points[5] = b.Z();
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown verified edge failure"); return 2;
 }
}

extern "C" int om9_occt_point_edge_nurbs(const char* source, size_t length,
    const double* point, double tolerance, const double* segment, double segment_tolerance, double* resolved_point, double* circle_data, size_t max_edges,
    size_t max_points, size_t max_nodes, size_t max_bytes, int max_degree,
    int (*sink)(void*, int, const double*, const double*, size_t, const double*, size_t),
    void* context, char* message, int capacity) {
 try {
  if (!point || !resolved_point || !sink || !std::isfinite(tolerance) || tolerance <= 0.0 ||
      !std::isfinite(point[0]) || !std::isfinite(point[1]) || !std::isfinite(point[2]))
   throw std::invalid_argument("source edge pick: invalid input");
  if (circle_data) std::fill(circle_data, circle_data + 14, 0.0);
  if (segment && (!std::isfinite(segment_tolerance) || segment_tolerance <= 0.0 ||
      !std::all_of(segment, segment + 6, [](double v) { return std::isfinite(v); })))
   throw std::invalid_argument("source edge pick: invalid segment");
  const TopoDS_Shape shape = read_shape(source, length, "source edge pick");
  TopTools_IndexedMapOfShape edges;
  TopExp::MapShapes(shape, TopAbs_EDGE, edges);
  if (edges.Extent() <= 0 || static_cast<size_t>(edges.Extent()) > max_edges)
   throw std::length_error("source edge pick: edge budget");
  gp_Pnt selected_point;
  const int selected = resolve_source_edge(edges, point, tolerance, segment, segment_tolerance, max_nodes, selected_point);
  if (circle_data) {
   const TopoDS_Edge edge = TopoDS::Edge(edges(selected + 1));
   BRepAdaptor_Curve adaptor(edge);
   if (adaptor.GetType() == GeomAbs_Circle) {
    const gp_Circ circle = adaptor.Circle();
    const gp_Pnt center = circle.Location();
    const gp_Dir x = circle.Position().XDirection(), y = circle.Position().YDirection();
    circle_data[0] = 1.0;
    circle_data[1] = center.X(); circle_data[2] = center.Y(); circle_data[3] = center.Z();
    circle_data[4] = x.X(); circle_data[5] = x.Y(); circle_data[6] = x.Z();
    circle_data[7] = y.X(); circle_data[8] = y.Y(); circle_data[9] = y.Z();
    circle_data[10] = circle.Radius(); circle_data[11] = adaptor.FirstParameter(); circle_data[12] = adaptor.LastParameter();
    circle_data[13] = edge.Orientation() == TopAbs_REVERSED ? 1.0 : 0.0;
   }
  }
  emit_edge_curves(edges, &selected, 1, max_points, max_nodes, max_bytes, max_degree, sink, context);
  resolved_point[0] = selected_point.X(); resolved_point[1] = selected_point.Y(); resolved_point[2] = selected_point.Z();
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown source edge pick failure"); return 2;
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

// Emit source-local centers of analytically circular edges without promoting
// the NURBS conversion to a circle or retaining native shape handles.
extern "C" int om9_occt_circular_edge_centers(const char* source, size_t length,
 size_t max_edges, size_t max_outputs, int (*checkpoint)(void*),
 int (*sink)(void*, const double*), void* context, char* message, int capacity) {
 try {
  if (!source || !length || !checkpoint || !sink) throw std::invalid_argument("edge centers: missing input");
  const TopoDS_Shape shape=read_shape(source,length,"edge centers");
  TopTools_IndexedMapOfShape edges; TopExp::MapShapes(shape,TopAbs_EDGE,edges);
  if (size_t(edges.Extent())>max_edges) throw std::length_error("edge centers: edge budget");
  size_t count=0;
  for (int i=1;i<=edges.Extent();++i) {
   if (checkpoint(context)) throw std::runtime_error("edge centers: canceled or refused");
   const TopoDS_Edge edge=TopoDS::Edge(edges(i));
   if (BRep_Tool::Degenerated(edge)) continue;
   const BRepAdaptor_Curve curve(edge);
   if (curve.GetType()!=GeomAbs_Circle) continue;
   if (++count>max_outputs) throw std::length_error("edge centers: output budget");
   const gp_Pnt center=curve.Circle().Location();
   const double xyz[3]={center.X(),center.Y(),center.Z()};
   if (!std::all_of(xyz,xyz+3,[](double value){return std::isfinite(value);}))
    throw std::runtime_error("edge centers: nonfinite center");
   if (sink(context,xyz)) throw std::runtime_error("edge centers: output refused");
  }
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) {
  if (message && capacity>0) std::snprintf(message,capacity,"Unknown edge centers failure"); return 2;
 }
}

extern "C" int om9_occt_source_vertices(const char* source, size_t length,
 size_t max_nodes, size_t max_outputs, int (*checkpoint)(void*),
 int (*sink)(void*, const double*), void* context, char* message, int capacity) {
 try {
  if (!source || !length || !checkpoint || !sink) throw std::invalid_argument("source vertices: missing input");
  const TopoDS_Shape shape=read_shape(source,length,"source vertices");
  TopTools_IndexedMapOfShape vertices; TopExp::MapShapes(shape,TopAbs_VERTEX,vertices);
  if (size_t(vertices.Extent())>max_nodes || size_t(vertices.Extent())>max_outputs)
   throw std::length_error("source vertices: output budget");
  for (int i=1;i<=vertices.Extent();++i) {
   if (checkpoint(context)) throw std::runtime_error("source vertices: canceled or refused");
   const gp_Pnt point=BRep_Tool::Pnt(TopoDS::Vertex(vertices(i)));
   const double xyz[3]={point.X(),point.Y(),point.Z()};
   if (!std::all_of(xyz,xyz+3,[](double value){return std::isfinite(value);}))
    throw std::runtime_error("source vertices: nonfinite point");
   if (sink(context,xyz)) throw std::runtime_error("source vertices: output refused");
  }
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) {
  if (message && capacity>0) std::snprintf(message,capacity,"Unknown source vertices failure"); return 2;
 }
}

extern "C" int om9_occt_fillet_project_edge(const char* source, size_t length, int index,
                                             const double* point, const double* segment, double tolerance, double segment_tolerance,
                                             size_t max_nodes, double* output, char* message, int capacity) {
 try {
  if (!point || !output || !std::isfinite(point[0]) || !std::isfinite(point[1]) || !std::isfinite(point[2]))
   throw std::invalid_argument("fillet edge projection: invalid point");
  const TopoDS_Shape shape = read_shape(source, length, "fillet edge projection");
  TopTools_IndexedMapOfShape edge_map;
  if (segment) {
   TopExp::MapShapes(shape, TopAbs_EDGE, edge_map);
   if (edge_map.Extent() == 0 || size_t(edge_map.Extent()) > max_nodes) throw std::length_error("fillet: source edge budget");
   const double midpoint[3] = {(segment[0]+segment[3])*0.5, (segment[1]+segment[4])*0.5, (segment[2]+segment[5])*0.5};
   gp_Pnt resolved;
   index = resolve_source_edge(edge_map, midpoint, tolerance, segment, segment_tolerance, max_nodes, resolved);
  }
  const TopoDS_Edge edge = segment ? TopoDS::Edge(edge_map(index + 1)) : edge_at(shape, index, "fillet edge projection");
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
                                                size_t count, const double* segments, size_t anchor_slot,
                                                double tolerance, double segment_tolerance, size_t max_nodes,
                                                unsigned char* masks, char* message, int capacity) {
 try {
  if (!candidates || !masks || count == 0 || (side != 0 && side != 1)) throw std::invalid_argument("fillet shared endpoint: invalid argument");
  const TopoDS_Shape shape = read_shape(source, length, "fillet shared endpoint");
  TopTools_IndexedMapOfShape edge_map;
  TopExp::MapShapes(shape, TopAbs_EDGE, edge_map);
  if (segments && (anchor_slot >= count || edge_map.Extent() == 0 || size_t(edge_map.Extent()) > max_nodes))
   throw std::invalid_argument("fillet shared endpoint: invalid source edges");
  auto resolve = [&](size_t slot) {
   const double* segment = segments + 6 * slot;
   const double midpoint[3] = {(segment[0]+segment[3])*0.5, (segment[1]+segment[4])*0.5, (segment[2]+segment[5])*0.5};
   gp_Pnt resolved;
   return resolve_source_edge(edge_map, midpoint, tolerance, segment, segment_tolerance, max_nodes, resolved);
  };
  if (segments) anchor_index = resolve(anchor_slot);
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
   const int candidate_index = segments ? resolve(i) : candidates[i];
   if (candidate_index < 0 || candidate_index >= edge_map.Extent()) throw std::invalid_argument("fillet shared endpoint: invalid candidate");
   const TopoDS_Edge candidate = TopoDS::Edge(edge_map(candidate_index + 1));
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
                                         const double* segments, double pick_tolerance, double segment_tolerance,
                                         double deflection, size_t max_points, size_t max_nodes,
                                         size_t max_bytes, Om9BrepSink sink, void* context,
                                         char* message, int capacity) {
 try {
  if (!edges || !starts || !ends || count == 0 || (control_count && (!control_edges || !fractions || !control_radii))) throw std::invalid_argument("fillet: no edges or controls");
  const TopoDS_Shape shape = read_shape(source, length, "fillet shape");
  TopTools_IndexedMapOfShape source_edges;
  if (segments) {
   if (!std::isfinite(pick_tolerance) || pick_tolerance <= 0 || !std::isfinite(segment_tolerance) || segment_tolerance <= 0)
    throw std::invalid_argument("fillet: invalid edge verification tolerance");
   TopExp::MapShapes(shape, TopAbs_EDGE, source_edges);
   if (source_edges.Extent() == 0 || size_t(source_edges.Extent()) > max_nodes)
    throw std::length_error("fillet: source edge budget");
  }
  TopTools_IndexedMapOfShape naked;
  naked_edge_count(shape, naked);
  BRepFilletAPI_MakeFillet maker(shape);
  struct FilletInput { TopoDS_Edge edge; int index; double start, end; int contour; };
  std::vector<FilletInput> selected;
  selected.reserve(count);
  for (size_t i = 0; i < count; ++i) {
   if (!std::isfinite(starts[i]) || !std::isfinite(ends[i]) || starts[i] <= 0 || ends[i] <= 0)
    throw std::invalid_argument("fillet: radii must be positive");
   int source_index = edges[i];
   if (segments) {
    const double* segment = segments + 6 * i;
    if (!std::all_of(segment, segment + 6, [](double v) { return std::isfinite(v); }))
     throw std::invalid_argument("fillet: invalid display segment");
    const double point[3] = {(segment[0] + segment[3]) * 0.5, (segment[1] + segment[4]) * 0.5, (segment[2] + segment[5]) * 0.5};
    gp_Pnt resolved;
    source_index = resolve_source_edge(source_edges, point, pick_tolerance, segment, segment_tolerance, max_nodes, resolved);
   }
   const TopoDS_Edge edge = segments ? TopoDS::Edge(source_edges(source_index + 1)) : edge_at(shape, source_index, "fillet");
   if (std::any_of(selected.begin(), selected.end(), [&](const auto& item) { return item.edge.IsSame(edge); }))
    throw std::invalid_argument("fillet: duplicate source edge");
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

// A display triangle is only an identity witness: output still comes from the
// exact source face. Reject if no unique trimmed source face matches.
int resolve_source_face(const TopTools_IndexedMapOfShape& faces, const double* triangle,
                               double tolerance, double vertex_tolerance, size_t max_nodes) {
 if (!triangle || !std::isfinite(tolerance) || tolerance <= 0 || !std::isfinite(vertex_tolerance) || vertex_tolerance <= 0 ||
     !std::all_of(triangle, triangle + 9, [](double v) { return std::isfinite(v); }))
  throw std::invalid_argument("source face pick: invalid triangle");
 if (faces.Extent() == 0 || size_t(faces.Extent()) > max_nodes) throw std::length_error("source face pick: face budget");
 const gp_Pnt p[3] = {gp_Pnt(triangle[0],triangle[1],triangle[2]), gp_Pnt(triangle[3],triangle[4],triangle[5]), gp_Pnt(triangle[6],triangle[7],triangle[8])};
 const gp_Vec a(p[0],p[1]), b(p[0],p[2]);
 if (a.Crossed(b).Magnitude() <= vertex_tolerance * vertex_tolerance)
  throw std::invalid_argument("source face pick: degenerate triangle");
 const gp_Pnt center(triangle[0]/3.+triangle[3]/3.+triangle[6]/3., triangle[1]/3.+triangle[4]/3.+triangle[7]/3., triangle[2]/3.+triangle[5]/3.+triangle[8]/3.);
 int selected = -1;
 for (int i = 1; i <= faces.Extent(); ++i) {
  const TopoDS_Face face = TopoDS::Face(faces(i));
  bool matched = true;
  for (int j = 0; j < 4; ++j) {
   const gp_Pnt& sample = j == 3 ? center : p[j];
   BRepExtrema_DistShapeShape dist(BRepBuilderAPI_MakeVertex(sample).Vertex(), face);
   dist.Perform();
   if (!dist.IsDone() || !std::isfinite(dist.Value())) throw std::runtime_error("source face pick: distance failed");
   if (dist.Value() > (j == 3 ? tolerance : vertex_tolerance)) { matched = false; break; }
  }
  if (!matched) continue;
  if (selected >= 0) throw std::invalid_argument("source face pick: ambiguous faces");
  selected = i - 1;
 }
 if (selected < 0) throw FaceWitnessMiss("source face pick: no matching face");
 return selected;
}

// A strict certificate for the only outer boundary where a thin overlap with
// an inner-wire box can safely bound *all* of the triangle's outside area.
// Any representation or coordinate uncertainty falls back to the full check.
static bool axis_rectangle_outer(const TopoDS_Wire& wire, const gp_Pln& plane,
                                 int& axis_u, int& axis_v, double (&lo)[2], double (&hi)[2]) {
 const gp_Dir n=plane.Axis().Direction();
 const double nc[3]={n.X(),n.Y(),n.Z()}; int normal=-1;
 for (int i=0;i<3;++i) if (std::abs(nc[i])==1. && nc[(i+1)%3]==0. && nc[(i+2)%3]==0.) normal=i;
 if (normal<0) return false;
 axis_u=(normal+1)%3; axis_v=(normal+2)%3;
 std::array<std::array<gp_Pnt,2>,4> ends; size_t count=0;
 for (TopExp_Explorer it(wire,TopAbs_EDGE);it.More();it.Next()) {
  if (count==4) return false;
  const TopoDS_Edge edge=TopoDS::Edge(it.Current());
  if (BRep_Tool::Degenerated(edge)) return false;
  BRepAdaptor_Curve curve(edge);
  if (curve.GetType()==GeomAbs_BSplineCurve) {
   if (curve.Degree()!=1 || curve.NbPoles()!=2 || curve.NbKnots()!=2 || curve.IsRational() || curve.IsPeriodic()) return false;
  } else if (curve.GetType()!=GeomAbs_Line) return false;
  TopoDS_Vertex a,b; TopExp::Vertices(edge,a,b);
  if (a.IsNull() || b.IsNull()) return false;
  ends[count]={BRep_Tool::Pnt(a),BRep_Tool::Pnt(b)};
  const gp_Pnt first=curve.Value(curve.FirstParameter()), last=curve.Value(curve.LastParameter());
  if (!((first.IsEqual(ends[count][0],0.) && last.IsEqual(ends[count][1],0.)) ||
        (first.IsEqual(ends[count][1],0.) && last.IsEqual(ends[count][0],0.)))) return false;
  ++count;
 }
 if (count!=4) return false;
 lo[0]=lo[1]=std::numeric_limits<double>::infinity(); hi[0]=hi[1]=-lo[0];
 for (const auto& edge:ends) for (const gp_Pnt& p:edge) {
  const double v[2]={p.Coord(axis_u+1),p.Coord(axis_v+1)};
  for (int i=0;i<2;++i) {if (!std::isfinite(v[i])) return false;lo[i]=std::min(lo[i],v[i]);hi[i]=std::max(hi[i],v[i]);}
 }
 if (!(lo[0]<hi[0] && lo[1]<hi[1])) return false;
 unsigned seen=0;
 for (const auto& edge:ends) {
  unsigned corners[2];
  for (int j=0;j<2;++j) {
   const double x=edge[j].Coord(axis_u+1),y=edge[j].Coord(axis_v+1);
   if ((x!=lo[0] && x!=hi[0]) || (y!=lo[1] && y!=hi[1])) return false;
   corners[j]=(x==hi[0]?1:0)|(y==hi[1]?2:0);
  }
  const unsigned a=std::min(corners[0],corners[1]),b=std::max(corners[0],corners[1]);
  const unsigned bit=(a==0 && b==1)?1:(a==0 && b==2)?2:(a==1 && b==3)?4:(a==2 && b==3)?8:0;
  if (!bit || (seen&bit)) return false;
  seen|=bit;
 }
 return seen==15;
}

// Validate every displayed triangle of the selected face against the resolved
// source face. A representative triangle alone cannot authenticate the one hit.
extern "C" int om9_occt_verify_face_display(const char* source, size_t length,
    const double* witness, double deflection, double vertex_tolerance,
    const double* points, size_t point_count, const uint32_t* triangles, size_t triangle_count,
    uint32_t display_face, size_t max_nodes, int (*canceled)(void*), void* cancel_context, char* message, int capacity) {
 try {
  if (canceled && canceled(cancel_context)) return 3;
  if (!source || !length || !witness || !points || !triangles || !std::isfinite(deflection) || deflection <= 0 ||
      !std::isfinite(vertex_tolerance) || vertex_tolerance <= 0 || triangle_count > max_nodes)
   throw std::invalid_argument("source face display: invalid input or budget");
  const TopoDS_Shape shape = read_shape(source, length, "source face display");
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(shape, TopAbs_FACE, faces);
  const int index = resolve_source_face(faces, witness, deflection + vertex_tolerance, vertex_tolerance, max_nodes);
  if (canceled && canceled(cancel_context)) return 3;
  const TopoDS_Face face = TopoDS::Face(faces(index + 1));
  const Handle(Geom_Surface) support = BRep_Tool::Surface(face);
  if (support.IsNull()) throw std::invalid_argument("source face display: missing surface");
  const GeomLib_IsPlanarSurface planar(support, 1e-10);
  std::vector<Bnd_Box> hole_boxes;
  int rect_u=-1,rect_v=-1; double rect_lo[2],rect_hi[2]; bool rectangular_outer=false;
  if (planar.IsPlanar()) {
   const TopoDS_Wire outer = BRepTools::OuterWire(face);
   if (outer.IsNull()) throw std::invalid_argument("source face display: missing outer wire");
   rectangular_outer=axis_rectangle_outer(outer,planar.Plan(),rect_u,rect_v,rect_lo,rect_hi);
   for (TopExp_Explorer wire(face, TopAbs_WIRE); wire.More(); wire.Next()) {
    if (canceled && canceled(cancel_context)) return 3;
    if (wire.Current().IsSame(outer)) continue;
    if (hole_boxes.size() >= max_nodes) throw std::length_error("source face display: wire budget");
    Bnd_Box bounds;
    BRepBndLib::Add(wire.Current(), bounds);
    if (bounds.IsVoid() || bounds.IsWhole()) throw std::invalid_argument("source face display: invalid hole bounds");
    bounds.Enlarge(vertex_tolerance);
    hole_boxes.push_back(bounds);
   }
  }
  // Shared mesh vertices need only one distance test per selected face. This
  // fixed 4 KiB bitmap has no heap cost; high indices simply remain uncached.
  std::array<uint64_t, 512> verified_vertices{};
  BRepExtrema_DistShapeShape distance;
  distance.LoadS2(face);
  BRepExtrema_ExtPF projected;
  projected.Initialize(face, Extrema_ExtFlag_MIN);
  const auto verify_point = [&](const gp_Pnt& point, double tolerance) {
   const TopoDS_Vertex vertex = BRepBuilderAPI_MakeVertex(point).Vertex();
   // ExtPF classifies its candidates against the trimmed face. Finding any
   // candidate within tolerance is sufficient; if it has no such candidate
   // (e.g. a boundary minimum), retain the full shape-distance fallback.
   try {
    projected.Perform(vertex, face);
    if (projected.IsDone()) for (int i = 1; i <= projected.NbExt(); ++i) {
     const double squared = projected.SquareDistance(i);
     if (std::isfinite(squared) && squared >= 0. && squared <= tolerance * tolerance) return;
    }
   } catch (const Standard_Failure&) { /* full OCCT distance handles this case */ }
   distance.LoadS1(vertex);
   distance.Perform();
   if (!distance.IsDone() || !std::isfinite(distance.Value()) || distance.Value() > tolerance)
    throw std::invalid_argument("source face display: triangle does not match selected face");
  };
  size_t checked = 0;
  for (size_t i = 0; i < triangle_count; ++i) {
   if (canceled && canceled(cancel_context)) return 3;
   const uint32_t* tri = triangles + 4*i;
   if (tri[3] != display_face) continue;
   gp_Pnt vertices[3];
   for (int j = 0; j < 3; ++j) {
    if (tri[j] >= point_count) throw std::invalid_argument("source face display: invalid vertex");
    const double* p = points + size_t(tri[j])*3;
    if (!std::all_of(p, p+3, [](double v) { return std::isfinite(v); })) throw std::invalid_argument("source face display: nonfinite vertex");
    vertices[j] = gp_Pnt(p[0],p[1],p[2]);
    const size_t word = size_t(tri[j]) / 64;
    const uint64_t bit = uint64_t(1) << (tri[j] % 64);
    if (word >= verified_vertices.size() || !(verified_vertices[word] & bit)) {
     verify_point(vertices[j], vertex_tolerance);
     if (word < verified_vertices.size()) verified_vertices[word] |= bit;
    }
   }
   const gp_Pnt center((vertices[0].X()+vertices[1].X()+vertices[2].X())/3.,
                       (vertices[0].Y()+vertices[1].Y()+vertices[2].Y())/3.,
                       (vertices[0].Z()+vertices[1].Z()+vertices[2].Z())/3.);
   verify_point(center, deflection + vertex_tolerance);
   if (!hole_boxes.empty()) {
    Bnd_Box bounds;
    for (const gp_Pnt& point : vertices) bounds.Add(point);
    bounds.Enlarge(vertex_tolerance);
    bool triangle_in_rectangle=rectangular_outer;
    if (triangle_in_rectangle) for (const gp_Pnt& p:vertices) {
     const double v[2]={p.Coord(rect_u+1),p.Coord(rect_v+1)};
     for (int k=0;k<2;++k) if (v[k]<rect_lo[k] || v[k]>rect_hi[k]) triangle_in_rectangle=false;
    }
    bool near_hole = false;
    double strip_sum = 0.;
    for (const Bnd_Box& hole:hole_boxes) {
     if (canceled && canceled(cancel_context)) return 3;
     if (bounds.IsOut(hole)) continue;
     // A small hole can have less area than the perimeter-based allowance
     // below while still containing a selectable point far from the face.
     // Check the hole-box centre when it lies strictly inside this triangle.
     // It is only a witness: irregular holes may not contain their box centre.
     double hxmin,hymin,hzmin,hxmax,hymax,hzmax;
     hole.Get(hxmin,hymin,hzmin,hxmax,hymax,hzmax);
     gp_Pnt middle((hxmin+hxmax)*0.5,(hymin+hymax)*0.5,(hzmin+hzmax)*0.5);
     const gp_Pln plane=planar.Plan();
     const gp_Vec normal(plane.Axis().Direction());
     middle.Translate(normal.Multiplied(-gp_Vec(plane.Location(),middle).Dot(normal)));
     const gp_Vec a(vertices[0],vertices[1]), b(vertices[0],vertices[2]), p(vertices[0],middle);
     const double aa=a.Dot(a), ab=a.Dot(b), bb=b.Dot(b), pa=p.Dot(a), pb=p.Dot(b);
     const double denominator=aa*bb-ab*ab;
     if (std::isfinite(denominator) && denominator>0.) {
      const double u=(bb*pa-ab*pb)/denominator, v=(aa*pb-ab*pa)/denominator;
      if (std::isfinite(u) && std::isfinite(v) && u>1e-8 && v>1e-8 && u+v<1.-1e-8)
       verify_point(middle,vertex_tolerance);
     }
     if (!triangle_in_rectangle) {near_hole=true;break;}
     double t0,t1,t2,t3,t4,t5,h0,h1,h2,h3,h4,h5;
     bounds.Get(t0,t1,t2,t3,t4,t5); hole.Get(h0,h1,h2,h3,h4,h5);
     const double tl[3]={t0,t1,t2},th[3]={t3,t4,t5},hl[3]={h0,h1,h2},hh[3]={h3,h4,h5};
     double strip=std::numeric_limits<double>::infinity();
     for (int axis:{rect_u,rect_v}) {
      const double scale=std::max({1.,std::abs(tl[axis]),std::abs(th[axis]),std::abs(hl[axis]),std::abs(hh[axis])});
      const double margin=64.*std::numeric_limits<double>::epsilon()*scale;
      const double overlap=std::min(th[axis],hh[axis])-std::max(tl[axis],hl[axis]);
      if (std::isfinite(overlap) && std::isfinite(margin)) strip=std::min(strip,std::max(0.,overlap)+margin);
     }
     strip_sum+=strip;
     // Sum bounds over all holes: several individually small overlaps can
     // exceed the outside-area tolerance together.
     if (!std::isfinite(strip_sum) || strip_sum>4.*vertex_tolerance) {near_hole=true;break;}
    }
    if (near_hole) {
     const gp_Pln plane = planar.Plan();
     const gp_Vec normal(plane.Axis().Direction());
     BRepBuilderAPI_MakePolygon polygon;
     double perimeter = 0.;
     for (int j = 0; j < 3; ++j) {
      const gp_Vec offset(plane.Location(), vertices[j]);
      polygon.Add(vertices[j].Translated(normal.Multiplied(-offset.Dot(normal))));
      perimeter += vertices[j].Distance(vertices[(j+1)%3]);
     }
     polygon.Close();
     if (!polygon.IsDone()) throw std::invalid_argument("source face display: invalid triangle wire");
     BRepBuilderAPI_MakeFace triangle(polygon.Wire());
     if (!triangle.IsDone()) throw std::invalid_argument("source face display: invalid triangle face");
     if (canceled && canceled(cancel_context)) return 3;
     BRepAlgoAPI_Cut outside(triangle.Face(), face);
     if (canceled && canceled(cancel_context)) return 3;
     if (!outside.IsDone() || outside.HasErrors()) throw std::runtime_error("source face display: triangle trim check failed");
     GProp_GProps area;
     BRepGProp::SurfaceProperties(outside.Shape(), area);
     if (!std::isfinite(area.Mass()) || area.Mass() > 4. * vertex_tolerance * perimeter)
      throw std::invalid_argument("source face display: triangle crosses trim boundary");
    }
   }
   ++checked;
  }
  if (!checked) throw std::invalid_argument("source face display: no triangles");
  if (canceled && canceled(cancel_context)) return 3;
  return 0;
 } catch (const Standard_Failure& e) { if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2; }
 catch (const std::exception& e) { if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1; }
 catch (...) { if (message && capacity > 0) std::snprintf(message, capacity, "source face display: native failure"); return 2; }
}

extern "C" int om9_occt_verify_face_triangles(const char* source, size_t length, const double* triangles,
    size_t count, double tolerance, double vertex_tolerance, size_t max_nodes, size_t max_bytes, char* message, int capacity) {
 try {
  if (!triangles || !count || count > max_nodes) throw std::invalid_argument("source face pick: invalid count");
  const TopoDS_Shape shape = read_shape(source, length, "source face pick");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  if (faces.Extent() == 0 || size_t(faces.Extent()) > max_nodes || size_t(faces.Extent()) > max_bytes)
   throw std::length_error("source face pick: face budget");
  std::vector<bool> seen(size_t(faces.Extent()), false);
  for (size_t i = 0; i < count; ++i) {
   const int index = resolve_source_face(faces, triangles + 9*i, tolerance, vertex_tolerance, max_nodes);
   if (seen[size_t(index)]) throw std::invalid_argument("source face pick: duplicate source face");
   seen[size_t(index)] = true;
  }
  return 0;
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown source face failure"); return 2;
 }
}

extern "C" int om9_occt_extract_face(const char* source, size_t length, int face_index,
                                     const double* triangle, double vertex_tolerance, double witness_deflection,
                                     double deflection, size_t max_points, size_t max_nodes,
                                     size_t max_bytes, Om9BrepSink sink, void* context,
                                     char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "extract face");
  TopTools_IndexedMapOfShape faces;
  if (triangle) { TopExp::MapShapes(shape, TopAbs_FACE, faces); face_index = resolve_source_face(faces, triangle, witness_deflection + vertex_tolerance, vertex_tolerance, max_nodes); }
  const TopoDS_Face face = triangle ? TopoDS::Face(faces(face_index + 1)) : face_at(shape, face_index, "extract face");
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

extern "C" int om9_occt_remove_region(const char* source, size_t length, int face_index,
                                    double deflection, size_t max_points, size_t max_nodes,
                                    size_t max_bytes, Om9BrepSink sink, void* context,
                                    char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "remove region");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  if (faces.Extent() < 2 || face_index < 0 || face_index >= faces.Extent())
   throw std::invalid_argument("remove region: invalid face index or empty remainder");
  BRep_Builder builder;
  TopoDS_Compound compound;
  builder.MakeCompound(compound);
  for (int i = 1; i <= faces.Extent(); ++i)
   if (i != face_index + 1) builder.Add(compound, faces(i));
  if (!BRepCheck_Analyzer(compound).IsValid()) throw std::runtime_error("invalid region remainder");
  return emit(compound, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown remove-region failure"); return 2;
 }
}

extern "C" int om9_occt_unify_surface_regions(const char* source, size_t length,
                                    double deflection, size_t max_points, size_t max_nodes,
                                    size_t max_bytes, Om9BrepSink sink, void* context,
                                    char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "unify surface regions");
  ShapeUpgrade_UnifySameDomain operation(shape, false, true, false);
  operation.SetSafeInputMode(true);
  operation.Build();
  const TopoDS_Shape result = operation.Shape();
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid())
   throw std::runtime_error("unify surface regions produced invalid topology");
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown unify surface regions failure"); return 2;
 }
}

extern "C" int om9_occt_unify_surface_preserving_edges(
    const char* source, size_t length, const char* original_source, size_t original_length,
    double deflection, size_t max_points, size_t max_nodes, size_t max_bytes,
    Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "unify surface regions");
  const TopoDS_Shape original = read_shape(original_source, original_length, "original surface");
  TopTools_IndexedMapOfShape current_edges, original_edges;
  TopExp::MapShapes(shape, TopAbs_EDGE, current_edges);
  TopExp::MapShapes(original, TopAbs_EDGE, original_edges);
  const size_t count = static_cast<size_t>(current_edges.Extent());
  const size_t source_count = static_cast<size_t>(original_edges.Extent());
  if (source_count && count > max_nodes / source_count)
   throw std::invalid_argument("unify surface edge comparison budget");
  ShapeUpgrade_UnifySameDomain operation(shape, false, true, false);
  operation.SetSafeInputMode(true);
  for (int i = 1; i <= current_edges.Extent(); ++i) {
   const TopoDS_Shape edge = current_edges(i);
   GProp_GProps length_props;
   BRepGProp::LinearProperties(edge, length_props);
   const double length_mm = std::abs(length_props.Mass());
   if (length_mm <= 1e-9) { operation.KeepShape(edge); continue; }
   for (int j = 1; j <= original_edges.Extent(); ++j) {
    BRepAlgoAPI_Common common(edge, original_edges(j));
    common.Build();
    if (!common.IsDone() || common.HasErrors())
     throw std::runtime_error("unify surface edge comparison failed");
    GProp_GProps overlap;
    BRepGProp::LinearProperties(common.Shape(), overlap);
    if (!std::isfinite(overlap.Mass())) throw std::runtime_error("unify surface invalid edge overlap");
    if (std::abs(overlap.Mass()) > 0.0) {
     operation.KeepShape(edge); break;
    }
   }
  }
  operation.Build();
  const TopoDS_Shape result = operation.Shape();
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid())
   throw std::runtime_error("unify surface produced invalid topology");
  return emit(result, deflection, max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown unify surface failure"); return 2;
 }
}

extern "C" int om9_occt_remove_face(const char* source, size_t length, int face_index,
                                    const double* triangle, double vertex_tolerance, double witness_deflection,
                                    double deflection, size_t max_points, size_t max_nodes,
                                    size_t max_bytes, Om9BrepSink sink, void* context,
                                    char* message, int capacity) {
 try {
  const TopoDS_Shape shape = read_shape(source, length, "remove face");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  if (triangle) face_index = resolve_source_face(faces, triangle, witness_deflection + vertex_tolerance, vertex_tolerance, max_nodes);
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
                                     size_t count, const double* triangles, double vertex_tolerance, double witness_deflection, double deflection, size_t max_points,
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
   const int index = triangles ? resolve_source_face(faces, triangles + 9*i, witness_deflection + vertex_tolerance, vertex_tolerance, max_nodes) : indices[i];
   if (index < 0 || index >= faces.Extent() || removed[index])
    throw std::invalid_argument("remove faces: invalid or duplicate index");
   removed[index] = true;
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
                            size_t length_b, int reverse_b, int preserve_surfaces, double tolerance, double deflection, size_t max_points,
                            size_t max_nodes, size_t max_bytes, Om9BrepSink sink, void* context,
                            char* message, int capacity) {
 try {
  if (!std::isfinite(tolerance) || tolerance <= 0)
   throw std::invalid_argument("sew: tolerance must be positive");
  const TopoDS_Shape a = read_shape(source_a, length_a, "sew argument");
  TopoDS_Shape b = read_shape(source_b, length_b, "sew tool");
  if (reverse_b != 0) b.Reverse();
  struct SurfaceSnapshot {
   Handle(Geom_Surface) surface;
   TopLoc_Location location;
   std::string serialized;
  };
  class LimitedSurfaceBuffer : public std::streambuf {
   public:
    explicit LimitedSurfaceBuffer(size_t limit) : limit_(limit) {}
    std::string text;
   protected:
    std::streamsize xsputn(const char* source, std::streamsize size) override {
     if (size<0 || size_t(size)>limit_-text.size()) throw std::length_error("sew: surface snapshot budget");
     text.append(source,size_t(size));return size;
    }
    int_type overflow(int_type ch) override {
     if (traits_type::eq_int_type(ch,traits_type::eof())) return traits_type::not_eof(ch);
     if (text.size()>=limit_) throw std::length_error("sew: surface snapshot budget");
     text.push_back(traits_type::to_char_type(ch));return ch;
    }
   private:
    size_t limit_;
  };
  auto serialize_surface=[](const Handle(Geom_Surface)& surface,size_t budget) {
   LimitedSurfaceBuffer buffer(budget);
   std::ostream stream(&buffer);
   stream.exceptions(std::ios::badbit | std::ios::failbit);
   stream << std::setprecision(std::numeric_limits<double>::max_digits10);
   GeomTools::Write(surface,stream);
   if (!stream || buffer.text.empty()) throw std::runtime_error("sew: cannot serialize surface");
   return std::move(buffer.text);
  };
  std::vector<SurfaceSnapshot> originals;
  size_t surface_bytes=0;
  if (preserve_surfaces) {
   for (const auto& shape : {a,b}) {
    TopTools_IndexedMapOfShape faces; TopExp::MapShapes(shape,TopAbs_FACE,faces);
    for (int i=1;i<=faces.Extent();++i) {
     TopLoc_Location location;
     Handle(Geom_Surface) surface=BRep_Tool::Surface(TopoDS::Face(faces(i)),location);
     if (surface.IsNull()) throw std::runtime_error("sew: missing input surface");
     std::string serialized=serialize_surface(surface,max_bytes-surface_bytes);
     if (originals.size()>=max_nodes || serialized.size()>max_bytes-surface_bytes)
      throw std::invalid_argument("sew: surface snapshot budget");
     surface_bytes+=serialized.size();
     originals.push_back({surface,location,std::move(serialized)});
    }
   }
  }
  BRepBuilderAPI_Sewing sewing(tolerance);
  sewing.Add(a);
  sewing.Add(b);
  sewing.Perform();
  const TopoDS_Shape sewed = sewing.SewedShape();
  if (sewed.IsNull()) throw std::runtime_error("sew produced no shape");
  if (preserve_surfaces) {
   TopTools_IndexedMapOfShape faces;TopExp::MapShapes(sewed,TopAbs_FACE,faces);
   if (size_t(faces.Extent())!=originals.size()) throw std::invalid_argument("sew changed face count");
   for (int i=1;i<=faces.Extent();++i) {
    TopLoc_Location location;
    Handle(Geom_Surface) surface=BRep_Tool::Surface(TopoDS::Face(faces(i)),location);
    const auto found=std::find_if(originals.begin(),originals.end(),[&](const SurfaceSnapshot& item) {
     return item.surface==surface && item.location.IsEqual(location);
    });
    if (found==originals.end()) throw std::invalid_argument("sew changed underlying surface");
    if (serialize_surface(surface,found->serialized.size())!=found->serialized) throw std::invalid_argument("sew mutated underlying surface");
    originals.erase(found);
   }
  }
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

// Sew many parts in one pass (one BRepBuilderAPI_Sewing over all of them, one snapshot). A
// closed sewn shell becomes an exact solid; an open result stays a shell. Folding pairwise
// re-reads, re-sews and re-tessellates the growing result at every step, which grows with the
// square of the part count (70 faces: ~23 s pairwise, ~0.1 s in one pass).
extern "C" int om9_occt_sew_many(const char* const* sources, const size_t* lengths, size_t count,
                                 double tolerance, double deflection, size_t max_points,
                                 size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
                                 void* context, char* message, int capacity) {
 try {
  if (!sources || !lengths || count < 2) throw std::invalid_argument("sew: at least two parts");
  if (!std::isfinite(tolerance) || tolerance <= 0)
   throw std::invalid_argument("sew: tolerance must be positive");
  BRepBuilderAPI_Sewing sewing(tolerance);
  for (size_t i = 0; i < count; ++i) sewing.Add(read_shape(sources[i], lengths[i], "sew part"));
  sewing.Perform();
  const TopoDS_Shape sewed = sewing.SewedShape();
  if (sewed.IsNull()) throw std::runtime_error("sew produced no shape");
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

extern "C" int om9_occt_shell(const char* source, size_t length, const size_t* face_indices, size_t face_count,
                              const double* triangles, double vertex_tolerance, double witness_deflection, double thickness,
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
  if (faces.Extent() == 0 || size_t(faces.Extent()) > max_nodes || size_t(faces.Extent()) > max_bytes)
   throw std::length_error("shell: face budget");
  std::vector<bool> selected(size_t(faces.Extent()), false);
  TopTools_ListOfShape closing;
  for (size_t i = 0; i < face_count; ++i) {
   const size_t index = triangles ? size_t(resolve_source_face(faces, triangles + 9*i, witness_deflection + vertex_tolerance, vertex_tolerance, max_nodes)) : face_indices[i];
   if (index >= size_t(faces.Extent()) || selected[index]) throw std::invalid_argument("shell face index or duplicate");
   selected[index] = true;
   closing.Append(faces.FindKey(int(index) + 1));
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
