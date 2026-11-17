// Diagnostic only. This is not used by the product.
#pragma once
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <Precision.hxx>
#include <cmath>
#include <cstdio>

static bool experimental_full_sphere(const TopoDS_Shape& shape) {
 if (shape.IsNull() || shape.ShapeType()!=TopAbs_SOLID || shape.Orientation()!=TopAbs_FORWARD) return false;
 int face_count=0,edge_count=0,vertex_count=0,shell_count=0;
 for (TopExp_Explorer e(shape,TopAbs_FACE);e.More();e.Next()) if (++face_count>1) return false;
 for (TopExp_Explorer e(shape,TopAbs_EDGE);e.More();e.Next()) if (++edge_count>4) return false;
 for (TopExp_Explorer e(shape,TopAbs_VERTEX);e.More();e.Next()) if (++vertex_count>8) return false;
 for (TopExp_Explorer e(shape,TopAbs_SHELL);e.More();e.Next()) if (++shell_count>1) return false;
 if (face_count!=1 || edge_count!=4 || vertex_count!=8 || shell_count!=1) return false;
 TopTools_IndexedMapOfShape shells,faces,wires,edges,vertices;
 TopExp::MapShapes(shape,TopAbs_SHELL,shells); TopExp::MapShapes(shape,TopAbs_FACE,faces);
 TopExp::MapShapes(shape,TopAbs_WIRE,wires); TopExp::MapShapes(shape,TopAbs_EDGE,edges);
 TopExp::MapShapes(shape,TopAbs_VERTEX,vertices);
 if (shells.Extent()!=1 || faces.Extent()!=1 || wires.Extent()!=1 || edges.Extent()!=3 || vertices.Extent()!=2) return false;
 auto face=TopoDS::Face(faces(1)); if (face.Orientation()!=TopAbs_FORWARD) return false;
 BRepAdaptor_Surface surf(face); if (surf.GetType()!=GeomAbs_Sphere) return false;
 auto sphere=surf.Sphere(); const auto center=sphere.Location(); const double radius=sphere.Radius();
 if (!std::isfinite(radius) || radius<=2.*Precision::Confusion() || !std::isfinite(center.X()) || !std::isfinite(center.Y()) || !std::isfinite(center.Z())) return false;
 const double pi=std::acos(-1.); double u0,u1,v0,v1;
 BRepTools::UVBounds(face,u0,u1,v0,v1);
 if (u0!=0. || u1!=2.*pi || v0!=-pi/2. || v1!=pi/2.) return false;
 const double tolerance=BRep_Tool::Tolerance(face);
 if (!std::isfinite(tolerance) || tolerance<Precision::Confusion() || tolerance>1e-6) return false;
 int sides[4]={}; int seam_id=0; int seam_orientation=0;
 for (TopExp_Explorer e(face,TopAbs_EDGE);e.More();e.Next()) {
  auto edge=TopoDS::Edge(e.Current());
  if (BRep_Tool::Tolerance(edge)!=tolerance) return false;
  double first,last; auto curve=BRep_Tool::CurveOnSurface(edge,face,first,last);
  if (curve.IsNull() || !std::isfinite(first) || !std::isfinite(last) || !(first<last)) return false;
  Geom2dAdaptor_Curve c(curve,first,last); if (c.GetType()!=GeomAbs_Line) return false;
  const auto a=c.Value(first), b=c.Value(last);
  if (a.X()==u0 && b.X()==u1 && a.Y()==v1 && b.Y()==v1 && BRep_Tool::Degenerated(edge)) ++sides[0];
  else if (a.X()==u0 && b.X()==u1 && a.Y()==v0 && b.Y()==v0 && BRep_Tool::Degenerated(edge)) ++sides[1];
  else if (a.X()==b.X() && (a.X()==u0 || a.X()==u1) && a.Y()==v0 && b.Y()==v1 && !BRep_Tool::Degenerated(edge) && BRep_Tool::IsClosed(edge,face)) {
   ++sides[a.X()==u0 ? 2 : 3];
   if (seam_id && seam_id!=edges.FindIndex(edge)) return false;
   seam_id=edges.FindIndex(edge);
   if (edge.Orientation()==TopAbs_FORWARD) seam_orientation|=1;
   else if (edge.Orientation()==TopAbs_REVERSED) seam_orientation|=2;
   else return false;
   BRepAdaptor_Curve seam(edge); if (seam.GetType()!=GeomAbs_Circle) return false;
   const auto circle=seam.Circle(); if (circle.Radius()!=radius || !circle.Location().IsEqual(center,0.)) return false;
   const auto normal=circle.Axis().Direction();
   const gp_Dir expected(gp_Vec(sphere.Position().XDirection()) * std::sin(u1) - gp_Vec(sphere.Position().YDirection()) * std::cos(u1));
   if (!((normal.X()==expected.X() && normal.Y()==expected.Y() && normal.Z()==expected.Z()) ||
         (normal.X()==-expected.X() && normal.Y()==-expected.Y() && normal.Z()==-expected.Z()))) return false;
   const auto circle_y=circle.Position().YDirection(), sphere_z=sphere.Position().Direction();
   if (circle_y.X()!=sphere_z.X() || circle_y.Y()!=sphere_z.Y() || circle_y.Z()!=sphere_z.Z()) return false;
  } else return false;
 }
 for (int side:sides) if (side!=1) return false;
 if (!seam_id || seam_orientation!=3) return false;
 // Compare with OCCT's own canonical construction, not an independent
 // re-evaluation whose operation ordering changes floating-point results.
 // Diagnostic only: equality of these vertices is not a whole-shape proof.
 BRepPrimAPI_MakeSphere reference(sphere.Position().Ax2(),radius);
 const TopoDS_Shape canonical=reference.Shape();
 if (!reference.IsDone() || canonical.IsNull()) return false;
 TopTools_IndexedMapOfShape reference_vertices;
 TopExp::MapShapes(canonical,TopAbs_VERTEX,reference_vertices);
 if (reference_vertices.Extent()!=2) return false;
 int poles=0;
 for (int i=1;i<=2;++i) {
  const auto vertex=TopoDS::Vertex(vertices(i)); const double t=BRep_Tool::Tolerance(vertex);
  if (!std::isfinite(t) || t<Precision::Confusion() || t>1e-6) return false;
  const auto p=BRep_Tool::Pnt(vertex);
  const auto r1=BRep_Tool::Pnt(TopoDS::Vertex(reference_vertices(1)));
  const auto r2=BRep_Tool::Pnt(TopoDS::Vertex(reference_vertices(2)));
  if (p.IsEqual(r1,0.)) poles|=1; else if (p.IsEqual(r2,0.)) poles|=2; else return false;
 }
 if (poles!=3) return false;
 if (!BRepCheck_Analyzer(shape).IsValid()) return false;
 BRepClass3d_SolidClassifier inside(shape,center,tolerance);
 return inside.State()==TopAbs_IN;
}
