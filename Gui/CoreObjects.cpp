// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreObjects.h"
#include "CoreUnits.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/GeoFeature.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Base/Exception.h>
#include <Base/Interpreter.h>
#include <QApplication>
#include <QThread>
#include <fastsignals/signal.h>
#include <cmath>
#include <cstring>
#include <map>
#include <memory>
#include <set>
#include <stdexcept>

extern "C" {
std::uint64_t om9_object_document_open();
void om9_object_document_close(std::uint64_t);
bool om9_object_document_invalidate(std::uint64_t);
bool om9_object_observe(std::uint64_t,std::uint64_t,OpenMatrix9Gui::ObjectStamp*);
bool om9_object_changed(std::uint64_t,std::uint64_t);
void om9_object_removed(std::uint64_t,std::uint64_t);
bool om9_object_validate(const OpenMatrix9Gui::ObjectStamp*);
}

namespace OpenMatrix9Gui {
namespace {
constexpr const char* capsuleName="OpenMatrix9.ObjectSnapshot.v1";
void guiThread() {
    if(!qApp||QThread::currentThread()!=qApp->thread())
        throw std::runtime_error("Object snapshots require the GUI thread");
}
class Store {
public:
    Store() {
        auto& app=App::GetApplication();
        deleted=app.signalDeleteDocument.connect([this](const App::Document& doc){
            if(auto found=documents.find(&doc);found!=documents.end()){
                om9_object_document_close(found->second);documents.erase(found);
            }
        });
        removed=app.signalDeletedObject.connect([this](const App::DocumentObject& object){retire(object);});
        created=app.signalNewObject.connect([this](const App::DocumentObject& object){retire(object);});
        changed=app.signalChangedObject.connect([this](const App::DocumentObject& object,const App::Property& property){
            // App properties can be native feature inputs or retained payloads.
            // GUI visibility also mirrors an App property and propagates through
            // _GroupTouched; neither changes geometry or placement ownership.
            const char* name=object.getPropertyName(&property);
            if(&property==&object.Visibility||&property==&object.Label||
               (name&&(std::strcmp(name,"Label2")==0||std::strcmp(name,"_GroupTouched")==0)))return;
            if(auto found=documents.find(object.getDocument());found!=documents.end())
                om9_object_changed(found->second,object.getID());
        });
        undone=app.signalUndoDocument.connect([this](const App::Document& doc){invalidate(doc);});
        redone=app.signalRedoDocument.connect([this](const App::Document& doc){invalidate(doc);});
    }
    std::uint64_t document(const App::Document& doc) {
        if(auto found=documents.find(&doc);found!=documents.end())return found->second;
        const auto id=om9_object_document_open();
        if(!id)throw std::runtime_error("Object identity registry is unavailable");
        try {documents.emplace(&doc,id);}catch(...){om9_object_document_close(id);throw;}
        return id;
    }
    ObjectStamp observe(const App::DocumentObject& object) {
        ObjectStamp result;
        if(!object.getDocument()||object.getID()<=0||
           !om9_object_observe(document(*object.getDocument()),object.getID(),&result))
            throw std::runtime_error("Cannot capture native object identity");
        return result;
    }
private:
    void retire(const App::DocumentObject& object) {
        if(auto found=documents.find(object.getDocument());found!=documents.end())
            om9_object_removed(found->second,object.getID());
    }
    void invalidate(const App::Document& doc) {
        if(auto found=documents.find(&doc);found!=documents.end()){
            if(!om9_object_document_invalidate(found->second)){
                om9_object_document_close(found->second);documents.erase(found);
            }
        }
    }
    // Native-only lookup: erased synchronously before Document destruction.
    std::map<const App::Document*,std::uint64_t> documents;
    fastsignals::scoped_connection deleted,removed,created,changed,undone,redone;
};
Store& store(){static auto* value=new Store;return *value;}

std::array<double,16> world(const App::DocumentObject& object) {
    const auto matrix=App::GeoFeature::getGlobalPlacement(&object).toMatrix();
    std::array<double,16> values;
    for(unsigned row=0;row<4;++row)for(unsigned col=0;col<4;++col){
        const auto value=matrix[row][col];
        if(!std::isfinite(value))throw std::runtime_error("Nonfinite source world placement");
        values[row*4+col]=value;
    }
    return values;
}
std::vector<ObjectStamp> ancestors(const App::DocumentObject& object) {
    std::vector<ObjectStamp> result;
    std::set<const App::DocumentObject*> seen{&object};
    auto* parent=App::GeoFeatureGroupExtension::getGroupOfObject(&object);
    while(parent){
        if(result.size()>=256||!seen.insert(parent).second||parent->getDocument()!=object.getDocument())
            throw std::runtime_error("Invalid or excessively deep placement ownership");
        result.push_back(store().observe(*parent));
        parent=App::GeoFeatureGroupExtension::getGroupOfObject(parent);
    }
    return result;
}
bool sameStamp(const ObjectStamp& left,const ObjectStamp& right) {
    return left.document==right.document&&left.object==right.object&&left.generation==right.generation&&
           left.revision==right.revision&&left.epoch==right.epoch;
}
[[noreturn]] void stale(const ObjectSnapshot& snapshot) {
    throw std::runtime_error("Source object changed or was replaced: "+snapshot.objectName+
                             "; cancel and reselect before accepting");
}
App::Document& document(const char* name) {
    guiThread();auto* doc=App::GetApplication().getDocument(name);
    if(!doc||doc->testStatus(App::Document::Restoring))throw std::runtime_error("Document is missing or restoring");
    return *doc;
}
template<class F> PyObject* api(F&& function) {
    try{return function();}
    catch(const Base::Exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());}
    catch(const std::exception& error){PyErr_SetString(PyExc_ValueError,error.what());}
    catch(...){PyErr_SetString(PyExc_RuntimeError,"Native object snapshot operation failed");}
    return nullptr;
}
void releaseSnapshot(PyObject* capsule) {
    delete static_cast<ObjectSnapshot*>(PyCapsule_GetPointer(capsule,capsuleName));
}
PyObject* captureAPI(PyObject*,PyObject* args) {
    const char *doc=nullptr,*name=nullptr,*sub="";
    if(!PyArg_ParseTuple(args,"ss|s",&doc,&name,&sub))return nullptr;
    return api([&]()->PyObject*{
        auto snapshot=std::make_unique<ObjectSnapshot>(captureObject(document(doc),name,sub));
        const auto& s=snapshot->stamp;
        PyObject* capsule=PyCapsule_New(snapshot.get(),capsuleName,releaseSnapshot);
        if(!capsule)return nullptr;
        auto* value=snapshot.release();
        PyObject* result=Py_BuildValue("{s:i,s:O,s:K,s:K,s:K,s:K,s:K,s:s,s:s,s:s}",
            "schema_version",1,"token",capsule,"document_session",static_cast<unsigned long long>(s.document),
            "object_id",static_cast<unsigned long long>(s.object),"generation",static_cast<unsigned long long>(s.generation),
            "geometry_revision",static_cast<unsigned long long>(s.revision),"document_epoch",static_cast<unsigned long long>(s.epoch),
            "document_uid",value->documentUid.c_str(),"object_name",value->objectName.c_str(),
            "subelement",value->subelement.c_str());
        Py_DECREF(capsule);return result;
    });
}
PyObject* validateAPI(PyObject*,PyObject* args) {
    const char* name=nullptr;PyObject* capsule=nullptr;
    if(!PyArg_ParseTuple(args,"sO",&name,&capsule))return nullptr;
    return api([&]()->PyObject*{
        auto* snapshot=static_cast<ObjectSnapshot*>(PyCapsule_GetPointer(capsule,capsuleName));
        if(!snapshot)return nullptr;
        verifyObject(document(name),*snapshot);Py_RETURN_TRUE;
    });
}
}

