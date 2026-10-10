// SPDX-License-Identifier: LGPL-2.1-or-later
#include "LayerDocumentAdapter.h"
#include "LayerExchangeAdapter.h"
#include "CoreNotes.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <Base/Console.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Gui/ViewProviderGeometryObject.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <QCoreApplication>
#include <QThread>
#include <map>
#include <stdexcept>
#include <vector>
#include <cmath>
#include <algorithm>
#include <set>

namespace OpenMatrix9Gui {
// Native API exception: supplying the persisted FreeCAD type/view-provider name
// requires a DocumentObject subclass. It owns no layer decisions or geometry.
class LayerDocumentObject final:public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::LayerDocumentObject);
public:
    LayerDocumentObject()=default;
    const char* getViewProviderName() const override { return "Gui::ViewProviderDocumentObject"; }
};
PROPERTY_SOURCE(OpenMatrix9Gui::LayerDocumentObject,App::DocumentObject)
namespace { void initializeNativeLayerObserver(); }
void registerLayerDocumentTypes() { LayerDocumentObject::init();initializeNativeLayerObserver(); }
namespace {
using namespace ThreeDm;
constexpr const char* markerName="OM9LayerDocumentVersion";
constexpr const char* snapshotName="OM9LayerSnapshot";
std::map<std::string,std::size_t> receivingScopes;
std::string documentKey(const App::Document& doc){return std::string(doc.getName())+":"+doc.Uid.getValueStr();}
void requireStatus(std::uint32_t code) {
    if(code) throw std::runtime_error("Layer controller rejected: Rust code "+std::to_string(code));
}
Om9LayerByteView bytes(const std::string& text) {
    return {reinterpret_cast<const unsigned char*>(text.data()),text.size()};
}
void requireGuiThread() {
    auto* app=QCoreApplication::instance();
    if(!app || QThread::currentThread()!=app->thread()) throw std::runtime_error("Layer controller requires GUI thread");
}
void requireEditable(App::Document& doc) {
    requireGuiThread();
    auto* gui=Gui::Application::Instance->getDocument(&doc);
    if(App::GetApplication().getActiveDocument()!=&doc || doc.isReadOnlyFile() || !gui || gui->getInEdit() || !Gui::Control().isAllowedAlterDocument(&doc))
        throw std::runtime_error("Layer document is not editable");
    if(doc.hasPendingTransaction() || doc.getBookedTransactionID()!=0 || App::GetApplication().getGlobalTransaction()!=0 || doc.isPerformingTransaction())
        throw std::runtime_error("Layer controller refuses an unrelated transaction");
}
template<class T> T* property(App::PropertyContainer& container,const char* name) {
    auto* raw=container.getPropertyByName(name);
    if(!raw) return nullptr;
    auto* typed=dynamic_cast<T*>(raw);
    if(!typed) throw std::runtime_error(std::string("Layer property has incompatible type: ")+name);
    return typed;
}
template<class T> T& ensureProperty(App::PropertyContainer& container,const char* name,const char* type) {
    if(auto* existing=property<T>(container,name)) return *existing;
    auto* created=dynamic_cast<T*>(container.addDynamicProperty(type,name,"OpenMatrix9 Layers",nullptr,
        App::Prop_NoRecompute|App::Prop_ReadOnly|App::Prop_Hidden,true,true));
    if(!created) throw std::runtime_error(std::string("Cannot create layer property: ")+name);
    return *created;
}
App::DocumentObject* storage(App::Document& doc) {
    App::DocumentObject* result=nullptr;
    for(auto* object:doc.getObjects()) {
        auto* marker=property<App::PropertyString>(*object,markerName);
        const bool nativeStorage=object->isDerivedFrom(LayerDocumentObject::getClassTypeId());
        if(!marker) {
            if(nativeStorage)throw std::runtime_error("Registered layer storage has no version marker");
            continue;
        }
        if(!nativeStorage)throw std::runtime_error(std::string("Model object uses a reserved layer storage marker: ")+object->getNameInDocument());
        if(std::string(marker->getValue())!="1") throw std::runtime_error("Unsupported layer document version");
        // Registered type alone does not justify omitting an augmented object.
        // This factory has exactly two native metadata fields. Unknown dynamic
        // data, including geometry/retained payload or group links, is an
        // explicit whole-document preflight error, never a silent model skip.
        for(const auto& name:object->getDynamicPropertyNames())
            if(name!=markerName && name!=snapshotName)
                throw std::runtime_error("Layer metadata contains unsupported model payload: "+name);
        if(result) throw std::runtime_error("Duplicate layer document storage");
        result=object;
    }
    return result;
}
NativeLayerSnapshot readState(App::Document& doc) {
    auto* owner=storage(doc);
    if(!owner) throw std::runtime_error("Document has no canonical layer state");
    auto* payload=property<App::PropertyString>(*owner,snapshotName);
    if(!payload) throw std::runtime_error("Layer document storage is incomplete");
    return nativeLayerSnapshotFromJson(payload->getValue());
}
struct Binding { App::DocumentObject* object; Gui::ViewProviderDocumentObject* view; NativeObjectLayerRow row; };
std::string nativeLayerObjectId(App::DocumentObject& object) {
    auto* logical=property<App::PropertyString>(object,"OM9LayerObjectId");
    auto* anchor=property<App::PropertyString>(object,"OM9LayerHostName");
    // FreeCAD's internal name is immutable. A copied dynamic property still
    // names the old native object and is not proof of this object's binding.
    const std::string name=object.getNameInDocument();
    if(anchor && std::string(anchor->getValue())!=name)return name;
    return logical?logical->getValue():name;
}
std::vector<Binding> bindObjects(App::Document& doc,std::uint64_t snapshot) {
    auto* gui=Gui::Application::Instance->getDocument(&doc);
    if(!gui) throw std::runtime_error("Missing GUI layer document");
    std::map<std::string,App::DocumentObject*> actual;
    auto* metadata=storage(doc);
    for(auto* object:doc.getObjects()) {
        if(object==metadata || CoreNotes::isStorageObject(object)) continue;
        const auto id=nativeLayerObjectId(*object);
        if(!actual.emplace(id,object).second) throw std::runtime_error("Duplicate native layer object binding");
    }
    auto rows=nativeObjectLayersFromSnapshot(snapshot);
    if(actual.size()!=rows.size()) throw std::runtime_error("Canonical layer state does not cover document objects");
    std::vector<Binding> result;
    result.reserve(rows.size());
    for(const auto& row:rows) {
        auto found=actual.find(row.id);
        if(found==actual.end()) throw std::runtime_error("Missing native layer object binding");
        auto* object=found->second;
        auto* view=dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(object));
        if(!view) throw std::runtime_error("Missing native layer view provider");
        // Check property types before any mutation/transaction.
        property<App::PropertyString>(*object,"OM9LayerId");
        property<App::PropertyBool>(*object,"OM9Locked");
        result.push_back({object,view,row});
    }
    return result;
}
class OwnedTransaction final {
    App::Document& doc_;int id_=0;bool done_=false;
public:
    explicit OwnedTransaction(App::Document& doc):doc_(doc) {
        id_=doc_.openTransaction(std::string("OM9 Layer state"));
        if(id_==0 || doc_.getBookedTransactionID()!=id_) throw std::runtime_error("Cannot own layer transaction");
    }
    bool owns() const {
        return doc_.getBookedTransactionID()==id_ && (!doc_.hasPendingTransaction() || doc_.getTransactionID(true)==id_);
    }
    ~OwnedTransaction() {
        if(!done_ && owns()) { try { doc_.abortTransaction(); } catch(...) {} }
    }
    void commit() {
        if(!owns()) throw std::runtime_error("Layer transaction ownership changed");
        doc_.commitTransaction();done_=true;
    }
};
void project(const std::vector<Binding>& bindings,std::uint64_t snapshot) {
    for(const auto& binding:bindings) {
        Om9LayerEffective state{};
        requireStatus(om9_layer_snapshot_effective(snapshot,bytes(binding.row.id),&state));
        auto& logical=ensureProperty<App::PropertyString>(*binding.object,"OM9LayerObjectId","App::PropertyString");
        if(std::string(logical.getValue())!=binding.row.id) logical.setValue(binding.row.id.c_str());
        auto& anchor=ensureProperty<App::PropertyString>(*binding.object,"OM9LayerHostName","App::PropertyString");
        if(std::string(anchor.getValue())!=binding.object->getNameInDocument())anchor.setValue(binding.object->getNameInDocument());
        auto& layer=ensureProperty<App::PropertyString>(*binding.object,"OM9LayerId","App::PropertyString");
        if(std::string(layer.getValue())!=binding.row.layerId) layer.setValue(binding.row.layerId.c_str());
        auto& locked=ensureProperty<App::PropertyBool>(*binding.object,"OM9Locked","App::PropertyBool");
        if(locked.getValue()!=static_cast<bool>(state.locked)) locked.setValue(state.locked!=0);
        if(binding.view->Visibility.getValue()!=static_cast<bool>(state.visible)) binding.view->Visibility.setValue(state.visible!=0);
        if(auto* geometry=dynamic_cast<Gui::ViewProviderGeometryObject*>(binding.view)) {
            if(geometry->Selectable.getValue()!=static_cast<bool>(state.selectable)) geometry->Selectable.setValue(state.selectable!=0);
            const Base::Color color(state.rgb[0]/255.0F,state.rgb[1]/255.0F,state.rgb[2]/255.0F);
            if(geometry->ShapeAppearance.getDiffuseColor()!=color) geometry->ShapeAppearance.setDiffuseColor(color);
            if(auto* line=property<App::PropertyColor>(*geometry,"LineColor"))if(line->getValue()!=color)line->setValue(color);
        }
    }
}
struct OwnedPlan {
    std::uint64_t handle=0;
    ~OwnedPlan(){if(handle)om9_layer_plan_free(handle);}
};
NativeLayerSnapshot stateOrDefault(App::Document& doc,const std::set<std::string>& excludedNames={}) {
    auto* metadata=storage(doc);
    struct Fact {std::string id,path;bool hasPath,locked,visible;std::array<unsigned char,3> rgb;};
    std::vector<Fact> facts;facts.reserve(doc.countObjects());
    std::vector<std::string> verifiedHelpers;
    auto* gui=Gui::Application::Instance->getDocument(&doc);
    if(!gui)throw std::runtime_error("Missing GUI legacy document");
    for(auto* object:doc.getObjects()) {
        if(object==metadata)continue;
        if(CoreNotes::isStorageObject(object)){verifiedHelpers.push_back(nativeLayerObjectId(*object));continue;}
        if(excludedNames.count(object->getNameInDocument()))continue;
        auto* view=dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(object));
        if(!view)throw std::runtime_error("Missing legacy object view provider");
        auto* path=property<App::PropertyString>(*object,"OM9LayerPath");
        auto* locked=property<App::PropertyBool>(*object,"OM9Locked");
        Base::Color color(180/255.F,180/255.F,180/255.F);
        if(auto* geometry=dynamic_cast<Gui::ViewProviderGeometryObject*>(view))color=geometry->ShapeAppearance.getDiffuseColor();
        // Native topology kind is O(1), not BRep serialization/point-array access.
        // Curves display LineColor; solids/mesh use their diffuse object color.
        if(auto* shape=property<Part::PropertyPartShape>(*object,"Shape")) {
            const auto& value=shape->getValue();
            if(!value.IsNull() && (value.ShapeType()==TopAbs_EDGE || value.ShapeType()==TopAbs_WIRE))
                if(auto* line=property<App::PropertyColor>(*view,"LineColor"))color=line->getValue();
        }
        std::array<unsigned char,3> rgb{};
        const std::array<float,3> channels{color.r,color.g,color.b};
        for(std::size_t i=0;i<3;++i) {
            const auto channel=channels[i];
            if(!std::isfinite(channel) || channel<0 || channel>1)throw std::runtime_error("Invalid legacy object color");
            rgb[i]=static_cast<unsigned char>(std::lround(channel*255));
        }
        facts.push_back({nativeLayerObjectId(*object),path?path->getValue():"",path!=nullptr,
            locked && locked->getValue(),view->Visibility.getValue(),rgb});
    }
    std::vector<Om9LayerLegacyObjectView> views;views.reserve(facts.size());
    for(const auto& fact:facts) {
        Om9LayerLegacyObjectView view{};view.id=bytes(fact.id);view.path=bytes(fact.path);
        std::copy(fact.rgb.begin(),fact.rgb.end(),view.rgb);view.locked=fact.locked;view.visible=fact.visible;view.path_present=fact.hasPath;
        views.push_back(view);
    }
    std::uint64_t handle=0;
    if(metadata) {
        auto current=readState(doc);OwnedPlan plan;
        if(!verifiedHelpers.empty()) {
            std::vector<Om9LayerByteView> ids;ids.reserve(verifiedHelpers.size());
            for(const auto& id:verifiedHelpers)ids.push_back(bytes(id));
            std::uint64_t filtered=0;
            requireStatus(om9_layer_snapshot_filter_metadata(current.get(),ids.data(),ids.size(),&filtered));
            current=NativeLayerSnapshot(filtered);
        }
        requireStatus(om9_layer_document_reconcile(current.get(),views.data(),views.size(),&plan.handle));
        requireStatus(om9_layer_plan_after_snapshot(plan.handle,&handle));
        return NativeLayerSnapshot(handle);
    }
    const std::string id=doc.Uid.getValueStr();
    requireStatus(om9_layer_document_legacy(bytes(id),views.data(),views.size(),&handle));
    return NativeLayerSnapshot(handle);
}
App::PropertyString& writeState(App::Document& doc,const std::string& canonical) {
    auto* owner=storage(doc);
    if(!owner) {
        owner=doc.addObject("OpenMatrix9Gui::LayerDocumentObject","OM9LayerState");
        if(!owner) throw std::runtime_error("Cannot create layer storage object");
        ensureProperty<App::PropertyString>(*owner,markerName,"App::PropertyString").setValue("1");
        auto* gui=Gui::Application::Instance->getDocument(&doc);
        auto* view=dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(owner));
        if(!view) throw std::runtime_error("Cannot create layer storage view provider");
        view->ShowInTree.setValue(false);view->Visibility.setValue(false);
    }
    auto& payload=ensureProperty<App::PropertyString>(*owner,snapshotName,"App::PropertyString");
    payload.setValue(canonical.c_str());
    return payload;
}
std::string applyDocumentPlan(App::Document& doc,std::uint64_t before,std::uint64_t plan) {
    std::uint64_t handle=0;
    requireStatus(om9_layer_plan_after_snapshot(plan,&handle));
    NativeLayerSnapshot after(handle);
    const auto canonical=nativeLayerSnapshotJson(after.get());
    auto bindings=bindObjects(doc,after.get());
    if(canonical==nativeLayerSnapshotJson(before)) return canonical;
    auto current=stateOrDefault(doc);
    requireStatus(om9_layer_plan_validate(plan,current.get()));
    OwnedTransaction transaction(doc);
    writeState(doc,canonical);
    project(bindings,after.get());
    transaction.commit();
    return canonical;
}
void supplementNativeCreation(App::Document& doc,int transactionId,const std::set<std::string>& createdNames) {
    requireGuiThread();
    if(doc.isPerformingTransaction() || !doc.hasPendingTransaction() || doc.getTransactionID(true)!=transactionId)return;
    auto* metadata=storage(doc);
    std::string rawBefore;
    std::set<std::string> known;
    if(metadata) {
        auto state=readState(doc);rawBefore=nativeLayerSnapshotJson(state.get());
        for(const auto& row:nativeObjectLayersFromSnapshot(state.get()))known.insert(row.id);
    }
    std::set<std::string> unboundNames;
    for(const auto& name:createdNames) {
        auto* object=doc.getObject(name.c_str());
        if(object && !isLayerStorageObject(object) && !CoreNotes::isStorageObject(object) && !known.count(nativeLayerObjectId(*object)))unboundNames.insert(name);
    }
    // Explicit OM9 transactions already completed their layer plan. Never
    // replace received own flags or ByObject state with a creation default.
    if(unboundNames.empty())return;
    auto before=stateOrDefault(doc,unboundNames);
    std::vector<std::string> live;
    for(auto* object:doc.getObjects())if(object!=metadata && !CoreNotes::isStorageObject(object))live.push_back(nativeLayerObjectId(*object));
    std::vector<Om9LayerByteView> ids;ids.reserve(live.size());for(const auto& id:live)ids.push_back(bytes(id));
    OwnedPlan plan;requireStatus(om9_layer_document_observed(before.get(),ids.data(),ids.size(),&plan.handle));
    std::uint64_t handle=0;requireStatus(om9_layer_plan_after_snapshot(plan.handle,&handle));
    NativeLayerSnapshot after(handle);const auto canonical=nativeLayerSnapshotJson(after.get());auto bindings=bindObjects(doc,after.get());
    requireStatus(om9_layer_plan_validate(plan.handle,before.get()));
    if(metadata) {auto current=readState(doc);if(nativeLayerSnapshotJson(current.get())!=rawBefore)throw std::runtime_error("Native creation layer metadata became stale");}
    else if(storage(doc))throw std::runtime_error("Native creation layer metadata changed");
    if(!doc.hasPendingTransaction() || doc.getTransactionID(true)!=transactionId || doc.isPerformingTransaction())throw std::runtime_error("Native creation transaction ownership changed");
    // Shared-SDK runtime proof pins this before-close write inside the native
    // caller's existing Undo. The observer never opens/commits/aborts a record.
    writeState(doc,canonical);project(bindings,after.get());
}
void hookError(const char* message) noexcept {
    try {Base::Console().error("OM9 native layer lifecycle: %s\n",message);}catch(...){}
}
class NativeLayerObserver final {
    using Frames=std::map<int,std::set<std::string>>;
    std::map<std::string,Frames> created;
    bool supplementing=false;
    fastsignals::scoped_connection newObject,beforeClose,committed,aborted,deleted;
    static std::string key(const App::Document& doc){return documentKey(doc);}
    struct Guard {
        bool& value;
        explicit Guard(bool& flag):value(flag){value=true;}
        ~Guard(){value=false;}
    };
    void record(const App::DocumentObject& object) noexcept {
        try {
            auto* doc=object.getDocument();
            if(supplementing || !doc || doc->isPerformingTransaction() || isLayerStorageObject(&object) || CoreNotes::isStorageObject(&object) || !doc->hasPendingTransaction())return;
            if(receivingScopes.count(key(*doc)))return;
            // Native copy/merge imports set both Restoring and Importing. They
            // create new model objects; FCStd reopen sets Restoring alone.
            if(doc->testStatus(App::Document::Restoring) && !doc->testStatus(App::Document::Importing))return;
            created[key(*doc)][doc->getTransactionID(true)].insert(object.getNameInDocument());
        }catch(const std::exception& error){hookError(error.what());}catch(...){hookError("Cannot observe native object creation");}
    }
    void close(bool abort) noexcept {
        // TransactionSignaller invokes this from its constructor. An exception
        // must never escape and corrupt FreeCAD's nested signaller accounting.
        try {
            if(supplementing || abort)return;
            auto documents=App::GetApplication().getDocuments();
            std::set<int> closing;
            for(auto* doc:documents)if(doc->hasPendingTransaction() && doc->transacting() && !doc->isPerformingTransaction())closing.insert(doc->getTransactionID(true));
            Guard guard(supplementing);
            for(auto* doc:documents) {
                if(!doc->hasPendingTransaction() || doc->isPerformingTransaction())continue;
                const int id=doc->getTransactionID(true);
                if(!closing.count(id))continue;
                auto found=created.find(key(*doc));if(found==created.end())continue;
                auto frame=found->second.find(id);if(frame==found->second.end())continue;
                supplementNativeCreation(*doc,id,frame->second);
            }
        }catch(const std::exception& error){hookError(error.what());}catch(...){hookError("Cannot supplement native creation transaction");}
    }
    void clear(const App::Document& doc) noexcept {
        try {created.erase(key(doc));}catch(...){hookError("Cannot clear native creation observation");}
    }
public:
    NativeLayerObserver() {
        auto& app=App::GetApplication();
        newObject=app.signalNewObject.connect([this](const App::DocumentObject& object){record(object);});
        beforeClose=app.signalBeforeCloseTransaction.connect([this](bool abort){close(abort);});
        committed=app.signalCommitTransaction.connect([this](const App::Document& doc){clear(doc);});
        aborted=app.signalAbortTransaction.connect([this](const App::Document& doc){clear(doc);});
        deleted=app.signalDeleteDocument.connect([this](const App::Document& doc){clear(doc);});
    }
};
void initializeNativeLayerObserver(){static NativeLayerObserver observer;}
} // namespace

