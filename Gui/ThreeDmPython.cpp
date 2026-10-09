#include <Python.h>
#ifdef _WIN64
#undef WIN32
#endif
#include "ThreeDmArchive.h"
#include "ThreeDmModeling.h"
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
#include <cstring>
#include <memory>
#include <chrono>
#include <cstdlib>
#include <TopoDS_Compound.hxx>
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
using namespace OpenMatrix9Gui::ThreeDm;
extern "C" bool om9_3dm_export_manifest_valid(const unsigned char*,std::size_t);
extern "C" bool om9_retained_dependencies_valid(const unsigned char*,std::size_t);
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
        const auto manifestBytes=QJsonDocument(manifest).toJson(QJsonDocument::Compact);
        if(!om9_retained_dependencies_valid(reinterpret_cast<const unsigned char*>(manifestBytes.constData()),manifestBytes.size()))throw ExchangeError("3DM archive has invalid, missing or cyclic source dependencies");
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
static PyObject* prepareModeling3dm(PyObject*,PyObject* args){const char *path,*dir;double scale=0;if(!PyArg_ParseTuple(args,"ss|d",&path,&dir,&scale))return nullptr;
    try{auto json=QJsonDocument(prepareModelingArchive(std::filesystem::u8path(path),std::filesystem::u8path(dir),scale)).toJson(QJsonDocument::Compact);return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");}
    catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* read3dm(PyObject*,PyObject* args){const char *path,*dir;double scale=0;
    if(!PyArg_ParseTuple(args,"ss|d",&path,&dir,&scale))return nullptr;
    try{auto model=readArchive(std::filesystem::u8path(path),scale);QJsonArray items;int i=0;
        for(auto& item:model.items)items.append(encodeExchangeItem(item,std::filesystem::u8path(dir),i++));
        auto json=QJsonDocument(QJsonObject{{"items",items},{"tolerance",model.tolerance}}).toJson(QJsonDocument::Compact);
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
    try{if(!om9_3dm_export_manifest_valid(reinterpret_cast<const unsigned char*>(json),std::strlen(json)))throw ExchangeError("Invalid geometry-only export manifest: check tuple sizes, mesh indices, types, tolerance and byte budget");
        QJsonParseError error;auto doc=QJsonDocument::fromJson(QByteArray(json),&error);if(error.error!=QJsonParseError::NoError||!doc.isObject())throw ExchangeError("Invalid export manifest");
        ExchangeModel model;model.tolerance=doc.object()["tolerance"].toDouble(1e-6);if(!std::isfinite(model.tolerance)||model.tolerance<=0)throw ExchangeError("Invalid export tolerance");
        quint64 cloudBytes=0;for(auto value:doc.object()["items"].toArray()){auto row=value.toObject();if(row.contains("point_cloud_fields")){QFile file(row["point_cloud_fields"].toString());if(!file.exists()||file.size()<=0)throw ExchangeError("Missing PointCloud current-field file");cloudBytes+=static_cast<quint64>(file.size());if(cloudBytes>512ULL*1024*1024)throw ExchangeError("Combined PointCloud current fields exceed512MiB");}}
        for(auto value:doc.object()["items"].toArray()){auto o=value.toObject();ExchangeItem item;
            item.name=o["name"].toString().toStdString();item.layer=o["layer"].toString().toStdString();item.visible=o["visible"].toBool(true);item.locked=o["locked"].toBool();
            if(o.contains("color")){auto color=o["color"].toArray();if(!o["color"].isArray()||color.size()!=3)throw ExchangeError("Export color requires3 integer RGB bytes");for(int i=0;i<3;++i){auto value=color[i];double channel=value.toDouble(-1);if(!value.isDouble()||!std::isfinite(channel)||channel<0||channel>255||channel!=std::floor(channel))throw ExchangeError("Invalid export RGB byte");item.color[i]=static_cast<int>(channel);}}
            if(o.contains("point_cloud_fields")||o.contains("point_cloud_sha256")||o.contains("point_cloud_transform")){
                if(o.contains("brep")||o.contains("vertices")||o.contains("faces"))throw ExchangeError("Ambiguous PointCloud geometry manifest");item.geometry=stagedCloud(o);
            }else if(o.contains("brep")){TopoDS_Shape shape;BRep_Builder builder;auto file=o["brep"].toString().toStdString();if(!BRepTools::Read(shape,file.c_str(),builder)||shape.IsNull())throw ExchangeError("Cannot read selected shape");item.geometry=shape;}
            else{MeshData mesh;for(auto v:o["vertices"].toArray()){auto a=v.toArray();mesh.vertices.push_back({a[0].toDouble(),a[1].toDouble(),a[2].toDouble()});}for(auto v:o["faces"].toArray()){auto a=v.toArray();mesh.faces.push_back({a[0].toInt(),a[1].toInt(),a[2].toInt(),a[3].toInt()});}item.geometry=mesh;}
            model.items.push_back(std::move(item));
        }writeArchive5(model,std::filesystem::u8path(path));Py_RETURN_NONE;
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
static PyObject* commit3dm(PyObject*,PyObject* args){const char* name;PyObject* prepared;if(!PyArg_ParseTuple(args,"sO",&name,&prepared))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);auto* gui=Gui::Application::Instance->activeDocument();
    if(!doc||doc!=App::GetApplication().getActiveDocument()||!gui||gui->getInEdit()||!Gui::Control().isAllowedAlterDocument(doc)){PyErr_SetString(PyExc_RuntimeError,"The active project is not editable");return nullptr;}
    doc->openTransaction(std::string("Import Rhino 5 3DM"));
    auto* module=PyImport_ImportModule("ThreeDm");auto* result=module?PyObject_CallMethod(module,"_insert_prepared","sO",name,prepared):nullptr;Py_XDECREF(module);
    if(!result){
        // Rollback invokes Python document observers. Preserve the binding
        // exception while those callbacks run, then return its original type.
        PyObject *type=nullptr,*value=nullptr,*trace=nullptr;PyErr_Fetch(&type,&value,&trace);
        doc->abortTransaction();PyErr_Restore(type,value,trace);return nullptr;
    }doc->commitTransaction();return result;
}
static bool ready3dm(const char* name){auto* doc=App::GetApplication().getDocument(name);auto* gui=Gui::Application::Instance->activeDocument();if(doc&&doc==App::GetApplication().getActiveDocument()&&gui&&!gui->getInEdit()&&Gui::Control().isAllowedAlterDocument(doc))return true;PyErr_SetString(PyExc_RuntimeError,"The active project is not editable");return false;}
static PyObject* validateDocument(PyObject*,PyObject* args){const char* name;if(!PyArg_ParseTuple(args,"s",&name)||!ready3dm(name))return nullptr;Py_RETURN_NONE;}
static PyObject* import3dm(PyObject*,PyObject* args){const char *path,*name;double scale=0;if(!PyArg_ParseTuple(args,"ss|d",&path,&name,&scale)||!ready3dm(name))return nullptr;auto* module=PyImport_ImportModule("ThreeDm");auto* result=module?PyObject_CallMethod(module,"import_file","sOd",path,Py_None,scale):nullptr;Py_XDECREF(module);return result;}
static PyObject* export3dm(PyObject*,PyObject* args){const char *path,*name;PyObject* names;if(!PyArg_ParseTuple(args,"ssO",&path,&name,&names)||!ready3dm(name))return nullptr;auto* module=PyImport_ImportModule("ThreeDm");auto* result=module?PyObject_CallMethod(module,"export_named","ssO",path,name,names):nullptr;Py_XDECREF(module);return result;}
void AddThreeDmMethods(PyObject* module){static PyMethodDef methods[]={{"modelingCapabilities3dm",modelingCapabilities3dm,METH_VARARGS,"Read shared Rust working-geometry policy."},{"prepareModeling3dm",prepareModeling3dm,METH_VARARGS,"Prepare independent working geometry without source archive dependencies."},{"editHatchLoops3dm",editHatchLoops3dm,METH_VARARGS,"Edit detached native Hatch loop values; the host validates before document commit."},{"hatchBoundary3dm",hatchBoundary3dm,METH_VARARGS,"Derive boundary BRep from verified current native Hatch fields."},{"transform3dmPointCloud",transform3dmPointCloud,METH_VARARGS,"Derive native affine PointCloud preview fields."},{"writePreserved3dm",writePreserved3dm,METH_VARARGS,"Write selected native archive records as Rhino 5."},{"writeGeometryStaging3dm",writeGeometryStaging3dm,METH_VARARGS,"Internal SDK80 current graph staging for geometry-only flattening; not Rhino5 output."},{"archiveLegacyExportAllowed",archiveLegacyExportAllowed,METH_VARARGS,"Rust preservation export policy."},{"inspect3dm",inspect3dm,METH_VARARGS,"Inventory all native source records."},{"prepare3dmArchive",prepare3dmArchive,METH_VARARGS,"Prepare geometry and immutable source snapshot."},{"measure3dmBrep",measure3dmBrep,METH_VARARGS,"Measure staged CAD with adaptive integration."},{"read3dm",read3dm,METH_VARARGS,"Read 3DM into a staged geometry manifest."},{"write3dm",write3dm,METH_VARARGS,"Write staged geometry as Rhino 5 3DM."},{"commit3dm",commit3dm,METH_VARARGS,"Insert prepared host objects in one native transaction."},{"validateDocument",validateDocument,METH_VARARGS,"Check the active project and task permissions."},{"import3dm",import3dm,METH_VARARGS,"Import 3DM into the active named document."},{"export3dm",export3dm,METH_VARARGS,"Export named whole objects as Rhino 5 3DM."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
