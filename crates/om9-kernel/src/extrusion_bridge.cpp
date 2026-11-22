// SPDX-License-Identifier: LGPL-2.1-or-later
// Capability-scoped OCCT construction only. No document/session state or curve fitting.
#include "brep_snapshot.hxx"
#include <BRepAlgoAPI_Splitter.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepOffsetAPI_ThruSections.hxx>
#include <BRepOffsetAPI_MakeDraft.hxx>
#include <BRepBuilderAPI_FindPlane.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <Geom_Plane.hxx>
#include <Geom_BezierCurve.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <BRepLib.hxx>
#include <Geom_Circle.hxx>
#include <Geom_BSplineCurve.hxx>
#include <GeomConvert.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <gp_Ax2.hxx>
#include <gp_Vec.hxx>
#include <algorithm>
#include <vector>

struct Om9ExtrusionEdge {
 int kind, degree, reversed;
 double params[12];
 const double* poles;
 size_t pole_count;
 const double* weights;
 size_t weight_count;
 const double* knots;
 size_t knot_count;
 double domain[2];
};
// Keep the portion of the prism that contains its original cap. A partial
// intersection (including a hole through the cutter) must not leave a path
// through that portion to the far cap.
static void split_at_boundary(BRepAlgoAPI_Splitter& split, const TopoDS_Shape& source, const char* boundary, size_t length) {
 std::istringstream stream(std::string(boundary, length));
 TopoDS_Shape tool;
 BRep_Builder reader;
 BRepTools::Read(tool, stream, reader);
 TopTools_IndexedMapOfShape faces;
 if (!tool.IsNull()) TopExp::MapShapes(tool, TopAbs_FACE, faces);
 if (faces.Extent() != 1) throw std::invalid_argument("Select one boundary face");
 NCollection_List<TopoDS_Shape> targets, tools;
 targets.Append(source);
 tools.Append(faces(1));
 split.SetArguments(targets);
 split.SetTools(tools);
 split.Build();
 if (!split.IsDone() || split.HasErrors()) throw std::runtime_error("Cannot split extrusion at boundary");
}

static TopoDS_Shape cut_prism(BRepPrimAPI_MakePrism& prism, const char* boundary, size_t length) {
 BRepAlgoAPI_Splitter split;
 split_at_boundary(split, prism.Shape(), boundary, length);
 TopTools_IndexedMapOfShape solids;
 TopExp::MapShapes(split.Shape(), TopAbs_SOLID, solids);
 if (solids.Extent() < 2) throw std::invalid_argument("Boundary does not cut through the extrusion");
 const TopoDS_Shape base = prism.FirstShape(), far_cap = prism.LastShape();
 auto contains = [&](const TopTools_IndexedMapOfShape& part, const TopoDS_Shape& cap) {
  if (part.Contains(cap)) return true;
  for (const auto& fragment : split.Modified(cap)) if (part.Contains(fragment)) return true;
  return false;
 };
 TopoDS_Shape result;
 for (const auto& solid : solids) {
  TopTools_IndexedMapOfShape part;
  TopExp::MapShapes(solid, TopAbs_FACE, part);
  if (!contains(part, base)) continue;
  if (contains(part, far_cap) || !result.IsNull())
   throw std::invalid_argument("Boundary does not cover the extrusion profile");
  result = solid;
 }
 if (result.IsNull()) throw std::invalid_argument("Cannot identify the base of the extrusion");
 // A planar intersection can introduce an extra vertex inside one cap edge.
 // Join edge fragments shared by the same faces while keeping all side faces.
 ShapeUpgrade_UnifySameDomain join_edges(result, true, false, true);
 join_edges.Build();
 return join_edges.Shape();
}

