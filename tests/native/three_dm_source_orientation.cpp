#include "ThreeDmArchive.h"
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <BRepBndLib.hxx>
#include <Bnd_Box.hxx>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QCryptographicHash>
#include <set>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
int main(){try{
    ON::Begin();std::filesystem::path fixture=OM9_SOURCE_ORIENTATION_FIXTURE;ONX_Model model;if(!model.Read(fixture.c_str()))return 2;
    auto component=ON_ModelGeometryComponent::Cast(model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString("8fb04ca0-f109-4111-9f70-e3c5b4492a56")).ModelComponent());if(!component)return 2;auto brep=ON_Brep::Cast(component->Geometry(nullptr));if(!brep||brep->SolidOrientation()!=1||brep->m_F.Count()!=101)return 2;
    std::cout<<"diamond source direction="<<brep->SolidOrientation()<<'\n';
    QFile evidence(QString::fromStdWString((fixture.parent_path()/"rhino5-source-ring-orientation.json").wstring()));if(!evidence.open(QIODevice::ReadOnly))return 2;auto expected=QJsonDocument::fromJson(evidence.readAll()).object();auto placements=expected["placements"].toArray();if(placements.size()!=4)return 2;
    QFile input(QString::fromStdWString(fixture.wstring()));if(!input.open(QIODevice::ReadOnly)||QString(QCryptographicHash::hash(input.readAll(),QCryptographicHash::Sha256).toHex())!=expected["fixture_sha256"].toString())return 2;
    auto all=readArchive(fixture);std::set<int> matched;
    for(auto& item:all.items)if(item.sourceUuid=="8fb04ca0-f109-4111-9f70-e3c5b4492a56"){
        auto shape=std::get<TopoDS_Shape>(item.geometry);Bnd_Box box;BRepBndLib::AddOptimal(shape,box,false,false);double bounds[6];box.Get(bounds[0],bounds[1],bounds[2],bounds[3],bounds[4],bounds[5]);int match=-1;
        for(int i=0;i<placements.size();++i){double deviation=0;auto reference=placements[i].toObject()["bounds"].toArray();for(int j=0;j<6;++j)deviation=std::max(deviation,std::abs(bounds[j]-reference[j].toDouble()));if(deviation<1e-3){if(match!=-1)return 1;match=i;}}
        if(match<0||!matched.insert(match).second)return 1;
        GProp_GProps p;BRepGProp::VolumeProperties(shape,p,1e-10);std::cout<<item.name<<" placement="<<match<<" signed volume="<<p.Mass()<<'\n';if(std::abs(p.Mass()-placements[match].toObject()["volume"].toDouble())>1e-8)return 1;
        auto exported=exportBrep(shape,1e-6);GProp_GProps after;BRepGProp::VolumeProperties(importBrep(*exported,1e-6),after,1e-10);if(std::abs(after.Mass()-p.Mass())>1e-8)return 1;
    }
    return matched.size()==4?0:1;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 2;}}
