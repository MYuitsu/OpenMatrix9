#include "ThreeDmArchive.h"
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
int main(int argc,char** argv){try{
    if(argc!=2)return 2;ON::Begin();ONX_Model model;model.AddDefaultLayer(L"Default",ON_Color(180,120,220));
    model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    ON_DimStyle style;style.SetName(L"Fixture dimensions");model.AddModelComponent(style);
    ON_3dmObjectAttributes a;a.m_layer_index=0;
    for(int i=0;i<8;++i){ON_Point point(i,i+1,i+2);a.m_name=ON_wString(("point-"+std::to_string(i)).c_str());if(model.AddModelGeometryComponent(&point,&a).IsEmpty())return 3;}
    ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(1,0,0),ON_3dPoint(1,1,0),ON_3dPoint(0,1,0),ON_3dPoint(0,0,1),ON_3dPoint(1,0,1),ON_3dPoint(1,1,1),ON_3dPoint(0,1,1)};
    std::unique_ptr<ON_SubD> subd(ON_SubD::CreateSubDBox(corners,ON_SubDEdgeTag::Smooth,1,1,1,nullptr));
    if(!subd||!subd->IsValid())return 4;a.m_name=L"retained-subd";if(model.AddModelGeometryComponent(subd.get(),&a).IsEmpty())return 5;
    ON_Light light;light.SetStyle(ON::world_point_light);a.m_name=L"retained-light";if(model.AddModelGeometryComponent(&light,&a).IsEmpty())return 6;
    ON_InstanceDefinition definition;definition.SetName(L"Eight members");definition.SetInstanceDefinitionType(ON_InstanceDefinition::IDEF_UPDATE_TYPE::Static);definition.SetUnitSystem(ON::LengthUnitSystem::Millimeters);
    for(int i=0;i<8;++i){ON_Point point(i,20,i+2);a.m_name=ON_wString(("member-"+std::to_string(i)).c_str());a.SetMode(ON::idef_object);auto c=model.AddModelGeometryComponent(&point,&a);if(c.IsEmpty())return 11;definition.AddInstanceGeometryId(c.ModelComponent()->Id());}
    definition.SetBoundingBox(ON_BoundingBox(ON_3dPoint(0,20,2),ON_3dPoint(7,20,9)));auto added=model.AddModelComponent(definition);if(added.IsEmpty())return 12;
    ON_InstanceRef instance;instance.m_instance_definition_uuid=added.ModelComponent()->Id();instance.m_xform=ON_Xform::IdentityTransformation;instance.m_bbox=definition.BoundingBox();a.SetMode(ON::normal_object);a.m_name=L"member-root";if(model.AddModelGeometryComponent(&instance,&a).IsEmpty())return 13;
    if(!model.Write(argv[1],80,nullptr))return 7;
    ONX_Model read;if(!read.Read(argv[1],nullptr))return 8;
    auto exchange=readArchive(argv[1],0,true);if(exchange.items.size()!=11)return 9;
    int retained=0;for(auto& item:exchange.items)retained+=item.retained;if(retained!=3)return 10;
    std::cout<<"Mixed points/SubD/render-light worker fixture PASS\n";return 0;
}catch(const std::exception& error){std::cerr<<error.what()<<'\n';return 1;}}