// Each ruled side touches its base edge. Keep the split side(s) touching the
// complete original wire, or the single solid containing it for a capped loft.
static TopoDS_Shape cut_apex(const TopoDS_Shape& loft, const TopoDS_Shape& base,
                            const gp_Pnt& apex, bool solid, const char* boundary, size_t length) {
 BRepAlgoAPI_Splitter split;
 split_at_boundary(split, loft, boundary, length);
 TopTools_IndexedMapOfShape parts, base_edges, kept, covered;
 TopExp::MapShapes(split.Shape(), solid ? TopAbs_SOLID : TopAbs_FACE, parts);
 TopExp::MapShapes(base, TopAbs_EDGE, base_edges);
 for (const auto& part : parts) {
  TopTools_IndexedMapOfShape edges;
  TopExp::MapShapes(part, TopAbs_EDGE, edges);
  for (const auto& base_edge : base_edges) {
   bool has_base = edges.Contains(base_edge);
   for (const auto& changed : split.Modified(base_edge)) has_base |= edges.Contains(changed);
   if (!has_base) continue;
   covered.Add(base_edge);
   if (kept.Contains(part)) continue;
   for (TopExp_Explorer v(part, TopAbs_VERTEX); v.More(); v.Next())
    if (BRep_Tool::Pnt(TopoDS::Vertex(v.Current())).Distance(apex) <= Precision::Confusion())
     throw std::invalid_argument("Boundary does not fully separate the apex from the profile");
   kept.Add(part);
  }
 }
 if (kept.IsEmpty() || covered.Extent() != base_edges.Extent() || (solid && kept.Extent() != 1))
  throw std::invalid_argument("Cannot identify base side of boundary (pieces " + std::to_string(parts.Extent()) + ", retained " + std::to_string(kept.Extent()) + ", base edges " + std::to_string(covered.Extent()) + "/" + std::to_string(base_edges.Extent()) + ")");
 TopoDS_Shape result = kept(1);
 if (kept.Extent() > 1) {
  BRepBuilderAPI_Sewing sewing;
  for (const auto& side : kept) sewing.Add(side);
  sewing.Perform();
  result = sewing.SewedShape();
 }
 ShapeUpgrade_UnifySameDomain unify(result, true, false, true);
 unify.Build();
 return unify.Shape();
}

using DraftCornerHandle = double (*)(double chord, double tangent_cosine);

// A rounded transition in MakeDraft is a ruled wedge with a singular edge at
// the source corner. Replace only this wedge; its two rails remain coincident
// with the neighboring draft faces. Source faces themselves stay exact.
static TopoDS_Shape smooth_draft(const TopoDS_Shape& shell, DraftCornerHandle handle) {
 BRepBuilderAPI_Sewing sewing(Precision::Confusion());
 bool replaced = false;
 for (TopExp_Explorer it(shell, TopAbs_FACE); it.More(); it.Next()) {
  const auto& face = it.Current();
  TopTools_IndexedMapOfShape edges;
  TopExp::MapShapes(face, TopAbs_EDGE, edges);
  TopoDS_Vertex apex;
  for (const auto& item : edges) {
   const auto& edge = TopoDS::Edge(item);
   if (BRep_Tool::Degenerated(edge)) apex = TopExp::FirstVertex(edge);
  }
  if (apex.IsNull()) { sewing.Add(face); continue; }
  TopoDS_Edge far;
  for (const auto& item : edges) {
   const auto& edge = TopoDS::Edge(item);
   if (!TopExp::FirstVertex(edge).IsSame(apex) && !TopExp::LastVertex(edge).IsSame(apex)) {
    if (!far.IsNull()) throw std::invalid_argument("Ambiguous smooth taper boundary");
    far = edge;
   }
  }
  if (far.IsNull()) throw std::invalid_argument("Cannot identify smooth taper boundary");
  BRepAdaptor_Curve curve(far);
  gp_Pnt a,b;
  gp_Vec ta,tb;
  curve.D1(curve.FirstParameter(),a,ta);
  curve.D1(curve.LastParameter(),b,tb);
  ta.Normalize(); tb.Normalize();
  const double length = handle(a.Distance(b),ta.Dot(tb));
  TColgp_Array1OfPnt poles(1,4);
  poles(1)=a; poles(2)=a.Translated(ta*length);
  poles(3)=b.Translated(tb*(-length)); poles(4)=b;
  BRepBuilderAPI_MakeEdge top(new Geom_BezierCurve(poles));
  BRepOffsetAPI_ThruSections wedge(false,true,Precision::Confusion());
  wedge.CheckCompatibility(false);
  wedge.AddVertex(apex);
  wedge.AddWire(BRepBuilderAPI_MakeWire(top.Edge()).Wire());
  wedge.Build();
  if (!wedge.IsDone()) throw std::invalid_argument("Cannot build smooth taper corner");
  sewing.Add(wedge.Shape());
  replaced = true;
 }
 if (!replaced) return shell;
 sewing.Perform();
 return sewing.SewedShape();
}

