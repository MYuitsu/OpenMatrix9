#include "ThreeDmArchive.h"
#include <opennurbs.h>
#include <filesystem>
#include <iostream>
int main(int argc,char** argv){
    if(argc!=2)return 2;ON::Begin();std::filesystem::create_directories(argv[1]);
    for(int density:{-1,0,1,3,33}){
        ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
        model.AddDefaultLayer(L"Default",ON_Color(255,128,128));ON_3dmObjectAttributes attributes;attributes.m_layer_index=0;attributes.m_wire_density=density;
        auto brep=std::unique_ptr<ON_Brep>(ON_BrepSphere(ON_Sphere(ON_3dPoint::Origin,5)));
        if(!brep||model.AddModelGeometryComponent(brep.get(),&attributes).IsEmpty())return 3;
        auto path=std::filesystem::path(argv[1])/("density-"+std::to_string(density)+".3dm");
        if(!OpenMatrix9Gui::ThreeDm::writeModelRhino5(model,path))return 4;
    }
    std::cout<<"Five independent V5 sphere density fixtures written\n";return 0;
}
