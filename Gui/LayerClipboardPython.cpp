// SPDX-License-Identifier: LGPL-2.1-or-later
#define PY_SSIZE_T_CLEAN
#include <Python.h>
#include "LayerClipboard.h"
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <memory>
#include <stdexcept>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {
PyObject* receipt(const Om9LayerClipboardInfo& info,const QJsonObject& extra={}){
    auto value=extra;value["version"]=static_cast<int>(info.version);value["evidence"]=static_cast<int>(info.evidence);
    value["scope"]=static_cast<int>(info.scope);value["geometry_version"]=static_cast<int>(info.geometry_version);
    value["geometry_bytes"]=static_cast<qint64>(info.geometry_length);value["metadata_bytes"]=static_cast<qint64>(info.metadata_length);
    const auto data=QJsonDocument(value).toJson(QJsonDocument::Compact);return PyUnicode_DecodeUTF8(data.constData(),data.size(),"strict");
}
// NewOnly prevents target replacement. Rollback removes this call's files only.
struct OwnedOutput {
    QFile file;bool created=false,committed=false;
    OwnedOutput(const char* path,const QByteArray& value):file(QString::fromUtf8(path)){
        if(!file.open(QIODevice::WriteOnly|QIODevice::NewOnly))throw std::runtime_error("Cannot create fresh clipboard staging file");
        created=true;
        if(file.write(value)!=value.size()||!file.flush()){
            file.close();file.remove();created=false;throw std::runtime_error("Cannot write clipboard staging file");
        }
        file.close();
    }
    ~OwnedOutput(){if(created&&!committed)file.remove();}
};
PyObject* publish(PyObject*,PyObject* args){
    const char* geometry=nullptr;Py_ssize_t length=0;PyObject* layerJson=nullptr;unsigned scope=0;
    if(!PyArg_ParseTuple(args,"y#OI",&geometry,&length,&layerJson,&scope))return nullptr;
    try{
        const auto status=om9_layer_clipboard_check_size(1,static_cast<std::size_t>(length));
        if(status)throw std::runtime_error("Clipboard geometry size rejected: Rust code "+std::to_string(status));
        NativeLayerSnapshot source;
        if(layerJson!=Py_None){
            Py_ssize_t jsonLength=0;const auto json=PyUnicode_AsUTF8AndSize(layerJson,&jsonLength);if(!json)return nullptr;
            const auto code=om9_layer_clipboard_check_size(2,static_cast<std::size_t>(jsonLength));
            if(code)throw std::runtime_error("Clipboard metadata size rejected: Rust code "+std::to_string(code));
            source=nativeLayerSnapshotFromJson(std::string(json,static_cast<std::size_t>(jsonLength)));
        }
        // QMimeData can outlive Python input. Its binary format is an owned Qt
        // byte array, not a fromRawData view into a released Python buffer.
        const QByteArray body(geometry,static_cast<qsizetype>(length));
        return receipt(publishLayerClipboard(body,source.get(),scope));
    }catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
PyObject* capture(PyObject*,PyObject* args){
    const char* geometryPath=nullptr;const char* metadataPath=nullptr;
    if(!PyArg_ParseTuple(args,"ss",&geometryPath,&metadataPath))return nullptr;
    try{
        const auto value=captureLayerClipboard();
        OwnedOutput geometry(geometryPath,value.geometry);
        std::unique_ptr<OwnedOutput> metadata;
        if(value.metadataPresent)metadata=std::make_unique<OwnedOutput>(metadataPath,value.metadata);
        auto result=receipt(value.info,{{"geometry_file",QString::fromUtf8(geometryPath)},
            {"metadata_file",value.metadataPresent?QJsonValue(QString::fromUtf8(metadataPath)):QJsonValue(QJsonValue::Null)}});
        if(!result)return nullptr;
        geometry.committed=true;if(metadata)metadata->committed=true;return result;
    }catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
}
void AddLayerClipboardMethods(PyObject* module){
    static PyMethodDef methods[]={
        {"publishLayerClipboard3dm",publish,METH_VARARGS,"Publish native Rhino geometry and Rust layer metadata together; GUI thread only."},
        {"captureLayerClipboard3dm",capture,METH_VARARGS,"Capture and validate both formats into fresh owned staging files; SDK preparation/GUI layer commit remain required."},
        {nullptr,nullptr,0,nullptr}};
    PyModule_AddFunctions(module,methods);
}
