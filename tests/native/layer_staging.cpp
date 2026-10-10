// SPDX-License-Identifier: LGPL-2.1-or-later
#include "ThreeDmStaging.h"
#include "ThreeDmModeling.h"
#include <QJsonDocument>
#include <QFile>
#include <filesystem>
#include <iostream>
#include <stdexcept>
#include <limits>
using namespace OpenMatrix9Gui::ThreeDm;
namespace{int checks=0;void check(bool value,const char* text){++checks;if(!value)throw std::runtime_error(text);}}
int main(){try{
    const auto fixtures=std::filesystem::path(OM9_LAYER_FIXTURES);
    auto model=readArchive(fixtures/"layer-source.3dm");
    auto encoded=encodeExchangeLayers(model,"staging-test",7);
    const auto bytes=QJsonDocument(encoded).toJson(QJsonDocument::Compact);
    auto snapshot=nativeLayerSnapshotFromJson(std::string(bytes.constData(),bytes.size()));
    Om9LayerCounts counts{};check(!om9_layer_snapshot_counts(snapshot.get(),&counts),"Rust accepts staged full snapshot");
    check(counts.layer_count==4&&counts.object_count==2&&counts.generation==7,"Staging carries complete table and selected metadata");
    auto highest=encodeExchangeLayers(model,"generation-lossless",std::numeric_limits<std::uint64_t>::max());
    auto highestBytes=QJsonDocument(highest).toJson(QJsonDocument::Compact);
    auto highestHandle=nativeLayerSnapshotFromJson(std::string(highestBytes.constData(),highestBytes.size()));
    check(!om9_layer_snapshot_counts(highestHandle.get(),&counts)&&counts.generation==std::numeric_limits<std::uint64_t>::max(),"Qt staging preserves full uint64 generation exactly");
    const auto directory=fixtures/"staged-geometry";std::filesystem::create_directories(directory);
    auto item=encodeExchangeItem(model.items[0],directory,0);
    check(item["layer_object_id"].toString().toStdString()==model.items[0].ownState->id,"Geometry row binds its exact Rust metadata object ID");
    check(item["layer_id"].toString().toStdString()==model.items[0].ownState->layerId,"Geometry row keeps native layer ID");
    auto restored=model;restored.layerTable={};
    for(auto& value:restored.items){value.ownState->layerId="lost";value.ownState->locked=true;}
    decodeExchangeLayers(restored,encoded);
    check(restored.layerTable.rows.size()==4&&!restored.items[0].ownState->locked,"Decode owns canonical table and own flags through Rust");
    check(restored.items[1].ownState->rgb==std::array<unsigned char,3>{23,24,25},"Decode preserves ByObject fields");
    auto missing=model;missing.items[0].ownState->id="unknown";auto before=missing.layerTable.rows;
    bool rejected=false;try{decodeExchangeLayers(missing,encoded);}catch(const std::exception&){rejected=true;}
    check(rejected&&missing.layerTable.rows.size()==before.size(),"Missing geometry metadata binding rejects before model mutation");
    ON_UUID run;ON_CreateUuid(run);char runId[37]{};ON_UuidToString(run,runId);
    auto staging=fixtures/(std::string("working-prepare-")+runId);std::filesystem::create_directory(staging);
    auto prepared=prepareModelingArchive(fixtures/"layer-source.3dm",staging,0);
    check(prepared["schema_version"].toInt()==2,"Working preparation uses schema 2");
    check(prepared["items"].toArray().size()==2&&prepared["layer_session"].isObject(),"Working preparation contains independent palette");
    auto paletteStaging=fixtures/(std::string("palette-prepare-")+runId);std::filesystem::create_directory(paletteStaging);
    auto palette=prepareModelingArchive(fixtures/"layer-palette.3dm",paletteStaging,0);
    check(palette["items"].toArray().isEmpty()&&palette["layer_session"].isObject(),"Palette-only preparation allowed");
    auto paletteBytes=QJsonDocument(palette["layer_session"].toObject()).toJson(QJsonDocument::Compact);
    auto paletteHandle=nativeLayerSnapshotFromJson(std::string(paletteBytes.constData(),paletteBytes.size()));
    check(!om9_layer_snapshot_counts(paletteHandle.get(),&counts)&&counts.layer_count==4&&!counts.object_count,"Palette-only metadata validates through Rust");
    const auto clipboardPath=fixtures/"layer-staging-clipboard.3dm";writeArchive5(model,clipboardPath);
    QFile clipboard(QString::fromStdWString(clipboardPath.wstring()));check(clipboard.open(QIODevice::ReadOnly),"Read real clipboard geometry");
    auto geometry=clipboard.readAll();clipboard.close();
    auto byteView=[](const QByteArray& body)->Om9LayerByteView{return {reinterpret_cast<const unsigned char*>(body.constData()),static_cast<std::size_t>(body.size())};};
    std::uint64_t binding=0;check(!om9_layer_clipboard_prepare(highestHandle.get(),2,byteView(geometry),&binding),"Bind current archive and full source metadata");
    std::size_t needed=0;check(om9_layer_clipboard_bytes(binding,nullptr,0,&needed)==15,"Query extended metadata size");
    QByteArray metadata(static_cast<qsizetype>(needed),'\0');check(!om9_layer_clipboard_bytes(binding,reinterpret_cast<unsigned char*>(metadata.data()),metadata.size(),&needed),"Get exact bound metadata");om9_layer_clipboard_free(binding);
    const auto metadataPath=fixtures/"layer-staging-clipboard.json";
    auto writeBytes=[](const std::filesystem::path& path,const QByteArray& body){QFile file(QString::fromStdWString(path.wstring()));if(!file.open(QIODevice::WriteOnly|QIODevice::Truncate)||file.write(body)!=body.size())throw std::runtime_error("Fixture write failed");};
    writeBytes(metadataPath,metadata);
    writeBytes(fixtures/"layer-staging-clipboard.bound.json",metadata);
    auto extendedDir=fixtures/(std::string("extended-prepare-")+runId);std::filesystem::create_directory(extendedDir);
    auto extended=prepareModelingArchive(clipboardPath,extendedDir,0,metadataPath);
    auto extendedBytes=QJsonDocument(extended["layer_session"].toObject()).toJson(QJsonDocument::Compact);
    auto extendedHandle=nativeLayerSnapshotFromJson(std::string(extendedBytes.constData(),extendedBytes.size()));
    check(!om9_layer_snapshot_counts(extendedHandle.get(),&counts)&&counts.generation==std::numeric_limits<std::uint64_t>::max(),"Worker keeps exact source generation");
    check(extended["layer_session"].toObject()["snapshot"].toObject()["document_id"].toString()=="generation-lossless","Worker keeps source document identity");
    check(extended["clipboard"].toObject()["evidence"].toInt()==2&&extended["clipboard"].toObject()["scope"].toInt()==2,"Worker reports verified extended Session evidence");
    auto sourceObjects=extended["layer_session"].toObject()["snapshot"].toObject()["objects"].toArray();
    check(sourceObjects.size()==2&&sourceObjects[0].toObject()["id"]!=highest["snapshot"].toObject()["objects"].toArray()[0].toObject()["id"],"Worker rebinds source metadata to physical UUIDs");
    check(extended["items"].toArray()[0].toObject()["layer_object_id"]==sourceObjects[0].toObject()["id"],"Staged geometry binds rebound metadata exactly");
    auto malformedDir=fixtures/(std::string("malformed-prepare-")+runId);std::filesystem::create_directory(malformedDir);
    writeBytes(metadataPath,metadata.left(metadata.size()-1));rejected=false;
    try{prepareModelingArchive(clipboardPath,malformedDir,0,metadataPath);}catch(const std::exception&){rejected=true;}
    check(rejected&&!std::filesystem::exists(malformedDir/"geometry"),"Truncated metadata refuses before geometry staging");
    auto mismatchDir=fixtures/(std::string("digest-prepare-")+runId);std::filesystem::create_directory(mismatchDir);
    writeBytes(metadataPath,metadata);auto changed=geometry;changed[changed.size()-1]^=1;writeBytes(clipboardPath,changed);rejected=false;
    try{prepareModelingArchive(clipboardPath,mismatchDir,0,metadataPath);}catch(const std::exception&){rejected=true;}
    check(rejected&&!std::filesystem::exists(mismatchDir/"geometry"),"Digest mismatch refuses before geometry staging");
    writeBytes(clipboardPath,geometry);writeBytes(metadataPath,QByteArray());
    auto emptyDir=fixtures/(std::string("empty-meta-prepare-")+runId);std::filesystem::create_directory(emptyDir);rejected=false;
    try{prepareModelingArchive(clipboardPath,emptyDir,0,metadataPath);}catch(const std::exception&){rejected=true;}
    check(rejected&&!std::filesystem::exists(emptyDir/"geometry"),"Present empty metadata never downgrades");
    writeBytes(metadataPath,metadata);
    auto protectedDir=fixtures/(std::string("protected-stage-")+runId);std::filesystem::create_directories(protectedDir/"geometry");
    auto protectedFile=protectedDir/"geometry"/"0.brep";writeBytes(protectedFile,"keep staging");rejected=false;
    try{prepareModelingArchive(clipboardPath,protectedDir,0,metadataPath);}catch(const std::exception&){rejected=true;}
    QFile protectedRead(QString::fromStdWString(protectedFile.wstring()));check(protectedRead.open(QIODevice::ReadOnly),"Read protected existing staging");
    check(rejected&&protectedRead.readAll()=="keep staging","Existing geometry staging refuses without overwrite");
    auto currentState=readArchive(clipboardPath);applyExtendedLayerOverlay(currentState,highestHandle.get());
    currentState.items[0].ownState->locked=true;
    auto fresh=encodeExchangeLayers(currentState,"fallback-context",0);
    check(fresh["snapshot"].toObject()["document_id"]=="generation-lossless"&&fresh["snapshot"].toObject()["generation"]==QString::number(std::numeric_limits<std::uint64_t>::max()),"Re-encoding keeps origin context");
    check(fresh["snapshot"].toObject()["objects"].toArray()[0].toObject()["locked"].toBool(),"Re-encoding validates current metadata rather than a cached snapshot");
    // Accepted geometry supports 64 block levels; metadata IDs must not impose
    // a smaller limit merely by concatenating all placement UUIDs.
    ONX_Model deep;deep.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    ON_Layer deepLayer;deepLayer.SetName(L"Deep");deep.AddModelComponent(deepLayer);
    ON_3dmObjectAttributes memberAttributes;memberAttributes.SetMode(ON::idef_object);memberAttributes.SetColorSource(ON::color_from_parent);
    ON_Point deepPoint(1,2,3);auto current=deep.AddModelGeometryComponent(&deepPoint,&memberAttributes);
    for(int depth=0;depth<40;++depth){
        ON_InstanceDefinition definition;definition.SetName(ON_wString(("Depth"+std::to_string(depth)).c_str()));definition.AddInstanceGeometryId(current.ModelComponent()->Id());
        auto added=deep.AddModelComponent(definition);ON_InstanceRef instance;instance.m_instance_definition_uuid=added.ModelComponent()->Id();instance.m_xform=ON_Xform::IdentityTransformation;
        auto attributes=memberAttributes;if(depth==39){attributes.SetMode(ON::normal_object);attributes.SetColorSource(ON::color_from_layer);}
        current=deep.AddModelGeometryComponent(&instance,&attributes);
    }
    const auto deepPath=fixtures/"layer-deep-blocks.3dm";check(deep.Write(deepPath.c_str(),5,nullptr),"Deep block source fixture");
    auto expanded=readArchive(deepPath);auto deepMetadata=encodeExchangeLayers(expanded,"deep-blocks");
    check(expanded.items.size()==1&&expanded.items[0].ownState->id.size()<=1024&&deepMetadata.isEmpty()==false,"Deep accepted block geometry keeps bounded metadata identity");
    std::cout<<"Layer staging PASS; checks="<<checks<<'\n';return 0;
}catch(const std::exception& error){std::cerr<<"Layer staging FAIL after "<<checks<<" checks: "<<error.what()<<'\n';return 1;}}
