// SPDX-License-Identifier: LGPL-2.1-or-later
#include "LayerSourceArchiveAdapter.h"
#include "LayerSourcePayloadAbi.h"
#include "ThreeDmInventory.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <QCoreApplication>
#include <QThread>
#include <QFile>
#include <QTemporaryDir>
#include <QJsonDocument>
#include <QJsonObject>
#include <QUuid>
#include <cmath>
#include <algorithm>
#include <vector>
#include <stdexcept>
#include <map>
namespace OpenMatrix9Gui {
namespace {
Om9LayerByteView bytes(const std::string& value){return {reinterpret_cast<const unsigned char*>(value.data()),value.size()};}
void status(std::uint32_t value){if(value)throw std::runtime_error("Source archive proof rejected: Rust code "+std::to_string(value));}
template<class T> T& field(App::DocumentObject& object,const char* name) {
    auto* property=dynamic_cast<T*>(object.getPropertyByName(name));
    if(!property)throw std::runtime_error(std::string("Source archive field missing or wrong type: ")+name);
    return *property;
}
struct Payload {std::uint64_t handle=0;~Payload(){if(handle)om9_layer_source_payload_free(handle);}};
void requireNativeStorageFields(App::DocumentObject& object) {
    if(object.getTypeId()!=Base::Type::fromName("App::FeaturePython"))
        throw std::runtime_error("Source archive owner is not the native storage object type");
    // Native binding contract only. Do not read Shape/Mesh/point arrays to
    // decide whether the plugin-owned storage helper carries extra payload.
    static const std::map<std::string,std::string> types{
        {"OM9ArchiveSchema","App::PropertyInteger"},{"OM9ArchiveMode","App::PropertyInteger"},
        {"OM9ImportNamespace","App::PropertyString"},{"OM9ArchiveManifest","App::PropertyString"},
        {"OM9ArchiveHash","App::PropertyString"},{"OM9SourceArchiveStorageVersion","App::PropertyInteger"},
        {"OM9SourceArchiveDigest","App::PropertyString"},{"OM9SourceArchiveChunks","App::PropertyStringList"},
        {"OM9SourceArchiveCachePath","App::PropertyString"},
        {"OM9LayerObjectId","App::PropertyString"},{"OM9LayerHostName","App::PropertyString"},
        {"OM9LayerId","App::PropertyString"},{"OM9Locked","App::PropertyBool"}
    };
    for(const auto& name:object.getDynamicPropertyNames()) {
        auto* property=object.getPropertyByName(name.c_str());
        const std::string type(property?property->getTypeId().getName():std::string_view{});
        // A migrated legacy FileIncluded may remain, while the verified owned
        // chunks are authoritative. Never read its path/file for this proof.
        if(name=="OM9SourceArchive" && (type=="App::PropertyString" || type=="App::PropertyFileIncluded"))continue;
        const auto expected=types.find(name);
        if(expected==types.end() || expected->second!=type)
            throw std::runtime_error("Source archive storage has an unknown or wrong-type field: "+name);
    }
    const auto& nameSpace=field<App::PropertyString>(object,"OM9ImportNamespace");
    const std::string identity=nameSpace.getValue();
    const QUuid uuid(QString::fromStdString(identity));
    if(uuid.isNull() || uuid.toString(QUuid::WithoutBraces).toStdString()!=identity)
        throw std::runtime_error("Source archive import namespace is not a canonical native identity");
}
}
QJsonObject verifyLayerSourceArchive(App::Document& doc,App::DocumentObject& object) {
    auto* app=QCoreApplication::instance();
    if(!app || QThread::currentThread()!=app->thread() || App::GetApplication().getActiveDocument()!=&doc || object.getDocument()!=&doc)
        throw std::runtime_error("Source archive proof requires the active native GUI document");
    requireNativeStorageFields(object);
    if(field<App::PropertyInteger>(object,"OM9ArchiveSchema").getValue()!=1 ||
       field<App::PropertyInteger>(object,"OM9ArchiveMode").getValue()!=1 ||
       field<App::PropertyInteger>(object,"OM9SourceArchiveStorageVersion").getValue()!=1)
        throw std::runtime_error("Unsupported source archive/storage version");
    const std::string digest=field<App::PropertyString>(object,"OM9SourceArchiveDigest").getValue();
    if(digest!=field<App::PropertyString>(object,"OM9ArchiveHash").getValue())throw std::runtime_error("Source archive hash differs from stored payload identity");
    const auto& chunks=field<App::PropertyStringList>(object,"OM9SourceArchiveChunks").getValues();
    if(chunks.size()>8192)throw std::runtime_error("Source archive chunk limit exceeded");
    std::vector<Om9LayerByteView> views;views.reserve(chunks.size());for(const auto& chunk:chunks)views.push_back(bytes(chunk));
    Payload payload;status(om9_layer_source_payload_prepare(views.data(),views.size(),bytes(digest),&payload.handle));
    const std::string manifest=field<App::PropertyString>(object,"OM9ArchiveManifest").getValue();
    if(manifest.size()>32ULL*1024*1024)throw std::runtime_error("Source archive manifest exceeds limit");
    QJsonParseError error;auto stored=QJsonDocument::fromJson(QByteArray::fromStdString(manifest),&error);
    if(error.error!=QJsonParseError::NoError || !stored.isObject() || !stored.object().value("scale_mm").isDouble())throw std::runtime_error("Invalid source archive manifest");
    const double scale=stored.object().value("scale_mm").toDouble();
    if(!std::isfinite(scale) || scale<=0)throw std::runtime_error("Invalid source archive native scale");
    std::size_t size=0;const auto query=om9_layer_source_payload_bytes(payload.handle,nullptr,0,&size);if(query!=15)status(query);
    QTemporaryDir directory;if(!directory.isValid())throw std::runtime_error("Cannot create owned source proof staging");
    const auto path=directory.filePath("verified-source.3dm");QFile file(path);
    if(!file.open(QIODevice::WriteOnly))throw std::runtime_error("Cannot write owned source proof staging");
    std::vector<unsigned char> buffer(64*1024);
    for(std::size_t offset=0;offset<size;) {
        const auto count=std::min(buffer.size(),size-offset);std::size_t written=0;
        status(om9_layer_source_payload_range(payload.handle,offset,buffer.data(),count,&written));
        if(written!=count || file.write(reinterpret_cast<const char*>(buffer.data()),qint64(count))!=qint64(count))throw std::runtime_error("Incomplete source proof staging write");
        offset+=count;
    }
    file.close();
    auto native=ThreeDm::inspectArchive(std::filesystem::u8path(path.toUtf8().constData()),scale);
    const auto inventory=ThreeDm::inventoryJson(native);
    const auto verified=om9_layer_source_manifest_verify(payload.handle,bytes(manifest),bytes(inventory));
    if(verified==1) {
        // Read the SAME verified temporary bytes independently. Never supply a
        // stored manifest as a native witness. V5 missing/nil DimStyle IDs can
        // be generated by the SDK per read; Rust permits only unreferenced,
        // uniquely matched unstable style IDs while keeping all else exact.
        auto witness=ThreeDm::inspectArchive(std::filesystem::u8path(path.toUtf8().constData()),scale);
        const auto second=ThreeDm::inventoryJson(witness);
        status(om9_layer_source_manifest_verify_witness(payload.handle,bytes(manifest),bytes(inventory),bytes(second)));
    } else {status(verified);}
    // Return proved stored context so FCStd retains its stable context labels.
    // An accepted ephemeral style UUID cannot bind any model/dependency.
    return stored.object();
}
}
