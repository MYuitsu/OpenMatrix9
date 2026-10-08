#include "ThreeDmModeling.h"
#include "ThreeDmArchive.h"
#include "ThreeDmStaging.h"
#include "ThreeDmInventory.h"
#include <QJsonArray>
#include <QFile>
#include <QCryptographicHash>
extern "C" unsigned om9_modeling_capabilities(unsigned kind);
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject modelingCapabilities(unsigned kind){
    const auto bits=om9_modeling_capabilities(kind);
    QJsonObject result;
    const char* names[]={"select","snap","transform","curve-edit","surface-edit","boolean","export-v5"};
    for(unsigned i=0;i<7;++i){bool allowed=bits&(1u<<i);result[names[i]]=QJsonObject{{"allowed",allowed},{"support",allowed?(i==1||i==5?"scoped":"supported"):"unsupported"},{"reason",allowed?(i==1?"Existing CAD End/Mid/Point snap; bounded queries follow in Phase 2":i==5?"Native Part kernel tools":"Working geometry"):"Not available for this geometry kind in Phase 1"}};}
    return result;
}
QJsonObject prepareModelingArchive(const std::filesystem::path& input,const std::filesystem::path& staging,double scale){
    // Strict reader finishes the entire conversion before any host transaction.
    // It rejects opaque geometry and expands embedded placements in world space.
    if(std::filesystem::file_size(input)>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    auto snapshot=staging/"working-source.3dm";
    std::filesystem::copy_file(input,snapshot);
    auto inventory=inspectArchive(snapshot,scale);
    auto manifest=inventory.document;
    // All geometry dependencies are resolved by the strict reader in workers.
    // History/render issues are outside this independent geometry workflow.
    auto prepared=stagePreservedArchive(snapshot,staging,scale,false,inventory.nativeModel.get(),true);
    QFile current(QString::fromStdWString(input.wstring()));
    QCryptographicHash digest(QCryptographicHash::Sha256);
    if(!current.open(QIODevice::ReadOnly)||!digest.addData(&current)||QString::fromLatin1(digest.result().toHex())!=manifest["archive_sha256"].toString())throw ExchangeError("Source archive changed during modeling preparation");
    QJsonArray items;
    for(auto value:prepared){auto row=value.toObject();unsigned kind=row["geometry_kind"].toInt(9);
        if(row["capability"]=="retained"||!(om9_modeling_capabilities(kind)&64))throw ExchangeError("Unsupported independent working geometry: "+row["name"].toString().toStdString()+" ["+row["class_name"].toString().toStdString()+"]");
        row["working_mode"]=true;items.append(row);
    }
    const double tolerance=items.isEmpty()?1e-6:items[0].toObject()["tolerance"].toDouble(1e-6);
    return QJsonObject{{"schema_version",1},{"items",items},{"tolerance",tolerance},{"scale_mm",manifest["scale_mm"]},{"scope_omissions",QJsonArray{"history","render","materials","textures","lights","layouts","userdata"}}};
}
}
