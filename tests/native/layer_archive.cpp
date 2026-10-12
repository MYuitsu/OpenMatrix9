// SPDX-License-Identifier: LGPL-2.1-or-later
#include "ThreeDmArchive.h"
#include "LayerExchangeAdapter.h"
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <filesystem>
#include <iostream>
#include <stdexcept>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {
int checks=0;
void check(bool value,const char* message){++checks;if(!value)throw std::runtime_error(message);}
std::string id(){ON_UUID value;ON_CreateUuid(value);char text[37]{};ON_UuidToString(value,text);return text;}
NativeLayerRow row(const std::string& name,const std::string& parent={},const std::string& path={}){
    NativeLayerRow value;value.id=id();value.name=name;value.parentId=parent;value.path=path.empty()?name:path;return value;
}
const NativeLayerRow& layer(const ExchangeModel& model,const std::string& path){
    for(const auto& value:model.layerTable.rows)if(value.path==path)return value;
    throw std::runtime_error("Missing reread layer "+path);
}
}
int main(){try{
    ON::Begin();const auto directory=std::filesystem::path(OM9_LAYER_FIXTURES);std::filesystem::create_directories(directory);
    ONX_Model native;native.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    NativeLayerTable table;
    auto parent=row("Parent");parent.rgb={14,37,211};parent.locked=true;
    auto child=row("Child",parent.id,"Parent::Child");child.rgb={63,19,181};child.persistentLocked=false;child.persistentVisible=true;
    auto working=row("Working");working.rgb={211,17,42};
    auto empty=row("Empty");empty.rgb={7,89,143};empty.visible=false;
    table.rows={parent,child,working,empty};table.activeId=working.id;
    auto indexes=writeNativeLayerTable(native,table);
    ON_Point first(1,2,3),second(4,5,6);ON_3dmObjectAttributes a,b;
    a.m_name=L"inherited";a.m_layer_index=indexes.at(child.id);a.SetColorSource(ON::color_from_layer);a.SetMode(ON::normal_object);a.SetVisible(true);a.m_color=ON_Color(17,18,19);
    b.m_name=L"own";b.m_layer_index=indexes.at(working.id);b.SetColorSource(ON::color_from_object);b.SetMode(ON::locked_object);b.SetVisible(false);b.SetUserString(L"OpenMatrix9.Locked",L"1");b.m_color=ON_Color(23,24,25);
    auto firstRef=native.AddModelGeometryComponent(&first,&a);auto secondRef=native.AddModelGeometryComponent(&second,&b);
    check(!secondRef.IsEmpty()&&ON_ModelGeometryComponent::Cast(secondRef.ModelComponent())->Attributes(nullptr)->IsValid(),"Inserted hidden+legacy-lock fixture is valid");
    char firstId[37]{};ON_UuidToString(firstRef.ModelComponent()->Id(),firstId);
    const auto source=directory/"layer-source.3dm";check(native.Write(source.c_str(),5,nullptr),"Write source fixture");
    auto imported=readArchive(source);
    check(imported.layerTable.rows.size()==4,"Import full table including empty layers");
    check(imported.layerTable.activeId==working.id,"Import active layer");
    check(imported.items.size()==2,"Import objects");
    check(imported.items[0].locked&&imported.items[0].visible,"Imported inherited effective state");
    check(imported.items[0].ownState&& !imported.items[0].ownState->locked,"Layer lock is not own object lock");
    check(imported.items[0].ownState->colorSource==1,"Keep ByLayer source");
    check(imported.items[1].ownState&&imported.items[1].ownState->locked&&!imported.items[1].ownState->visible,"Keep own lock and visibility");
    check(imported.items[1].ownState->colorSource==2,"Keep ByObject source");
    check(layer(imported,"Parent::Child").persistentLocked==false,"Keep child desired unlock");
    auto subset=readArchiveSubset(source,1,false,{firstId},false);
    check(subset.items.size()==1&&subset.layerTable.rows.size()==4,"Selected subset carries entire palette");
    const auto selected=directory/"layer-selected.3dm";writeArchive5(subset,selected);
    auto reread=readArchive(selected);
    check(reread.items.size()==1&&reread.layerTable.rows.size()==4,"Selected archive keeps empty palette");
    check(reread.items[0].ownState&&!reread.items[0].ownState->locked,"Output does not bake inherited lock into object");
    check(reread.items[0].ownState->colorSource==1,"Output keeps ByLayer");
    check(reread.items[0].transferObjectId==subset.items[0].ownState->id,"Export transfer tag binds original metadata identity");
    check(reread.items[0].ownState->id!=reread.items[0].transferObjectId,"Native identity remains separate from transfer tag");
    auto selectedState=createNativeLayerSnapshot(subset.layerTable,{*subset.items[0].ownState},"selected-payload",9);
    const auto physicalIdentity=reread.items[0].ownState->id;
    applyExtendedLayerOverlay(reread,selectedState.get());
    check(reread.layerTable.rows[0].id==subset.layerTable.rows[0].id&&reread.layerTable.activeId==subset.layerTable.activeId,"Extended overlay restores source palette identity and active layer");
    check(reread.items[0].ownState->id==physicalIdentity&&reread.items[0].ownState->layerId==subset.items[0].ownState->layerId,"Extended overlay uses actual physical geometry identity");
    check(!reread.items[0].ownState->locked&&reread.items[0].locked,"Extended overlay projects inherited lock separately");
    check(layer(reread,"Parent::Child").rgb==child.rgb&&layer(reread,"Empty").rgb==empty.rgb,"Output source RGB exact");
    const auto both=directory/"layer-session.3dm";writeArchive5(imported,both);
    auto all=readArchive(both);
    check(all.items.size()==2,"Output hidden locked object is not silently dropped");
    check(all.items[1].ownState->rgb==std::array<unsigned char,3>{23,24,25},"ByObject RGB survives session");
    check(all.items[1].ownState->locked&&!all.items[1].ownState->visible,"Own states survive session");
    // Rhino Copy duplicates user strings but allocates a new object UUID. Native-only
    // archive import must keep both physical objects, never adopt duplicate tags as IDs.
    ONX_Model copied;check(copied.Read(selected.c_str(),nullptr),"Read tagged native copy fixture");
    ONX_ModelComponentIterator tagged(copied,ON_ModelComponent::Type::ModelGeometry);
    auto original=ON_ModelGeometryComponent::Cast(tagged.FirstComponent());
    auto copiedAttributes=*original->Attributes(nullptr);copiedAttributes.m_uuid=ON_nil_uuid;
    check(!copied.AddModelGeometryComponent(original->Geometry(nullptr),&copiedAttributes).IsEmpty(),"Duplicate tagged object with fresh UUID");
    const auto duplicatePath=directory/"layer-duplicate-tags.3dm";check(copied.Write(duplicatePath.c_str(),5,nullptr),"Write duplicate tags fixture");
    auto duplicate=readArchive(duplicatePath);
    check(duplicate.items.size()==2&&duplicate.items[0].ownState->id!=duplicate.items[1].ownState->id,"Duplicate tags never replace native-only physical IDs");
    check(duplicate.items[0].transferObjectId==duplicate.items[1].transferObjectId,"Both raw tags remain separate native facts");
    const auto duplicateActive=duplicate.layerTable.activeId;const auto duplicateOwn=duplicate.items[0].ownState->layerId;
    auto fullState=createNativeLayerSnapshot(imported.layerTable,{*imported.items[0].ownState,*imported.items[1].ownState},"session-payload",9);
    bool rejectedTags=false;try{applyExtendedLayerOverlay(duplicate,fullState.get());}catch(const std::exception&){rejectedTags=true;}
    check(rejectedTags&&duplicate.layerTable.activeId==duplicateActive&&duplicate.items[0].ownState->layerId==duplicateOwn,"Invalid extended overlay leaves detached model metadata unchanged");
    ExchangeModel inward;inward.layerTable=table;
    ExchangeItem signedItem;signedItem.name="signed";signedItem.geometry=BRepPrimAPI_MakeBox(2,3,4).Shape();std::get<TopoDS_Shape>(signedItem.geometry).Reverse();
    NativeObjectLayerRow signedState;signedState.id="signed-object";signedState.layerId=working.id;signedItem.ownState=signedState;inward.items.push_back(signedItem);
    const auto signedPath=directory/"layer-signed-tag.3dm";writeArchive5(inward,signedPath);auto signedRead=readArchive(signedPath);
    check(signedRead.items.size()==1&&signedRead.items[0].transferObjectId=="signed-object","Signed logical placement retains transfer tag");
    ONX_Model signedNative;check(signedNative.Read(signedPath.c_str(),nullptr),"Read signed native fixture");
    ONX_ModelComponentIterator signedObjects(signedNative,ON_ModelComponent::Type::ModelGeometry);
    for(auto c=signedObjects.FirstComponent();c;c=signedObjects.NextComponent()){
        auto object=ON_ModelGeometryComponent::Cast(c);if(object->Attributes(nullptr)->Mode()==ON::idef_object){ON_wString tag;check(!object->Attributes(nullptr)->GetUserString(L"OpenMatrix9.LayerObjectId",tag),"Generated definition member carries no transfer tag");}
    }
    ExchangeModel palette;palette.layerTable=table;
    const auto palettePath=directory/"layer-palette.3dm";writeArchive5(palette,palettePath);
    auto emptyRead=readArchive(palettePath);
    check(emptyRead.items.empty()&&emptyRead.layerTable.rows.size()==4,"Palette-only file transfer");
    check(layer(emptyRead,"Empty").visible==false,"Palette-only hidden flag");
    // An invalid canonical reference must reject before touching an existing file.
    auto invalid=subset;invalid.items[0].ownState->layerId="missing";
    auto before=std::filesystem::file_size(selected);bool rejected=false;
    try{writeArchive5(invalid,selected);}catch(const std::exception&){rejected=true;}
    check(rejected&&std::filesystem::file_size(selected)==before,"Reject invalid layer reference before output mutation");
    // Legacy callers with only flat item fields remain readable; no metadata guessed.
    ExchangeModel legacy;ExchangeItem item;item.name="legacy";item.layer="Legacy::Geometry";item.color={111,112,113};item.locked=true;item.geometry=BRepBuilderAPI_MakeVertex(gp_Pnt(7,8,9)).Shape();legacy.items.push_back(item);
    const auto legacyPath=directory/"layer-legacy.3dm";writeArchive5(legacy,legacyPath);
    auto migrated=readArchive(legacyPath);
    check(migrated.items.size()==1&&migrated.items[0].ownState->locked,"Legacy true lock migrates conservatively");
    check(migrated.items[0].ownState->colorSource==2,"Legacy flat export keeps ByObject");
    std::cout<<"Layer archive PASS; checks="<<checks<<"; fixtures=7\n";return 0;
}catch(const std::exception& error){std::cerr<<"Layer archive FAIL after "<<checks<<" checks: "<<error.what()<<'\n';return 1;}}
