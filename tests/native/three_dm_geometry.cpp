// OM9-FILE-012: independent kernel fixtures, not just converter round trips.
#include "ThreeDmGeometry.h"
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Circ.hxx>
#include <Geom_Surface.hxx>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <stdexcept>
using namespace OpenMatrix9Gui::ThreeDm;
void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
void closeTo(double a,double b){require(std::abs(a-b)<1e-7,"numerical mismatch");}
int topologyCount(const TopoDS_Shape& shape,TopAbs_ShapeEnum kind)
{
    int count=0;
    for(TopExp_Explorer item(shape,kind);item.More();item.Next())++count;
    return count;
}
TopoDS_Shape curvedSurfaceArea(const char* label,const TopoDS_Shape& source,double exactArea,int wires)
{
    auto exported=exportBrep(source,1e-7);
    require(exported->IsValid(),"curved surface ON brep invalid");
    require(!exported->IsSolid(),"open surface exported as solid");
    auto imported=importBrep(*exported,1e-7);
    require(BRepCheck_Analyzer(imported).IsValid(),"curved surface OCC shape invalid");
    require(topologyCount(imported,TopAbs_FACE)==1,"curved surface face count");
    require(topologyCount(imported,TopAbs_WIRE)==wires,"curved surface wire count");
    require(topologyCount(imported,TopAbs_SOLID)==0,"open surface imported as solid");
    require(!BRep_Tool::IsClosed(imported),"open surface imported as closed shape");

    GProp_GProps sourceArea,adaptiveArea,convergedArea,defaultArea;
    BRepGProp::SurfaceProperties(source,sourceArea,1e-10);
    // FreeCAD Shape.Area uses the nonadaptive overload. Rational circular trims
    // require adaptive integration to compare geometry against an exact area.
    BRepGProp::SurfaceProperties(imported,defaultArea);
    BRepGProp::SurfaceProperties(imported,adaptiveArea,1e-10);
    // The periodic rational cylinder needs further refinement to satisfy the
    // exact-area assertion; tighten integration, never the assertion tolerance.
    BRepGProp::SurfaceProperties(imported,convergedArea,1e-14);
    std::cout<<std::setprecision(17)<<label<<" area exact="<<exactArea
             <<" source-adaptive="<<sourceArea.Mass()
             <<" imported-default="<<defaultArea.Mass()
             <<" imported-adaptive-1e-10="<<adaptiveArea.Mass()
             <<" imported-adaptive-1e-14="<<convergedArea.Mass()<<std::endl;
    closeTo(sourceArea.Mass(),exactArea);
    closeTo(convergedArea.Mass(),exactArea);
    return imported;
}
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
    auto ringOuter=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1)),10)).Edge()).Wire();
    auto ringInner=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1)),3)).Edge()).Wire();
    BRepBuilderAPI_MakeFace ring(ringOuter);ring.Add(TopoDS::Wire(ringInner.Reversed()));
    auto importedRing=curvedSurfaceArea("circular ring",ring.Face(),91*ON_PI,2);
    for(TopExp_Explorer edge(importedRing,TopAbs_EDGE);edge.More();edge.Next()){
        double first,last;auto curve=BRep_Tool::Curve(TopoDS::Edge(edge.Current()),first,last);
        const auto start=curve->Value(first);const double radius=std::hypot(start.X(),start.Y());
        require(std::abs(radius-10)<1e-7||std::abs(radius-3)<1e-7,"ring boundary radius");
        for(int i=0;i<=64;++i){const auto point=curve->Value(first+(last-first)*i/64.0);closeTo(std::hypot(point.X(),point.Y()),radius);closeTo(point.Z(),0);}
    }
    auto importedCylinder=curvedSurfaceArea("open lateral cylinder",BRepPrimAPI_MakeCylinder(4,7).Face(),56*ON_PI,1);
    TopExp_Explorer cylinderFace(importedCylinder,TopAbs_FACE);const auto lateral=TopoDS::Face(cylinderFace.Current());
    double u0,u1,v0,v1;BRepTools::UVBounds(lateral,u0,u1,v0,v1);auto surface=BRep_Tool::Surface(lateral);
    for(int i=0;i<=64;++i)for(int j=0;j<=8;++j){
        const auto point=surface->Value(u0+(u1-u0)*i/64.0,v0+(v1-v0)*j/8.0);
        closeTo(std::hypot(point.X(),point.Y()),4);require(point.Z()>=-1e-7&&point.Z()<=7+1e-7,"cylinder height range");
    }
    ON_Mesh mesh;mesh.m_V.Append(ON_3fPoint(0,0,0));mesh.m_V.Append(ON_3fPoint(1,0,0));mesh.m_V.Append(ON_3fPoint(0,1,0));mesh.m_F.Append(ON_MeshFace{{0,1,2,2}});auto m=importMesh(mesh);require(m.vertices.size()==3&&m.faces.size()==1,"mesh count");require(exportMesh(m)->IsValid(),"mesh invalid");
    std::cout<<"OM9-FILE-012 geometry PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
