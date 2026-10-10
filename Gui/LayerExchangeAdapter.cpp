// SPDX-License-Identifier: LGPL-2.1-or-later
#include "LayerExchangeAdapter.h"
#include <algorithm>
#include <stdexcept>
#include <utility>
#include <set>

namespace OpenMatrix9Gui::ThreeDm {
namespace {
std::string uuid(const ON_UUID& value) {
    if (ON_UuidIsNil(value)) return {};
    char buffer[37]{}; ON_UuidToString(value,buffer); return buffer;
}
std::string utf8(const ON_wString& value) {
    const ON_String text(value); return static_cast<const char*>(text);
}
std::array<unsigned char,3> rgb(const ON_Color& color) {
    return {static_cast<unsigned char>(color.Red()),static_cast<unsigned char>(color.Green()),static_cast<unsigned char>(color.Blue())};
}
Om9LayerByteView bytes(const std::string& value) {
    return {reinterpret_cast<const unsigned char*>(value.data()),value.size()};
}
std::int8_t flag(const std::optional<bool>& value) { return value ? (*value ? 1 : 0) : -1; }
void rustError(std::uint32_t error) {
    if (error) throw std::runtime_error("Rust layer metadata preflight rejected snapshot; code="+std::to_string(error));
}
void writePersistentWitness(ON_Layer& layer,const NativeLayerRow& row) {
    std::array<unsigned char,256> buffer{};std::size_t needed=0;
    rustError(om9_layer_native_persistent_encode(!row.parentId.empty(),row.locked,row.visible,flag(row.persistentLocked),flag(row.persistentVisible),buffer.data(),buffer.size(),&needed));
    const std::string value(reinterpret_cast<const char*>(buffer.data()),needed);
    if(!layer.SetUserString(L"OpenMatrix9.LayerPersistent.v1",ON_wString(value.c_str())))throw std::runtime_error("Native SDK refused persistent layer witness");
}
std::string snapshotText(std::uint64_t handle,unsigned kind,std::size_t index,unsigned field){
    std::size_t needed=0;auto status=om9_layer_snapshot_text(handle,kind,index,field,nullptr,0,&needed);
    if(status!=15)rustError(status);
    if(needed>4096)throw std::runtime_error("Rust layer text exceeds native adapter bound");
    std::string value(needed,'\0');rustError(om9_layer_snapshot_text(handle,kind,index,field,reinterpret_cast<unsigned char*>(value.data()),value.size(),&needed));
    return value;
}
}
NativeLayerSnapshot::~NativeLayerSnapshot() { if (handle_) om9_layer_snapshot_free(handle_); }
std::string nativeLayerSnapshotJson(std::uint64_t handle) {
    std::size_t needed=0; const auto status=om9_layer_snapshot_json(handle,nullptr,0,&needed);
    if (status!=15) rustError(status);
    if (needed>256ULL*1024*1024) throw std::runtime_error("Rust layer JSON exceeds metadata limit");
    std::string result(needed,'\0');
    rustError(om9_layer_snapshot_json(handle,reinterpret_cast<unsigned char*>(result.data()),result.size(),&needed));
    if (needed!=result.size()) throw std::runtime_error("Layer JSON size changed during immutable snapshot read");
    return result;
}
NativeLayerSnapshot nativeLayerSnapshotFromJson(const std::string& json) {
    std::uint64_t handle=0; rustError(om9_layer_snapshot_from_json(bytes(json),&handle));
    return NativeLayerSnapshot(handle);
}
NativeLayerSnapshot::NativeLayerSnapshot(NativeLayerSnapshot&& other) noexcept : handle_(std::exchange(other.handle_,0)) {}
NativeLayerSnapshot& NativeLayerSnapshot::operator=(NativeLayerSnapshot&& other) noexcept {
    if (this!=&other) { if (handle_) om9_layer_snapshot_free(handle_); handle_=std::exchange(other.handle_,0); }
    return *this;
}
NativeLayerSnapshot createNativeLayerSnapshot(const NativeLayerTable& table,
    const std::vector<NativeObjectLayerRow>& objects,const std::string& document,std::uint64_t generation,bool legacyNativeNames) {
    std::vector<Om9LayerView> layers; layers.reserve(table.rows.size());
    for (const auto& row:table.rows) {
        Om9LayerView view{}; view.id=bytes(row.id); view.parent=bytes(row.parentId);
        view.name=bytes(row.name); view.path=bytes(row.path);
        std::copy(row.rgb.begin(),row.rgb.end(),view.rgb);
        view.locked=row.locked; view.visible=row.visible;
        view.persistent_locked=flag(row.persistentLocked); view.persistent_visible=flag(row.persistentVisible);
        layers.push_back(view);
    }
    std::vector<Om9LayerObjectView> nativeObjects; nativeObjects.reserve(objects.size());
    for (const auto& object:objects) {
        Om9LayerObjectView view{}; view.id=bytes(object.id); view.layer=bytes(object.layerId);
        std::copy(object.rgb.begin(),object.rgb.end(),view.rgb);
        view.locked=object.locked; view.visible=object.visible; view.color_source=object.colorSource;
        nativeObjects.push_back(view);
    }
    Om9LayerSnapshotView snapshot{}; snapshot.version=1; snapshot.document=bytes(document);
    snapshot.generation=generation; snapshot.active=bytes(table.activeId);
    snapshot.layers=layers.data(); snapshot.layer_count=layers.size();
    snapshot.objects=nativeObjects.data(); snapshot.object_count=nativeObjects.size();
    std::uint64_t handle=0;
    rustError(legacyNativeNames?om9_layer_snapshot_create_native(&snapshot,&handle):om9_layer_snapshot_create(&snapshot,&handle));
    return NativeLayerSnapshot(handle);
}
NativeLayerTable nativeLayerTableFromSnapshot(std::uint64_t handle) {
    Om9LayerCounts counts{};rustError(om9_layer_snapshot_counts(handle,&counts));
    if(counts.layer_count>65536)throw std::runtime_error("Rust layer count exceeds native adapter bound");
    auto optional=[](std::int8_t value)->std::optional<bool>{return value<0?std::nullopt:std::optional<bool>(value!=0);};
    NativeLayerTable table;table.rows.reserve(counts.layer_count);
    for(std::size_t i=0;i<counts.layer_count;++i){
        Om9LayerInfo info{};rustError(om9_layer_snapshot_layer(handle,i,&info));NativeLayerRow row;
        row.id=snapshotText(handle,1,i,0);row.parentId=snapshotText(handle,1,i,1);row.name=snapshotText(handle,1,i,2);row.path=snapshotText(handle,1,i,3);
        std::copy(std::begin(info.rgb),std::end(info.rgb),row.rgb.begin());row.locked=info.locked;row.visible=info.visible;
        row.persistentLocked=optional(info.persistent_locked);row.persistentVisible=optional(info.persistent_visible);
        if(i==counts.active_layer_index)table.activeId=row.id;table.rows.push_back(std::move(row));
    }
    return table;
}
NativeLayerContext nativeLayerContextFromSnapshot(std::uint64_t handle) {
    Om9LayerCounts counts{};rustError(om9_layer_snapshot_counts(handle,&counts));
    return {snapshotText(handle,0,0,0),counts.generation};
}
std::vector<NativeObjectLayerRow> nativeObjectLayersFromSnapshot(std::uint64_t handle){
    Om9LayerCounts counts{};rustError(om9_layer_snapshot_counts(handle,&counts));
    if(counts.object_count>2000000)throw std::runtime_error("Rust object count exceeds native adapter bound");
    std::vector<NativeObjectLayerRow> result;result.reserve(counts.object_count);
    for(std::size_t i=0;i<counts.object_count;++i){
        Om9LayerObjectInfo info{};rustError(om9_layer_snapshot_object(handle,i,&info));NativeObjectLayerRow row;
        row.id=snapshotText(handle,2,i,0);row.layerId=snapshotText(handle,2,i,1);
        std::copy(std::begin(info.rgb),std::end(info.rgb),row.rgb.begin());
        row.locked=info.locked;row.visible=info.visible;row.colorSource=info.color_source;result.push_back(std::move(row));
    }
    return result;
}
NativeLayerTable readNativeLayerTable(const ONX_Model& model) {
    NativeLayerTable result; std::map<std::string,std::size_t> byId; std::map<int,std::string> byIndex;
    ONX_ModelComponentIterator iterator(model,ON_ModelComponent::Type::Layer);
    for (auto component=iterator.FirstComponent();component;component=iterator.NextComponent()) {
        auto layer=ON_Layer::Cast(component); if (!layer || layer->IsDeleted()) continue;
        if (result.rows.size()>=65536) throw std::runtime_error("Native layer table exceeds supported metadata limit");
        NativeLayerRow row; row.id=uuid(layer->Id()); row.parentId=uuid(layer->ParentId());
        row.name=utf8(layer->Name()); row.rgb=rgb(layer->Color());
        // ON_Layer::Write stores these desired fields, distinct from parent effect.
        row.locked=layer->PersistentLocking(); row.visible=layer->PersistentVisibility();
        ON_wString marker;layer->GetUserString(L"OpenMatrix9.LayerPersistent.v1",marker);
        const auto text=utf8(marker);Om9LayerInfo fields{};
        rustError(om9_layer_native_persistent_decode(!row.parentId.empty(),row.locked,row.visible,bytes(text),&fields));
        if(fields.persistent_locked>=0)row.persistentLocked=fields.persistent_locked!=0;
        if(fields.persistent_visible>=0)row.persistentVisible=fields.persistent_visible!=0;
        byId.emplace(row.id,result.rows.size()); byIndex.emplace(layer->Index(),row.id);
        result.rows.push_back(std::move(row));
    }
    for (auto& row:result.rows) {
        std::vector<std::string> components; const NativeLayerRow* current=&row;
        for (std::size_t depth=0;;++depth) {
            if (depth>=128) throw std::runtime_error("Cannot collect native layer path: depth/cycle limit");
            components.push_back(current->name); if (current->parentId.empty()) break;
            auto found=byId.find(current->parentId);
            if (found==byId.end()) throw std::runtime_error("Cannot collect native layer path: parent is missing");
            current=&result.rows[found->second];
        }
        for (auto component=components.rbegin();component!=components.rend();++component) {
            if (!row.path.empty()) row.path+="::"; row.path+=*component;
        }
    }
    result.activeId=uuid(model.m_settings.CurrentLayerId());
    if (result.activeId.empty()) {
        auto found=byIndex.find(model.m_settings.CurrentLayerIndex());
        if (found!=byIndex.end()) result.activeId=found->second;
    }
    // Native facts are not accepted merely because the SDK returned them.
    auto validated=createNativeLayerSnapshot(result,{},"native-layer-table",0,true);
    return nativeLayerTableFromSnapshot(validated.get());
}
std::vector<NativeObjectLayerRow> readNativeObjectLayers(const ONX_Model& model,bool modelSpaceOnly) {
    std::vector<NativeObjectLayerRow> result;
    ONX_ModelComponentIterator iterator(model,ON_ModelComponent::Type::ModelGeometry);
    for (auto component=iterator.FirstComponent();component;component=iterator.NextComponent()) {
        auto geometry=ON_ModelGeometryComponent::Cast(component); if (!geometry || geometry->IsDeleted()) continue;
        const auto* attributes=geometry->Attributes(nullptr); if (!attributes) throw std::runtime_error("Native geometry has no object attributes");
        if (modelSpaceOnly && attributes->Mode()==ON::idef_object) continue;
        if (result.size()>=2000000) throw std::runtime_error("Native object metadata exceeds supported limit");
        auto layerRef=model.LayerFromIndex(attributes->m_layer_index);
        auto layer=ON_Layer::Cast(layerRef.ModelComponent());
        if (!layer || layer->IsDeleted()) throw std::runtime_error("Native object references missing layer");
        NativeObjectLayerRow row; row.id=uuid(geometry->Id()); row.layerId=uuid(layer->Id());
        row.locked=attributes->Mode()==ON::locked_object; row.visible=attributes->IsVisible();
        ON_wString legacyLock;
        if (attributes->GetUserString(L"OpenMatrix9.Locked",legacyLock) && legacyLock==L"1") row.locked=true;
        if (attributes->ColorSource()==ON::color_from_layer) row.colorSource=1;
        else if (attributes->ColorSource()==ON::color_from_object) row.colorSource=2;
        else throw std::runtime_error("Native object color source needs instance-context resolution before layer transfer: "+row.id);
        row.rgb=rgb(attributes->m_color); result.push_back(std::move(row));
    }
    return result;
}
std::map<std::string,int> writeNativeLayerTable(ONX_Model& model,const NativeLayerTable& table) {
    auto validated=createNativeLayerSnapshot(table,{},"native-layer-export",0);
    ONX_ModelComponentIterator existing(model,ON_ModelComponent::Type::Layer);
    if (existing.FirstComponent()) throw std::runtime_error("Native layer writer requires a fresh archive model");
    std::map<std::string,int> mapping; std::map<std::string,ON_UUID> nativeIds;
    for (const auto& row:table.rows) {
        auto id=ON_UuidFromString(row.id.c_str()); if (ON_UuidIsNil(id)) ON_CreateUuid(id);
        nativeIds.emplace(row.id,id);
    }
    std::vector<bool> inserted(table.rows.size(),false);
    std::size_t remaining=table.rows.size();
    while (remaining) {
        const auto previous=remaining;
        for (std::size_t i=0;i<table.rows.size();++i) {
            if (inserted[i]) continue; const auto& row=table.rows[i];
            if (!row.parentId.empty() && !mapping.contains(row.parentId)) continue;
            ON_Layer layer; layer.SetId(nativeIds.at(row.id)); layer.SetName(ON_wString(row.name.c_str()));
            if (!row.parentId.empty()) layer.SetParentId(nativeIds.at(row.parentId));
            layer.SetColor(ON_Color(row.rgb[0],row.rgb[1],row.rgb[2]));
            layer.SetLocked(row.locked); layer.SetVisible(row.visible);
            if (row.persistentLocked) layer.SetPersistentLocking(*row.persistentLocked); else layer.UnsetPersistentLocking();
            if (row.persistentVisible) layer.SetPersistentVisibility(*row.persistentVisible); else layer.UnsetPersistentVisibility();
            writePersistentWitness(layer,row);
            auto reference=model.AddModelComponent(layer); auto stored=ON_Layer::Cast(reference.ModelComponent());
            if (!stored) throw std::runtime_error("Native SDK refused validated layer insertion");
            // AddModelComponent may remap a conflicting physical UUID. Children
            // and current-layer settings must reference the actual stored ID.
            nativeIds[row.id]=stored->Id();
            mapping.emplace(row.id,stored->Index()); inserted[i]=true; --remaining;
        }
        if (remaining==previous) throw std::runtime_error("Native layer insertion made no progress");
    }
    if (!table.activeId.empty()) {
        model.m_settings.SetCurrentLayerId(nativeIds.at(table.activeId));
        model.m_settings.SetV5CurrentLayerIndex(mapping.at(table.activeId));
    }
    return mapping;
}
NativeRetainedLayerResult applyRetainedLayerOverlay(ONX_Model& model,std::uint64_t source,
    const std::vector<NativeLayerObjectBinding>& bindings) {
    Om9LayerCounts sourceCounts{};rustError(om9_layer_snapshot_counts(source,&sourceCounts));
    if(bindings.size()!=sourceCounts.object_count)throw std::runtime_error("Retained layer object binding count mismatch");
    const auto before=readNativeLayerTable(model);
    auto destination=createNativeLayerSnapshot(before,{},"retained-native-archive",0);
    auto less=[](const ON_UUID& a,const ON_UUID& b){return ON_UuidCompare(a,b)<0;};
    std::set<ON_UUID,decltype(less)> members(less);
    ONX_ModelComponentIterator definitions(model,ON_ModelComponent::Type::InstanceDefinition);
    for(auto component=definitions.FirstComponent();component;component=definitions.NextComponent()){
        auto definition=ON_InstanceDefinition::Cast(component);const auto& ids=definition->InstanceGeometryIdList();
        for(int i=0;i<ids.Count();++i)members.insert(ids[i]);
    }
    std::vector<Om9LayerGeometryBindingView> facts;facts.reserve(bindings.size());
    std::map<std::string,ON_3dmObjectAttributes*> attributes;
    std::map<std::string,std::string> sourceIds;
    for(const auto& binding:bindings){
        auto component=ON_ModelGeometryComponent::Cast(model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(binding.physicalId.c_str())).ModelComponent());
        auto native=component?component->ExclusiveAttributes():nullptr;
        if(!component||!native||uuid(component->Id())!=binding.physicalId)throw std::runtime_error("Missing exact native retained object identity");
        if(native->m_space!=ON::model_space||native->IsInstanceDefinitionObject()||members.contains(component->Id()))throw std::runtime_error("Canonical retained binding requires a model-space object; native definition context stays in its preservation graph");
        facts.push_back({bytes(binding.physicalId),bytes(binding.sourceId)});
        attributes.emplace(binding.physicalId,native);sourceIds.emplace(binding.physicalId,binding.sourceId);
    }
    std::uint64_t handle=0;rustError(om9_layer_retained_overlay(source,destination.get(),facts.data(),facts.size(),&handle));
    NativeLayerSnapshot plan(handle);
    NativeRetainedLayerResult result{nativeLayerTableFromSnapshot(plan.get()),nativeObjectLayersFromSnapshot(plan.get())};
    std::map<std::string,const NativeLayerRow*> prior;
    std::map<std::string,ON_UUID> nativeIds;std::map<std::string,int> indices;
    for(const auto& row:before.rows){prior.emplace(row.id,&row);auto reference=model.ComponentFromId(ON_ModelComponent::Type::Layer,ON_UuidFromString(row.id.c_str()));auto layer=ON_Layer::Cast(reference.ModelComponent());nativeIds.emplace(row.id,layer->Id());indices.emplace(row.id,layer->Index());}
    auto unchanged=[](const NativeLayerRow& a,const NativeLayerRow& b){return a.id==b.id&&a.parentId==b.parentId&&a.name==b.name&&a.rgb==b.rgb&&a.locked==b.locked&&a.visible==b.visible&&a.persistentLocked==b.persistentLocked&&a.persistentVisible==b.persistentVisible;};
    std::vector<bool> done(result.table.rows.size(),false);std::size_t remaining=done.size();
    while(remaining){const auto start=remaining;
        for(std::size_t i=0;i<done.size();++i){
            if(done[i])continue;const auto& row=result.table.rows[i];
            if(!row.parentId.empty()&&!nativeIds.contains(row.parentId))continue;
            ON_Layer fresh;ON_Layer* layer=&fresh;
            auto existing=prior.find(row.id);
            if(existing!=prior.end()){
                if(unchanged(row,*existing->second)){done[i]=true;--remaining;continue;}
                layer=const_cast<ON_Layer*>(ON_Layer::Cast(model.ComponentFromId(ON_ModelComponent::Type::Layer,nativeIds.at(row.id)).ModelComponent()));
            }else{auto identifier=ON_UuidFromString(row.id.c_str());if(ON_UuidIsNil(identifier))ON_CreateUuid(identifier);fresh.SetId(identifier);}
            // Source-wins fullpath matches change existing spelling only. They
            // retain UUID/index and parent identity; no geometry reindexing.
            if(!layer->SetName(ON_wString(row.name.c_str())))throw std::runtime_error("Native SDK refused canonical layer spelling");
            layer->SetParentId(row.parentId.empty()?ON_nil_uuid:nativeIds.at(row.parentId));
            layer->SetColor(ON_Color(row.rgb[0],row.rgb[1],row.rgb[2]));layer->SetLocked(row.locked);layer->SetVisible(row.visible);
            if(row.persistentLocked)layer->SetPersistentLocking(*row.persistentLocked);else layer->UnsetPersistentLocking();
            if(row.persistentVisible)layer->SetPersistentVisibility(*row.persistentVisible);else layer->UnsetPersistentVisibility();
            writePersistentWitness(*layer,row);
            if(existing==prior.end()){
                auto added=model.AddModelComponent(fresh);auto stored=ON_Layer::Cast(added.ModelComponent());
                if(!stored||utf8(stored->Name())!=row.name||stored->ParentId()!=(row.parentId.empty()?ON_nil_uuid:nativeIds.at(row.parentId)))throw std::runtime_error("Native SDK changed canonical layer path");
                nativeIds[row.id]=stored->Id();indices[row.id]=stored->Index();
            }
            done[i]=true;--remaining;
        }
        if(remaining==start)throw std::runtime_error("Retained layer insertion made no progress");
    }
    if(!result.table.activeId.empty()){model.m_settings.SetCurrentLayerId(nativeIds.at(result.table.activeId));model.m_settings.SetV5CurrentLayerIndex(indices.at(result.table.activeId));}
    for(auto& row:result.objects){
        auto native=attributes.at(row.id);native->m_layer_index=indices.at(row.layerId);
        native->m_color=ON_Color(row.rgb[0],row.rgb[1],row.rgb[2]);native->SetColorSource(row.colorSource==1?ON::color_from_layer:ON::color_from_object);
        native->SetMode(row.locked?ON::locked_object:ON::normal_object);native->SetVisible(row.visible);
        native->SetUserString(L"OpenMatrix9.Locked",row.locked?L"1":L"0");
        native->SetUserString(L"OpenMatrix9.LayerObjectId",ON_wString(sourceIds.at(row.id).c_str()));
        row.layerId=uuid(nativeIds.at(row.layerId));
    }
    for(auto& row:result.table.rows){row.id=uuid(nativeIds.at(row.id));if(!row.parentId.empty())row.parentId=uuid(nativeIds.at(row.parentId));}
    if(!result.table.activeId.empty())result.table.activeId=uuid(model.m_settings.CurrentLayerId());
    const auto context=nativeLayerContextFromSnapshot(plan.get());
    auto checked=createNativeLayerSnapshot(result.table,result.objects,context.document,context.generation);
    return result;
}
std::map<std::string,std::string> mergeRetainedNativePalette(ONX_Model& model,const NativeLayerTable& table){
    auto source=createNativeLayerSnapshot(table,{},"namespace-native-palette",0);
    auto destination=createNativeLayerSnapshot(readNativeLayerTable(model),{},"retained-native-archive",0);
    struct OwnedPlan{std::uint64_t handle=0;~OwnedPlan(){if(handle)om9_layer_plan_free(handle);}} plan;
    rustError(om9_layer_plan_receive(source.get(),destination.get(),1,nullptr,0,&plan.handle));
    std::uint64_t afterHandle=0;rustError(om9_layer_plan_after_snapshot(plan.handle,&afterHandle));NativeLayerSnapshot after(afterHandle);
    const auto planned=nativeLayerTableFromSnapshot(after.get());
    std::map<std::string,std::string> sourceToPlanned;
    for(const auto& row:table.rows){std::size_t needed=0;auto status=om9_layer_plan_map(plan.handle,1,bytes(row.id),nullptr,0,&needed);if(status!=15)rustError(status);if(needed>1024)throw std::runtime_error("Native mapped layer ID exceeds bound");std::string value(needed,'\0');rustError(om9_layer_plan_map(plan.handle,1,bytes(row.id),reinterpret_cast<unsigned char*>(value.data()),value.size(),&needed));sourceToPlanned.emplace(row.id,std::move(value));}
    auto applied=applyRetainedLayerOverlay(model,source.get(),{});
    if(planned.rows.size()!=applied.table.rows.size())throw std::runtime_error("Native retained palette plan changed during detached apply");
    std::map<std::string,std::string> plannedToNative;
    for(std::size_t i=0;i<planned.rows.size();++i){if(planned.rows[i].path!=applied.table.rows[i].path)throw std::runtime_error("Native retained palette ordering mismatch");plannedToNative.emplace(planned.rows[i].id,applied.table.rows[i].id);}
    std::map<std::string,std::string> result;for(const auto& [sourceId,plannedId]:sourceToPlanned)result.emplace(sourceId,plannedToNative.at(plannedId));return result;
}
}
