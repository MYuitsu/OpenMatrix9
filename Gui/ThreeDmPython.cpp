#include <Python.h>
#ifdef _WIN64
#undef WIN32
#endif
#include "ThreeDmArchive.h"
#include "LayerDocumentAdapter.h"
#include "LayerCollectionAdapter.h"
#include "LayerSourceArchiveAdapter.h"
#include "ThreeDmModeling.h"
#include "ThreeDmThreadPool.h"
#include "ThreeDmStaging.h"
#include "ThreeDmInventory.h"
#include "ThreeDmClassRegistry.h"
#include "ThreeDmMerge.h"
#include "ThreeDmPointCloud.h"
#include "ThreeDmHatch.h"
#include "ThreeDmHatchDialog.h"
#include "RustBridge.h"
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <Standard_Failure.hxx>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QFile>
#include <QCryptographicHash>
#include <cmath>
#include <memory>
#include <chrono>
#include <cstdlib>
#include <TopoDS_Compound.hxx>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/DocumentObjectPy.h>
#include <Gui/Selection/Selection.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
using namespace OpenMatrix9Gui::ThreeDm;
void AddLayerClipboardMethods(PyObject* module);
static PyObject* inspect3dmRegistry(PyObject*,PyObject*){
    try{const auto json=QJsonDocument(linkedClassRegistry()).toJson(QJsonDocument::Compact);return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");}
    catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
void AddThreeDmRegistryMethods(PyObject* module){
    static PyMethodDef methods[]={{"inspect3dmRegistry",inspect3dmRegistry,METH_NOARGS,"Read actual linked openNURBS class registry; does not imply exchange support."},{nullptr,nullptr,0,nullptr}};
    PyModule_AddFunctions(module,methods);
}
static PyObject* inspect3dm(PyObject*,PyObject* args){const char* path;double scale=0;if(!PyArg_ParseTuple(args,"s|d",&path,&scale))return nullptr;
    try{auto json=inventoryJson(inspectArchive(std::filesystem::u8path(path),scale));return PyUnicode_DecodeUTF8(json.data(),json.size(),"strict");}
    catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
static PyObject* archiveLegacyExportAllowed(PyObject*,PyObject* args){unsigned mode;if(!PyArg_ParseTuple(args,"I",&mode))return nullptr;return PyBool_FromLong(om9_3dm_archive_legacy_export_allowed(mode));}
static PyObject* prepare3dmArchive(PyObject*,PyObject* args){const char *path,*dir;double scale;unsigned mode;
    if(!PyArg_ParseTuple(args,"ssdI",&path,&dir,&scale,&mode))return nullptr;
    try{
        QJsonObject timings;auto tick=std::chrono::steady_clock::now();
        auto mark=[&](const char* stage){auto now=std::chrono::steady_clock::now();timings[stage]=timings[stage].toDouble()+std::chrono::duration<double>(now-tick).count();tick=now;};
        if(!om9_3dm_archive_mode_valid(mode))throw ExchangeError("Invalid 3DM archive mode");
        auto input=std::filesystem::u8path(path),staging=std::filesystem::u8path(dir);
        auto snapshot=staging/"source.3dm";
        if(std::filesystem::exists(snapshot))throw ExchangeError("Snapshot destination already exists");
        auto manifest=inspectArchive(input,scale).document;
        mark("source_inventory");
        if(!manifest["issues"].toArray().isEmpty())throw ExchangeError("3DM archive has unresolved dependencies");
        std::filesystem::copy_file(input,snapshot);
        try{
            auto snapshotInventory=inspectArchive(snapshot,scale);auto snapshotManifest=snapshotInventory.document;
            if(snapshotManifest["archive_sha256"]!=manifest["archive_sha256"])throw ExchangeError("Source archive changed during preparation");
            mark("snapshot_inventory");
            QJsonArray geometry,retained;int i=0;
            auto rows=mode==1?stagePreservedArchive(snapshot,staging,scale,false,snapshotInventory.nativeModel.get()):QJsonArray{};
            if(mode!=1){auto model=readArchive(snapshot,scale);for(auto& item:model.items){auto row=encodeExchangeItem(item,staging,i++);row["tolerance"]=model.tolerance;rows.append(row);}}
            for(auto value:rows){auto row=value.toObject();if(row["capability"]=="retained")retained.append(row);else geometry.append(row);}
            mark("top_level_conversion_and_staging");
            QJsonArray definitionGeometry,definitionRetained;
            if(mode==1)for(auto value:stagePreservedArchive(snapshot,staging,scale,true,snapshotInventory.nativeModel.get())){auto row=value.toObject();if(row["capability"]=="retained")definitionRetained.append(row);else definitionGeometry.append(row);}
            mark("definition_conversion_and_staging");
            // Affine previews have their own staging namespace.
            auto previewStaging=staging/"affine";std::filesystem::create_directory(previewStaging);
            QJsonObject pointCloudData;
            if(mode==1)for(auto value:manifest["records"].toArray()){auto record=value.toObject();if(record["class_name"]!="ON_PointCloud")continue;auto uuid=record["source_uuid"].toString();auto component=ON_ModelGeometryComponent::Cast(snapshotInventory.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData())).ModelComponent());auto cloud=component?ON_PointCloud::Cast(component->Geometry(nullptr)):nullptr;if(!cloud)throw ExchangeError("Missing native PointCloud source payload");auto current=*cloud;double mm=manifest["scale_mm"].toDouble();if(mm!=1)transformNativeGeometry(current,ON_Xform::DiagonalTransformation(mm));auto fields=staging/(uuid.toStdWString()+L".point-cloud.json");auto digest=writePointCloudFields(current,fields);pointCloudData[uuid]=QJsonObject{{"file",QString::fromStdWString(fields.wstring())},{"sha256",digest}};}
            QJsonArray previews;
            if(mode==1)for(auto value:manifest["records"].toArray()){auto record=value.toObject();if(record["class_name"]!="ON_InstanceRef")continue;auto matrix=record["instance_matrix"].toArray();bool rigid=true;for(int a=0;a<3;++a)for(int b=0;b<3;++b){double dot=0;for(int k=0;k<3;++k)dot+=matrix[a*4+k].toDouble()*matrix[b*4+k].toDouble();if(std::abs(dot-(a==b?1:0))>1e-9)rigid=false;}ON_Xform transform;for(int k=0;k<16;++k)transform[k/4][k%4]=matrix[k].toDouble();if(rigid&&std::abs(transform.Determinant()-1)<1e-9)continue;
                QJsonObject preview{{"source_uuid",record["source_uuid"]}};QJsonArray items;
                try{auto expanded=readArchive(snapshot,scale,false,false,record["source_uuid"].toString().toStdString());mark("affine_conversion");for(auto& item:expanded.items)items.append(encodeExchangeItem(item,previewStaging,i++));preview["items"]=items;mark("affine_staging");}
                catch(const ExchangeError& error){preview["unavailable_reason"]=error.what();}
                previews.append(preview);
            }
            auto payload=QJsonObject{{"manifest",manifest},{"snapshot",QString::fromStdWString(snapshot.wstring())},{"prepared_geometry",geometry},{"retained_records",retained},{"definition_geometry",definitionGeometry},{"definition_retained",definitionRetained},{"point_cloud_data",pointCloudData},{"affine_previews",previews},{"mode",static_cast<int>(mode)}};
            if(std::getenv("OM9_3DM_PROFILE"))payload["_performance_seconds"]=timings;
            auto json=QJsonDocument(payload).toJson(QJsonDocument::Compact);
            return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");
        }catch(...){std::filesystem::remove(snapshot);throw;}
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* modelingCapabilities3dm(PyObject*,PyObject* args){unsigned kind;if(!PyArg_ParseTuple(args,"I",&kind))return nullptr;
    auto json=QJsonDocument(modelingCapabilities(kind)).toJson(QJsonDocument::Compact);return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");
}
static PyObject* prepareModeling3dm(PyObject*,PyObject* args){const char *path,*dir,*metadata=nullptr;double scale=0;if(!PyArg_ParseTuple(args,"ss|ds",&path,&dir,&scale,&metadata))return nullptr;
    try{
        auto input=std::filesystem::u8path(path),staging=std::filesystem::u8path(dir);QByteArray json;
        {struct ReleaseGIL {PyThreadState* state=PyEval_SaveThread();~ReleaseGIL(){PyEval_RestoreThread(state);}} release;
        json=QJsonDocument(prepareModelingArchive(input,staging,scale,metadata?std::filesystem::u8path(metadata):std::filesystem::path{})).toJson(QJsonDocument::Compact);}
        return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");}
    catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* read3dm(PyObject*,PyObject* args){const char *path,*dir;double scale=0;
    if(!PyArg_ParseTuple(args,"ss|d",&path,&dir,&scale))return nullptr;
    try{auto model=readArchive(std::filesystem::u8path(path),scale);QJsonArray items;int i=0;
        for(auto& item:model.items)items.append(encodeExchangeItem(item,std::filesystem::u8path(dir),i++));
        auto json=QJsonDocument(QJsonObject{{"schema_version",2},{"layer_session",encodeExchangeLayers(model,"read-archive")},{"items",items},{"tolerance",model.tolerance}}).toJson(QJsonDocument::Compact);
        return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* measure3dmBrep(PyObject*,PyObject* args){const char* path;double precision=1e-10;if(!PyArg_ParseTuple(args,"s|d",&path,&precision))return nullptr;
    try{if(!std::isfinite(precision)||precision<1e-14||precision>1e-3)throw ExchangeError("BRep integration precision must be between1e-14 and1e-3");
        QFile input(QString::fromUtf8(path));if(!input.exists()||input.size()<=0||input.size()>512LL*1024*1024)throw ExchangeError("BRep measurement input is missing, empty or exceeds512MiB");
        TopoDS_Shape shape;BRep_Builder builder;if(!BRepTools::Read(shape,path,builder)||shape.IsNull()||!BRepCheck_Analyzer(shape).IsValid())throw ExchangeError("Cannot measure invalid BRep");
        GProp_GProps volume,area;const double volumeError=BRepGProp::VolumeProperties(shape,volume,precision);const double areaError=BRepGProp::SurfaceProperties(shape,area,precision);
        return Py_BuildValue("{s:d,s:d,s:d,s:d}","volume",volume.Mass(),"area",area.Mass(),"volume_error",volumeError,"area_error",areaError);
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static ON_PointCloud stagedCloud(const QJsonObject& row){
    if(!row["point_cloud_fields"].isString()||!row["point_cloud_sha256"].isString())throw ExchangeError("PointCloud requires a current-field file and SHA256");
    ON_PointCloud cloud;applyPointCloudFields(cloud,std::filesystem::path(row["point_cloud_fields"].toString().toStdWString()),row["point_cloud_sha256"].toString());
    if(row.contains("point_cloud_transform")){
        if(!row["point_cloud_transform"].isArray()||row["point_cloud_transform"].toArray().size()!=16)throw ExchangeError("PointCloud transform requires16 doubles");
        auto values=row["point_cloud_transform"].toArray();ON_Xform transform;
        for(int i=0;i<16;++i){if(!values[i].isDouble()||!std::isfinite(values[i].toDouble()))throw ExchangeError("Invalid PointCloud transform numeric value");transform[i/4][i%4]=values[i].toDouble();}
        transformNativeGeometry(cloud,transform);
    }return cloud;
}
static PyObject* transform3dmPointCloud(PyObject*,PyObject* args){const char *json,*dir;
    if(!PyArg_ParseTuple(args,"ss",&json,&dir))return nullptr;
    try{QJsonParseError error;auto document=QJsonDocument::fromJson(QByteArray(json),&error);if(error.error!=QJsonParseError::NoError||!document.isObject())throw ExchangeError("Invalid PointCloud preview request");ExchangeItem item;item.geometry=stagedCloud(document.object());auto result=QJsonDocument(encodeExchangeItem(item,std::filesystem::u8path(dir),0)).toJson(QJsonDocument::Compact);return PyUnicode_DecodeUTF8(result.constData(),result.size(),"strict");
    }catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
static PyObject* editHatchLoops3dm(PyObject*,PyObject* args){const char* json;const char* message="";
    if(!PyArg_ParseTuple(args,"s|s",&json,&message))return nullptr;
    try{
        if(std::strlen(json)>32ULL*1024*1024||std::strlen(message)>65536)throw ExchangeError("Hatch editor input exceeds its bounded data/message size");
        QJsonParseError error;auto document=QJsonDocument::fromJson(QByteArray(json),&error);if(error.error!=QJsonParseError::NoError||!document.isObject())throw ExchangeError("Invalid Hatch loop editor data");auto value=document.object();
        if(!editHatchLoops(value,QString::fromUtf8(message)))Py_RETURN_NONE;auto raw=QJsonDocument(value).toJson(QJsonDocument::Compact);return PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict");
    }catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
static PyObject* hatchBoundary3dm(PyObject*,PyObject* args){const char *json,*dir;
    if(!PyArg_ParseTuple(args,"ss",&json,&dir))return nullptr;
    try{
        if(std::strlen(json)>32ULL*1024*1024)throw ExchangeError("Hatch boundary request exceeds32MiB");
        QJsonParseError error;auto document=QJsonDocument::fromJson(QByteArray(json),&error);if(error.error!=QJsonParseError::NoError||!document.isObject())throw ExchangeError("Invalid Hatch boundary request");auto row=document.object();
        const QStringList keys{"snapshot","archive_sha256","scale_mm","source_uuid","hatch_fields","geometry_matrix"};for(auto key:row.keys())if(!keys.contains(key))throw ExchangeError("Unknown Hatch boundary request field");
        if(!row["snapshot"].isString()||!row["archive_sha256"].isString()||!row["source_uuid"].isString()||!row["scale_mm"].isDouble()||!std::isfinite(row["scale_mm"].toDouble())||row["scale_mm"].toDouble()<=0)throw ExchangeError("Incomplete Hatch boundary source identity");
        auto source=inspectArchive(std::filesystem::path(row["snapshot"].toString().toStdWString()),row["scale_mm"].toDouble());if(source.document["archive_sha256"]!=row["archive_sha256"])throw ExchangeError("Hatch boundary source hash differs");
        auto component=ON_ModelGeometryComponent::Cast(source.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData())).ModelComponent());auto original=component?ON_Hatch::Cast(component->Geometry(nullptr)):nullptr;if(!original)throw ExchangeError("Hatch boundary requires verified native ON_Hatch");
        auto current=*original;const double scale=source.document["scale_mm"].toDouble();if(scale!=1)transformHatchNative(current,ON_Xform::DiagonalTransformation(scale),source.nativeModel.get());applyHatchFields(current,row["hatch_fields"],*source.nativeModel);
        if(row.contains("geometry_matrix")){auto values=row["geometry_matrix"].toArray();if(!row["geometry_matrix"].isArray()||values.size()!=16)throw ExchangeError("Hatch boundary matrix requires16 doubles");ON_Xform transform;for(int i=0;i<16;++i){if(!values[i].isDouble()||!std::isfinite(values[i].toDouble()))throw ExchangeError("Invalid Hatch boundary matrix");transform[i/4][i%4]=values[i].toDouble();}transformNativeGeometry(current,transform,source.nativeModel.get());}
        BRep_Builder builder;TopoDS_Compound shape;builder.MakeCompound(shape);
        for(int i=0;i<current.LoopCount();++i){std::unique_ptr<ON_Curve> curve(current.LoopCurve3d(i));if(!curve||!curve->IsValid())throw ExchangeError("Invalid native Hatch world boundary curve");builder.Add(shape,importCurve(*curve,1e-6));}
        if(!BRepCheck_Analyzer(shape).IsValid())throw ExchangeError("Invalid derived Hatch BRep boundary");ExchangeItem item;item.geometry=TopoDS_Shape(shape);auto result=QJsonDocument(encodeExchangeItem(item,std::filesystem::u8path(dir),0)).toJson(QJsonDocument::Compact);return PyUnicode_DecodeUTF8(result.constData(),result.size(),"strict");
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* write3dm(PyObject*,PyObject* args){const char *json,*path;
    if(!PyArg_ParseTuple(args,"ss",&json,&path))return nullptr;
    try{{
        auto request=QByteArray(json);auto output=std::filesystem::u8path(path);
        struct ReleaseGIL {PyThreadState* state=PyEval_SaveThread();~ReleaseGIL(){PyEval_RestoreThread(state);}} release;
        std::lock_guard sdkLock(sdkArchiveMutex());
        QJsonParseError error;auto doc=QJsonDocument::fromJson(request,&error);if(error.error!=QJsonParseError::NoError||!doc.isObject())throw ExchangeError("Invalid export manifest");
        const auto envelope=doc.object();const bool canonical=envelope.contains("layer_session")||envelope["schema_version"].toDouble(-1)==2;
        if(canonical&&(envelope["schema_version"].toDouble(-1)!=2||!envelope["layer_session"].isObject()||!envelope["items"].isArray()))throw ExchangeError("Canonical export requires schema2, layer_session and an items array");
        if(envelope.contains("schema_version")&&envelope["schema_version"].toDouble(-1)!=1&&envelope["schema_version"].toDouble(-1)!=2)throw ExchangeError("Unsupported export manifest schema version");
        ExchangeModel model;model.tolerance=doc.object()["tolerance"].toDouble(1e-6);if(!std::isfinite(model.tolerance)||model.tolerance<=0)throw ExchangeError("Invalid export tolerance");
        quint64 cloudBytes=0;for(auto value:doc.object()["items"].toArray()){auto row=value.toObject();if(row.contains("point_cloud_fields")){QFile file(row["point_cloud_fields"].toString());if(!file.exists()||file.size()<=0)throw ExchangeError("Missing PointCloud current-field file");cloudBytes+=static_cast<quint64>(file.size());if(cloudBytes>512ULL*1024*1024)throw ExchangeError("Combined PointCloud current fields exceed512MiB");}}
        for(auto value:doc.object()["items"].toArray()){auto o=value.toObject();ExchangeItem item;
            item.name=o["name"].toString().toStdString();item.layer=o["layer"].toString().toStdString();item.visible=o["visible"].toBool(true);item.locked=o["locked"].toBool();
            if(canonical){
                if(!o["layer_object_id"].isString())throw ExchangeError("Canonical export geometry is missing its metadata object ID");
                item.ownState=NativeObjectLayerRow{};item.ownState->id=o["layer_object_id"].toString().toStdString();
            }
            if(o.contains("color")){auto color=o["color"].toArray();if(!o["color"].isArray()||color.size()!=3)throw ExchangeError("Export color requires3 integer RGB bytes");for(int i=0;i<3;++i){auto value=color[i];double channel=value.toDouble(-1);if(!value.isDouble()||!std::isfinite(channel)||channel<0||channel>255||channel!=std::floor(channel))throw ExchangeError("Invalid export RGB byte");item.color[i]=static_cast<int>(channel);}}
            if(o.contains("point_cloud_fields")||o.contains("point_cloud_sha256")||o.contains("point_cloud_transform")){
                if(o.contains("brep")||o.contains("vertices")||o.contains("faces"))throw ExchangeError("Ambiguous PointCloud geometry manifest");item.geometry=stagedCloud(o);
            }else if(o.contains("brep")){TopoDS_Shape shape;BRep_Builder builder;auto file=o["brep"].toString().toStdString();if(!BRepTools::Read(shape,file.c_str(),builder)||shape.IsNull())throw ExchangeError("Cannot read selected shape");item.geometry=shape;}
            else{MeshData mesh;for(auto v:o["vertices"].toArray()){auto a=v.toArray();mesh.vertices.push_back({a[0].toDouble(),a[1].toDouble(),a[2].toDouble()});}for(auto v:o["faces"].toArray()){auto a=v.toArray();mesh.faces.push_back({a[0].toInt(),a[1].toInt(),a[2].toInt(),a[3].toInt()});}item.geometry=mesh;}
            model.items.push_back(std::move(item));
        }if(canonical)decodeExchangeLayers(model,envelope["layer_session"].toObject());
        writeArchive5(model,output);}Py_RETURN_NONE;
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* writePreserved3dm(PyObject*,PyObject* args){PyObject* request;const char* path;if(!PyArg_ParseTuple(args,"Os",&request,&path))return nullptr;
    Py_ssize_t length=0;const char* json=PyUnicode_AsUTF8AndSize(request,&length);if(!json)return nullptr;
    try{if(length>32LL*1024*1024)throw ExchangeError("Preservation request exceeds32MiB");QJsonParseError error;auto document=QJsonDocument::fromJson(QByteArray(json,length),&error);if(error.error!=QJsonParseError::NoError||!document.isObject())throw ExchangeError("Invalid preservation export request");writePreservedArchive(document.object(),std::filesystem::u8path(path));Py_RETURN_NONE;
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* writeGeometryStaging3dm(PyObject*,PyObject* args){PyObject* request;const char* path;if(!PyArg_ParseTuple(args,"Os",&request,&path))return nullptr;
    Py_ssize_t length=0;const char* json=PyUnicode_AsUTF8AndSize(request,&length);if(!json)return nullptr;
    try{if(length>32LL*1024*1024)throw ExchangeError("Preservation request exceeds32MiB");QJsonParseError error;auto document=QJsonDocument::fromJson(QByteArray(json,length),&error);if(error.error!=QJsonParseError::NoError||!document.isObject())throw ExchangeError("Invalid preservation export request");writeGeometryStagingArchive(document.object(),std::filesystem::u8path(path));Py_RETURN_NONE;
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
namespace {
struct LayerPyRef {PyObject* value=nullptr;explicit LayerPyRef(PyObject* p):value(p){}~LayerPyRef(){Py_XDECREF(value);}LayerPyRef(const LayerPyRef&)=delete;};
struct ReceivePythonError {
    PyObject *type=nullptr,*value=nullptr,*trace=nullptr;
    ReceivePythonError(){PyErr_Fetch(&type,&value,&trace);}
    ~ReceivePythonError(){PyErr_Restore(type,value,trace);}
    ReceivePythonError(const ReceivePythonError&)=delete;
};
std::string layerPyText(PyObject* value) {
    if(!value || !PyUnicode_Check(value))throw ExchangeError("Layer binding must be UTF-8 text");
    Py_ssize_t size=0;const char* text=PyUnicode_AsUTF8AndSize(value,&size);
    if(!text || size<0)throw ExchangeError("Cannot read layer binding text");
    if(om9_layer_clipboard_check_size(2,static_cast<std::size_t>(size)))throw ExchangeError("Layer binding text exceeds metadata budget");
    return {text,static_cast<std::size_t>(size)};
}
std::vector<std::string> layerPyIds(PyObject* sequence,std::size_t count,bool pairs) {
    if(!sequence || (!PyList_Check(sequence) && !PyTuple_Check(sequence)) || PySequence_Size(sequence)!=static_cast<Py_ssize_t>(count))
        throw ExchangeError("Prepared geometry does not match canonical layer object count");
    std::vector<std::string> ids;ids.reserve(count);std::size_t total=0;
    for(std::size_t i=0;i<count;++i) {
        LayerPyRef entry(PySequence_GetItem(sequence,static_cast<Py_ssize_t>(i)));
        PyObject* value=entry.value;
        LayerPyRef row(pairs && value && PyTuple_Check(value) && PyTuple_Size(value)==2 ? PySequence_GetItem(value,0):nullptr);
        if(pairs) {
            if(!row.value || !PyDict_Check(row.value))throw ExchangeError("Invalid prepared geometry binding");
            value=PyDict_GetItemString(row.value,"layer_object_id");
        }
        auto id=layerPyText(value);
        if(id.size()>SIZE_MAX-total)throw ExchangeError("Layer binding size overflow");
        total+=id.size();if(om9_layer_clipboard_check_size(2,total))throw ExchangeError("Layer bindings exceed metadata budget");
        ids.push_back(std::move(id));
    }
    return ids;
}
void validateLayerBindings(std::uint64_t source,const std::vector<std::string>& ids) {
    Om9LayerCounts counts{};auto status=om9_layer_snapshot_counts(source,&counts);
    if(status || counts.object_count!=ids.size())throw ExchangeError("Canonical source and prepared geometry bindings disagree");
    std::vector<Om9LayerByteView> views;views.reserve(ids.size());for(const auto& id:ids)views.push_back({reinterpret_cast<const unsigned char*>(id.data()),id.size()});
    std::uint64_t checked=0;status=om9_layer_snapshot_subset(source,views.data(),views.size(),&checked);
    NativeLayerSnapshot subset(checked);
    if(status)throw ExchangeError("Layer geometry binding rejected: Rust code "+std::to_string(status));
}
std::string layerPyJson(PyObject* value) {
    LayerPyRef module(PyImport_ImportModule("json"));
    LayerPyRef encoded(module.value?PyObject_CallMethod(module.value,"dumps","O",value):nullptr);
    return layerPyText(encoded.value);
}
std::vector<std::string> receivedNativeNames(App::Document& doc,PyObject* result,std::size_t count) {
    if(!result || (!PyList_Check(result) && !PyTuple_Check(result)) || PySequence_Size(result)!=static_cast<Py_ssize_t>(count))
        throw ExchangeError("Native geometry receiver returned an incomplete binding");
    std::vector<std::string> names;names.reserve(count);
    for(std::size_t i=0;i<count;++i) {
        LayerPyRef item(PySequence_GetItem(result,static_cast<Py_ssize_t>(i)));
        if(!item.value || !PyObject_TypeCheck(item.value,&App::DocumentObjectPy::Type))throw ExchangeError("Receiver returned a non-native geometry object");
        auto* object=static_cast<App::DocumentObjectPy*>(item.value)->getDocumentObjectPtr();
        if(!object || object->getDocument()!=&doc || !object->getNameInDocument())throw ExchangeError("Receiver returned geometry from another document");
        names.emplace_back(object->getNameInDocument());
    }
    return names;
}
std::vector<std::string> selectedNativeNames(App::Document& doc,PyObject* selection) {
    if(!selection || (!PyList_Check(selection) && !PyTuple_Check(selection)))throw ExchangeError("Native export selection must be a list or tuple");
    const auto count=PyList_Check(selection)?PyList_Size(selection):PyTuple_Size(selection);
    if(count<0 || static_cast<std::size_t>(count)>doc.countObjects())throw ExchangeError("Native export selection exceeds document inventory");
    std::vector<std::string> names;names.reserve(static_cast<std::size_t>(count));
    for(Py_ssize_t i=0;i<count;++i) {
        // Direct native list/tuple access cannot invoke a user __getitem__
        // callback that closes the document while facts are being extracted.
        auto* item=PyList_Check(selection)?PyList_GetItem(selection,i):PyTuple_GetItem(selection,i);
        if(!item || !PyObject_TypeCheck(item,&App::DocumentObjectPy::Type))throw ExchangeError("Export selection contains a non-native object");
        auto* object=static_cast<App::DocumentObjectPy*>(item)->getDocumentObjectPtr();
        if(!object || object->getDocument()!=&doc || !object->getNameInDocument())throw ExchangeError("Export selection belongs to another document");
        names.emplace_back(object->getNameInDocument());
    }
    return names;
}
using LayerSelection=std::vector<std::pair<std::string,std::string>>;
void restoreReceiveSelection(App::Document& doc,const LayerSelection& previous) {
    Gui::Selection().clearSelection(doc.getName());
    for(const auto& [name,sub]:previous)if(doc.getObject(name.c_str()))Gui::Selection().addSelection(doc.getName(),name.c_str(),sub.c_str());
}
}
static PyObject* exportLayerSelection3dm(PyObject*,PyObject* args) {
    const char* name=nullptr;PyObject* selection=nullptr;
    if(!PyArg_ParseTuple(args,"sO",&name,&selection))return nullptr;
    try {
        auto* doc=App::GetApplication().getDocument(name);if(!doc)throw ExchangeError("Export document not found");
        auto value=OpenMatrix9Gui::layerDocumentExportSelection(*doc,selectedNativeNames(*doc,selection));
        QJsonParseError error;auto layer=QJsonDocument::fromJson(QByteArray::fromStdString(value.snapshotJson),&error);
        if(error.error!=QJsonParseError::NoError || !layer.isObject())throw ExchangeError("Cannot carry canonical export metadata");
        QJsonArray ids;for(const auto& id:value.objectIds)ids.append(QString::fromStdString(id));
        const auto result=QJsonDocument(QJsonObject{{"schema_version",2},{"layer_session",layer.object()},{"object_ids",ids}}).toJson(QJsonDocument::Compact);
        return PyUnicode_DecodeUTF8(result.constData(),result.size(),"strict");
    }catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
static PyObject* collectLayerTransfer3dm(PyObject*,PyObject* args) {
    const char* name=nullptr;PyObject* selection=nullptr;unsigned int scope=0;PyObject* result=nullptr;
    if(!PyArg_ParseTuple(args,"sOI",&name,&selection,&scope))return nullptr;
    try {
        auto* doc=App::GetApplication().getDocument(name);if(!doc)throw ExchangeError("Collection document not found");
        const auto names=OpenMatrix9Gui::collectLayerTransferObjects(*doc,selectedNativeNames(*doc,selection),scope);
        result=PyList_New(Py_ssize_t(names.size()));if(!result)return nullptr;
        for(std::size_t i=0;i<names.size();++i) {
            auto* object=doc->getObject(names[i].c_str());if(!object || object->getDocument()!=doc)throw ExchangeError("Collection native binding changed");
            auto* value=object->getPyObject();if(!value)throw ExchangeError("Cannot bind collected native object");
            PyList_SET_ITEM(result,Py_ssize_t(i),value);
        }
        return result;
    }catch(const std::exception& error){Py_XDECREF(result);PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
static PyObject* verifySourceArchive3dm(PyObject*,PyObject* args) {
    const char* name=nullptr;PyObject* owner=nullptr;
    if(!PyArg_ParseTuple(args,"sO",&name,&owner))return nullptr;
    try {
        auto* doc=App::GetApplication().getDocument(name);if(!doc)throw ExchangeError("Source archive document not found");
        if(!owner || !PyObject_TypeCheck(owner,&App::DocumentObjectPy::Type))throw ExchangeError("Source archive requires an actual native owner");
        auto* object=static_cast<App::DocumentObjectPy*>(owner)->getDocumentObjectPtr();
        if(!object || object->getDocument()!=doc)throw ExchangeError("Source archive owner belongs to a foreign document");
        auto inventory=OpenMatrix9Gui::verifyLayerSourceArchive(*doc,*object);
        const auto json=QJsonDocument(inventory).toJson(QJsonDocument::Compact);
        return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");
    }catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
static PyObject* validateLayerExport3dm(PyObject*,PyObject* args) {
    const char *name=nullptr,*json=nullptr;PyObject* selection=nullptr;
    if(!PyArg_ParseTuple(args,"sOs",&name,&selection,&json))return nullptr;
    try {
        auto* doc=App::GetApplication().getDocument(name);if(!doc)throw ExchangeError("Export document changed or closed");
        OpenMatrix9Gui::validateLayerDocumentExportSelection(*doc,selectedNativeNames(*doc,selection),json);Py_RETURN_NONE;
    }catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
static PyObject* validateLayerBindings3dm(PyObject*,PyObject* args) {
    const char* json=nullptr;PyObject* sequence=nullptr;
    if(!PyArg_ParseTuple(args,"sO",&json,&sequence))return nullptr;
    try {
        auto source=nativeLayerSnapshotFromJson(json);Om9LayerCounts counts{};
        auto status=om9_layer_snapshot_counts(source.get(),&counts);if(status)throw ExchangeError("Invalid canonical layer source");
        auto ids=layerPyIds(sequence,counts.object_count,false);validateLayerBindings(source.get(),ids);Py_RETURN_NONE;
    }catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
static PyObject* commit3dm(PyObject*,PyObject* args){const char* name;PyObject* prepared;if(!PyArg_ParseTuple(args,"sO",&name,&prepared))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);auto* gui=Gui::Application::Instance->activeDocument();
    if(!doc||doc!=App::GetApplication().getActiveDocument()||!gui||gui->getInEdit()||!om9AlterDocument(doc)){PyErr_SetString(PyExc_RuntimeError,"The active project is not editable");return nullptr;}
    const std::string documentName=doc->getName(),documentIdentity=doc->Uid.getValueStr();
    auto alive=[&]{auto* current=App::GetApplication().getDocument(documentName.c_str());return current && current==doc && current->Uid.getValueStr()==documentIdentity;};
    auto requireTarget=[&]{
        if(!alive())throw ExchangeError("3DM receive target changed or closed during binding");
        auto* currentGui=Gui::Application::Instance->getDocument(doc);
        if(App::GetApplication().getActiveDocument()!=doc || !currentGui || currentGui->isAboutToClose() || currentGui->getInEdit() || om9ReadOnlyFile(*doc) || !om9AlterDocument(doc))
            throw ExchangeError("3DM receive target is no longer editable");
    };
    int transactionId=0;PyObject* result=nullptr;LayerSelection selection;
    try {
    OpenMatrix9Gui::requireLayerGeometryEditable(*doc);
    for(const auto& entry:Gui::Selection().getSelection(doc->getName()))selection.emplace_back(entry.FeatName,entry.SubName?entry.SubName:"");
    PyObject* geometry=prepared;std::vector<std::string> sourceIds;
    std::unique_ptr<OpenMatrix9Gui::LayerDocumentReceivePlan> layers;
    if(PyDict_Check(prepared) && !PyDict_GetItemString(prepared,"manifest")) {
        auto* layerSession=PyDict_GetItemString(prepared,"layer_session");geometry=PyDict_GetItemString(prepared,"host_geometry");
        if(!layerSession || !geometry)throw ExchangeError("Prepared receive packet is incomplete");
        const auto json=layerPyJson(layerSession);auto source=nativeLayerSnapshotFromJson(json);Om9LayerCounts counts{};
        auto status=om9_layer_snapshot_counts(source.get(),&counts);if(status)throw ExchangeError("Invalid canonical layer source");
        sourceIds=layerPyIds(geometry,counts.object_count,true);validateLayerBindings(source.get(),sourceIds);
        requireTarget();
        layers=std::make_unique<OpenMatrix9Gui::LayerDocumentReceivePlan>(*doc,json,sourceIds);
    }
    transactionId=om9OpenTransaction(*doc,std::string("Import Rhino 5 3DM"));
    if(!OpenMatrix9Gui::ownsLayerGeometryTransaction(*doc,transactionId))throw std::runtime_error("Cannot own 3DM import transaction");
    OpenMatrix9Gui::LayerNativeReceiveScope receiving(*doc);
    auto* module=PyImport_ImportModule("ThreeDm");
    result=module?(layers?PyObject_CallMethod(module,"_insert_prepared","sOO",name,geometry,Py_False):PyObject_CallMethod(module,"_insert_prepared","sO",name,prepared)):nullptr;Py_XDECREF(module);
    requireTarget();
    if(!result){
        // Rollback invokes Python document observers. Preserve the binding
        // exception while those callbacks run, then return its original type.
        ReceivePythonError error;
        if(alive() && OpenMatrix9Gui::ownsLayerGeometryTransaction(*doc,transactionId)){om9AbortTransaction(*doc);if(alive())restoreReceiveSelection(*doc,selection);}return nullptr;
    }
    if(layers)layers->finish(transactionId,receivedNativeNames(*doc,result,sourceIds.size()));
    requireTarget();
    if(!OpenMatrix9Gui::ownsLayerGeometryTransaction(*doc,transactionId))throw std::runtime_error("3DM import transaction ownership changed");
    om9CommitTransaction(*doc);return result;
    } catch(const std::exception& error) {
        Py_XDECREF(result);
        if(alive() && OpenMatrix9Gui::ownsLayerGeometryTransaction(*doc,transactionId)){try{om9AbortTransaction(*doc);if(alive())restoreReceiveSelection(*doc,selection);}catch(...){}}
        PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;
    }
}
static bool ready3dm(const char* name){auto* doc=App::GetApplication().getDocument(name);auto* gui=Gui::Application::Instance->activeDocument();if(doc&&doc==App::GetApplication().getActiveDocument()&&gui&&!gui->getInEdit()&&om9AlterDocument(doc))return true;PyErr_SetString(PyExc_RuntimeError,"The active project is not editable");return false;}
static PyObject* validateDocument(PyObject*,PyObject* args){const char* name;if(!PyArg_ParseTuple(args,"s",&name)||!ready3dm(name))return nullptr;Py_RETURN_NONE;}
static PyObject* import3dm(PyObject*,PyObject* args){const char *path,*name;double scale=0;if(!PyArg_ParseTuple(args,"ss|d",&path,&name,&scale)||!ready3dm(name))return nullptr;auto* module=PyImport_ImportModule("ThreeDm");auto* result=module?PyObject_CallMethod(module,"import_file","sOd",path,Py_None,scale):nullptr;Py_XDECREF(module);return result;}
static PyObject* export3dm(PyObject*,PyObject* args){const char *path,*name;PyObject* names;if(!PyArg_ParseTuple(args,"ssO",&path,&name,&names)||!ready3dm(name))return nullptr;auto* module=PyImport_ImportModule("ThreeDm");auto* result=module?PyObject_CallMethod(module,"export_named","ssO",path,name,names):nullptr;Py_XDECREF(module);return result;}
void AddThreeDmMethods(PyObject* module){AddLayerClipboardMethods(module);static PyMethodDef sourceMethods[]={{"verifySourceArchive3dm",verifySourceArchive3dm,METH_VARARGS,"Verify stored source payload and manifest against actual openNURBS inventory; does not establish helper/model roles."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,sourceMethods);static PyMethodDef methods[]={{"collectLayerTransfer3dm",collectLayerTransfer3dm,METH_VARARGS,"Collect native object scope in Rust without extracting geometry arrays; scope1 Selected, scope2 Session."},{"exportLayerSelection3dm",exportLayerSelection3dm,METH_VARARGS,"Read canonical full palette and exact native object bindings for current geometry export; no geometry extraction."},{"validateLayerExport3dm",validateLayerExport3dm,METH_VARARGS,"Reject a changed canonical layer snapshot after detached geometry staging."},{"validateLayerBindings3dm",validateLayerBindings3dm,METH_VARARGS,"Validate full canonical palette and exact geometry bindings in Rust before host construction."},{"modelingCapabilities3dm",modelingCapabilities3dm,METH_VARARGS,"Read shared Rust working-geometry policy."},{"prepareModeling3dm",prepareModeling3dm,METH_VARARGS,"Prepare independent working geometry without source archive dependencies."},{"editHatchLoops3dm",editHatchLoops3dm,METH_VARARGS,"Edit detached native Hatch loop values; the host validates before document commit."},{"hatchBoundary3dm",hatchBoundary3dm,METH_VARARGS,"Derive boundary BRep from verified current native Hatch fields."},{"transform3dmPointCloud",transform3dmPointCloud,METH_VARARGS,"Derive native affine PointCloud preview fields."},{"writePreserved3dm",writePreserved3dm,METH_VARARGS,"Write selected native archive records as Rhino 5."},{"writeGeometryStaging3dm",writeGeometryStaging3dm,METH_VARARGS,"Internal SDK80 current graph staging for geometry-only flattening; not Rhino5 output."},{"archiveLegacyExportAllowed",archiveLegacyExportAllowed,METH_VARARGS,"Rust preservation export policy."},{"inspect3dm",inspect3dm,METH_VARARGS,"Inventory all native source records."},{"prepare3dmArchive",prepare3dmArchive,METH_VARARGS,"Prepare geometry and immutable source snapshot."},{"measure3dmBrep",measure3dmBrep,METH_VARARGS,"Measure staged CAD with adaptive integration."},{"read3dm",read3dm,METH_VARARGS,"Read 3DM into a staged geometry manifest."},{"write3dm",write3dm,METH_VARARGS,"Write staged geometry as Rhino 5 3DM."},{"commit3dm",commit3dm,METH_VARARGS,"Insert prepared host objects in one native transaction."},{"validateDocument",validateDocument,METH_VARARGS,"Check the active project and task permissions."},{"import3dm",import3dm,METH_VARARGS,"Import 3DM into the active named document."},{"export3dm",export3dm,METH_VARARGS,"Export named whole objects as Rhino 5 3DM."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
