// SPDX-License-Identifier: LGPL-2.1-or-later
// Test-only native journal tests; document savepoint/visible Undo integration
// remains a separate gate. The old API reports unavailable instead of emulating
// a merge or manufacturing successful values.
#include <Python.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/Transactions.h>
#include <App/TransactionalObject.h>
#include <App/PropertyStandard.h>
#include <functional>
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

namespace SavepointTest {
class CopyString: public App::PropertyString {
public:
    static inline std::function<void()> onCopy;
    static inline bool growCopy=false;
    static inline unsigned liveCopies=0;
    bool copied=false;
    ~CopyString() override {if(copied)--liveCopies;}
    App::Property* Copy() const override {
        if(onCopy) onCopy();
        auto copy=std::make_unique<CopyString>();copy->setValue(getValue());
        if(growCopy)copy->setValue(std::string(1024*1024,'x'));
        copy->copied=true;++liveCopies;
        return copy.release();
    }
};
class Tracked: public App::TransactionalObject {
    PROPERTY_HEADER_WITH_OVERRIDE(SavepointTest::Tracked);
public:
    static inline unsigned destroyed=0;
    CopyString Value,Later;
    Tracked() {ADD_PROPERTY(Value,("original"));ADD_PROPERTY(Later,("later-original"));}
    ~Tracked() override {++destroyed;}
};
PROPERTY_SOURCE(SavepointTest::Tracked,App::TransactionalObject)
}
namespace {
struct Check {std::string name;bool passed;};
template<class Journal> constexpr bool available=requires(Journal& a,const Journal& b){
    a.preparePropertyMerge(b,std::size_t(1000),std::size_t(16*1024*1024));
};
template<class Journal> std::vector<Check> exercise(App::Document& doc){
    using SavepointTest::Tracked;using SavepointTest::CopyString;
    std::vector<Check> checks;auto check=[&](const char* name,bool value){checks.push_back({name,value});};
    if constexpr(!available<Journal>){check("native staged property merge API is available",false);return checks;}
    else {
        check("native staged property merge API is available",true);
        {
            Tracked owner,independent;Journal caller,replay;caller.Name="Caller";const int id=caller.getID();
            caller.addObjectChange(&owner,&owner.Value);caller.addObjectChange(&independent,&independent.Value);
            owner.Value.setValue("caller-edited");independent.Value.setValue("independent-edited");
            replay.addObjectChange(&owner,&owner.Value);replay.addObjectChange(&owner,&owner.Later);
            owner.Value.setValue("replay-output");owner.Later.setValue("later-output");
            auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);
            check("prepare preserves current native property values",owner.Value.getValue()==std::string("replay-output")&&owner.Later.getValue()==std::string("later-output"));
            check("prepare does not alter caller first-write journal",caller.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original")&&owner.Later.getValue()==std::string("later-output"));
            owner.Value.setValue("replay-output");independent.Value.setValue("independent-edited");
            check("commit accepts the prepared native stage",caller.commitPropertyMerge(*stage));
            check("merge preserves caller ID and description",caller.getID()==id&&caller.Name=="Caller");
            check("merged native journal restores earliest same-property value",caller.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
            check("merged native journal restores private first-write value",owner.Later.getValue()==std::string("later-original"));
            check("independent caller entries remain executable",independent.Value.getValue()==std::string("original"));
            check("a prepared stage cannot commit twice",!caller.commitPropertyMerge(*stage));
        }
        {
            Tracked owner;Journal caller,replay;caller.addObjectChange(&owner,&owner.Value);owner.Value.setValue("caller-edited");
            replay.addObjectChange(&owner,&owner.Later);owner.Later.setValue("later-output");
            {auto abandoned=caller.preparePropertyMerge(replay,1000,16*1024*1024);}
            check("abandoned stage preserves caller journal without new entries",caller.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original")&&owner.Later.getValue()==std::string("later-output"));
        }
        {
            Tracked owner;Journal caller,replay;replay.addObjectChange(&owner,&owner.Later);owner.Later.setValue("output");
            bool rejected=false;try{auto stage=caller.preparePropertyMerge(replay,0,16*1024*1024);}catch(const std::exception&){rejected=true;}
            check("record limit rejects before altering empty destination",rejected&&caller.isEmpty());
            unsigned copies=0;CopyString::onCopy=[&]{++copies;};rejected=false;
            try{auto stage=caller.preparePropertyMerge(replay,1000,1);}catch(const std::exception&){rejected=true;}
            CopyString::onCopy={};check("byte limit rejects before copying or changing caller",rejected&&copies==0&&caller.isEmpty());
            CopyString::onCopy=[] {throw std::runtime_error("actual native Copy fault");};rejected=false;
            try{auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);}catch(const std::exception&){rejected=true;}
            CopyString::onCopy={};check("native Property Copy exception preserves caller and source",rejected&&caller.isEmpty()&&owner.Later.getValue()==std::string("output")&&replay.applyChecked(doc,false)&&owner.Later.getValue()==std::string("later-original"));
            CopyString::growCopy=true;rejected=false;
            try{auto stage=caller.preparePropertyMerge(replay,1000,64*1024);}catch(const std::exception&){rejected=true;}
            CopyString::growCopy=false;check("unexpected larger native copy rejects before transfer",rejected&&caller.isEmpty());
        }
        {
            Tracked owner;Journal destination,replay,wrong;replay.addObjectChange(&owner,&owner.Later);
            auto stage=destination.preparePropertyMerge(replay,1000,16*1024*1024);
            check("stage cannot transfer into another journal",!wrong.commitPropertyMerge(*stage)&&wrong.isEmpty());
            destination.addObjectChange(&owner,&owner.Value);
            check("changed destination rejects stale stage before any transfer",!destination.commitPropertyMerge(*stage));
            owner.Later.setValue("after-stale");check("stale stage does not merge private entries",destination.applyChecked(doc,false)&&owner.Later.getValue()==std::string("after-stale"));
        }
        {
            Tracked owner;Journal caller,replay;replay.addObjectChange(&owner,&owner.Value);
            auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);replay.addObjectChange(&owner,&owner.Later);
            check("changed source journal rejects prepared stage",!caller.commitPropertyMerge(*stage)&&caller.isEmpty());
        }
        {
            const unsigned before=Tracked::destroyed;
            {
                Journal caller,replay;auto* owner=new Tracked;caller.addObjectNew(owner);
                replay.addObjectChange(owner,&owner->Value);owner->Value.setValue("replayed");
                {auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);}
                check("abandoned merge never destroys caller-owned detached New object",Tracked::destroyed==before);
                auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);
                check("accepted merge retains native New-object journal ownership",caller.commitPropertyMerge(*stage)&&Tracked::destroyed==before&&caller.hasObject(owner));
            }
            check("caller-owned detached object is destroyed exactly once",Tracked::destroyed==before+1);
        }
        {
            auto owner=std::make_unique<Tracked>();const unsigned before=Tracked::destroyed;
            {Journal caller,replay;caller.addObjectDel(owner.get());replay.addObjectChange(owner.get(),&owner->Value);
                auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);check("merge preserves caller Del entry without acquiring object ownership",caller.commitPropertyMerge(*stage)&&caller.hasObject(owner.get()));}
            check("destroying merged Del journal does not destroy external object",Tracked::destroyed==before);
        }
        {
            Journal destination,replay;auto* owner=new Tracked;replay.addObjectNew(owner);bool rejected=false;
            try{auto stage=destination.preparePropertyMerge(replay,1000,16*1024*1024);}catch(const std::exception&){rejected=true;}
            check("private object creation deletion journal is refused",rejected&&destination.isEmpty());
        }
        {
            Tracked owner;Journal destination,replay;replay.renameProperty(&owner,&owner.Value,"Other");bool rejected=false;
            try{auto stage=destination.preparePropertyMerge(replay,1000,16*1024*1024);}catch(const std::exception&){rejected=true;}
            check("private property rename is refused before caller mutation",rejected&&destination.isEmpty());
        }
        {
            Tracked owner,target;Journal destination,replay;
            replay.arrangeMoveProperty(&owner,&owner.Value,&target,&target.Value);bool rejected=false;
            try{auto stage=destination.preparePropertyMerge(replay,1000,16*1024*1024);}catch(const std::exception&){rejected=true;}
            check("private property move is refused before caller mutation",rejected&&destination.isEmpty());
        }
        {
            Tracked owner;Journal caller,replay;caller.addObjectChange(&owner,&owner.Value);owner.Value.setValue("caller");replay.addObjectChange(&owner,&owner.Value);
            CopyString::onCopy=[] {throw std::runtime_error("caller originals must never be copied");};
            auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);CopyString::onCopy={};
            check("duplicate replay first-write never copies caller originals",caller.commitPropertyMerge(*stage)&&caller.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
        }
        {
            Tracked owner;Journal caller,replay;auto* added=owner.addDynamicProperty("App::PropertyString","DynamicAdded");
            replay.addOrRemoveProperty(&owner,added,true);static_cast<App::PropertyString*>(added)->setValue("new");
            auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);
            check("dynamic property addition keeps native removal undo",caller.commitPropertyMerge(*stage)&&caller.applyChecked(doc,false)&&!owner.getDynamicPropertyByName("DynamicAdded"));
        }
        {
            Tracked owner;Journal caller,replay;auto* removed=owner.addDynamicProperty("App::PropertyString","DynamicRemoved");static_cast<App::PropertyString*>(removed)->setValue("before-remove");
            replay.addOrRemoveProperty(&owner,removed,false);owner.removeDynamicProperty("DynamicRemoved");
            auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);const bool restored=caller.commitPropertyMerge(*stage)&&caller.applyChecked(doc,false);
            auto* actual=dynamic_cast<App::PropertyString*>(owner.getDynamicPropertyByName("DynamicRemoved"));
            check("dynamic property removal restores native type and original value",restored&&actual&&actual->getValue()==std::string("before-remove"));
        }
        {
            Tracked owner;Journal journal;bool rejected=false;
            CopyString::onCopy=[] {throw std::runtime_error("first capture fault");};
            try{journal.addObjectChange(&owner,&owner.Value);}catch(const std::exception&){rejected=true;}
            CopyString::onCopy={};
            check("failed first capture leaves no object or property journal entry",rejected&&journal.isEmpty()&&journal.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
            journal.addObjectChange(&owner,&owner.Value);owner.Value.setValue("retry-output");
            check("successful retry after capture failure records original value",journal.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
        }
        {
            Tracked owner;Journal journal;journal.addObjectChange(&owner,&owner.Value);owner.Value.setValue("caller-output");bool rejected=false;
            Journal empty;auto before=empty.preparePropertyMerge(journal,1000,16*1024*1024);
            CopyString::onCopy=[] {throw std::runtime_error("later capture fault");};
            try{journal.addObjectChange(&owner,&owner.Later);}catch(const std::exception&){rejected=true;}
            CopyString::onCopy={};
            auto after=empty.preparePropertyMerge(journal,1000,16*1024*1024);
            check("failed later capture leaves existing journal metadata unchanged",rejected&&before->accountedRecords()==after->accountedRecords()&&before->accountedBytes()==after->accountedBytes());
            // Retry repairs an old partial entry before executing its rollback;
            // the assertion above still exposes that failed capture mutation.
            journal.addObjectChange(&owner,&owner.Later);
            check("existing first-write and retry remain executable after later capture fault",journal.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original")&&owner.Later.getValue()==std::string("later-original"));
        }
        {
            Tracked owner;Journal journal;journal.addObjectChange(&owner,&owner.Value);owner.Value.setValue("prior-output");bool rejected=false;
            CopyString::onCopy=[] {throw std::runtime_error("removal capture fault");};
            try{journal.addOrRemoveProperty(&owner,&owner.Value,false);}catch(const std::exception&){rejected=true;}
            CopyString::onCopy={};
            journal.addObjectChange(&owner,&owner.Value);
            check("failed removal capture retains existing original snapshot",rejected&&journal.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
        }
        if constexpr(requires(const Journal& journal){journal.preparePropertyInverse(1000,16*1024*1024);}) {
            {
                Tracked owner;std::unique_ptr<App::Transaction::PropertyInverse> inverse;
                {Journal source;source.addObjectChange(&owner,&owner.Value);owner.Value.setValue("published");inverse=source.preparePropertyInverse(1000,16*1024*1024);}
                check("native inverse restores after every source Property object is destroyed",inverse->applyChecked(doc)&&owner.Value.getValue()==std::string("original"));
            }
            {
                Tracked owner;Journal caller;std::unique_ptr<typename Journal::PropertyMerge> stage;
                {Journal source;source.addObjectChange(&owner,&owner.Value);owner.Value.setValue("published");stage=caller.preparePropertyMerge(source,1000,16*1024*1024);check("protected merge source can be sealed before retirement",caller.sealPropertyMergeSource(*stage));}
                check("sealed source retirement leaves a valid independent caller stage",stage->valid()&&caller.commitPropertyMerge(*stage)&&caller.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
            }
            {
                Tracked owner;Journal source;source.addObjectChange(&owner,&owner.Value);owner.Value.setValue("published");bool rejected=false;
                try{auto inverse=source.preparePropertyInverse(1000,1);}catch(const std::exception&){rejected=true;}
                check("inverse byte budget failure retains executable native source journal",rejected&&source.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
            }
            {
                Tracked owner;std::unique_ptr<App::Transaction::PropertyInverse> inverse;
                {Journal source;auto* added=owner.addDynamicProperty("App::PropertyString","InverseAdded");source.addOrRemoveProperty(&owner,added,true);inverse=source.preparePropertyInverse(1000,16*1024*1024);}
                check("inverse preserves dynamic addition removal after source retirement",inverse->applyChecked(doc)&&!owner.getDynamicPropertyByName("InverseAdded"));
            }
            {
                Tracked owner;std::unique_ptr<App::Transaction::PropertyInverse> inverse;
                {Journal source;auto* removed=owner.addDynamicProperty("App::PropertyString","InverseRemoved");static_cast<App::PropertyString*>(removed)->setValue("original-dynamic");source.addOrRemoveProperty(&owner,removed,false);owner.removeDynamicProperty("InverseRemoved");inverse=source.preparePropertyInverse(1000,16*1024*1024);}
                const bool restored=inverse->applyChecked(doc);auto* property=dynamic_cast<App::PropertyString*>(owner.getDynamicPropertyByName("InverseRemoved"));
                check("inverse recreates removed dynamic property with original native value",restored&&property&&property->getValue()==std::string("original-dynamic"));
            }
            {
                Tracked owner;Journal source;source.addObjectChange(&owner,&owner.Value);owner.Value.setValue("published");
                source.retirePropertyCopies(1000,16*1024*1024);
                check("retired property copies leave native journal executable",source.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
                owner.Value.setValue("published-again");source.retirePropertyCopies(1000,16*1024*1024);
                check("repeated retirement retains the same first-write inverse",source.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
            }
            {
                const unsigned before=Tracked::destroyed;
                {Journal source;auto* owner=new Tracked;source.addObjectNew(owner);source.addObjectChange(owner,&owner->Value);owner->Value.setValue("published");source.retirePropertyCopies(1000,16*1024*1024);
                    check("retirement preserves detached New object ownership and value inverse",Tracked::destroyed==before&&source.hasObject(owner)&&source.applyChecked(doc,false)&&owner->Value.getValue()==std::string("original"));}
                check("retired detached New object is still destroyed exactly once",Tracked::destroyed==before+1);
            }
            {
                auto owner=std::make_unique<Tracked>();const unsigned before=Tracked::destroyed;
                {Journal source;source.addObjectDel(owner.get());source.addObjectChange(owner.get(),&owner->Value);source.retirePropertyCopies(1000,16*1024*1024);}
                check("retirement never acquires or destroys externally owned Del object",Tracked::destroyed==before);
            }
            {
                Tracked owner;Journal source;source.addObjectChange(&owner,&owner.Value);
                source.retirePropertyCopies(1000,16*1024*1024);
                source.addObjectChange(&owner,&owner.Later);owner.Later.setValue("later-published");
                check("native changes appended after retirement remain executable",source.applyChecked(doc,false)&&owner.Later.getValue()==std::string("later-original"));
                owner.Later.setValue("later-again");source.retirePropertyCopies(1000,16*1024*1024);
                check("repeated retirement preserves both frozen and appended originals",source.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original")&&owner.Later.getValue()==std::string("later-original"));
                check("repeated retirement destroys appended native Property copies",CopyString::liveCopies==0);
            }
            {
                Tracked owner;Journal source;source.addObjectChange(&owner,&owner.Value);
                source.retirePropertyCopies(1000,16*1024*1024);bool unnecessaryCopy=false;
                CopyString::onCopy=[] {throw std::runtime_error("frozen first write must not be copied again");};
                try{source.addObjectChange(&owner,&owner.Value);}catch(const std::exception&){unnecessaryCopy=true;}
                CopyString::onCopy={};owner.Value.setValue("published-again");
                check("frozen first-write recapture retains original without Copy",!unnecessaryCopy&&source.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
            }
            {
                Tracked owner;Journal caller,replay;caller.addObjectChange(&owner,&owner.Value);
                caller.retirePropertyCopies(1000,16*1024*1024);
                replay.addObjectChange(&owner,&owner.Later);owner.Later.setValue("merged-later");
                auto stage=caller.preparePropertyMerge(replay,1000,16*1024*1024);
                check("merge into retired record preserves native additional values",caller.commitPropertyMerge(*stage)&&caller.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original")&&owner.Later.getValue()==std::string("later-original"));
            }
            {
                Tracked owner;Journal source;auto* added=owner.addDynamicProperty("App::PropertyString","CanceledAdded");
                source.addOrRemoveProperty(&owner,added,true);source.retirePropertyCopies(1000,16*1024*1024);
                source.addOrRemoveProperty(&owner,added,false);owner.removeDynamicProperty("CanceledAdded");
                check("retired dynamic addition cancellation remains an executable no-op",source.applyChecked(doc,false)&&!owner.getDynamicPropertyByName("CanceledAdded"));
            }
            {
                Tracked owner;Journal source,caller;source.addObjectChange(&owner,&owner.Value);owner.Value.setValue("published");
                source.retirePropertyCopies(1000,16*1024*1024);
                auto stage=caller.preparePropertyMerge(source,1000,16*1024*1024);
                check("merge FROM retired source preserves native original value",caller.commitPropertyMerge(*stage)&&caller.applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
            }
            {
                Tracked owner;Journal source;source.addObjectChange(&owner,&owner.Value);owner.Value.setValue("published");
                source.retirePropertyCopies(1000,16*1024*1024);
                auto recovery=source.preparePropertyRecovery(1000,16*1024*1024);
                check("recovery FROM retired source preserves native original value",recovery->applyChecked(doc,false)&&owner.Value.getValue()==std::string("original"));
            }
            {
                Tracked owner;Journal source;source.addObjectChange(&owner,&owner.Value);owner.Value.setValue("published");
                source.retirePropertyCopies(1000,16*1024*1024);
                auto inverse=source.preparePropertyInverse(1000,16*1024*1024);
                check("independent inverse FROM retired source preserves native original value",inverse->applyChecked(doc)&&owner.Value.getValue()==std::string("original"));
            }
        }
        CopyString::onCopy={};return checks;
    }
}
PyObject* run(PyObject*,PyObject* args){
    const char* name;if(!PyArg_ParseTuple(args,"s",&name))return nullptr;
    auto* doc=App::GetApplication().getDocument(name);if(!doc){PyErr_SetString(PyExc_ValueError,"Live test document required");return nullptr;}
    try {
        auto checks=exercise<App::Transaction>(*doc);auto* list=PyList_New(checks.size());if(!list)return nullptr;
        for(std::size_t i=0;i<checks.size();++i){auto* row=Py_BuildValue("{s:s,s:O}","name",checks[i].name.c_str(),"passed",checks[i].passed?Py_True:Py_False);if(!row){Py_DECREF(list);return nullptr;}PyList_SET_ITEM(list,i,row);}return list;
    }catch(const std::exception& error){SavepointTest::CopyString::onCopy={};SavepointTest::CopyString::growCopy=false;PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
PyMethodDef methods[]={{"run",run,METH_VARARGS,nullptr},{nullptr,nullptr,0,nullptr}};
PyModuleDef module={PyModuleDef_HEAD_INIT,"OM9PropertyMerge",nullptr,-1,methods};
}
PyMODINIT_FUNC PyInit_OM9PropertyMerge(){
    SavepointTest::Tracked::init();
    static App::TransactionProducer<App::TransactionObject> producer(SavepointTest::Tracked::getClassTypeId());
    return PyModule_Create(&module);
}
