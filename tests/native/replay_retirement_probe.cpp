// SPDX-License-Identifier: LGPL-2.1-or-later
// Real native property destruction callbacks, including swallowed violations.
#include <Python.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentPy.h>
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <App/PropertyPythonObject.h>
#include <App/Transactions.h>
#include <Base/Exception.h>
#include <Base/Writer.h>
#include <Base/Console.h>
#include <Base/Tools.h>
#include <functional>
#include <memory>
#include <string>

namespace SavepointLifetimeTest {
class RetirementString: public App::PropertyString {
    TYPESYSTEM_HEADER_WITH_OVERRIDE();
public:
    static inline std::function<void()> onRetire;
    static inline std::function<void()> onSave;
    static inline unsigned sequence=0,armedAfter=0;
    static inline bool retireRecovery=false;
    static inline bool retireDiscardedRedo=false;
    static inline bool retireMerge=false;
    enum class Kind { Live, Captured, Recovery, Merge };
    Kind kind=Kind::Live;
    unsigned serial=0;
    App::Property* Copy() const override {
        auto value=std::make_unique<RetirementString>();
        value->setValue(getValue());value->kind=kind==Kind::Live?Kind::Captured:Kind::Merge;value->serial=++sequence;
        return value.release();
    }
    void Paste(const App::Property& source) override {
        App::PropertyString::Paste(source);
        if(kind==Kind::Live && !getContainer()){kind=Kind::Recovery;serial=++sequence;}
    }
    void Save(Base::Writer& writer) const override {
        if(kind==Kind::Recovery && serial>armedAfter && onSave)onSave();
        App::PropertyString::Save(writer);
    }
    ~RetirementString() override {
        const auto target=retireMerge?Kind::Merge:retireRecovery?Kind::Recovery:Kind::Captured;
        const bool match=retireDiscardedRedo ? kind==Kind::Captured && serial<=armedAfter
                                            : kind==target && serial>armedAfter;
        if(match && onRetire)onRetire();
    }
};
TYPESYSTEM_SOURCE(SavepointLifetimeTest::RetirementString,App::PropertyString)
class Feature: public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(SavepointLifetimeTest::Feature);
public:
    static inline std::function<void()> onDispose;
    bool disposalProbe=false;
    RetirementString Value;
    App::PropertyString Output;
    Feature(){ADD_PROPERTY(Value,("original-value"));ADD_PROPERTY(Output,("original-output"));}
    ~Feature() override {if(disposalProbe && onDispose)onDispose();}
};
PROPERTY_SOURCE(SavepointLifetimeTest::Feature,App::DocumentObject)
class MemberFeature: public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(SavepointLifetimeTest::MemberFeature);
public:
    // C++ destroys Value first, then the Python member. Its real finalizer
    // must not access properties through a still-valid derived wrapper.
    App::PropertyPythonObject Payload;
    App::PropertyString Value;
    MemberFeature(){ADD_PROPERTY(Payload,(Py::Object(Py_None)));ADD_PROPERTY(Value,("member-value"));}
};
PROPERTY_SOURCE(SavepointLifetimeTest::MemberFeature,App::DocumentObject)
}
namespace {
template<class Scope>
void disposalResult(const Scope& scope,bool& verified,std::string& diagnostic){
    if constexpr(requires{scope.disposalVerified();scope.disposalDiagnostic();}){
        verified=scope.disposalVerified();diagnostic=scope.disposalDiagnostic();
    }
}
PyObject* create(PyObject*,PyObject* args){
    const char *name,*objectName="Child";int disposal=0;if(!PyArg_ParseTuple(args,"s|sp",&name,&objectName,&disposal))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);
    if(!doc){PyErr_SetString(PyExc_ValueError,"Live owner required");return nullptr;}
    try{
        auto owned=std::make_unique<SavepointLifetimeTest::Feature>();
        owned->disposalProbe=disposal!=0;
        doc->addObject(owned.get(),objectName);
        auto* child=owned.release();
        return PyUnicode_FromString(child->getNameInDocument());
    }catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
     catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
PyObject* createMember(PyObject*,PyObject* args){
    const char* name;if(!PyArg_ParseTuple(args,"s",&name))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);
    if(!doc){PyErr_SetString(PyExc_ValueError,"Live owner required");return nullptr;}
    try{
        auto owned=std::make_unique<SavepointLifetimeTest::MemberFeature>();
        doc->addObject(owned.get(),"MemberChild");auto* child=owned.release();
        return PyUnicode_FromString(child->getNameInDocument());
    }catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
    catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
PyObject* hasNativeId(PyObject*,PyObject* args){
    PyObject* object;long id;if(!PyArg_ParseTuple(args,"Ol",&object,&id))return nullptr;
    if(!PyObject_TypeCheck(object,&App::DocumentPy::Type)){
        PyErr_SetString(PyExc_TypeError,"Native document wrapper required");return nullptr;
    }
    auto* wrapper=static_cast<App::DocumentPy*>(object);
    if(!wrapper->Base::PyObjectBase::isValid()){
        PyErr_SetString(PyExc_ReferenceError,"Native document already deleted");return nullptr;
    }
    // Copy only the existence verdict. Never return or dereference the native
    // object pointer, which is precisely the lifetime being characterized.
    return PyBool_FromLong(wrapper->getDocumentPtr()->getObjectByID(id)!=nullptr);
}
PyObject* run(PyObject*,PyObject* args){
    const char *name,*action,*phase="original";PyObject* payload=nullptr;
    if(!PyArg_ParseTuple(args,"ss|sO",&name,&action,&phase,&payload))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);
    auto* child=doc?dynamic_cast<SavepointLifetimeTest::Feature*>(doc->getObject("Child")):nullptr;
    if(!child){PyErr_SetString(PyExc_ValueError,"Live native test child required");return nullptr;}
    using Property=SavepointLifetimeTest::RetirementString;
    unsigned callbacks=0,blocked=0,openCallbacks=0;bool accepted=false,rolled=false,complete=false,cleanupRetryComplete=false,disposalVerified=false;std::string error,blockedError,retryError,disposalDiagnostic;
    Property::armedAfter=Property::sequence;
    Property::retireRecovery=std::string(phase)=="recovery";
    Property::retireDiscardedRedo=std::string(phase)=="discarded_redo";
    Property::retireMerge=std::string(phase)=="merge_open";
    try{
        App::Document::PropertyReplayTransaction scope(*doc);
        auto callback=[&]{
            ++callbacks;
            try{
                if(std::string(action)=="close")App::GetApplication().closeDocument(name);
                else if(std::string(action)=="remove")doc->removeObject("Child");
                else if(std::string(action)=="clear")doc->clearUndos();
                else child->Value.setValue("tampered");
            }catch(const Base::Exception& e){++blocked;blockedError=e.what();}
             catch(const std::exception& e){++blocked;blockedError=e.what();}
        };
        const bool objectPhase=std::string(phase)=="redo_object";
        const bool savePhase=std::string(phase)=="save";
        const bool retryPhase=std::string(phase)=="open_retry";
        const bool releasePhase=std::string(phase)=="python_release";
        const bool cleanupPhase=std::string(phase)=="python_cleanup";
        Property::onRetire=objectPhase||savePhase||retryPhase||releasePhase||cleanupPhase?std::function<void()>{}:callback;
        Property::onSave=savePhase?callback:std::function<void()>{};
        SavepointLifetimeTest::Feature::onDispose=objectPhase?callback:std::function<void()>{};
        fastsignals::scoped_connection openConnection;
        if(Property::retireMerge||retryPhase||cleanupPhase){
            openConnection=doc->signalOpenTransaction.connect([&](const App::Document&,std::string){
                ++openCallbacks;throw std::runtime_error("native open notification failed");
            });
        }
        try{
            if(payload){
                auto* property=dynamic_cast<App::PropertyPythonObject*>(child->getPropertyByName("Payload"));
                if(!property)throw std::runtime_error("native Python replay property required");
                property->setValue(Py::Object(payload));
            }
            child->Value.setValue("replayed-value");child->Output.setValue("replayed-output");
            scope.verify();accepted=scope.commit();complete=accepted;
        }catch(const Base::Exception& e){error=e.what();}
         catch(const std::exception& e){error=e.what();}
        if(retryPhase&&!accepted){
            try{scope.commit();retryError="Unexpected accepted retry";}
            catch(const Base::Exception& e){retryError=e.what();}
            catch(const std::exception& e){retryError=e.what();}
        }
        if(!accepted){
            rolled=scope.rollback();
            if(rolled){
                child->Output.setValue("");
                if(cleanupPhase){
                    auto* property=dynamic_cast<App::PropertyPythonObject*>(child->getPropertyByName("Payload"));
                    property->setValue(Py::None());
                }
                complete=scope.completeErrorState();
                if(cleanupPhase)cleanupRetryComplete=scope.completeErrorState();
            }
        }
        disposalResult(scope,disposalVerified,disposalDiagnostic);
        Property::onRetire={};
        Property::onSave={};
        SavepointLifetimeTest::Feature::onDispose={};
    }catch(const Base::Exception& e){error=e.what();Property::onRetire={};Property::onSave={};SavepointLifetimeTest::Feature::onDispose={};}
     catch(const std::exception& e){error=e.what();Property::onRetire={};Property::onSave={};SavepointLifetimeTest::Feature::onDispose={};}
     catch(...){error="Unknown native retirement exception";Property::onRetire={};Property::onSave={};SavepointLifetimeTest::Feature::onDispose={};}
    return Py_BuildValue("{s:O,s:O,s:O,s:I,s:I,s:I,s:s,s:s,s:s,s:O,s:O,s:s}","accepted",accepted?Py_True:Py_False,
        "rollback",rolled?Py_True:Py_False,"complete",complete?Py_True:Py_False,
        "callbacks",callbacks,"blocked",blocked,"open_callbacks",openCallbacks,"error",error.c_str(),"blocked_error",blockedError.c_str(),"retry_error",retryError.c_str(),"cleanup_retry_complete",cleanupRetryComplete?Py_True:Py_False,"disposal_verified",disposalVerified?Py_True:Py_False,"disposal_diagnostic",disposalDiagnostic.c_str());
}
PyObject* rename(PyObject*,PyObject* args){
    const char *name,*from,*to;if(!PyArg_ParseTuple(args,"sss",&name,&from,&to))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);auto* child=doc?doc->getObject("Child"):nullptr;
    auto* property=child?child->getDynamicPropertyByName(from):nullptr;
    if(!property){PyErr_SetString(PyExc_ValueError,"Live native dynamic property required");return nullptr;}
    try{return PyBool_FromLong(child->renameDynamicProperty(property,to));}
    catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
    catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