ObjectSnapshot captureObject(App::Document& doc,const std::string& name,const std::string& sub) {
    guiThread();
    if(doc.testStatus(App::Document::Restoring))throw std::runtime_error("Cannot capture a restoring document");
    if(name.empty()||name.size()>1024||sub.size()>4096||name.find('\0')!=std::string::npos||sub.find('\0')!=std::string::npos)
        throw std::runtime_error("Invalid object or subelement reference");
    auto* object=doc.getObject(name.c_str());
    if(!object)throw std::runtime_error("Source object is missing: "+name);
    ObjectSnapshot result;
    result.stamp=store().observe(*object);
    result.objectName=name;result.subelement=sub;result.documentUid=doc.Uid.getValueStr();
    result.unitContext=unitContextSnapshot(doc);
    // Check cycles/depth before native placement recursively visits ancestors.
    result.ancestors=ancestors(*object);
    result.world=world(*object);
    return result;
}
void verifyObject(App::Document& doc,const ObjectSnapshot& snapshot) {
    guiThread();
    if(!snapshot.captured()||doc.testStatus(App::Document::Restoring)||
       store().document(doc)!=snapshot.stamp.document||doc.Uid.getValueStr()!=snapshot.documentUid||
       !om9_object_validate(&snapshot.stamp))stale(snapshot);
    auto* object=doc.getObject(snapshot.objectName.c_str());
    if(!object||object->getID()<=0||static_cast<std::uint64_t>(object->getID())!=snapshot.stamp.object||
       unitContextSnapshot(doc)!=snapshot.unitContext)stale(snapshot);
    const auto current=ancestors(*object);
    if(current.size()!=snapshot.ancestors.size())stale(snapshot);
    for(std::size_t i=0;i<current.size();++i)
        if(!sameStamp(current[i],snapshot.ancestors[i])||!om9_object_validate(&snapshot.ancestors[i]))stale(snapshot);
    if(world(*object)!=snapshot.world)stale(snapshot);
}
void addObjectSnapshotMethods(PyObject* module) {
    static PyMethodDef methods[]={
        {"objectSnapshot",captureAPI,METH_VARARGS,"Capture immutable session identity by document/object name without changing the document. Token is process-local, not persistent History metadata."},
        {"validateObjectSnapshot",validateAPI,METH_VARARGS,"Validate a captured token before native mutation; stale source/owner/context raises without changing the document."},
        {nullptr,nullptr,0,nullptr}};
    if(PyModule_AddFunctions(module,methods)<0)throw Base::RuntimeError("Cannot register object snapshot API");
}
}
