#include "ThreeDmModeling.h"
#include "ThreeDmArchive.h"
#include "ThreeDmStaging.h"
#include "ThreeDmInventory.h"
#include "ThreeDmThreadPool.h"
#include <QJsonArray>
#include <QFile>
#include <QCryptographicHash>
#include <QElapsedTimer>
extern "C" unsigned om9_modeling_capabilities(unsigned kind);
extern "C" unsigned om9_modeling_worker_target(unsigned available);
extern "C" unsigned om9_modeling_workers(unsigned available,unsigned tasks,unsigned ramSlots);
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject modelingCapabilities(unsigned kind){
    const auto bits=om9_modeling_capabilities(kind);
    QJsonObject result;
    const char* names[]={"select","snap","transform","curve-edit","surface-edit","boolean","export-v5"};
    for(unsigned i=0;i<7;++i){bool allowed=bits&(1u<<i);result[names[i]]=QJsonObject{{"allowed",allowed},{"support",allowed?(i==1||i==5?"scoped":"supported"):"unsupported"},{"reason",allowed?(i==1?"Existing CAD End/Mid/Point snap; bounded queries follow in Phase 2":i==5?"Native Part kernel tools":"Working geometry"):"Not available for this geometry kind in Phase 1"}};}
    return result;
}
QJsonObject prepareModelingArchive(const std::filesystem::path& input,const std::filesystem::path& staging,double scale,const std::filesystem::path& layerMetadata){
    // Strict reader finishes the entire conversion before any host transaction.
    // It rejects opaque geometry and expands embedded placements in world space.
    if(std::filesystem::file_size(input)>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    auto snapshot=staging/"working-source.3dm";
    std::filesystem::copy_file(input,snapshot);
    auto hash=[](const std::filesystem::path& path){QFile file(QString::fromStdWString(path.wstring()));QCryptographicHash digest(QCryptographicHash::Sha256);if(!file.open(QIODevice::ReadOnly)||!digest.addData(&file))throw ExchangeError("Cannot hash modeling source");return digest.result();};
    const auto sourceHash=hash(snapshot);
    NativeLayerSnapshot sourceLayers;Om9LayerClipboardInfo clipboardInfo{};
    if(!layerMetadata.empty()){
        auto readBounded=[](const std::filesystem::path& path,unsigned kind){
            QFile file(QString::fromStdWString(path.wstring()));
            if(!file.open(QIODevice::ReadOnly)||file.size()<0)throw ExchangeError("Cannot read clipboard input snapshot");
            const auto size=static_cast<std::size_t>(file.size());
            const auto status=om9_layer_clipboard_check_size(kind,size);
            if(status)throw ExchangeError("Clipboard input size rejected: Rust code "+std::to_string(status));
            auto body=file.read(static_cast<qint64>(size)+1);
            if(static_cast<std::size_t>(body.size())!=size||file.size()!=static_cast<qint64>(size)||file.error()!=QFileDevice::NoError)throw ExchangeError("Clipboard snapshot read was incomplete or changed");
            return body;
        };
        // Both paths are detached snapshots. Validate their exact bytes before
        // the SDK sees geometry, and never silently fall back on bad metadata.
        auto metadataSnapshot=staging/"working-layer-metadata.json";
        auto metadata=readBounded(layerMetadata,2),geometry=readBounded(snapshot,1);
        auto view=[](const QByteArray& body)->Om9LayerByteView{return {reinterpret_cast<const unsigned char*>(body.constData()),static_cast<std::size_t>(body.size())};};
        std::uint64_t handle=0;const auto error=om9_layer_clipboard_receive(view(geometry),view(metadata),1,&handle,&clipboardInfo);
        if(error)throw ExchangeError("Extended clipboard input rejected: Rust code "+std::to_string(error));
        sourceLayers=NativeLayerSnapshot(handle);
        QFile savedMetadata(QString::fromStdWString(metadataSnapshot.wstring()));
        if(!savedMetadata.open(QIODevice::WriteOnly|QIODevice::NewOnly))throw ExchangeError("Cannot create metadata snapshot");
        if(savedMetadata.write(metadata)!=metadata.size()){savedMetadata.close();savedMetadata.remove();throw ExchangeError("Cannot save metadata snapshot");}
        savedMetadata.close();
    }
    auto cancelled=[&]{if(std::filesystem::exists(staging/"cancel"))throw ExchangeError("3DM operation cancelled");};
    cancelled();QElapsedTimer clock;clock.start();QJsonObject timings;
    auto model=readWorkingArchiveDeferred(snapshot,scale);
    if(sourceLayers.get())applyExtendedLayerOverlay(model,sourceLayers.get());
    if(hash(snapshot)!=sourceHash)throw ExchangeError("Working snapshot changed during SDK preparation");
    timings["sdk_prepare"]=clock.restart()/1000.0;
    const auto available=availableCpuThreads(),target=om9_modeling_worker_target(available),ramSlots=memoryWorkerSlots();
    bool valid=false;auto requested=qEnvironmentVariableIntValue("OM9_MODELING_WORKERS",&valid);
    unsigned effective=om9_modeling_workers(available,static_cast<unsigned>(model.pendingBreps.size()),ramSlots);
    if(valid&&requested>0)effective=std::min(effective,static_cast<unsigned>(requested));
    if(model.pendingBreps.size()<4)effective=std::min(effective,1u);
    auto peak=runNativeJobs(model.pendingBreps.size(),effective,[&](size_t i){cancelled();finishDeferredBrep(model,i);});
    cancelled();timings["assemble_validate"]=clock.restart()/1000.0;
    auto base=staging/"geometry";if(!std::filesystem::create_directory(base))throw ExchangeError("Geometry staging target already exists");
    std::vector<QJsonObject> rows(model.items.size());std::vector<size_t> jobs;
    for(size_t i=0;i<model.items.size();++i){
        if(std::holds_alternative<ON_PointCloud>(model.items[i].geometry))rows[i]=encodeExchangeItem(model.items[i],base,static_cast<int>(i));
        else jobs.push_back(i);
    }
    unsigned stagingWorkers=om9_modeling_workers(available,static_cast<unsigned>(jobs.size()),ramSlots);
    if(valid&&requested>0)stagingWorkers=std::min(stagingWorkers,static_cast<unsigned>(requested));
    if(jobs.size()<4)stagingWorkers=std::min(stagingWorkers,1u);
    auto stagePeak=runNativeJobs(jobs.size(),stagingWorkers,[&](size_t j){cancelled();auto i=jobs[j];rows[i]=encodeExchangeItem(model.items[i],base,static_cast<int>(i));});
    QJsonArray prepared;for(auto row:rows){row["tolerance"]=model.tolerance;prepared.append(row);}
    cancelled();timings["stage"]=clock.restart()/1000.0;
    if(hash(input)!=sourceHash)throw ExchangeError("Source archive changed during modeling preparation");
    QJsonArray items;
    for(auto value:prepared){auto row=value.toObject();unsigned kind=row["geometry_kind"].toInt(9);
        if(row["capability"]=="retained"||!(om9_modeling_capabilities(kind)&64))throw ExchangeError("Unsupported independent working geometry: "+row["name"].toString().toStdString()+" ["+row["class_name"].toString().toStdString()+"]");
        row["working_mode"]=true;items.append(row);
    }
    auto layerSession=encodeExchangeLayers(model,"working-archive",0);
    QJsonObject result{{"schema_version",2},{"layer_session",layerSession},{"items",items},{"tolerance",model.tolerance},{"scale_mm",model.scaleMm},
        {"workers",QJsonObject{{"available",static_cast<int>(available)},{"target",static_cast<int>(target)},{"effective",static_cast<int>(effective)},{"peak",static_cast<int>(peak)},{"staging_effective",static_cast<int>(stagingWorkers)},{"staging_peak",static_cast<int>(stagePeak)},{"ram_slots",static_cast<int>(ramSlots)},{"tasks",static_cast<int>(model.pendingBreps.size())},{"reason","bounded by tasks, RAM (256 MiB/worker), small-job threshold and optional lower benchmark limit"}}},
        {"timings",timings},{"scope_omissions",QJsonArray{"history","render","materials","textures","lights","layouts","userdata"}}};
    result["clipboard"]=QJsonObject{{"evidence",sourceLayers.get()?2:1},{"scope",static_cast<int>(clipboardInfo.scope)}};
    return result;
}
}
