#include "ThreeDmArchive.h"
#include "ThreeDmInventory.h"
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_NurbsConvert.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepBndLib.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <Bnd_Box.hxx>
#include <gp_Pln.hxx>
#include <BRep_Tool.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Surface.hxx>
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <iostream>
#include <fstream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool v,const char* text){if(!v)throw ExchangeError(text);}
static double mass(const TopoDS_Shape& s,bool volume){GProp_GProps m;if(volume)BRepGProp::VolumeProperties(s,m,1e-13);else BRepGProp::SurfaceProperties(s,m,1e-13);return m.Mass();}
static TopoDS_Shell shell(const TopoDS_Shape& s){TopExp_Explorer it(s,TopAbs_SHELL);require(it.More(),"Fixture shell");return TopoDS::Shell(it.Current());}
static TopoDS_Wire square(double low,double high){BRepBuilderAPI_MakePolygon p;p.Add(gp_Pnt(low,low,0));p.Add(gp_Pnt(high,low,0));p.Add(gp_Pnt(high,high,0));p.Add(gp_Pnt(low,high,0));p.Close();return p.Wire();}
static int count(const TopoDS_Shape& s,TopAbs_ShapeEnum kind){int n=0;for(TopExp_Explorer it(s,kind);it.More();it.Next())++n;return n;}
static QJsonArray bounds(const TopoDS_Shape& s){
    Bnd_Box box;BRepBndLib::Add(s,box,false);double low[3],high[3];box.Get(low[0],low[1],low[2],high[0],high[1],high[2]);
    double distance=100;for(int i=0;i<3;++i)distance+=std::abs(low[i])+std::abs(high[i]);QJsonArray result;
    // Distances to six exterior planes measure trimmed native geometry. OCCT's
    // AddOptimal underbounds the Rhino-saved rational sphere in this SDK.
    for(int side:{-1,1})for(int axis=0;axis<3;++axis){gp_Pnt p(0,0,0);p.SetCoord(axis+1,side*distance);gp_Dir n(1,0,0);if(axis==1)n=gp_Dir(0,1,0);if(axis==2)n=gp_Dir(0,0,1);
        auto plane=BRepBuilderAPI_MakeFace(gp_Pln(p,n),-2*distance,2*distance,-2*distance,2*distance).Face();BRepExtrema_DistShapeShape extrema(s,plane,1e-9);extrema.Perform();require(extrema.IsDone(),"Native exterior-plane extent solved");result.append(side*(distance-extrema.Value()));}
    return result;
}
static TopoDS_Shape integrationFaces(const TopoDS_Shape& s){
    // Convert each face separately: whole-solid NurbsConvert in this OCCT SDK
    // drops cavity shells. Preserve original oriented face occurrences.
    BRep_Builder builder;TopoDS_Compound group;builder.MakeCompound(group);
    for(TopExp_Explorer it(s,TopAbs_FACE);it.More();it.Next()){
        auto face=BRepBuilderAPI_NurbsConvert(it.Current(),true).Shape();
        require(count(face,TopAbs_FACE)==1,"Mass normalization retains one face");
        face.Orientation(it.Current().Orientation());builder.Add(group,face);
    }
    require(count(group,TopAbs_FACE)==count(s,TopAbs_FACE),"Mass normalization retains every cavity face");return group;
}
static void pcurves(const TopoDS_Shape& s){for(TopExp_Explorer f(s,TopAbs_FACE);f.More();f.Next()){
    auto face=TopoDS::Face(f.Current());auto surface=BRep_Tool::Surface(face);
    for(TopExp_Explorer e(face,TopAbs_EDGE);e.More();e.Next()){
        auto edge=TopoDS::Edge(e.Current());double a,b,c,d;auto uv=BRep_Tool::CurveOnSurface(edge,face,a,b);require(!uv.IsNull(),"Native face retains pcurve including seams/poles");
        auto curve=BRep_Tool::Curve(edge,c,d);if(BRep_Tool::Degenerated(edge))continue;require(!curve.IsNull(),"Nondegenerate edge has 3D curve");
        for(int i=0;i<=16;++i){double t=a+(b-a)*i/16.;auto p=uv->Value(t);require(surface->Value(p.X(),p.Y()).Distance(curve->Value(t))<1e-5,"Common UV basis: pcurve agrees with current 3D edge");}
    }
}}
int main(int argc,char** argv){try{
    if(argc==2){std::ifstream stream(argv[1]);TopoDS_Shape shape;BRep_Builder builder;BRepTools::Read(shape,stream,builder);require(!shape.IsNull(),"Readable native measurement shape");GProp_GProps fast;BRepGProp::SurfaceProperties(shape,fast);auto converted=integrationFaces(shape);std::cout.precision(17);std::cout<<"{\"area\":"<<mass(shape,false)<<",\"area_nurbs\":"<<mass(converted,false)<<",\"area_default\":"<<fast.Mass()<<",\"volume\":"<<mass(shape,true)<<",\"volume_nurbs\":"<<mass(converted,true)<<",\"faces\":"<<count(shape,TopAbs_FACE)<<",\"bounds\":"<<QJsonDocument(bounds(shape)).toJson(QJsonDocument::Compact).constData()<<"}\n";return 0;}
    ON::Begin();QDir folder(QStringLiteral(OM9_BREP_FIXTURES));require(QDir().mkpath(folder.path()),"Fixture folder");QJsonArray rows;
    double pi=std::acos(-1.);const char* names[]={"cylinder-seam","sphere-poles","torus","cavity","open-cylinder-face","planar-hole","edited-cylinder-knots","open-cylinder-shell"};
    for(int k=0;k<8;++k){TopoDS_Shape original;double area=0,volume=0;bool closed=k<4||k==6;
        if(k==0){original=BRepPrimAPI_MakeCylinder(3,7).Shape();area=60*pi;volume=63*pi;}
        if(k==1){original=BRepPrimAPI_MakeSphere(5).Shape();area=100*pi;volume=500*pi/3;}
        if(k==2){original=BRepPrimAPI_MakeTorus(10,2).Shape();area=80*pi*pi;volume=80*pi*pi;}
        if(k==3){auto outer=BRepPrimAPI_MakeSphere(5).Shape(),inner=BRepPrimAPI_MakeSphere(3).Shape();auto cavity=shell(inner);cavity.Reverse();original=BRepBuilderAPI_MakeSolid(shell(outer),cavity).Solid();area=136*pi;volume=392*pi/3;}
        if(k==4){auto cylinder=BRepPrimAPI_MakeCylinder(3,7).Shape();for(TopExp_Explorer it(cylinder,TopAbs_FACE);it.More();it.Next())if(BRepAdaptor_Surface(TopoDS::Face(it.Current())).GetType()==GeomAbs_Cylinder)original=it.Current();area=42*pi;}
        if(k==5){BRepBuilderAPI_MakeFace face(square(0,10));auto inner=square(3,7);inner.Reverse();face.Add(inner);original=face.Face();area=84;}
        if(k==6){original=BRepPrimAPI_MakeCylinder(6,14).Shape();area=240*pi;volume=504*pi;}
        if(k==7){auto cylinder=BRepPrimAPI_MakeCylinder(3,7).Shape();BRep_Builder b;TopoDS_Shell open;b.MakeShell(open);
            for(TopExp_Explorer it(cylinder,TopAbs_FACE);it.More();it.Next()){auto face=TopoDS::Face(it.Current());if(BRepAdaptor_Surface(face).GetType()==GeomAbs_Cylinder||BRep_Tool::Surface(face)->Value(0,0).Z()<1e-7)b.Add(open,face);}original=open;area=51*pi;}
        require(BRepCheck_Analyzer(original).IsValid(),"Independent analytic fixture valid");require(std::abs(mass(original,false)-area)<1e-6,"Independent analytic area");if(closed)require(std::abs(mass(original,true)-volume)<1e-6,"Independent analytic volume");
        auto brep=exportBrep(k==6?BRepPrimAPI_MakeCylinder(3,7).Shape():original,1e-7);
        if(k==6){require(brep->Transform(ON_Xform::ScaleTransformation(ON_3dPoint::Origin,2)),"Coupled cylinder CV/edge edit");int inserted=0;
            for(int i=0;i<brep->m_S.Count();++i)if(auto* s=ON_NurbsSurface::Cast(brep->m_S[i])){auto u=s->Domain(0),v=s->Domain(1);ON_3dPoint samples[17][17];for(int a=0;a<=16;++a)for(int b=0;b<=16;++b)samples[a][b]=s->PointAt(u.ParameterAt(a/16.),v.ParameterAt(b/16.));
                require(s->InsertKnot(0,u.ParameterAt(.37),1)&&s->InsertKnot(1,v.ParameterAt(.61),1),"Edited cylinder inserts knots on common UV basis");require(s->Domain(0)==u&&s->Domain(1)==v,"Knot insertion preserves trim UV domains");
                for(int a=0;a<=16;++a)for(int b=0;b<=16;++b)require(samples[a][b].DistanceTo(s->PointAt(u.ParameterAt(a/16.),v.ParameterAt(b/16.)))<1e-10,"Knot insertion preserves interior physical surface");++inserted;}
            require(inserted>0,"Edited cylinder has rational surface knot proof");brep->DestroyRuntimeCache();}
        require(brep->IsValid()&&brep->IsSolid()==closed,"Native open/closed classification");
        auto exact=integrationFaces(original);require(std::abs(mass(exact,false)-area)<1e-5,"Normalized oracle independent analytic area includes cavities");if(closed)require(std::abs(mass(exact,true)-volume)<1e-5,"Normalized oracle signed material includes cavities");
        auto shape=importBrep(*brep,1e-7);require(BRepCheck_Analyzer(shape).IsValid(),"Imported CAD valid");require(count(shape,TopAbs_FACE)==count(original,TopAbs_FACE),"Independent topology face count");require(std::abs(mass(shape,false)-area)<1e-5,"Imported analytic area");if(closed)require(std::abs(mass(shape,true)-volume)<1e-5,"Imported analytic volume");else require(count(shape,TopAbs_SOLID)==0,"Open face never fabricated into solid");pcurves(shape);
        ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_3dmObjectAttributes attributes;attributes.m_name=ON_wString(names[k]);require(!model.AddModelGeometryComponent(brep.get(),&attributes).IsEmpty(),"Fixture model");
        auto name=QString::fromUtf8(names[k])+".3dm";auto file=std::filesystem::path(folder.filePath(name).toStdWString());require(writeModelRhino5(model,file),"Fixture V5 writer");auto inventory=inspectArchive(file);
        ONX_Model reread;require(reread.Read(file.c_str()),"Actual V5 fixture reread");ONX_ModelComponentIterator iterator(reread,ON_ModelComponent::Type::ModelGeometry);auto ref=iterator.FirstComponentReference();auto component=ON_ModelGeometryComponent::FromModelComponentRef(ref,nullptr);auto decoded=component?ON_Brep::Cast(component->Geometry(nullptr)):nullptr;require(decoded&&decoded->IsValid(),"V5 retains valid native BRep");pcurves(importBrep(*decoded,1e-7));
        rows.append(QJsonObject{{"name",QString::fromUtf8(names[k])},{"input",folder.filePath(name)},{"sha256",inventory.document["archive_sha256"]},{"area",area},{"volume",volume},{"closed",closed},{"faces",count(original,TopAbs_FACE)},{"bounds",bounds(original)}});
    }
    QFile out(folder.filePath("expected.json"));require(out.open(QIODevice::WriteOnly),"Oracle output");out.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",rows}}).toJson());std::cout<<"PASS: eight analytic BRep fixtures, edited-cylinder knots, seam/pole/hole pcurves and open shell\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
