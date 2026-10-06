#include "ThreeDmArchive.h"
#include "ThreeDmInventory.h"
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
int main(){try{
    ON::Begin();ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    ON_3dmObjectAttributes attributes;ON_Point point(1,2,3);ON_TextDot dot(ON_3dPoint(4,5,6),L"Retain",L"");
    model.AddModelGeometryComponent(&point,&attributes);model.AddModelGeometryComponent(&dot,&attributes);
    auto path=std::filesystem::temp_directory_path()/"om9-preservation.3dm";
    if(!model.Write(path.c_str(),5,nullptr))throw ExchangeError("fixture write failed");
    bool rejected=false;try{readArchive(path);}catch(const ExchangeError&){rejected=true;}
    if(!rejected)throw ExchangeError("geometry-only accepted unsupported object");
    auto result=readArchive(path,0,true);
    if(result.items.size()!=2)throw ExchangeError("missing retained object");
    unsigned retained=0;for(auto& item:result.items){retained+=item.retained;if(item.sourceUuid.empty())throw ExchangeError("missing source identity");}
    if(retained!=1)throw ExchangeError("retained capability mismatch");
    rejected=false;try{writeArchive5(result,path.parent_path()/"om9-unsafe-export.3dm");}catch(const ExchangeError&){rejected=true;}
    if(!rejected)throw ExchangeError("legacy writer discarded retained object");
    ONX_Model lights;lights.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_Light light;light.SetStyle(ON::world_point_light);lights.AddModelGeometryComponent(&light,&attributes);auto lightPath=path.parent_path()/"om9-preservation-light.3dm";if(!lights.Write(lightPath.c_str(),5,nullptr))throw ExchangeError("light fixture write");auto lightResult=readArchive(lightPath,0,true);if(lightResult.items.size()!=1||!lightResult.items[0].retained)throw ExchangeError("light not retained");
    ONX_Model tables;tables.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_Material material;material.SetName(L"Only material");tables.AddModelComponent(material);auto tablesPath=path.parent_path()/"om9-preservation-tables.3dm";if(!tables.Write(tablesPath.c_str(),5,nullptr)||!readArchive(tablesPath,0,true).items.empty())throw ExchangeError("table-only source preservation");
    std::cout<<"preservation PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