static TopoDS_Wire extrusion_wire(const Om9ExtrusionEdge* edges,size_t count,size_t max_points,size_t max_nodes) {
  BRepBuilderAPI_MakeWire wire;
  for (size_t i = 0; i < count; ++i) {
   const auto& e = edges[i];
   const auto* p = e.params;
   TopoDS_Edge edge;
   if (e.kind == 0) {
    BRepBuilderAPI_MakeEdge make(gp_Pnt(p[0],p[1],p[2]), gp_Pnt(p[3],p[4],p[5]));
    if (!make.IsDone()) throw std::invalid_argument("extrusion line edge");
    edge = make.Edge();
   } else if (e.kind == 1) {
    const gp_Vec x(p[3],p[4],p[5]), y(p[6],p[7],p[8]);
    Handle(Geom_Circle) curve = new Geom_Circle(gp_Ax2(gp_Pnt(p[0],p[1],p[2]), gp_Dir(x.Crossed(y)), gp_Dir(x)), p[9]);
    BRepBuilderAPI_MakeEdge make(curve,e.domain[0],e.domain[1]);
    if (!make.IsDone()) throw std::invalid_argument("extrusion circle edge");
    edge = make.Edge();
   } else if (e.kind == 2) {
    if (!e.poles || !e.knots || e.pole_count < 2 || e.pole_count > max_points
        || e.knot_count > max_nodes || e.knot_count < 2
        || (e.weight_count && (!e.weights || e.weight_count != e.pole_count)))
     throw std::invalid_argument("extrusion NURBS buffers");
    TColgp_Array1OfPnt poles(1,int(e.pole_count));
    TColStd_Array1OfReal weights(1,int(e.pole_count));
    for (int j=0;j<int(e.pole_count);++j) {
     poles.SetValue(j+1,gp_Pnt(e.poles[3*j],e.poles[3*j+1],e.poles[3*j+2]));
     weights.SetValue(j+1,e.weight_count ? e.weights[j] : 1.0);
    }
    std::vector<double> knots;
    std::vector<int> mults;
    for (size_t j=0;j<e.knot_count;++j) {
     if (knots.empty() || knots.back()!=e.knots[j]) {knots.push_back(e.knots[j]);mults.push_back(1);}
     else ++mults.back();
    }
    TColStd_Array1OfReal unique(1,int(knots.size()));
    TColStd_Array1OfInteger multiplicities(1,int(knots.size()));
    for (int j=0;j<int(knots.size());++j) {unique.SetValue(j+1,knots[j]);multiplicities.SetValue(j+1,mults[j]);}
    // The canonical expanded vector defines the basis; no SetPeriodic conversion.
    // A full closed basis has the same locus after a seam shift; trims are refused in Rust.
    const auto start_mult=std::count(e.knots,e.knots+e.knot_count,e.domain[0]);
    const auto end_mult=std::count(e.knots,e.knots+e.knot_count,e.domain[1]);
    if (count==1 && start_mult>1 && end_mult>1 && start_mult<=e.degree && end_mult<=e.degree) {
     // Normalize parameter units before constructing the trimmed bases: very
     // small or translated knot domains can destabilize OCCT mass integration.
     const double span=e.domain[1]-e.domain[0];
     TColStd_Array1OfReal normalized(1,int(knots.size()));
     for (int j=0;j<int(knots.size());++j) normalized.SetValue(j+1,(knots[j]-e.domain[0])/span);
     Handle(Geom_BSplineCurve) stable = new Geom_BSplineCurve(poles,weights,normalized,multiplicities,e.degree,false);
     auto left=GeomConvert::SplitBSplineCurve(stable,0.,0.5,1e-12);
     auto right=GeomConvert::SplitBSplineCurve(stable,0.5,1.,1e-12);
     BRepBuilderAPI_MakeEdge first(left), second(right);
     if (!first.IsDone() || !second.IsDone()) throw std::invalid_argument("extrusion split NURBS edge");
     TopoDS_Edge a=first.Edge(), b=second.Edge();
     if (e.reversed) {a.Reverse();b.Reverse();std::swap(a,b);}
     wire.Add(a);if (!wire.IsDone()) throw std::invalid_argument("extrusion split boundary is disconnected");
     wire.Add(b);if (!wire.IsDone()) throw std::invalid_argument("extrusion split boundary is disconnected");
     continue;
    }
    Handle(Geom_BSplineCurve) curve = new Geom_BSplineCurve(poles,weights,unique,multiplicities,e.degree,false);
    BRepBuilderAPI_MakeEdge make(curve,e.domain[0],e.domain[1]);
    if (!make.IsDone()) throw std::invalid_argument("extrusion NURBS edge");
    edge = make.Edge();
   } else throw std::invalid_argument("extrusion edge type");
   if (e.reversed) edge.Reverse();
   wire.Add(edge);
   if (!wire.IsDone()) throw std::invalid_argument("extrusion boundary is disconnected");
  }
 return wire.Wire();
}

