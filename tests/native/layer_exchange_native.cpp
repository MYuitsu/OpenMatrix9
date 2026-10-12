#include "LayerExchangeAdapter.h"
#include <algorithm>
#include <iostream>
#include <stdexcept>
#include <filesystem>
using namespace OpenMatrix9Gui::ThreeDm;
static int checks=0;
static void check(bool value,const char* message){++checks;if(!value)throw std::runtime_error(message);}
static const NativeLayerRow& at(const NativeLayerTable& table,const std::string& path){
    auto found=std::find_if(table.rows.begin(),table.rows.end(),[&](const auto& l){return l.path==path;});
    if(found==table.rows.end())throw std::runtime_error("missing native layer "+path);return *found;
}
int main(){try{
    ON::Begin();ONX_Model source;std::vector<const ON_Layer*> roots;
    source.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    for(int i=0;i<32;++i){
        ON_Layer layer;layer.SetName(ON_wString((std::string("Slot ")+std::to_string(i)).c_str()));
        layer.SetColor(ON_Color(i+10,100+i,210-i));layer.SetLocked(i==0);layer.SetVisible(i!=1);
        auto ref=source.AddModelComponent(layer);auto stored=ON_Layer::Cast(ref.ModelComponent());
        check(stored!=nullptr,"native palette insertion");roots.push_back(stored);
    }
    ON_Layer child;child.SetParentId(roots[0]->Id());child.SetName(L"Detail");child.SetColor(ON_Color(44,55,66));
    child.SetLocked(true);child.SetPersistentLocking(false);
    auto childRef=source.AddModelComponent(child);auto childLayer=ON_Layer::Cast(childRef.ModelComponent());
    check(childLayer!=nullptr,"inherited child insertion");
    source.m_settings.SetCurrentLayerId(roots[2]->Id());source.m_settings.SetV5CurrentLayerIndex(roots[2]->Index());
    ON_Point geometry(ON_3dPoint(1,2,3));ON_3dmObjectAttributes a;
    a.m_layer_index=childLayer->Index();a.m_name=L"ByLayer ring point";a.SetColorSource(ON::color_from_layer);
    check(!source.AddModelGeometryComponent(&geometry,&a).IsEmpty(),"ByLayer native geometry");
    a.m_uuid=ON_nil_uuid;a.m_name=L"ByObject locked point";a.m_color=ON_Color(77,88,99);
    a.SetColorSource(ON::color_from_object);a.SetMode(ON::locked_object);
    check(!source.AddModelGeometryComponent(&geometry,&a).IsEmpty(),"ByObject native geometry");
    const auto directory=std::filesystem::path(OM9_LAYER_FIXTURES);std::filesystem::create_directories(directory);
    auto original=directory/"native-source.3dm";check(source.Write(original.c_str(),5,nullptr),"native source v5 write");
    ONX_Model read;check(read.Read(original.c_str(),nullptr),"native source v5 read");
    auto table=readNativeLayerTable(read);auto objects=readNativeObjectLayers(read,true);
    check(table.rows.size()==33,"all32 slots and child preserved");check(objects.size()==2,"native typed object metadata");
    check(at(table,"Slot 0").locked&&!at(table,"Slot 1").visible,"root own flags");
    check(!at(table,"Slot 0::Detail").locked,"child desired lock separated from parent");
    check(at(table,"Slot 31").rgb==std::array<unsigned char,3>{41,131,179},"empty palette RGB preserved");
    check(table.activeId==at(table,"Slot 2").id,"native current layer retained");
    auto rust=createNativeLayerSnapshot(table,objects,"native-source",0);
    Om9LayerEffective effective{};
    auto id=Om9LayerByteView{reinterpret_cast<const unsigned char*>(objects[0].id.data()),objects[0].id.size()};
    check(om9_layer_snapshot_effective(rust.get(),id,&effective)==0,"native facts validate in Rust");
    check(effective.locked&&effective.rgb[0]==44,"effective parent lock and child color");
    check(!objects[0].locked&&objects[1].locked,"object own locks stay distinct");
    check(objects[1].colorSource==2&&objects[1].rgb==std::array<unsigned char,3>{77,88,99},"ByObject source preserved");
    auto json=nativeLayerSnapshotJson(rust.get());
    check(json.find("vertices")==std::string::npos&&json.find("brep")==std::string::npos,"metadata codec does not serialize geometry");
    auto decoded=nativeLayerSnapshotFromJson(json);Om9LayerCounts counts{};
    check(om9_layer_snapshot_counts(decoded.get(),&counts)==0&&counts.layer_count==33&&counts.object_count==2,"native Rust JSON keeps full palette and object records");
    Om9LayerEffective decodedEffective{};
    check(om9_layer_snapshot_effective(decoded.get(),id,&decodedEffective)==0&&decodedEffective.locked&&decodedEffective.rgb[0]==44,"native JSON roundtrip keeps local/effective separation");
    bool truncated=false;try{auto broken=nativeLayerSnapshotFromJson(json.substr(0,json.size()-1));}catch(const std::exception&){truncated=true;}
    check(truncated,"native JSON truncated metadata rejected before commit");
    ONX_Model palette;palette.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    auto mapping=writeNativeLayerTable(palette,table);
    check(mapping.size()==33,"palette-only native layer write");
    auto output=directory/"palette-only.3dm";check(palette.Write(output.c_str(),5,nullptr),"palette-only native file write");
    ONX_Model reread;check(reread.Read(output.c_str(),nullptr),"palette-only native file reread");
    auto again=readNativeLayerTable(reread);
    check(again.rows.size()==33,"palette-only keeps empty slots");
    check(at(again,"Slot 0").locked&&!at(again,"Slot 0::Detail").locked,"local nested state roundtrip");
    check(at(again,"Slot 31").rgb==at(table,"Slot 31").rgb,"empty palette exact color roundtrip");
    // Getters do not expose unset presence; OM9's versioned per-layer witness
    // keeps exact presence on unchanged native roundtrips, never stale state.
    {auto exact=table;exact.rows[0].persistentLocked=true;
    auto childRow=std::find_if(exact.rows.begin(),exact.rows.end(),[](const auto& row){return row.path=="Slot 0::Detail";});
    childRow->persistentLocked.reset();childRow->persistentVisible.reset();
    ONX_Model witnessModel;writeNativeLayerTable(witnessModel,exact);
    auto witnessFile=directory/"persistent-presence.3dm";check(witnessModel.Write(witnessFile.c_str(),5,nullptr),"persistent presence V5 file write");
    ONX_Model witnessRead;check(witnessRead.Read(witnessFile.c_str(),nullptr),"persistent presence V5 file read");
    auto exactRead=readNativeLayerTable(witnessRead);
    check(at(exactRead,"Slot 0").persistentLocked==std::optional<bool>(true),"explicit root field presence survives V5");
    check(!at(exactRead,"Slot 0::Detail").persistentLocked&& !at(exactRead,"Slot 0::Detail").persistentVisible,"unset child fields survive V5");
    auto reference=witnessRead.ComponentFromId(ON_ModelComponent::Type::Layer,ON_UuidFromString(at(exactRead,"Slot 0::Detail").id.c_str()));
    auto edited=const_cast<ON_Layer*>(ON_Layer::Cast(reference.ModelComponent()));edited->SetLocked(true);
    auto editedRead=readNativeLayerTable(witnessRead);
    check(at(editedRead,"Slot 0::Detail").locked&&at(editedRead,"Slot 0::Detail").persistentLocked==std::optional<bool>(true),"native layer edit overrides old presence witness");
    edited->SetUserString(L"OpenMatrix9.LayerPersistent.v1",L"malformed");bool corrupt=false;
    try{readNativeLayerTable(witnessRead);}catch(const std::exception&){corrupt=true;}
    check(corrupt,"malformed supplied presence witness is rejected");}
    auto invalid=table;invalid.rows.back().parentId="nonexistent-parent";
    bool refused=false;try{auto invalidHandle=createNativeLayerSnapshot(invalid,{},"invalid",0);}catch(const std::exception&){refused=true;}
    check(refused,"invalid native parent preflight rejected");
    // Logical IDs remain distinct even if parsing them yields the same UUID.
    {NativeLayerTable aliases;NativeLayerRow one;one.id="abcdef01-2345-6789-abcd-ef0123456789";one.name=one.path="First";
    auto two=one;two.id="ABCDEF01-2345-6789-ABCD-EF0123456789";two.name=two.path="Second";
    auto child=one;child.id="logical-child";child.parentId=two.id;child.name="Child";child.path="Second::Child";
    aliases.rows={one,two,child};aliases.activeId=child.id;
    ONX_Model aliasModel;auto aliasMap=writeNativeLayerTable(aliasModel,aliases);
    check(aliasMap.size()==3,"UUID spelling alias mapping is complete");
    auto aliasRead=readNativeLayerTable(aliasModel);
    check(aliasRead.rows.size()==3,"UUID remapping cannot merge logical layers");
    check(at(aliasRead,"Second::Child").parentId==at(aliasRead,"Second").id,"Child points to actual remapped native parent ID");
    check(aliasRead.activeId==at(aliasRead,"Second::Child").id,"Current uses actual mapped native UUID");}
    std::cout<<"{\"ok\":true,\"checks\":"<<checks<<",\"fixtures\":2,\"scope\":\"native3DM palette metadata; no host UI claim\"}\n";return 0;
}catch(const std::exception& error){std::cerr<<error.what()<<"\n";return 1;}}
