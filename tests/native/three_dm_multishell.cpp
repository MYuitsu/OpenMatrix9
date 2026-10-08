#include "ThreeDmArchive.h"
#include "ThreeDmInventory.h"
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepCheck_Result.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
#include <filesystem>
#include <QDir>
#include <QFile>
#include <QJsonDocument>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool value,const char* text){if(!value)throw ExchangeError(text);}
static TopoDS_Shell shell(const TopoDS_Shape& shape){TopExp_Explorer it(shape,TopAbs_SHELL);require(it.More(),"Fixture shell");return TopoDS::Shell(it.Current());}
static double volume(const TopoDS_Shape& shape){GProp_GProps properties;BRepGProp::VolumeProperties(shape,properties,1e-10);return properties.Mass();}
int main(){try{ON::Begin();int checks=0;QJsonArray evidence;QDir directory(QStringLiteral(OM9_MULTISHELL_DIR));require(QDir().mkpath(directory.path()),"Multi-shell evidence directory");
    for(int kind=0;kind<3;++kind)for(int direction:{1,-1}){
        BRep_Builder builder;TopoDS_Shape fixture;
        if(kind==0){TopoDS_Compound group;builder.MakeCompound(group);builder.Add(group,BRepPrimAPI_MakeBox(2,3,4).Shape());builder.Add(group,BRepPrimAPI_MakeBox(gp_Pnt(10,0,0),2,3,4).Shape());fixture=group;}
        else {auto outer=shell(BRepPrimAPI_MakeBox(10,10,10).Shape());auto inner=shell(BRepPrimAPI_MakeBox(gp_Pnt(2,2,2),6,6,6).Shape());inner.Reverse();fixture=BRepBuilderAPI_MakeSolid(outer,inner).Solid();
            if(kind==2){TopoDS_Compound group;builder.MakeCompound(group);builder.Add(group,fixture);builder.Add(group,BRepPrimAPI_MakeBox(gp_Pnt(4,4,4),2,2,2).Shape());fixture=group;}}
        std::cerr<<"Multi-shell fixture kind="<<kind<<" direction="<<direction<<" positive_valid="<<BRepCheck_Analyzer(fixture).IsValid()<<'\n';
        BRepCheck_Analyzer validation(fixture);
        if(!validation.IsValid()){for(TopExp_Explorer solid(fixture,TopAbs_SOLID);solid.More();solid.Next()){auto result=validation.Result(solid.Current());if(!result.IsNull())for(auto state:result->Status())std::cerr<<"solid status="<<int(state)<<'\n';}}
        require(validation.IsValid(),"Valid independent cavity/island/disjoint fixture");
        const double expected=direction*(kind==0?48:kind==1?784:792);require(std::abs(volume(fixture)-std::abs(expected))<1e-8,"Independent analytical material volume");
        auto native=exportBrep(fixture,1e-7);require(native->IsValid()&&native->IsSolid(),"Multi-shell native BRep is solid");
        if(direction<0)native->Flip();
        ON_Brep fresh;fresh.Append(*native);native=std::make_unique<ON_Brep>(fresh);
        const auto beforeDirection=native->SolidOrientation();require(beforeDirection==2,"Append-built fixture genuinely exercises unknown orientation; expert setter2 is a no-op");auto crc=native->DataCRC(0);
        const auto fixtureName=QString("multishell-%1-direction-%2.3dm").arg(kind).arg(direction);const auto fixturePath=std::filesystem::path(directory.filePath(fixtureName).toStdWString());
        ONX_Model fixtureModel;fixtureModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);fixtureModel.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance=1e-7;require(!fixtureModel.AddModelGeometryComponent(native.get(),nullptr).IsEmpty(),"Copy independent source into fixture model");require(fixtureModel.Write(fixturePath.c_str(),5,nullptr),"Rhino5 multi-shell fixture");
        auto inventory=inspectArchive(fixturePath);require(inventory.document["issues"].toArray().isEmpty(),"Multi-shell source graph complete");auto record=inventory.document["records"].toArray()[0].toObject();
        evidence.append(QJsonObject{{"fixture",fixtureName},{"kind",kind},{"direction",direction},{"volume_mm3",expected},{"faces",kind==0?12:kind==1?12:18},{"editable",kind==0||direction>0},{"uuid",record["source_uuid"]},{"source_sha256",inventory.document["archive_sha256"]},{"source_version",inventory.document["source_version"]}});
        if(kind>0&&direction<0){
            ONX_Model block;block.m_settings=fixtureModel.m_settings;
            ON_3dmObjectAttributes memberAttributes;memberAttributes.SetMode(ON::idef_object);
            auto member=block.AddModelGeometryComponent(native.get(),&memberAttributes);
            require(!member.IsEmpty(),"Independent native-only block member fixture");
            ON_InstanceDefinition definition;definition.SetName(L"Native inward cavity");definition.AddInstanceGeometryId(member.ModelComponent()->Id());
            auto owner=block.AddModelComponent(definition);require(!owner.IsEmpty(),"Independent native-only block definition fixture");
            ON_InstanceRef instance;instance.m_instance_definition_uuid=owner.ModelComponent()->Id();instance.m_xform=ON_Xform::IdentityTransformation;
            require(!block.AddModelGeometryComponent(&instance,nullptr).IsEmpty(),"Independent native-only block root fixture");
            auto blockPath=std::filesystem::path(directory.filePath(QString("native-only-block-%1-v5.3dm").arg(kind)).toStdWString());
            require(block.Write(blockPath.c_str(),5,nullptr),"Native-only V5 block fixture write");
            auto blockInventory=inspectArchive(blockPath);require(blockInventory.document["source_version"]==50&&blockInventory.document["issues"].toArray().isEmpty(),"Native-only V5 block fixture reference closure");
            bool unavailable=false;try{importBrep(*native,1e-7);}catch(const GeometryRepresentationUnavailable&){unavailable=true;}
            require(unavailable,"Inward cavity must be reported as a native-only representation, not invalid source");++checks;
            require(resolvedBrepOrientation(*native,1e-7)==-1,"Native-only inward cavity +2 direction remains classifiable without an invalid editable solid");++checks;
            require(native->DataCRC(0)==crc&&native->SolidOrientation()==2,"Native-only orientation classification leaves original graph and cache intact");++checks;
            ON_Brep mirrored(*native);ON_Xform mirror=ON_Xform::IdentityTransformation;mirror[0][0]=-1;transformNativeGeometry(mirrored,mirror);require(mirrored.IsValid()&&mirrored.SolidOrientation()==1&&std::abs(volume(importBrep(mirrored,1e-7))+expected)<1e-6,"Unknown native-only inward cavity reflection yields valid outward material with correct volume");++checks;
            ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_3dmObjectAttributes attributes;model.AddModelGeometryComponent(native.get(),&attributes);
            auto file=std::filesystem::temp_directory_path()/("om9-inward-cavity-"+std::to_string(kind)+".3dm");require(model.Write(file.c_str(),80,nullptr),"Native-only source fixture");
            auto retained=readArchive(file,0,true);require(retained.items.size()==1&&retained.items[0].retained&&!retained.items[0].representationIssue.empty(),"Preserve-mode native-only BRep retained with reason");++checks;
            bool rejected=false;try{readArchive(file,0,false);}catch(const ExchangeError&){rejected=true;}require(rejected,"Geometry-only import cannot claim native-only cavity is editable");++checks;continue;
        }
        auto host=importBrep(*native,1e-7);require(std::abs(volume(host)-expected)<1e-6,"Multi-shell import retains cavity/island/disjoint material and sign");++checks;
        require(resolvedBrepOrientation(*native,1e-7)==direction,"SDK +2 multi-shell orientation classification");++checks;
        if(native->DataCRC(0)!=crc||native->SolidOrientation()!=beforeDirection)std::cerr<<"source changed crc="<<crc<<" after="<<native->DataCRC(0)<<" dir="<<beforeDirection<<" after="<<native->SolidOrientation()<<'\n';
        require(native->DataCRC(0)==crc&&native->SolidOrientation()==beforeDirection,"Classification leaves authoritative native source unchanged");++checks;
        auto output=exportBrep(host,1e-7);require(std::abs(volume(importBrep(*output,1e-7))-expected)<1e-6,"Multi-shell export reimport preserves signed volume");++checks;
        ON_Xform mirror=ON_Xform::IdentityTransformation;mirror[0][0]=-1;transformNativeGeometry(*native,mirror);
        if(kind==0)require(std::abs(volume(importBrep(*native,1e-7))+expected)<1e-6,"Reflection preserves disjoint signed parity");
        else{bool retained=false;try{importBrep(*native,1e-7);}catch(const GeometryRepresentationUnavailable&){retained=true;}require(retained&&native->IsValid()&&native->SolidOrientation()==-1,"Reflected cavity retains valid native winding without an invalid editable OCCT solid");}++checks;
    }
    QFile report(directory.filePath("native-results.json"));require(report.open(QIODevice::WriteOnly),"Multi-shell evidence report");report.write(QJsonDocument(QJsonObject{{"ok",true},{"checks",checks},{"cases",evidence}}).toJson());
    std::cout<<"Multi-shell BRep PASS "<<checks<<" checks\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
