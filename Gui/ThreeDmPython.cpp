#include <Python.h>
#ifdef _WIN64
#undef WIN32
#endif
#include "ThreeDmArchive.h"
#include "ThreeDmInventory.h"
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
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
using namespace OpenMatrix9Gui::ThreeDm;
static QJsonObject encode(const ExchangeItem& item,const std::filesystem::path& dir,int index){
    QJsonObject o{{"name",QString::fromStdString(item.name)},{"layer",QString::fromStdString(item.layer)},{"visible",item.visible},{"locked",item.locked}};
    o["color"]=QJsonArray{item.color[0],item.color[1],item.color[2]};
    o["source_uuid"]=QString::fromStdString(item.sourceUuid);o["class_name"]=QString::fromStdString(item.sourceClass);
    if(item.retained){o["capability"]="retained";return o;}
    o["capability"]="editable";
    if(auto shape=std::get_if<TopoDS_Shape>(&item.geometry)){
        auto file=dir/(std::to_string(index)+".brep");
        if(!BRepTools::Write(*shape,file.string().c_str()))throw ExchangeError("Cannot stage imported shape");
        o["brep"]=QString::fromStdString(file.string());
    }else{auto& mesh=std::get<MeshData>(item.geometry);QJsonArray vertices,faces;
        for(auto p:mesh.vertices)vertices.append(QJsonArray{p[0],p[1],p[2]});
        for(auto f:mesh.faces)faces.append(QJsonArray{f[0],f[1],f[2],f[3]});
        o["vertices"]=vertices;o["faces"]=faces;
    }return o;
}
static PyObject* inspect3dm(PyObject*,PyObject* args){const char* path;double scale=0;if(!PyArg_ParseTuple(args,"s|d",&path,&scale))return nullptr;
    try{auto json=inventoryJson(inspectArchive(std::filesystem::u8path(path),scale));return PyUnicode_DecodeUTF8(json.data(),json.size(),"strict");}
    catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
static PyObject* archiveLegacyExportAllowed(PyObject*,PyObject* args){unsigned mode;if(!PyArg_ParseTuple(args,"I",&mode))return nullptr;return PyBool_FromLong(om9_3dm_archive_legacy_export_allowed(mode));}
static PyObject* prepare3dmArchive(PyObject*,PyObject* args){const char *path,*dir;double scale;unsigned mode;
    if(!PyArg_ParseTuple(args,"ssdI",&path,&dir,&scale,&mode))return nullptr;
    try{
        if(!om9_3dm_archive_mode_valid(mode))throw ExchangeError("Invalid 3DM archive mode");
        auto input=std::filesystem::u8path(path),staging=std::filesystem::u8path(dir);
        auto snapshot=staging/"source.3dm";
        if(std::filesystem::exists(snapshot))throw ExchangeError("Snapshot destination already exists");
        auto manifest=inspectArchive(input,scale).document;
        if(!manifest["issues"].toArray().isEmpty())throw ExchangeError("3DM archive has unresolved dependencies");
        std::filesystem::copy_file(input,snapshot);
        try{
            auto snapshotManifest=inspectArchive(snapshot,scale).document;
            if(snapshotManifest["archive_sha256"]!=manifest["archive_sha256"])throw ExchangeError("Source archive changed during preparation");
            auto model=readArchive(snapshot,scale,mode==1);QJsonArray geometry,retained;int i=0;
            for(auto& item:model.items){auto row=encode(item,staging,i++);row["tolerance"]=model.tolerance;if(item.retained)retained.append(row);else geometry.append(row);}
            auto json=QJsonDocument(QJsonObject{{"manifest",manifest},{"snapshot",QString::fromStdWString(snapshot.wstring())},{"prepared_geometry",geometry},{"retained_records",retained},{"mode",static_cast<int>(mode)}}).toJson(QJsonDocument::Compact);
            return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");
        }catch(...){std::filesystem::remove(snapshot);throw;}
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* read3dm(PyObject*,PyObject* args){const char *path,*dir;double scale=0;
    if(!PyArg_ParseTuple(args,"ss|d",&path,&dir,&scale))return nullptr;
    try{auto model=readArchive(std::filesystem::u8path(path),scale);QJsonArray items;int i=0;
        for(auto& item:model.items)items.append(encode(item,std::filesystem::u8path(dir),i++));
        auto json=QJsonDocument(QJsonObject{{"items",items},{"tolerance",model.tolerance}}).toJson(QJsonDocument::Compact);
        return PyUnicode_DecodeUTF8(json.constData(),json.size(),"strict");
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* measure3dmBrep(PyObject*,PyObject* args){const char* path;if(!PyArg_ParseTuple(args,"s",&path))return nullptr;
    try{TopoDS_Shape shape;BRep_Builder builder;if(!BRepTools::Read(shape,path,builder)||shape.IsNull()||!BRepCheck_Analyzer(shape).IsValid())throw ExchangeError("Cannot measure invalid BRep");
        GProp_GProps volume,area;const double volumeError=BRepGProp::VolumeProperties(shape,volume,1e-10);const double areaError=BRepGProp::SurfaceProperties(shape,area,1e-10);
        return Py_BuildValue("{s:d,s:d,s:d,s:d}","volume",volume.Mass(),"area",area.Mass(),"volume_error",volumeError,"area_error",areaError);
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* write3dm(PyObject*,PyObject* args){const char *json,*path;
    if(!PyArg_ParseTuple(args,"ss",&json,&path))return nullptr;
    try{QJsonParseError error;auto doc=QJsonDocument::fromJson(QByteArray(json),&error);if(error.error!=QJsonParseError::NoError||!doc.isObject())throw ExchangeError("Invalid export manifest");
        ExchangeModel model;model.tolerance=doc.object()["tolerance"].toDouble(1e-6);if(!std::isfinite(model.tolerance)||model.tolerance<=0)throw ExchangeError("Invalid export tolerance");for(auto value:doc.object()["items"].toArray()){auto o=value.toObject();ExchangeItem item;
            item.name=o["name"].toString().toStdString();item.layer=o["layer"].toString().toStdString();item.visible=o["visible"].toBool(true);item.locked=o["locked"].toBool();auto color=o["color"].toArray();for(int i=0;i<3;++i)item.color[i]=color[i].toInt();
            if(o.contains("brep")){TopoDS_Shape shape;BRep_Builder builder;auto file=o["brep"].toString().toStdString();if(!BRepTools::Read(shape,file.c_str(),builder)||shape.IsNull())throw ExchangeError("Cannot read selected shape");item.geometry=shape;}
            else{MeshData mesh;for(auto v:o["vertices"].toArray()){auto a=v.toArray();mesh.vertices.push_back({a[0].toDouble(),a[1].toDouble(),a[2].toDouble()});}for(auto v:o["faces"].toArray()){auto a=v.toArray();mesh.faces.push_back({a[0].toInt(),a[1].toInt(),a[2].toInt(),a[3].toInt()});}item.geometry=mesh;}
            model.items.push_back(std::move(item));
        }writeArchive5(model,std::filesystem::u8path(path));Py_RETURN_NONE;
    }catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
static PyObject* commit3dm(PyObject*,PyObject* args){const char* name;PyObject* prepared;if(!PyArg_ParseTuple(args,"sO",&name,&prepared))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);auto* gui=Gui::Application::Instance->activeDocument();
    if(!doc||doc!=App::GetApplication().getActiveDocument()||!gui||gui->getInEdit()||!Gui::Control().isAllowedAlterDocument(doc)){PyErr_SetString(PyExc_RuntimeError,"The active project is not editable");return nullptr;}
    doc->openTransaction(std::string("Import Rhino 5 3DM"));
    auto* module=PyImport_ImportModule("ThreeDm");auto* result=module?PyObject_CallMethod(module,"_insert_prepared","sO",name,prepared):nullptr;Py_XDECREF(module);
    if(!result){doc->abortTransaction();return nullptr;}doc->commitTransaction();return result;
}
static bool ready3dm(const char* name){auto* doc=App::GetApplication().getDocument(name);auto* gui=Gui::Application::Instance->activeDocument();if(doc&&doc==App::GetApplication().getActiveDocument()&&gui&&!gui->getInEdit()&&Gui::Control().isAllowedAlterDocument(doc))return true;PyErr_SetString(PyExc_RuntimeError,"The active project is not editable");return false;}
static PyObject* validateDocument(PyObject*,PyObject* args){const char* name;if(!PyArg_ParseTuple(args,"s",&name)||!ready3dm(name))return nullptr;Py_RETURN_NONE;}
static PyObject* import3dm(PyObject*,PyObject* args){const char *path,*name;double scale=0;if(!PyArg_ParseTuple(args,"ss|d",&path,&name,&scale)||!ready3dm(name))return nullptr;auto* module=PyImport_ImportModule("ThreeDm");auto* result=module?PyObject_CallMethod(module,"import_file","sOd",path,Py_None,scale):nullptr;Py_XDECREF(module);return result;}
static PyObject* export3dm(PyObject*,PyObject* args){const char *path,*name;PyObject* names;if(!PyArg_ParseTuple(args,"ssO",&path,&name,&names)||!ready3dm(name))return nullptr;auto* module=PyImport_ImportModule("ThreeDm");auto* result=module?PyObject_CallMethod(module,"export_named","ssO",path,name,names):nullptr;Py_XDECREF(module);return result;}
void AddThreeDmMethods(PyObject* module){static PyMethodDef methods[]={{"archiveLegacyExportAllowed",archiveLegacyExportAllowed,METH_VARARGS,"Rust preservation export policy."},{"inspect3dm",inspect3dm,METH_VARARGS,"Inventory all native source records."},{"prepare3dmArchive",prepare3dmArchive,METH_VARARGS,"Prepare geometry and immutable source snapshot."},{"measure3dmBrep",measure3dmBrep,METH_VARARGS,"Measure staged CAD with adaptive integration."},{"read3dm",read3dm,METH_VARARGS,"Read 3DM into a staged geometry manifest."},{"write3dm",write3dm,METH_VARARGS,"Write staged geometry as Rhino 5 3DM."},{"commit3dm",commit3dm,METH_VARARGS,"Insert prepared host objects in one native transaction."},{"validateDocument",validateDocument,METH_VARARGS,"Check the active project and task permissions."},{"import3dm",import3dm,METH_VARARGS,"Import 3DM into the active named document."},{"export3dm",export3dm,METH_VARARGS,"Export named whole objects as Rhino 5 3DM."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
