// SPDX-License-Identifier: LGPL-2.1-or-later
// Capability-scoped OCCT construction only. No document/session state or curve fitting.
#include "brep_snapshot.hxx"
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepLib.hxx>
#include <Geom_Circle.hxx>
#include <Geom_BSplineCurve.hxx>
#include <GeomConvert.hxx>
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
extern "C" int om9_occt_extrude_exact(
    const Om9ExtrusionEdge* edges, size_t count, const double* direction, int capped, double deflection,
    size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink, void* context,
    char* message, int capacity) {
 try {
  if (!edges || !count || count > 4096 || !direction) throw std::invalid_argument("extrusion edges");
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
  if (!wire.Wire().Closed()) throw std::invalid_argument("extrusion boundary is open");
  BRepBuilderAPI_MakeFace face(wire.Wire(), true);
  if (!face.IsDone()) throw std::invalid_argument("extrusion boundary is not planar");
  if (capped < 0 || capped > 2) throw std::invalid_argument("extrusion cap mode");
  if (capped == 2 && !BRepCheck_Analyzer(face.Face()).IsValid())
   throw std::runtime_error("invalid planar extrusion profile");
  if (capped == 2)
   return om9_emit_brep_snapshot(face.Face(),deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
  const TopoDS_Shape profile = capped ? TopoDS_Shape(face.Face()) : TopoDS_Shape(wire.Wire());
  BRepPrimAPI_MakePrism prism(profile,gp_Vec(direction[0],direction[1],direction[2]));
  if (!prism.IsDone()) throw std::runtime_error("exact extrusion failed");
  TopoDS_Shape shape = prism.Shape();
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
