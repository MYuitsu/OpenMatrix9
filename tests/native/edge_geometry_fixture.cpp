// SPDX-License-Identifier: LGPL-2.1-or-later
// OCCT authoring APIs unavailable in Python; test-only prepared native inputs.
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>
#include <filesystem>
#include <iostream>
#include <stdexcept>
int main(int argc,char** argv){
 try{
  if(argc!=2)throw std::runtime_error("output directory required");const std::filesystem::path output(argv[1]);std::filesystem::create_directories(output);
  BRep_Builder builder;Handle(Geom_Plane) plane=new Geom_Plane(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)));
  auto face=BRepBuilderAPI_MakeFace(plane,0,10,0,5,1e-7).Face();auto edge=TopoDS::Edge(TopExp_Explorer(face,TopAbs_EDGE).Current());
  double first=0,last=0;auto original=BRep_Tool::CurveOnSurface(edge,face,first,last);
  NCollection_Array1<gp_Pnt2d> poles(1,2);poles(1)=original->Value(first);poles(2)=original->Value(last);
  NCollection_Array1<double> knots(1,2);knots(1)=0;knots(2)=1;NCollection_Array1<int> mult(1,2);mult(1)=2;mult(2)=2;
  Handle(Geom2d_BSplineCurve) different=new Geom2d_BSplineCurve(poles,knots,mult,1);
  builder.UpdateEdge(edge,different,plane,TopLoc_Location(),1e-7);builder.Range(edge,plane,TopLoc_Location(),0,1);builder.SameParameter(edge,false);builder.SameRange(edge,false);
  double a=0,b=0;BRep_Tool::CurveOnSurface(edge,face,a,b);double c=0,d=0;TopLoc_Location location;BRep_Tool::Curve(edge,location,c,d);
  if(BRep_Tool::SameParameter(edge)||BRep_Tool::SameRange(edge)||a!=0||b!=1||(c==a&&d==b))throw std::runtime_error("authored parameter domains not distinct");
  if(!BRepTools::Write(face,(output/"different-parameter-domains.brep").string().c_str()))throw std::runtime_error("write face failed");
  // Clearing3D support preserves the native pcurve and topological endpoints.
  builder.UpdateEdge(edge,Handle(Geom_Curve)(),1e-7);
  if(!BRep_Tool::Curve(edge,location,c,d).IsNull()||BRep_Tool::Degenerated(edge))throw std::runtime_error("missing nondegenerate curve fixture failed");
  if(!BRepTools::Write(face,(output/"missing-curve3d.brep").string().c_str()))throw std::runtime_error("write missing curve failed");
  const auto vertex=BRepBuilderAPI_MakeVertex(gp_Pnt(1e151,2,3)).Vertex();
  if(!BRepTools::Write(vertex,(output/"out-of-policy-vertex.brep").string().c_str()))throw std::runtime_error("write vertex failed");
  const auto sphere=BRepPrimAPI_MakeSphere(3).Shape();bool poleFound=false;
  for(TopExp_Explorer ex(sphere,TopAbs_EDGE);ex.More();ex.Next()){
   auto pole=TopoDS::Edge(ex.Current());double u=0,v=0;TopLoc_Location supportLocation;
   if(!BRep_Tool::Degenerated(pole)||!BRep_Tool::Curve(pole,supportLocation,u,v).IsNull())continue;
   gp_Trsf pose;pose.SetRotation(gp_Ax1(gp_Pnt(0,0,0),gp_Dir(1,2,3)),.7);pose.SetTranslationPart(gp_Vec(23,29,31));pole.Location(TopLoc_Location(pose));
   if(!BRepTools::Write(pole,(output/"located-null-degenerate-edge.brep").string().c_str()))throw std::runtime_error("write located pole failed");
   poleFound=true;break;
  }
  if(!poleFound)throw std::runtime_error("sphere has no native null-degenerate pole edge");
  // No CurveRepresentation at all: unlike a pole retaining a face pcurve,
  // BRep_Tool has no supporting representation location to return.
  TopoDS_Edge barePole;builder.MakeEdge(barePole);builder.Degenerated(barePole,true);
  auto endpoint=BRepBuilderAPI_MakeVertex(gp_Pnt(0,0,3)).Vertex();builder.Add(barePole,endpoint.Oriented(TopAbs_FORWARD));builder.Add(barePole,endpoint.Oriented(TopAbs_REVERSED));
  gp_Trsf barePose;barePose.SetRotation(gp_Ax1(gp_Pnt(0,0,0),gp_Dir(1,2,3)),.7);barePose.SetTranslationPart(gp_Vec(23,29,31));barePole.Location(TopLoc_Location(barePose));
  if(!BRepTools::Write(barePole,(output/"located-bare-degenerate-edge.brep").string().c_str()))throw std::runtime_error("write bare pole failed");
  std::cout<<"Authored distinct parameter domains, missing nondegenerate Curve3D and out-of-policy native vertex\n";return 0;
 }catch(const std::exception& error){std::cerr<<error.what()<<'\n';return 1;}
}