PyObject* sourceRoundtrip(PyObject*,PyObject* args){
    const char *name,*propertyName,*operation;PyObject* mutate;
    if(!PyArg_ParseTuple(args,"sssO",&name,&propertyName,&operation,&mutate))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);
    auto* child=doc?dynamic_cast<SavepointLifetimeTest::Feature*>(doc->getObject("Child")):nullptr;
    auto* property=child?child->getPropertyByName(propertyName):nullptr;
    if(!property||!PyCallable_Check(mutate)){PyErr_SetString(PyExc_ValueError,"Live property and native setter callback required");return nullptr;}
    try{
        App::Document::PropertyReplayTransaction scope(*doc);
        auto source=std::make_unique<App::Transaction>(-1);
        source->addObjectChange(child,property);source->addObjectChange(child,&child->Value);
        Py::Callable callback(mutate);callback.apply(Py::Tuple());
        child->Value.setValue("published-source");
        source->retirePropertyCopies(1000,16*1024*1024);
        bool prepared=false,restored=false;
        if(std::string(operation)=="merge"){
            App::Transaction caller(-1);auto stage=caller.preparePropertyMerge(*source,1000,16*1024*1024);
            prepared=caller.sealPropertyMergeSource(*stage);source.reset();
            restored=prepared&&caller.commitPropertyMerge(*stage)&&caller.applyChecked(*doc,false);
        }else if(std::string(operation)=="recovery"){
            auto recovery=source->preparePropertyRecovery(1000,16*1024*1024);prepared=true;
            source.reset();restored=recovery->applyChecked(*doc,false);
        }else{
            auto inverse=source->preparePropertyInverse(1000,16*1024*1024);prepared=true;
            source.reset();restored=inverse->applyChecked(*doc);
        }
        const bool rolled=scope.rollback(),complete=rolled&&scope.completeErrorState();
        return Py_BuildValue("{s:O,s:O,s:O,s:O}","prepared",prepared?Py_True:Py_False,
            "restored",restored?Py_True:Py_False,"rollback",rolled?Py_True:Py_False,"complete",complete?Py_True:Py_False);
    }catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
    catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
    catch(const Py::Exception&){Base::PyException e;PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
PyObject* historyDisposal(PyObject*,PyObject* args){
    const char *name,*objectName;int throwing=0;
    if(!PyArg_ParseTuple(args,"ssp",&name,&objectName,&throwing))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);
    auto* child=doc?doc->getObject(objectName):nullptr;
    if(!child){PyErr_SetString(PyExc_ValueError,"Live History child required");return nullptr;}
    unsigned callbacks=0,blocked=0;
    struct Logger:Base::ILogger{
        bool throwing;unsigned calls=0;std::string message;
        explicit Logger(bool value):throwing(value){}
        void sendLog(const std::string&,const std::string& value,Base::LogStyle,Base::IntendedRecipient,Base::ContentType)override{
            if(value.find("Circle History")!=std::string::npos&&value.find("disposal diagnostic")!=std::string::npos){
                ++calls;message=value;
                if(throwing)throw std::runtime_error("native diagnostic observer failed");
            }
        }
    } logger(throwing!=0);
    Base::Console().setConnectionMode(Base::ConsoleSingleton::Direct);
    Base::Console().attachObserver(&logger);
    Base::ScopeGuard cleanup([&]{SavepointLifetimeTest::Feature::onDispose={};Base::Console().detachObserver(&logger);});
    SavepointLifetimeTest::Feature::onDispose=[&]{
        ++callbacks;
        try{child->Label.setValue("tampered-label");}
        catch(const Base::Exception&){++blocked;}
    };
    try{
        doc->recompute();
        return Py_BuildValue("{s:I,s:I,s:I,s:s}","callbacks",callbacks,"blocked",blocked,"logger_calls",logger.calls,"message",logger.message.c_str());
    }catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
    catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
PyObject* failedEmptyRedo(PyObject*,PyObject* args){
    const char* name;if(!PyArg_ParseTuple(args,"s",&name))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);
    if(!doc){PyErr_SetString(PyExc_ValueError,"Live owner required");return nullptr;}
    unsigned callbacks=0,blocked=0;std::string error;
    SavepointLifetimeTest::Feature::onDispose=[&]{
        ++callbacks;
        try{doc->lockTransaction();}
        catch(const Base::Exception& e){++blocked;error=e.what();}
    };
    Base::ScopeGuard cleanup([&]{SavepointLifetimeTest::Feature::onDispose={};
        if(doc->isTransactionLocked())doc->unlockTransaction();});
    auto failing=doc->signalBecameStable.connect([](const App::Document&){
        throw std::runtime_error("deliberate empty preserving close failure");
    });
    fastsignals::scoped_connection connection(std::move(failing));
    try{
        const int id=doc->openTransactionPreservingRedo("Empty failed group");
        const bool accepted=App::GetApplication().commitTransaction(id);
        return Py_BuildValue("{s:O,s:I,s:I,s:O,s:s}","accepted",accepted?Py_True:Py_False,
            "callbacks",callbacks,"blocked",blocked,"locked",doc->isTransactionLocked()?Py_True:Py_False,"error",error.c_str());
    }catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
    catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
PyMethodDef methods[]={{"create",create,METH_VARARGS,nullptr},{"create_member",createMember,METH_VARARGS,nullptr},{"has_native_id",hasNativeId,METH_VARARGS,nullptr},{"run",run,METH_VARARGS,nullptr},{"rename",rename,METH_VARARGS,nullptr},{"source_roundtrip",sourceRoundtrip,METH_VARARGS,nullptr},{"history_disposal",historyDisposal,METH_VARARGS,nullptr},{"failed_empty_redo",failedEmptyRedo,METH_VARARGS,nullptr},{nullptr,nullptr,0,nullptr}};
PyModuleDef module={PyModuleDef_HEAD_INIT,"OM9ReplayRetirement",nullptr,-1,methods};
}
PyMODINIT_FUNC PyInit_OM9ReplayRetirement(){
    SavepointLifetimeTest::RetirementString::init();
    SavepointLifetimeTest::Feature::init();
    SavepointLifetimeTest::MemberFeature::init();
    static App::TransactionProducer<App::TransactionDocumentObject> producer(SavepointLifetimeTest::Feature::getClassTypeId());
    static App::TransactionProducer<App::TransactionDocumentObject> memberProducer(SavepointLifetimeTest::MemberFeature::getClassTypeId());
    return PyModule_Create(&module);
}
