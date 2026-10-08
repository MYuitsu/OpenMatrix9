#include "ThreeDmArchive.h"
#include "ThreeDmInventory.h"
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool value,const char* message){if(!value)throw ExchangeError(message);}
static TopoDS_Shell shell(const TopoDS_Shape& shape){TopExp_Explorer it(shape,TopAbs_SHELL);require(it.More(),"Independent curved shell");return TopoDS::Shell(it.Current());}
static double volume(const TopoDS_Shape& shape){GProp_GProps mass;BRepGProp::VolumeProperties(shape,mass,1e-13);return mass.Mass();}
int main(){try{ON::Begin();QDir directory(QStringLiteral(OM9_CURVED_SHELL_DIR));require(QDir().mkpath(directory.path()),"Curved shell proof folder");QJsonArray cases;
    const double pi=std::acos(-1.);
    for(int kind=0;kind<4;++kind){TopoDS_Shape fixture;double expected=0;
        if(kind==0){BRep_Builder builder;TopoDS_Compound group;builder.MakeCompound(group);builder.Add(group,BRepPrimAPI_MakeCylinder(3,7).Shape());builder.Add(group,BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(20,0,0),gp_Dir(0,0,1)),3,7).Shape());fixture=group;expected=126*pi;}
        else {TopoDS_Shape outside,inside;
            if(kind==1){outside=BRepPrimAPI_MakeCylinder(5,10).Shape();inside=BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(0,0,2),gp_Dir(0,0,1)),3,6).Shape();expected=(250-54)*pi;}
            if(kind==2){outside=BRepPrimAPI_MakeSphere(5).Shape();inside=BRepPrimAPI_MakeSphere(3).Shape();expected=4*pi*(125-27)/3;}
            if(kind==3){outside=BRepPrimAPI_MakeTorus(10,2).Shape();inside=BRepPrimAPI_MakeTorus(10,1).Shape();expected=60*pi*pi;}
            auto inner=shell(inside);inner.Reverse();fixture=BRepBuilderAPI_MakeSolid(shell(outside),inner).Solid();}
        require(BRepCheck_Analyzer(fixture).IsValid()&&std::abs(volume(fixture)-expected)<1e-6,"Independent analytical curved cavity material");
        auto native=exportBrep(fixture,1e-7);require(native->IsValid()&&native->IsSolid(),"Independent curved native closed graph");ON_Brep fresh;fresh.Append(*native);native=std::make_unique<ON_Brep>(fresh);require(native->SolidOrientation()==2,"Unknown direction curved-shell source");auto crc=native->DataCRC(0);
        std::cerr<<"Curved shell kind="<<kind<<" faces="<<native->m_F.Count()<<'\n';
        auto imported=importBrep(*native,1e-7);std::cerr.precision(17);std::cerr<<"Imported valid="<<BRepCheck_Analyzer(imported).IsValid()<<" volume="<<volume(imported)<<" expected="<<expected<<'\n';require(BRepCheck_Analyzer(imported).IsValid()&&std::abs(volume(imported)-expected)<1e-6,"Curved closed graph retains exact material and editable topology");
        require(resolvedBrepOrientation(*native,1e-7)==1&&native->DataCRC(0)==crc&&native->SolidOrientation()==2,"Curved +2 classification does not alter source graph/cache");
        auto name=QString("curved-shell-%1.3dm").arg(kind);auto file=std::filesystem::path(directory.filePath(name).toStdWString());ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);require(!model.AddModelGeometryComponent(native.get(),nullptr).IsEmpty(),"Independent curved model source");require(writeModelRhino5(model,file),"Curved Rhino5 writer");auto inventory=inspectArchive(file);auto record=inventory.document["records"].toArray()[0].toObject();
        cases.append(QJsonObject{{"fixture",name},{"source_sha256",inventory.document["archive_sha256"]},{"objects",QJsonArray{QJsonObject{{"uuid",record["source_uuid"]},{"faces",native->m_F.Count()},{"volume_mm3",expected}}}},{"passed",true}});
    }
    QFile output(directory.filePath("native-results.json"));require(output.open(QIODevice::WriteOnly),"Curved native oracle");output.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",cases}}).toJson());std::cout<<"Four independent curved shell profiles PASS\n";return 0;
}catch(const std::exception& error){std::cerr<<error.what()<<'\n';return 1;}}