void initializeLayerDocument(App::Document& doc,const std::string& json) {
    requireEditable(doc);
    if(storage(doc)) throw std::runtime_error("Canonical layer document already initialized");
    auto state=nativeLayerSnapshotFromJson(json);
    const auto canonical=nativeLayerSnapshotJson(state.get());
    auto bindings=bindObjects(doc,state.get());
    OwnedTransaction transaction(doc);
    writeState(doc,canonical);
    project(bindings,state.get());
    transaction.commit();
}
std::string layerDocumentSnapshot(App::Document& doc) {
    requireGuiThread();
    auto state=stateOrDefault(doc);
    bindObjects(doc,state.get());
    return nativeLayerSnapshotJson(state.get());
}
std::string layerDocumentCommand(App::Document& doc,const std::string& json) {
    requireEditable(doc);
    auto before=stateOrDefault(doc);
    OwnedPlan plan;
    requireStatus(om9_layer_document_plan(before.get(),bytes(json),&plan.handle));
    return applyDocumentPlan(doc,before.get(),plan.handle);
}
std::string layerDocumentText(App::Document& doc,const std::string& text,const std::vector<std::string>& selected) {
    requireEditable(doc);
    auto before=stateOrDefault(doc);
    std::vector<Om9LayerByteView> ids;ids.reserve(selected.size());
    for(const auto& id:selected) ids.push_back(bytes(id));
    OwnedPlan plan;
    requireStatus(om9_layer_document_text(before.get(),bytes(text),ids.data(),ids.size(),&plan.handle));
    return applyDocumentPlan(doc,before.get(),plan.handle);
}
std::string layerDocumentPanel(App::Document& doc) {
    requireGuiThread();auto state=stateOrDefault(doc);bindObjects(doc,state.get());
    std::size_t size=0;
    auto status=om9_layer_document_panel(state.get(),nullptr,0,&size);
    if(status!=15) requireStatus(status);
    std::string json(size,'\0');
    requireStatus(om9_layer_document_panel(state.get(),reinterpret_cast<unsigned char*>(json.data()),json.size(),&size));
    return json;
}
LayerExportSelection layerDocumentExportSelection(App::Document& doc,const std::vector<std::string>& names) {
    requireGuiThread();
    auto* gui=Gui::Application::Instance->getDocument(&doc);
    if(App::GetApplication().getActiveDocument()!=&doc || !gui || gui->isAboutToClose() || gui->getInEdit() || !Gui::Control().isAllowedAlterDocument(&doc))
        throw std::runtime_error("Layer export document is not available");
    auto state=stateOrDefault(doc);bindObjects(doc,state.get());
    LayerExportSelection result;result.objectIds.reserve(names.size());
    for(const auto& name:names) {
        auto* object=doc.getObject(name.c_str());
        if(!object || isLayerStorageObject(object) || CoreNotes::isStorageObject(object))throw std::runtime_error("Select native model objects for layer export");
        result.objectIds.push_back(nativeLayerObjectId(*object));
    }
    std::vector<Om9LayerByteView> ids;ids.reserve(result.objectIds.size());for(const auto& id:result.objectIds)ids.push_back(bytes(id));
    // Export includes locked/hidden objects. A zero-object Rust subset is a
    // palette-only transfer; edit/delete eligibility is not reused for export.
    if(!ids.empty())requireStatus(om9_layer_snapshot_can_mutate(state.get(),ids.data(),ids.size(),5,{}));
    std::uint64_t handle=0;requireStatus(om9_layer_snapshot_subset(state.get(),ids.data(),ids.size(),&handle));
    NativeLayerSnapshot subset(handle);result.snapshotJson=nativeLayerSnapshotJson(subset.get());return result;
}
void validateLayerDocumentExportSelection(App::Document& doc,const std::vector<std::string>& names,const std::string& expectedJson) {
    auto expected=nativeLayerSnapshotFromJson(expectedJson);auto current=layerDocumentExportSelection(doc,names);
    if(current.snapshotJson!=nativeLayerSnapshotJson(expected.get()))throw std::runtime_error("Layer export snapshot changed during geometry staging");
}
bool isLayerStorageObject(const App::DocumentObject* object) {
    return object && object->isDerivedFrom(LayerDocumentObject::getClassTypeId());
}
void requireLayerGeometryEditable(App::Document& doc){requireEditable(doc);}
LayerNativeReceiveScope::LayerNativeReceiveScope(App::Document& doc):identity(documentKey(doc)) {
    requireGuiThread();++receivingScopes[identity];
}
LayerNativeReceiveScope::~LayerNativeReceiveScope() {
    auto found=receivingScopes.find(identity);
    if(found!=receivingScopes.end() && --found->second==0)receivingScopes.erase(found);
}
std::uint64_t layerMutationGeneration(App::Document& doc,const std::vector<std::string>& names,std::uint32_t operation) {
    requireGuiThread();auto state=stateOrDefault(doc);bindObjects(doc,state.get());
    std::vector<std::string> owned;owned.reserve(names.size());
    for(const auto& name:names) {
        auto* object=doc.getObject(name.c_str());
        if(!object || isLayerStorageObject(object) || CoreNotes::isStorageObject(object))throw std::runtime_error("Missing layer mutation object");
        owned.emplace_back(nativeLayerObjectId(*object));
    }
    std::vector<Om9LayerByteView> ids;ids.reserve(owned.size());for(const auto& id:owned)ids.push_back(bytes(id));
    requireStatus(om9_layer_snapshot_can_mutate(state.get(),ids.data(),ids.size(),operation,{}));
    Om9LayerCounts counts{};requireStatus(om9_layer_snapshot_counts(state.get(),&counts));
    return counts.generation;
}
bool ownsLayerGeometryTransaction(App::Document& doc,int id) {
    return id!=0 && doc.getBookedTransactionID()==id && (!doc.hasPendingTransaction() || doc.getTransactionID(true)==id);
}
LayerDocumentReceivePlan::LayerDocumentReceivePlan(App::Document& document,const std::string& json,const std::vector<std::string>& sourceIds):doc(document),identity(document.Uid.getValueStr()) {
    requireEditable(doc);
    auto incoming=nativeLayerSnapshotFromJson(json);
    Om9LayerCounts counts{};requireStatus(om9_layer_snapshot_counts(incoming.get(),&counts));
    if(counts.object_count!=sourceIds.size())throw std::runtime_error("Prepared geometry does not cover source layer bindings");
    auto current=stateOrDefault(doc);bindObjects(doc,current.get());
    hadState=storage(doc)!=nullptr;
    if(hadState)canonicalBefore=property<App::PropertyString>(*storage(doc),snapshotName)->getValue();
    std::vector<Om9LayerByteView> ids;ids.reserve(sourceIds.size());for(const auto& id:sourceIds)ids.push_back(bytes(id));
    OwnedPlan planned;requireStatus(om9_layer_plan_receive(incoming.get(),current.get(),1,ids.data(),ids.size(),&planned.handle));
    std::vector<std::string> mappings;mappings.reserve(sourceIds.size());
    for(const auto& id:sourceIds) {
        std::size_t length=0;const auto status=om9_layer_plan_map(planned.handle,2,bytes(id),nullptr,0,&length);
        if(status!=15)requireStatus(status);
        std::string mapped(length,'\0');requireStatus(om9_layer_plan_map(planned.handle,2,bytes(id),reinterpret_cast<unsigned char*>(mapped.data()),mapped.size(),&length));
        mappings.push_back(std::move(mapped));
    }
    std::uint64_t afterHandle=0;requireStatus(om9_layer_plan_after_snapshot(planned.handle,&afterHandle));
    NativeLayerSnapshot result(afterHandle);
    mappedIds=std::move(mappings);before=current.release();after=result.release();plan=planned.handle;planned.handle=0;
}
LayerDocumentReceivePlan::~LayerDocumentReceivePlan(){if(before)om9_layer_snapshot_free(before);if(after)om9_layer_snapshot_free(after);if(plan)om9_layer_plan_free(plan);}
void LayerDocumentReceivePlan::finish(int transactionId,const std::vector<std::string>& names) {
    requireGuiThread();
    if(finished || identity!=doc.Uid.getValueStr() || !ownsLayerGeometryTransaction(doc,transactionId) || names.size()!=mappedIds.size())
        throw std::runtime_error("Layer receive transaction or binding became stale");
    std::set<std::string> added(names.begin(),names.end());
    if(added.size()!=names.size())throw std::runtime_error("Received geometry duplicates a native object");
    auto* metadata=storage(doc);
    if(hadState) {
        if(!metadata || std::string(property<App::PropertyString>(*metadata,snapshotName)->getValue())!=canonicalBefore)
            throw std::runtime_error("Layer metadata changed during receive");
    }else if(metadata)throw std::runtime_error("Layer metadata appeared during receive");
    // Reconcile only the old inventory. Unexpected additions/removals or a
    // foreign layer change cannot be hidden by the incoming source IDs.
    auto current=stateOrDefault(doc,added);requireStatus(om9_layer_plan_validate(plan,current.get()));
    std::set<std::string> oldNames;
    for(auto* object:doc.getObjects())if(object!=metadata && !CoreNotes::isStorageObject(object) && !added.count(object->getNameInDocument()))oldNames.insert(object->getNameInDocument());
    std::vector<App::DocumentObject*> objects;objects.reserve(names.size());
    for(const auto& name:names) {
        auto* object=doc.getObject(name.c_str());
        if(!object || object==metadata || CoreNotes::isStorageObject(object) || oldNames.count(name))throw std::runtime_error("Missing received geometry object");
        property<App::PropertyString>(*object,"OM9LayerObjectId");property<App::PropertyString>(*object,"OM9LayerHostName");
        objects.push_back(object);
    }
    for(std::size_t i=0;i<objects.size();++i) {
        ensureProperty<App::PropertyString>(*objects[i],"OM9LayerObjectId","App::PropertyString").setValue(mappedIds[i].c_str());
        ensureProperty<App::PropertyString>(*objects[i],"OM9LayerHostName","App::PropertyString").setValue(names[i].c_str());
    }
    auto bindings=bindObjects(doc,after);const auto canonical=nativeLayerSnapshotJson(after);
    writeState(doc,canonical);project(bindings,after);
    if(!ownsLayerGeometryTransaction(doc,transactionId))throw std::runtime_error("Layer receive ownership changed during projection");
    finished=true;
}
LayerGeometryTransaction::LayerGeometryTransaction(App::Document& document,int id):doc(document),identity(document.Uid.getValueStr()),transactionId(id) {
    requireGuiThread();
    if(!ownsLayerGeometryTransaction(doc,id))throw std::runtime_error("Layer creation requires caller-owned transaction");
    hadState=storage(doc)!=nullptr;
    if(hadState) {
        auto* payload=property<App::PropertyString>(*storage(doc),snapshotName);
        if(!payload)throw std::runtime_error("Layer document storage is incomplete");
        canonicalBefore=payload->getValue();
    }
    auto state=stateOrDefault(doc);bindObjects(doc,state.get());
    before=state.release();
}
LayerGeometryTransaction::~LayerGeometryTransaction(){if(before)om9_layer_snapshot_free(before);}
void LayerGeometryTransaction::finish() {
    requireGuiThread();
    if(finished || doc.Uid.getValueStr()!=identity || !ownsLayerGeometryTransaction(doc,transactionId))
        throw std::runtime_error("Layer geometry transaction became stale");
    std::vector<std::string> owned;
    auto* metadata=storage(doc);
    for(auto* object:doc.getObjects())if(object!=metadata && !CoreNotes::isStorageObject(object)) {
        owned.emplace_back(nativeLayerObjectId(*object));
    }
    std::vector<Om9LayerByteView> ids;ids.reserve(owned.size());for(const auto& id:owned)ids.push_back(bytes(id));
    OwnedPlan plan;requireStatus(om9_layer_document_observed(before,ids.data(),ids.size(),&plan.handle));
    if(hadState) {
        if(!metadata)throw std::runtime_error("Layer metadata removed during geometry creation");
        auto* payload=property<App::PropertyString>(*metadata,snapshotName);
        if(!payload || std::string(payload->getValue())!=canonicalBefore)throw std::runtime_error("Layer metadata changed during geometry creation");
        requireStatus(om9_layer_plan_validate(plan.handle,before));
    }
    else if(metadata)throw std::runtime_error("Layer metadata changed during geometry creation");
    std::uint64_t handle=0;requireStatus(om9_layer_plan_after_snapshot(plan.handle,&handle));
    NativeLayerSnapshot after(handle);const auto canonical=nativeLayerSnapshotJson(after.get());auto bindings=bindObjects(doc,after.get());
    writeState(doc,canonical);project(bindings,after.get());
    if(!ownsLayerGeometryTransaction(doc,transactionId))throw std::runtime_error("Layer geometry transaction ownership changed");
    finished=true;
}
} // namespace OpenMatrix9Gui
