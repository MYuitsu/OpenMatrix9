// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <Precision.hxx>
#include <Standard_OutOfMemory.hxx>
#include <algorithm>
#include <array>
#include <cmath>
#include <set>
#include <new>

namespace om9_axis_box {
using Point=std::array<double,3>;
static Point point(const gp_Pnt& p) {return {p.X(),p.Y(),p.Z()};}
static bool certify(const TopoDS_Shape& s, Point& lo, Point& hi, double& tolerance) {
 if (s.IsNull() || s.ShapeType()!=TopAbs_SOLID) return false;
 // Bound exploration before allocating topology maps for arbitrary source shapes.
 int face_occurrences=0,edge_occurrences=0,vertex_occurrences=0,shell_occurrences=0;
 for (TopExp_Explorer e(s,TopAbs_FACE);e.More();e.Next()) if (++face_occurrences>6) return false;
 for (TopExp_Explorer e(s,TopAbs_EDGE);e.More();e.Next()) if (++edge_occurrences>24) return false;
 for (TopExp_Explorer e(s,TopAbs_VERTEX);e.More();e.Next()) if (++vertex_occurrences>48) return false;
 for (TopExp_Explorer e(s,TopAbs_SHELL);e.More();e.Next()) if (++shell_occurrences>1) return false;
 if (face_occurrences!=6 || edge_occurrences!=24 || vertex_occurrences!=48 || shell_occurrences!=1) return false;
 TopTools_IndexedMapOfShape shells,faces,edges,vertices;
 TopExp::MapShapes(s,TopAbs_SHELL,shells); TopExp::MapShapes(s,TopAbs_FACE,faces);
 TopExp::MapShapes(s,TopAbs_EDGE,edges); TopExp::MapShapes(s,TopAbs_VERTEX,vertices);
 if (shells.Extent()!=1 || faces.Extent()!=6 || edges.Extent()!=12 || vertices.Extent()!=8) return false;
 tolerance=BRep_Tool::Tolerance(TopoDS::Vertex(vertices(1)));
 if (!std::isfinite(tolerance) || tolerance<Precision::Confusion() || tolerance>1e-6) return false;
 lo=hi=point(BRep_Tool::Pnt(TopoDS::Vertex(vertices(1))));
 for (int i=1;i<=8;++i) {
  const auto v=TopoDS::Vertex(vertices(i)); if (BRep_Tool::Tolerance(v)!=tolerance) return false;
  const auto p=point(BRep_Tool::Pnt(v));
  for (int k=0;k<3;++k) {if (!std::isfinite(p[k])) return false; lo[k]=std::min(lo[k],p[k]); hi[k]=std::max(hi[k],p[k]);}
 }
 for (int k=0;k<3;++k) if (!std::isfinite(hi[k]-lo[k]) || hi[k]-lo[k]<=2.*tolerance) return false;
 auto corner=[&](const Point& p)->int {
  int mask=0; for (int k=0;k<3;++k) {if (p[k]==hi[k]) mask|=1<<k; else if (p[k]!=lo[k]) return -1;} return mask;
 };
 std::set<int> corners;
 for (int i=1;i<=8;++i) {int c=corner(point(BRep_Tool::Pnt(TopoDS::Vertex(vertices(i))))); if (c<0 || !corners.insert(c).second) return false;}
 std::set<std::pair<int,int>> edge_keys;
 for (int i=1;i<=12;++i) {
  auto edge=TopoDS::Edge(edges(i)); if (BRep_Tool::Degenerated(edge) || BRep_Tool::Tolerance(edge)!=tolerance) return false;
  BRepAdaptor_Curve curve(edge); if (curve.GetType()!=GeomAbs_BSplineCurve || curve.Degree()!=1 || curve.NbPoles()!=2 || curve.NbKnots()!=2 || curve.IsRational() || curve.IsPeriodic()) return false;
  auto spline=curve.BSpline(); if (spline->Degree()!=1 || spline->NbPoles()!=2 || spline->NbKnots()!=2 || spline->IsRational() || spline->IsPeriodic() || spline->Weight(1)!=spline->Weight(2)) return false;
  TopoDS_Vertex v1,v2; TopExp::Vertices(edge,v1,v2); if (v1.IsNull() || v2.IsNull()) return false;
  Point a=point(BRep_Tool::Pnt(v1)), b=point(BRep_Tool::Pnt(v2));
  Point p=point(spline->Pole(1)), q=point(spline->Pole(2));
  if (!((p==a && q==b)||(p==b && q==a))) return false;
  const auto start=point(curve.Value(curve.FirstParameter())),end=point(curve.Value(curve.LastParameter()));
  if (!((start==a && end==b)||(start==b && end==a))) return false;
  int x=corner(a),y=corner(b); if (x<0 || y<0 || (x^y)!=1 && (x^y)!=2 && (x^y)!=4) return false;
  if (!edge_keys.emplace(std::min(x,y),std::max(x,y)).second) return false;
 }
 TopTools_IndexedDataMapOfShapeListOfShape adjacency; TopExp::MapShapesAndAncestors(s,TopAbs_EDGE,TopAbs_FACE,adjacency);
 if (adjacency.Extent()!=12) return false;
 for (int i=1;i<=12;++i) if (adjacency.FindFromIndex(i).Extent()!=2) return false;
 std::set<int> sides;
 for (int i=1;i<=6;++i) {
  auto face=TopoDS::Face(faces(i)); if (BRep_Tool::Tolerance(face)!=tolerance) return false;
  TopTools_IndexedMapOfShape wires,face_edges,face_vertices;
  TopExp::MapShapes(face,TopAbs_WIRE,wires); TopExp::MapShapes(face,TopAbs_EDGE,face_edges); TopExp::MapShapes(face,TopAbs_VERTEX,face_vertices);
  if (wires.Extent()!=1 || face_edges.Extent()!=4 || face_vertices.Extent()!=4) return false;
  int edge_occurrences=0; for (TopExp_Explorer e(face,TopAbs_EDGE);e.More();e.Next()) if (++edge_occurrences>4) return false;
  if (edge_occurrences!=4) return false;
  for (int j=1;j<=4;++j) if (BRep_Tool::IsClosed(TopoDS::Edge(face_edges(j)),face)) return false;
  std::set<int> face_corners;
  for (int j=1;j<=4;++j) {int c=corner(point(BRep_Tool::Pnt(TopoDS::Vertex(face_vertices(j))))); if (c<0 || !face_corners.insert(c).second) return false;}
  int side=-1;
  for (int axis=0;axis<3;++axis) for (int bit=0;bit<2;++bit) {
   if (std::all_of(face_corners.begin(),face_corners.end(),[&](int c){return ((c>>axis)&1)==bit;})) {
    if (side!=-1) return false; side=2*axis+bit;
   }
  }
  if (side<0 || !sides.insert(side).second) return false;
  BRepAdaptor_Surface surface(face); if (surface.GetType()!=GeomAbs_BSplineSurface || surface.UDegree()!=1 || surface.VDegree()!=1 || surface.NbUPoles()!=2 || surface.NbVPoles()!=2 || surface.NbUKnots()!=2 || surface.NbVKnots()!=2 || surface.IsURational() || surface.IsVRational() || surface.IsUPeriodic() || surface.IsVPeriodic()) return false;
  auto spline=surface.BSpline();
  if (spline->UDegree()!=1 || spline->VDegree()!=1 || spline->NbUPoles()!=2 || spline->NbVPoles()!=2 || spline->NbUKnots()!=2 || spline->NbVKnots()!=2 ||
      spline->IsURational() || spline->IsVRational() || spline->IsUPeriodic() || spline->IsVPeriodic()) return false;
  std::set<int> poles; int net[2][2]; const double weight=spline->Weight(1,1);
  for (int u=1;u<=2;++u) for (int v=1;v<=2;++v) {
   if (spline->Weight(u,v)!=weight) return false;
   int c=corner(point(spline->Pole(u,v))); if (c<0 || !face_corners.count(c) || !poles.insert(c).second) return false; net[u-1][v-1]=c;
  }
  const int du=net[0][0]^net[1][0],dv=net[0][0]^net[0][1];
  if ((du!=1 && du!=2 && du!=4) || (dv!=1 && dv!=2 && dv!=4) || du==dv || net[1][1]!=(net[0][0]^du^dv)) return false;
  for (int j=1;j<=4;++j) {
   auto edge=TopoDS::Edge(face_edges(j)); double first=0.,last=0.;
   auto pcurve=BRep_Tool::CurveOnSurface(edge,face,first,last); if (pcurve.IsNull() || !std::isfinite(first) || !std::isfinite(last) || !(first<last)) return false;
   Geom2dAdaptor_Curve adapted(pcurve,first,last);
   if (adapted.GetType()==GeomAbs_BSplineCurve) {
    if (adapted.Degree()!=1 || adapted.NbPoles()!=2 || adapted.NbKnots()!=2 || adapted.IsRational() || adapted.IsPeriodic()) return false;
    auto bspline=adapted.BSpline(); if (bspline->Weight(1)!=bspline->Weight(2)) return false;
   } else if (adapted.GetType()!=GeomAbs_Line) return false;
   auto uv1=pcurve->Value(first),uv2=pcurve->Value(last);
   Point p1=point(surface.Value(uv1.X(),uv1.Y())),p2=point(surface.Value(uv2.X(),uv2.Y()));
   BRepAdaptor_Curve curve(edge); Point a=point(curve.Value(curve.FirstParameter())),b=point(curve.Value(curve.LastParameter()));
   if (!((p1==a && p2==b)||(p1==b && p2==a))) return false;
  }
 }
 gp_Pnt center(lo[0]+(hi[0]-lo[0])/2.,lo[1]+(hi[1]-lo[1])/2.,lo[2]+(hi[2]-lo[2])/2.);
 if (!BRepCheck_Analyzer(s).IsValid()) return false;
 BRepClass3d_SolidClassifier inside(s,center,tolerance);
 return inside.State()==TopAbs_IN;
}

// Null means the source did not meet every condition; keep the original operand.
static TopoDS_Shape make_operand(const TopoDS_Shape& source) {
 try {
  Point lo,hi; double tolerance=0.;
  if (!certify(source,lo,hi,tolerance)) return {};
  BRepPrimAPI_MakeBox maker(gp_Pnt(lo[0],lo[1],lo[2]),gp_Pnt(hi[0],hi[1],hi[2]));
  TopoDS_Shape result=maker.Shape();
  if (!maker.IsDone() || result.IsNull()) return {};
  TopTools_IndexedMapOfShape faces,edges,vertices;
  TopExp::MapShapes(result,TopAbs_FACE,faces); TopExp::MapShapes(result,TopAbs_EDGE,edges);
  TopExp::MapShapes(result,TopAbs_VERTEX,vertices);
  if (faces.Extent()!=6 || edges.Extent()!=12 || vertices.Extent()!=8) return {};
  BRep_Builder builder;
  for (int i=1;i<=faces.Extent();++i) builder.UpdateFace(TopoDS::Face(faces(i)),tolerance);
  for (int i=1;i<=edges.Extent();++i) builder.UpdateEdge(TopoDS::Edge(edges(i)),tolerance);
  for (int i=1;i<=vertices.Extent();++i) builder.UpdateVertex(TopoDS::Vertex(vertices(i)),tolerance);
  for (int i=1;i<=faces.Extent();++i) if (BRep_Tool::Tolerance(TopoDS::Face(faces(i)))!=tolerance) return {};
  for (int i=1;i<=edges.Extent();++i) if (BRep_Tool::Tolerance(TopoDS::Edge(edges(i)))!=tolerance) return {};
  std::set<int> corners;
  for (int i=1;i<=vertices.Extent();++i) {
   const auto vertex=TopoDS::Vertex(vertices(i));
   if (BRep_Tool::Tolerance(vertex)!=tolerance) return {};
   Point p=point(BRep_Tool::Pnt(vertex)); int mask=0;
   for (int k=0;k<3;++k) {if (p[k]==hi[k]) mask|=1<<k; else if (p[k]!=lo[k]) return {};}
   if (!corners.insert(mask).second) return {};
  }
  if (!BRepCheck_Analyzer(result).IsValid()) return {};
  return result;
 } catch (const Standard_OutOfMemory&) { throw; }
   catch (const Standard_Failure&) { return {}; }
   catch (const std::bad_alloc&) { throw; }
   catch (const std::exception&) { return {}; }
}
} // namespace om9_axis_box
