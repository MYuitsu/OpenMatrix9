#include "ThreeDmStaging.h"
#include "ThreeDmPointCloud.h"
#include <BRepTools.hxx>
#include <QJsonArray>
#include <cstdlib>
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject encodeExchangeItem(const ExchangeItem& item,const std::filesystem::path& dir,int index){
    QJsonObject o{{"name",QString::fromStdString(item.name)},{"layer",QString::fromStdString(item.layer)},{"visible",item.visible},{"locked",item.locked}};
    o["color"]=QJsonArray{item.color[0],item.color[1],item.color[2]};o["wire_density"]=item.wireDensity;
    if(std::getenv("OM9_3DM_PROFILE"))o["_conversion_seconds"]=item.conversionSeconds;
    o["source_uuid"]=QString::fromStdString(item.sourceUuid);o["class_name"]=QString::fromStdString(item.sourceClass);
    if(item.retained){o["capability"]="retained";if(!item.representationIssue.empty())o["representation_issue"]=QString::fromStdString(item.representationIssue);return o;}
    o["capability"]="editable";
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