extern "C" int om9_occt_extrude_exact(
    const Om9ExtrusionEdge* edges, size_t count, const double* direction, int capped, double angle, DraftCornerHandle corner_handle, const char* boundary, size_t boundary_len, double deflection,
    size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink, void* context,
    char* message, int capacity) {
 try {
  if (!edges || !count || count > 4096 || !direction) throw std::invalid_argument("extrusion edges");
  const TopoDS_Wire wire = extrusion_wire(edges,count,max_points,max_nodes);
  if (capped < 0 || capped > 8 || (boundary && ((capped != 1 && capped != 4 && capped != 5) || !boundary_len || boundary_len > max_bytes)))
   throw std::invalid_argument("extrusion cap mode or boundary");
  if (capped >= 6 && capped <= 8) {
   const gp_Vec travel(direction[0],direction[1],direction[2]);
   const gp_Dir axis(travel);
   bool internal = angle > 0.;
   if (wire.Closed()) {
    BRepBuilderAPI_FindPlane support(wire);
    if (!support.Found()) throw std::invalid_argument("Taper of closed spatial wire requires a projected orientation");
    gp_Pln plane = support.Plane()->Pln();
    if (plane.Axis().Direction().Dot(axis) < 0.) plane.UReverse();
    BRepBuilderAPI_MakeFace oriented(plane,wire,false);
    if (!oriented.IsDone()) throw std::invalid_argument("Cannot orient taper profile");
    GProp_GProps area;
    BRepGProp::SurfaceProperties(oriented.Face(),area);
    if (area.Mass() < 0.) internal = !internal;
   }
   BRepOffsetAPI_MakeDraft draft(wire,axis,std::abs(angle));
   draft.SetDraft(internal);
   draft.SetOptions(capped != 6 ? BRepBuilderAPI_RoundCorner : BRepBuilderAPI_RightCorner);
   draft.Perform(travel.Magnitude()/std::cos(angle));
   if (!draft.IsDone()) throw std::invalid_argument("Cannot construct the drafted sides");
   auto shape = capped == 8 ? smooth_draft(draft.Shape(),corner_handle) : draft.Shape();
   if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
    throw std::invalid_argument("Cannot draft this profile at the chosen height and angle");
   return om9_emit_brep_snapshot(shape,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
  }
  // Apex modes use the same admitted wire and BRep snapshot as the prism.
  if (capped == 4 || capped == 5) {
   const bool solid = capped == 5;
   if (solid && (!wire.Closed() || !BRepBuilderAPI_MakeFace(wire, true).IsDone()))
    throw std::invalid_argument("solid to point requires a closed planar profile");
   BRepOffsetAPI_ThruSections loft(solid, true, Precision::Confusion());
   loft.CheckCompatibility(false);
   loft.AddWire(wire);
   gp_Pnt apex(direction[0], direction[1], direction[2]);
   loft.AddVertex(BRepBuilderAPI_MakeVertex(apex));
   loft.Build();
   if (!loft.IsDone() || loft.Shape().IsNull() || !BRepCheck_Analyzer(loft.Shape()).IsValid())
    throw std::invalid_argument("cannot extrude this profile to the selected point");
   auto result = boundary ? cut_apex(loft.Shape(), wire, apex, solid, boundary, boundary_len) : loft.Shape();
   if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid()) throw std::invalid_argument("invalid extrusion to point");
   return om9_emit_brep_snapshot(result,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
  }
  TopoDS_Shape profile = wire;
  if (capped != 3) {
   if (!wire.Closed()) throw std::invalid_argument("extrusion boundary is open");
   BRepBuilderAPI_MakeFace face(wire, true);
   if (!face.IsDone()) throw std::invalid_argument("extrusion boundary is not planar");
   if (capped == 2)
    return om9_emit_brep_snapshot(face.Face(),deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
   if (capped == 1) profile = face.Face();
  }
  BRepPrimAPI_MakePrism prism(profile,gp_Vec(direction[0],direction[1],direction[2]));
  if (!prism.IsDone()) throw std::runtime_error("exact extrusion failed");
  TopoDS_Shape shape = boundary ? cut_prism(prism, boundary, boundary_len) : prism.Shape();
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid()) throw std::runtime_error("invalid exact extrusion");
  return om9_emit_brep_snapshot(shape,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity>0) std::snprintf(message,capacity,"%s",e.what()); return 1;
 } catch (...) {
  if (message && capacity>0) std::snprintf(message,capacity,"Unknown exact extrusion failure"); return 2;
 }
}

// Keep the component touching the base. Section edges disconnect the two sides
// of a split open shell. On solids the splitter already returns separate pieces.
extern "C" int om9_occt_clip_draft(const Om9ExtrusionEdge* edges,size_t count,const char* body,size_t body_len,
 const char* boundary,size_t boundary_len,double tolerance,double deflection,size_t max_points,size_t max_nodes,
 size_t max_bytes,Om9BrepSink sink,void* context,char* message,int capacity) {
 try {
  if (!edges || !count || count > 4096 || !body || !body_len || !boundary || !boundary_len)
   throw std::invalid_argument("draft boundary inputs");
  const auto wire=extrusion_wire(edges,count,max_points,max_nodes);
  std::istringstream stream(std::string(body,body_len));
  TopoDS_Shape source;
  BRep_Builder builder;
  BRepTools::Read(source,stream,builder);
  if(source.IsNull()) throw std::invalid_argument("draft boundary shape");
  BRepAlgoAPI_Splitter splitter;
  split_at_boundary(splitter,source,boundary,boundary_len);
  TopTools_IndexedMapOfShape solids,parts,profile_vertices,far_vertices,original,section;
  TopExp::MapShapes(source,TopAbs_SOLID,solids);
  const bool solid=!solids.IsEmpty();
  TopExp::MapShapes(splitter.Shape(),solid?TopAbs_SOLID:TopAbs_FACE,parts);
  TopExp::MapShapes(source,TopAbs_VERTEX,original);
  if (size_t(parts.Extent())>max_nodes || size_t(original.Extent())>max_nodes) throw std::runtime_error("draft split budget");
  BRepExtrema_DistShapeShape distance;
  distance.LoadS1(wire);
  for(const auto& v: original){
   distance.LoadS2(v);distance.Perform();
   if(!distance.IsDone()) throw std::invalid_argument("Cannot identify draft profile");
   auto& set = distance.Value()<=tolerance ? profile_vertices : far_vertices;
   set.Add(v);
   for(const auto& changed:splitter.Modified(v)) set.Add(changed);
  }
  for(const auto& e:splitter.SectionEdges()) section.Add(e);
  TopTools_IndexedDataMapOfShapeListOfShape edge_faces;
  if(!solid) TopExp::MapShapesAndAncestors(splitter.Shape(),TopAbs_EDGE,TopAbs_FACE,edge_faces);
  TopTools_IndexedMapOfShape kept;
  for(const auto& part:parts) {
   for(TopExp_Explorer v(part,TopAbs_VERTEX);v.More();v.Next()) if(profile_vertices.Contains(v.Current())) {
    kept.Add(part);break;
   }
  }
  for(int i=1;i<=kept.Extent();++i) {
   const auto part=kept(i);
   for(TopExp_Explorer v(part,TopAbs_VERTEX);v.More();v.Next()) if(far_vertices.Contains(v.Current()))
    throw std::invalid_argument("Boundary does not cut through the full taper");
   if(!solid) for(TopExp_Explorer e(part,TopAbs_EDGE);e.More();e.Next()) {
    if(section.Contains(e.Current())) continue;
    for(const auto& neighbor:edge_faces.FindFromKey(e.Current())) kept.Add(neighbor);
   }
  }
  if(kept.IsEmpty() || kept.Extent()==parts.Extent() || (solid && kept.Extent()!=1))
   throw std::invalid_argument("Cannot identify the base side of the tapered extrusion");
  TopoDS_Shape result=kept(1);
  if(kept.Extent()>1) {
   BRepBuilderAPI_Sewing sewing(tolerance);
   for(const auto& part:kept) sewing.Add(part);
   sewing.Perform();result=sewing.SewedShape();
  }
  ShapeUpgrade_UnifySameDomain join(result,true,false,true);
  join.Build();result=join.Shape();
  if(result.IsNull() || !BRepCheck_Analyzer(result).IsValid()) throw std::invalid_argument("Invalid clipped taper");
  return om9_emit_brep_snapshot(result,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 }catch(const Standard_Failure& e){
  if(message && capacity>0) std::snprintf(message,capacity,"%s",e.what());return 2;
 }catch(const std::exception& e){
  if(message && capacity>0) std::snprintf(message,capacity,"%s",e.what());return 1;
 }catch(...){if(message && capacity>0) std::snprintf(message,capacity,"Draft split failed");return 2;}
}
