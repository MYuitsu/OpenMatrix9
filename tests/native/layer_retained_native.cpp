// SPDX-License-Identifier: LGPL-2.1-or-later
#include "LayerExchangeAdapter.h"
#include <filesystem>
#include <iostream>
#include <stdexcept>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {int checks=0;void check(bool value,const char* text){++checks;if(!value)throw std::runtime_error(text);}std::string id(ON_UUID value){char text[37]{};ON_UuidToString(value,text);return text;}}
int main(){try{
    ON::Begin();ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    ON_Linetype linePattern;linePattern.SetName(L"Jewelry dash");linePattern.AppendSegment(ON_LinetypeSegment(2.0,ON_LinetypeSegment::eSegType::stLine));linePattern.AppendSegment(ON_LinetypeSegment(1.0,ON_LinetypeSegment::eSegType::stSpace));auto lineRef=model.AddModelComponent(linePattern);
    ON_Layer metal;metal.SetName(L"Metal");metal.SetColor(ON_Color(11,22,33));metal.SetUserString(L"CAD.Note",L"source stock");metal.SetPlotWeight(0.75);metal.SetLinetypeIndex(lineRef.ModelComponent()->Index());auto inserted=model.AddModelComponent(metal);auto nativeMetal=ON_Layer::Cast(inserted.ModelComponent());
    ON_Layer spare;spare.SetName(L"Native only");spare.SetColor(ON_Color(71,72,73));auto spareRef=model.AddModelComponent(spare);
    ON_3dmObjectAttributes member;member.m_layer_index=nativeMetal->Index();member.SetMode(ON::idef_object);member.SetColorSource(ON::color_from_parent);member.m_color=ON_Color(8,9,10);
    ON_Point point(1,2,3);auto memberRef=model.AddModelGeometryComponent(&point,&member);auto memberId=memberRef.ModelComponent()->Id();
    ON_InstanceDefinition definition;definition.SetName(L"Ring block");definition.AddInstanceGeometryId(memberId);definition.SetBoundingBox(ON_BoundingBox(ON_3dPoint(1,2,3),ON_3dPoint(1,2,3)));auto definitionRef=model.AddModelComponent(definition);
    ON_InstanceRef instance;instance.m_instance_definition_uuid=definitionRef.ModelComponent()->Id();instance.m_xform=ON_Xform::IdentityTransformation;instance.m_bbox=definition.BoundingBox();
    ON_3dmObjectAttributes root;root.m_layer_index=nativeMetal->Index();root.SetColorSource(ON::color_from_layer);auto rootRef=model.AddModelGeometryComponent(&instance,&root);auto rootId=id(rootRef.ModelComponent()->Id());
    auto rootComponent=ON_ModelGeometryComponent::Cast(rootRef.ModelComponent());auto memberComponent=ON_ModelGeometryComponent::Cast(memberRef.ModelComponent());
    const auto memberAttributes=memberComponent->Attributes(nullptr)->DataCRC(0),memberGeometry=memberComponent->Geometry(nullptr)->DataCRC(0),rootGeometry=rootComponent->Geometry(nullptr)->DataCRC(0);
    NativeLayerTable source;source.activeId="work";
    source.rows={{"source-metal","","METAL","METAL",{201,202,203},true,true},{"child","source-metal","Child","METAL::Child",{1,2,3},false,true,false,true},{"empty","","Empty custom","Empty custom",{7,89,143},false,false},{"work","","Work","Work",{56,78,90},false,true}};
    NativeObjectLayerRow own;own.id="host-root";own.layerId="source-metal";own.rgb={8,9,10};
    auto state=createNativeLayerSnapshot(source,{own},"retained-test",7);
    auto before=readNativeLayerTable(model);bool rejected=false;
    try{applyRetainedLayerOverlay(model,state.get(),{{"missing-physical","host-root"}});}catch(const std::exception&){rejected=true;}
    check(rejected&&readNativeLayerTable(model).rows.size()==before.rows.size(),"Unknown physical object rejects before layer mutation");
    auto applied=applyRetainedLayerOverlay(model,state.get(),{{rootId,"host-root"}});
    check(applied.table.rows.size()==5&&applied.objects.size()==1,"Retained overlay carries full palette and native-only layer");
    auto actualMetal=ON_Layer::Cast(model.ComponentFromId(ON_ModelComponent::Type::Layer,nativeMetal->Id()).ModelComponent());
    check(actualMetal&&actualMetal->Name()==L"METAL"&&actualMetal->Color()==ON_Color(201,202,203)&&actualMetal->IsLocked(),"Source-wins palette reuses UUID and exact native RGB/spelling/lock");
    check(rootComponent->Attributes(nullptr)->Mode()==ON::normal_object&&rootComponent->Attributes(nullptr)->ColorSource()==ON::color_from_layer,"Inherited lock never becomes own lock or ByObject color");
    check(applied.objects[0].id==rootId&&applied.objects[0].layerId==id(actualMetal->Id()),"Canonical own row maps to native physical layer and object IDs");
    check(memberComponent->Attributes(nullptr)->DataCRC(0)==memberAttributes&&memberComponent->Attributes(nullptr)->ColorSource()==ON::color_from_parent,"Untouched definition member retains native ByParent attributes");
    check(memberComponent->Geometry(nullptr)->DataCRC(0)==memberGeometry&&rootComponent->Geometry(nullptr)->DataCRC(0)==rootGeometry,"Layer overlay does not change native geometry");
    auto directory=std::filesystem::path(OM9_LAYER_FIXTURES);std::filesystem::create_directories(directory);auto file=directory/"retained-layer-block.3dm";
    check(model.Write(file.c_str(),5,nullptr),"Write native V5 block with full palette");
    ONX_Model reopened;check(reopened.Read(file.c_str(),nullptr),"Reread native V5 palette and block");
    check(readNativeLayerTable(reopened).rows.size()==5,"Unused/empty native palette survives V5");
    auto reopenedMember=ON_ModelGeometryComponent::Cast(reopened.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,memberId).ModelComponent());
    check(reopenedMember&&reopenedMember->Attributes(nullptr)->ColorSource()==ON::color_from_parent&&reopenedMember->Geometry(nullptr)->DataCRC(0)==memberGeometry,"V5 keeps untouched instance-context payload");
    check(ON_Layer::Cast(reopened.ComponentFromId(ON_ModelComponent::Type::Layer,nativeMetal->Id()).ModelComponent())->Color()==ON_Color(201,202,203),"V5 retains current layer colors");
    NativeLayerTable imported=readNativeLayerTable(reopened);const auto originalPalette=readNativeLayerTable(model);
    imported.rows[0].rgb={111,112,113};
    const auto sourceFirst=imported.rows[0].id;imported.rows[0].id="namespace-other-metal";
    for(auto& row:imported.rows)if(row.parentId==sourceFirst)row.parentId="namespace-other-metal";
    if(imported.activeId==sourceFirst)imported.activeId="namespace-other-metal";
    auto mapping=mergeRetainedNativePalette(model,imported);
    check(mapping.size()==imported.rows.size(),"Namespace palette maps every source layer");
    check(mapping.at("namespace-other-metal")==sourceFirst,"Native palettes merge same fullpath instead of namespace suffix");
    check(readNativeLayerTable(model).rows.size()==originalPalette.rows.size(),"Palette merge creates no duplicate hierarchy");
    check(ON_Layer::Cast(model.ComponentFromId(ON_ModelComponent::Type::Layer,ON_UuidFromString(sourceFirst.c_str())).ModelComponent())->Color()==ON_Color(111,112,113),"Palette source RGB wins through Rust native map");
    imported.rows.push_back({originalPalette.rows[0].id,"","Another empty","Another empty",{19,29,39},false,true});
    auto collision=mergeRetainedNativePalette(model,imported);
    check(collision.at(originalPalette.rows[0].id)!=originalPalette.rows[0].id&&model.ComponentFromId(ON_ModelComponent::Type::Layer,ON_UuidFromString(collision.at(originalPalette.rows[0].id).c_str())).ModelComponent(),"Colliding native UUID maps to actual SDK stored layer");
    check(memberComponent->Geometry(nullptr)->DataCRC(0)==memberGeometry&&memberComponent->Attributes(nullptr)->DataCRC(0)==memberAttributes,"Namespace palette mapping never extracts or edits member geometry");
    std::cout<<"Layer retained native PASS; checks="<<checks<<"; fixtures=1\n";return 0;
}catch(const std::exception& error){std::cerr<<"Layer retained native FAIL after "<<checks<<" checks: "<<error.what()<<'\n';return 1;}}
