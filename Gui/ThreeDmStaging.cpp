#include "ThreeDmStaging.h"
#include "ThreeDmPointCloud.h"
#include <BRepTools.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <cstdlib>
#include <unordered_map>
#include <unordered_set>
namespace OpenMatrix9Gui::ThreeDm {
void decodeExchangeLayers(ExchangeModel& model,const QJsonObject& encoded){
    auto bytes=QJsonDocument(encoded).toJson(QJsonDocument::Compact);
    auto snapshot=nativeLayerSnapshotFromJson(std::string(bytes.constData(),bytes.size()));
    auto table=nativeLayerTableFromSnapshot(snapshot.get());auto objects=nativeObjectLayersFromSnapshot(snapshot.get());
    auto context=nativeLayerContextFromSnapshot(snapshot.get());
    std::unordered_map<std::string,const NativeObjectLayerRow*> byId;byId.reserve(objects.size());
    for(const auto& row:objects)byId.emplace(row.id,&row);
    std::unordered_set<std::string> used;std::vector<NativeObjectLayerRow> matched;matched.reserve(model.items.size());
    for(const auto& item:model.items){
        if(!item.ownState)throw ExchangeError("Geometry row is missing its canonical layer object binding");
        auto found=byId.find(item.ownState->id);
        if(found==byId.end())throw ExchangeError("Geometry row references missing canonical object metadata: "+item.ownState->id);
        if(!used.insert(item.ownState->id).second)throw ExchangeError("Geometry rows duplicate a canonical object binding: "+item.ownState->id);
        matched.push_back(*found->second);
    }
    // The native bridge commits detached metadata only after every binding is valid.
    model.layerTable=std::move(table);
    model.layerContext=std::move(context);
    for(std::size_t i=0;i<model.items.size();++i)model.items[i].ownState=std::move(matched[i]);
}
QJsonObject encodeExchangeLayers(const ExchangeModel& model,const std::string& document,std::uint64_t generation){
    std::vector<NativeObjectLayerRow> objects;objects.reserve(model.items.size());
    for(const auto& item:model.items){
        if(!item.ownState)throw ExchangeError("Layer staging is missing own object metadata: "+item.name);
        if(item.ownState->colorSource!=1&&item.ownState->colorSource!=2)
            throw ExchangeError("Layer staging needs native instance color context: '"+item.name+"' ["+item.sourceClass+"]");
        objects.push_back(*item.ownState);
    }
    try{
        // Carry only origin facts; rebuild/validate current rows rather than
        // reusing a cached snapshot whose object/layer state may now be stale.
        const auto& source=model.layerContext;
        auto snapshot=createNativeLayerSnapshot(model.layerTable,objects,source?source->document:document,source?source->generation:generation);
        auto text=nativeLayerSnapshotJson(snapshot.get());QJsonParseError error;
        auto encoded=QJsonDocument::fromJson(QByteArray(text.data(),static_cast<qsizetype>(text.size())),&error);
        if(error.error!=QJsonParseError::NoError||!encoded.isObject())throw ExchangeError("Qt cannot carry validated Rust layer JSON");
        return encoded.object();
    }catch(const std::exception& error){throw ExchangeError(std::string("Cannot stage canonical layer metadata: ")+error.what());}
}
QJsonObject encodeExchangeItem(const ExchangeItem& item,const std::filesystem::path& dir,int index){
    QJsonObject o{{"name",QString::fromStdString(item.name)},{"layer",QString::fromStdString(item.layer)},{"visible",item.visible},{"locked",item.locked}};
    o["color"]=QJsonArray{item.color[0],item.color[1],item.color[2]};o["wire_density"]=item.wireDensity;
    if(std::getenv("OM9_3DM_PROFILE"))o["_conversion_seconds"]=item.conversionSeconds;
    o["source_uuid"]=QString::fromStdString(item.sourceUuid);o["class_name"]=QString::fromStdString(item.sourceClass);
    if(item.ownState){o["layer_object_id"]=QString::fromStdString(item.ownState->id);o["layer_id"]=QString::fromStdString(item.ownState->layerId);}
    if(item.retained){o["capability"]="retained";if(!item.representationIssue.empty())o["representation_issue"]=QString::fromStdString(item.representationIssue);return o;}
    o["capability"]="editable";
    o["root_uuid"]=QString::fromStdString(item.sourceRootUuid);
    if(auto shape=std::get_if<TopoDS_Shape>(&item.geometry)){o["geometry_kind"]=shape->ShapeType()==TopAbs_VERTEX?1:shape->ShapeType()==TopAbs_EDGE||shape->ShapeType()==TopAbs_WIRE?2:3;o["representation"]="native-cad";}
    else if(std::holds_alternative<MeshData>(item.geometry)){o["geometry_kind"]=4;o["representation"]="native-mesh";}
    else{o["geometry_kind"]=5;o["representation"]="native-points";}
    if(auto shape=std::get_if<TopoDS_Shape>(&item.geometry)){
        auto file=dir/(std::to_string(index)+".brep");
        if(!BRepTools::Write(*shape,file.string().c_str()))throw ExchangeError("Cannot stage imported shape");
        o["brep"]=QString::fromStdString(file.string());
    }else if(auto cloud=std::get_if<ON_PointCloud>(&item.geometry)){
        auto file=dir/(std::to_string(index)+".point-cloud.json");
        o["point_cloud_sha256"]=writePointCloudFields(*cloud,file);
        o["point_cloud_fields"]=QString::fromStdWString(file.wstring());
    }else{auto& mesh=std::get<MeshData>(item.geometry);QJsonArray vertices,faces;
        for(auto p:mesh.vertices)vertices.append(QJsonArray{p[0],p[1],p[2]});
        for(auto f:mesh.faces)faces.append(QJsonArray{f[0],f[1],f[2],f[3]});
        o["vertices"]=vertices;o["faces"]=faces;
    }return o;
}
}
