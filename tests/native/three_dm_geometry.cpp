// OM9-FILE-012: independent kernel fixtures, not just converter round trips.
#include "ThreeDmGeometry.h"
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <iostream>
#include <stdexcept>
using namespace OpenMatrix9Gui::ThreeDm;
void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
void closeTo(double a,double b){require(std::abs(a-b)<1e-7,"numerical mismatch");}
int main(){try {
    ON::Begin();
    ON_LineCurve line(ON_3dPoint(1,2,3),ON_3dPoint(11,2,3));
    auto edge=importCurve(line,1e-7);double a,b;auto c=BRep_Tool::Curve(TopoDS::Edge(edge),a,b);
    closeTo(c->Value(a).X(),1);closeTo(c->Value(b).X(),11);
    auto output=exportCurve(BRepBuilderAPI_MakeEdge(gp_Pnt(2,3,4),gp_Pnt(7,3,4)).Edge(),1e-7);
    closeTo(output->PointAtStart().x,2);closeTo(output->PointAtEnd().x,7);
    ON_ArcCurve arc(ON_Circle(ON_Plane::World_xy,5.0));auto circle=importCurve(arc,1e-7);
    GProp_GProps length;BRepGProp::LinearProperties(circle,length);closeTo(length.Mass(),2*ON_PI*5);
    ON_NurbsCurve rational(3,true,3,3);rational.SetCV(0,ON_4dPoint(1,0,0,1));rational.SetCV(1,ON_4dPoint(1,1,0,1));rational.SetCV(2,ON_4dPoint(0,2,0,2));rational.SetKnot(0,0);rational.SetKnot(1,0);rational.SetKnot(2,1);rational.SetKnot(3,1);
    auto nr=importCurve(rational,1e-7);auto nc=BRep_Tool::Curve(TopoDS::Edge(nr),a,b);
    for(int i=0;i<=10;++i){auto p=rational.PointAt(i/10.0);auto q=nc->Value(i/10.0);closeTo(p.x,q.X());closeTo(p.y,q.Y());}
    auto box=BRepPrimAPI_MakeBox(10,10,10).Shape();auto ob=exportBrep(box,1e-7);require(ob->IsValid(),"ON box invalid");require(ob->IsSolid(),"ON box not solid");auto round=importBrep(*ob,1e-7);require(BRepCheck_Analyzer(round).IsValid(),"OCC box invalid");GProp_GProps volume;BRepGProp::VolumeProperties(round,volume);closeTo(volume.Mass(),1000);
    ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(10,0,0),ON_3dPoint(10,10,0),ON_3dPoint(0,10,0),ON_3dPoint(0,0,10),ON_3dPoint(10,0,10),ON_3dPoint(10,10,10),ON_3dPoint(0,10,10)};std::unique_ptr<ON_Brep> independent(ON_BrepBox(corners));auto ib=importBrep(*independent,1e-7);BRepGProp::VolumeProperties(ib,volume);closeTo(volume.Mass(),1000);
    for(auto shape:{BRepPrimAPI_MakeCylinder(4,10).Shape(),BRepPrimAPI_MakeSphere(4).Shape()}){auto on=exportBrep(shape,1e-7);require(on->IsSolid(),"curved ON solid open");auto occ=importBrep(*on,1e-7);require(BRepCheck_Analyzer(occ).IsValid(),"curved OCC solid invalid");GProp_GProps expected,actual;BRepGProp::VolumeProperties(shape,expected,1e-10);BRepGProp::VolumeProperties(occ,actual,1e-10);std::cout<<"curved volumes "<<expected.Mass()<<" "<<actual.Mass()<<std::endl;require(std::abs(expected.Mass()-actual.Mass())<1e-5,"curved solid volume");}
    auto outer=BRepBuilderAPI_MakePolygon(gp_Pnt(0,0,0),gp_Pnt(10,0,0),gp_Pnt(10,10,0),gp_Pnt(0,10,0),true).Wire();auto hole=BRepBuilderAPI_MakePolygon(gp_Pnt(2,2,0),gp_Pnt(2,4,0),gp_Pnt(4,4,0),gp_Pnt(4,2,0),true).Wire();BRepBuilderAPI_MakeFace face(outer);face.Add(hole);auto trimmed=importBrep(*exportBrep(face.Face(),1e-7),1e-7);GProp_GProps area;BRepGProp::SurfaceProperties(trimmed,area);closeTo(area.Mass(),96);
    ON_Mesh mesh;mesh.m_V.Append(ON_3fPoint(0,0,0));mesh.m_V.Append(ON_3fPoint(1,0,0));mesh.m_V.Append(ON_3fPoint(0,1,0));mesh.m_F.Append(ON_MeshFace{{0,1,2,2}});auto m=importMesh(mesh);require(m.vertices.size()==3&&m.faces.size()==1,"mesh count");require(exportMesh(m)->IsValid(),"mesh invalid");
    std::cout<<"OM9-FILE-012 geometry PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
