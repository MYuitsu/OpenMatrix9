// SPDX-License-Identifier: LGPL-2.1-or-later
#include "RetainedArchive.h"
#include "CurveGeometry.h"
#include <QFile>
#include <QCryptographicHash>
#include <QTemporaryDir>
#include <cstdint>
#include <stdexcept>
extern "C" int om9_retained_manifest_validate(const char*,const char*,const char*,const char*,const char*,const char*,std::int64_t,double*);
extern "C" bool om9_retained_record_identity_valid(const char*,const char*);
extern "C" bool om9_retained_uuid_normalize(const char*,char*,std::size_t);
extern "C" bool om9_retained_instance_snapshot_unchanged(const unsigned char*,std::size_t,const char*,const char*,const char*);
namespace OpenMatrix9Gui {
namespace {
using Ref=CurvePyRef;
std::string string(PyObject* o,const char* key){Ref value(PyObject_GetAttrString(o,key));Py_ssize_t size=0;const auto* s=value.value?PyUnicode_AsUTF8AndSize(value.value,&size):nullptr;if(!s)throw std::runtime_error(std::string("Missing retained source field: ")+key);std::string result(s,static_cast<std::size_t>(size));if(result.find('\0')!=std::string::npos)throw std::runtime_error(std::string("Invalid retained source field: ")+key);return result;}
bool derived(PyObject* object,const char* type){Ref result(PyObject_CallMethod(object,"isDerivedFrom","s",type));return PyObject_IsTrue(result.value)==1;}
void verifyInstanceOwner(PyObject* source,PyObject* container){
    Ref owner(PyObject_GetAttrString(source,"OM9ArchiveOwner"));
    if(owner.value!=container||string(source,"Name")!=string(source,"OM9SourceHostID"))
        throw std::runtime_error("Retained instance has ambiguous current ownership; source preserved");
}
void verifyInstanceSnapshot(PyObject* source,const RetainedArchiveRecord& record,const std::string& digest){
    // Reuse the established host adapter for placements, retargeted definitions,
    // member/proxy edits and preview integrity. It stages only temporary files;
    // the portable accept/reject policy belongs to Rust, with no document edits.
    QTemporaryDir staging;if(!staging.isValid())throw std::runtime_error("Cannot verify current retained instance");
    Ref state(PyImport_ImportModule("ThreeDmArchiveState")),selected(Py_BuildValue("[O]",source));
    Ref request(PyObject_CallMethod(state.value,"preservation_request","Os",selected.value,staging.path().toUtf8().constData()));
    Ref json(PyImport_ImportModule("json")),serialized(PyObject_CallMethod(json.value,"dumps","O",request.value));
    Py_ssize_t size=0;const auto* raw=PyUnicode_AsUTF8AndSize(serialized.value,&size);
    if(!raw||!om9_retained_instance_snapshot_unchanged(reinterpret_cast<const unsigned char*>(raw),static_cast<std::size_t>(size),record.sourceUuid.c_str(),record.importNamespace.c_str(),digest.c_str()))
        throw std::runtime_error("Retained instance or its definition has changed; source preserved. Explode requires an unchanged source instance");
}
}
RetainedArchiveRecord verifiedRetainedArchive(PyObject* source){
    RetainedArchiveRecord result;result.sourceUuid=string(source,"OM9SourceUUID");result.sourceClass=string(source,"OM9SourceClass");result.importNamespace=string(source,"OM9ImportNamespace");
    if(!om9_retained_record_identity_valid(result.sourceUuid.c_str(),result.importNamespace.c_str()))throw std::runtime_error("Invalid retained record identity");
    char canonicalUuid[37]{};if(!om9_retained_uuid_normalize(result.sourceUuid.c_str(),canonicalUuid,sizeof canonicalUuid))throw std::runtime_error("Invalid retained record identity");result.sourceUuid=canonicalUuid;
    Ref document(PyObject_GetAttrString(source,"Document")),objects(document.value?PyObject_GetAttrString(document.value,"Objects"):nullptr),container(Py_NewRef(Py_None));
    if(!objects.value)throw std::runtime_error("Retained source has no native document");
    for(Py_ssize_t i=0;i<PySequence_Size(objects.value);++i){Ref object(PySequence_GetItem(objects.value,i));
        if(PyObject_HasAttrString(object.value,"OM9SourceArchive")==1&&PyObject_HasAttrString(object.value,"OM9ImportNamespace")==1&&string(object.value,"OM9ImportNamespace")==result.importNamespace){if(container.value!=Py_None)throw std::runtime_error("Ambiguous retained archive namespace");container=Ref(Py_NewRef(object.value));}}
    if(container.value==Py_None)throw std::runtime_error("Retained source archive is missing");result.file=std::filesystem::u8path(string(container.value,"OM9SourceArchive"));
    QFile file(QString::fromStdWString(result.file.wstring()));if(!file.open(QIODevice::ReadOnly))throw std::runtime_error("Cannot read retained source archive");
    const auto size=file.size();if(size<=0||size>512LL*1024*1024)throw std::runtime_error("Cannot read retained source archive");
    result.archiveBytes.resize(static_cast<std::size_t>(size));
    if(file.read(reinterpret_cast<char*>(result.archiveBytes.data()),size)!=size)throw std::runtime_error("Cannot read retained source archive");
    char extra;if(file.read(&extra,1)!=0||file.error()!=QFileDevice::NoError)throw std::runtime_error("Cannot read retained source archive");
    auto digest=string(container.value,"OM9ArchiveHash");QCryptographicHash hash(QCryptographicHash::Sha256);hash.addData(reinterpret_cast<const char*>(result.archiveBytes.data()),size);if(hash.result().toHex().toStdString()!=digest)throw std::runtime_error("Source archive integrity check failed");
    auto raw=string(container.value,"OM9ArchiveManifest");Ref schema(PyObject_GetAttrString(container.value,"OM9ArchiveSchema"));
    if(!schema.value)throw std::runtime_error("Missing retained manifest schema");
    auto schemaVersion=PyLong_AsLongLong(schema.value);if(PyErr_Occurred())throw std::runtime_error("Invalid retained manifest schema");
    auto capability=string(source,"OM9Capability");
    const bool instance=result.sourceClass=="ON_InstanceRef";
    if(instance)verifyInstanceOwner(source,container.value);
    if(capability=="display-retained"){
        // This host-only representation does not change the archive inventory.
        // Normalize only a verified affine instance preview, never arbitrary
        // retained geometry or an edited preview masquerading as native data.
        if(!instance||(!derived(source,"Part::Feature")&&!derived(source,"App::Part")))
            throw std::runtime_error("Invalid retained instance preview capability");
        Ref state(PyImport_ImportModule("ThreeDmArchiveState")),matches(PyObject_CallMethod(state.value,"_preview_matches","O",source));
        if(PyObject_IsTrue(matches.value)!=1)throw std::runtime_error("Retained instance preview changed; source preserved");
        capability="retained";
    }
    switch(om9_retained_manifest_validate(raw.c_str(),digest.c_str(),result.sourceUuid.c_str(),result.sourceClass.c_str(),capability.c_str(),result.importNamespace.c_str(),schemaVersion,&result.scaleMm)){
    case 0:break;
    case -1:throw std::runtime_error("Invalid retained manifest JSON or budget");
    case -2:throw std::runtime_error("Source manifest integrity check failed");
    case -3:throw std::runtime_error("Invalid retained source identity or capability");
    case -4:throw std::runtime_error("Invalid retained source units");
    case -5:throw std::runtime_error("Retained source has unresolved or invalid dependency reports");
    default:throw std::runtime_error("Retained manifest validation failed");
    }
    if(instance)verifyInstanceSnapshot(source,result,digest);
    return result;
}
}
