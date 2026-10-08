#include "ThreeDmArchive.h"
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
int main(){try{
    ON::Begin();auto root=std::filesystem::path(OM9_MODELING_FIXTURES);std::filesystem::create_directories(root);
    for(auto units:{ON::LengthUnitSystem::Millimeters,ON::LengthUnitSystem::Centimeters}){
        ONX_Model model;ON_Layer layer;layer.SetName(L"Working::Geometry");auto l=model.AddModelComponent(layer);
        model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(units);model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance=1e-7;
        ON_3dmObjectAttributes attributes;attributes.m_layer_index=ON_Layer::Cast(l.ModelComponent())->Index();
        auto add=[&](ON_Geometry& geometry,const wchar_t* name){attributes.m_uuid=ON_nil_uuid;attributes.m_name=name;require(!model.AddModelGeometryComponent(&geometry,&attributes).IsEmpty(),"fixture object insertion");};
        ON_ArcCurve arc(ON_Arc(ON_Circle(ON_xy_plane,2),ON_PI));add(arc,L"Arc");
        ON_NurbsCurve nurbs(3,false,3,3);nurbs.SetCV(0,ON_3dPoint(0,0,0));nurbs.SetCV(1,ON_3dPoint(2,3,0));nurbs.SetCV(2,ON_3dPoint(4,0,0));nurbs.SetKnot(0,0);nurbs.SetKnot(1,0);nurbs.SetKnot(2,1);nurbs.SetKnot(3,1);require(nurbs.IsValid(),"NURBS fixture validity");add(nurbs,L"Nurbs");
        ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(2,0,0),ON_3dPoint(2,3,0),ON_3dPoint(0,3,0),ON_3dPoint(0,0,4),ON_3dPoint(2,0,4),ON_3dPoint(2,3,4),ON_3dPoint(0,3,4)};
        std::unique_ptr<ON_Brep> box(ON_BrepBox(corners));require(bool(box),"box fixture");add(*box,L"Box");
        ON_PointCloud cloud;cloud.m_P.Append(ON_3dPoint(1,2,3));cloud.m_P.Append(ON_3dPoint(-1,4,5));add(cloud,L"Cloud");
        auto path=root/(units==ON::LengthUnitSystem::Millimeters?"mm.3dm":"cm.3dm");require(model.Write(path.c_str(),5,nullptr),"unit fixture write");
        auto received=readArchive(path);require(received.items.size()==4,"all typed geometry imported");double scale=units==ON::LengthUnitSystem::Millimeters?1:10;
        require(received.scaleMm==scale,"file-unit scaling");
        auto shape=std::get<TopoDS_Shape>(received.items[2].geometry);require(BRepCheck_Analyzer(shape).IsValid(),"trimmed solid valid");GProp_GProps volume;BRepGProp::VolumeProperties(shape,volume,1e-10);require(std::abs(volume.Mass()-24*scale*scale*scale)<1e-6,"scaled solid volume");
        require(std::get<ON_PointCloud>(received.items[3].geometry).m_P[0].y==2*scale,"native cloud unit scaling");
        ExchangeModel current;current.items={received.items[0],received.items[3]};writeArchive5(current,root/(scale==1?"mm-current.3dm":"cm-current.3dm"));require(readArchive(root/(scale==1?"mm-current.3dm":"cm-current.3dm")).items.size()==2,"selected current output");
        if(scale==1){
            ONX_Model blocks;blocks.AddModelComponent(layer);blocks.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Centimeters);
            ON_3dmObjectAttributes memberAttributes;memberAttributes.SetMode(ON::idef_object);
            auto solidMember=blocks.AddModelGeometryComponent(box.get(),&memberAttributes);
            auto cloudMember=blocks.AddModelGeometryComponent(&cloud,&memberAttributes);
            ON_InstanceDefinition inner;inner.SetName(L"Inner");inner.SetInstanceDefinitionType(ON_InstanceDefinition::IDEF_UPDATE_TYPE::Static);inner.AddInstanceGeometryId(solidMember.ModelComponent()->Id());inner.AddInstanceGeometryId(cloudMember.ModelComponent()->Id());auto innerRef=blocks.AddModelComponent(inner);
            ON_InstanceRef nested;nested.m_instance_definition_uuid=innerRef.ModelComponent()->Id();nested.m_xform=ON_Xform::TranslationTransformation(ON_3dVector(1,2,0));auto nestedMember=blocks.AddModelGeometryComponent(&nested,&memberAttributes);
            ON_InstanceDefinition outer;outer.SetName(L"Outer");outer.SetInstanceDefinitionType(ON_InstanceDefinition::IDEF_UPDATE_TYPE::Static);outer.AddInstanceGeometryId(nestedMember.ModelComponent()->Id());auto outerRef=blocks.AddModelComponent(outer);
            for(int i=0;i<9;++i){ON_InstanceRef instance;instance.m_instance_definition_uuid=outerRef.ModelComponent()->Id();instance.m_xform=ON_Xform::IdentityTransformation;instance.m_xform[0][0]=i%2?-1.1:1.1;instance.m_xform[1][1]=1.3;instance.m_xform[2][2]=0.9;instance.m_xform[0][3]=20*i;ON_3dmObjectAttributes a;a.m_name=ON_wString((std::string("Placed")+std::to_string(i)).c_str());blocks.AddModelGeometryComponent(&instance,&a);}
            require(blocks.Write((root/"blocks.3dm").c_str(),5,nullptr),"nested affine fixture write");
            require(readArchive(root/"blocks.3dm").items.size()==18,"nested affine roots expand all current geometry");
            ONX_Model layout;layout.AddModelComponent(layer);layout.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
            for(int i=0;i<9;++i){ON_LineCurve modelLine(ON_3dPoint(10*i,0,0),ON_3dPoint(10*i+5,0,0));ON_3dmObjectAttributes a;a.m_name=ON_wString((std::string("Model")+std::to_string(i)).c_str());layout.AddModelGeometryComponent(&modelLine,&a);}
            ON_UUID pageId;ON_CreateUuid(pageId);ON_3dmObjectAttributes page;page.m_space=ON::page_space;page.m_viewport_id=pageId;page.m_name=L"PageBorder";
            ON_LineCurve border(ON_3dPoint(10000,10000,0),ON_3dPoint(20000,10000,0));layout.AddModelGeometryComponent(&border,&page);
            page.m_uuid=ON_nil_uuid;page.m_name=L"PageNote";ON_TextDot pageNote(ON_3dPoint(20000,20000,0),L"Layout note",L"");layout.AddModelGeometryComponent(&pageNote,&page);
            require(layout.Write((root/"layout-omitted.3dm").c_str(),5,nullptr),"model and layout fixture write");
            require(readArchive(root/"layout-omitted.3dm",0,false,false,"",true).items.size()==9,"working model-space selection omits layout contents");
            require(readArchive(root/"layout-omitted.3dm",0,true).items.size()==11,"preservation still includes page-space records");
            ON_TextDot dot(ON_3dPoint(0,0,0),L"Opaque working geometry",L"");add(dot,L"Opaque");auto opaque=root/"opaque.3dm";require(model.Write(opaque.c_str(),5,nullptr),"opaque fixture write");bool rejected=false;try{readArchive(opaque);}catch(const ExchangeError& e){auto message=std::string(e.what());rejected=message.find("Opaque")!=std::string::npos&&message.find("ON_TextDot")!=std::string::npos;}require(rejected,"opaque geometry named preflight failure");
        }
    }
    std::cout<<"Modeling native: mm/cm, NURBS/arc/trimmed BRep/cloud, current selection and opaque preflight PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
