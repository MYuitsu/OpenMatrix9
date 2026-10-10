// SPDX-License-Identifier: LGPL-2.1-or-later
#include "LayerCollectionAdapter.h"
#include "LayerCollectionAbi.h"
#include "LayerDocumentAdapter.h"
#include "CoreNotes.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <App/PropertyLinks.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <QCoreApplication>
#include <QThread>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <set>
#include <stdexcept>
#include <limits>

namespace OpenMatrix9Gui {
namespace {
bool derived(const App::DocumentObject& obj,const char* name) {return obj.isDerivedFrom(Base::Type::fromName(name));}
Om9LayerByteView bytes(const std::string& text){return {reinterpret_cast<const unsigned char*>(text.data()),text.size()};}
void requireStatus(std::uint32_t code){if(code)throw std::runtime_error("Layer collection rejected: Rust code "+std::to_string(code));}
void charge(std::size_t& total,std::size_t amount){if(amount>std::numeric_limits<std::size_t>::max()-total)throw std::runtime_error("Layer collection size overflow");total+=amount;requireStatus(om9_layer_collection_check_size(total));}
struct Fact {std::string id,label,nativeType;std::uint32_t role=7;std::vector<std::string> children;std::vector<Om9LayerByteView> views;};
struct CollectionHandle {std::uint64_t id=0;~CollectionHandle(){if(id)om9_layer_collection_free(id);}};
bool cloudFields(const App::DocumentObject& object) {
    auto* schema=dynamic_cast<App::PropertyInteger*>(object.getPropertyByName("OM9PointCloudSchema"));
    return schema && schema->getValue()==1 && derived(object,"App::FeaturePython") &&
        dynamic_cast<App::PropertyVectorList*>(object.getPropertyByName("OM9CloudPoints")) && dynamic_cast<App::PropertyVectorList*>(object.getPropertyByName("OM9CloudNormals")) &&
        dynamic_cast<App::PropertyIntegerList*>(object.getPropertyByName("OM9CloudRGBA")) && dynamic_cast<App::PropertyFloatList*>(object.getPropertyByName("OM9CloudValues")) &&
        dynamic_cast<App::PropertyFloatList*>(object.getPropertyByName("OM9CloudPlane")) && dynamic_cast<App::PropertyBool*>(object.getPropertyByName("OM9CloudOrdered")) && dynamic_cast<App::PropertyBool*>(object.getPropertyByName("OM9CloudHasPlane"));
}
bool geometryProperties(const App::DocumentObject& object){return object.getPropertyByName("Shape") || object.getPropertyByName("Mesh") || object.getPropertyByName("OM9PointCloudSchema");}
std::set<const App::DocumentObject*> nativeOriginHelpers(App::Document& doc) {
    std::set<const App::DocumentObject*> result;
    for(auto* object:doc.getObjects())if(derived(*object,"App::Part") || derived(*object,"PartDesign::Body")) {
        auto* link=dynamic_cast<App::PropertyLink*>(object->getPropertyByName("Origin"));auto* origin=link?link->getValue():nullptr;
        if(!origin || origin->getDocument()!=&doc || !derived(*origin,"App::Origin"))continue;
        result.insert(origin);
        if(auto* features=dynamic_cast<App::PropertyLinkList*>(origin->getPropertyByName("OriginFeatures")))
            for(auto* child:features->getValues())if(child && child->getDocument()==&doc && derived(*child,"App::OriginFeature"))result.insert(child);
    }
    return result;
}
bool originMetadata(const App::DocumentObject& object) {
    if(geometryProperties(object))return false;
    static const std::set<std::string> bindings{"OM9LayerObjectId","OM9LayerHostName","OM9LayerId","OM9Locked"};
    for(const auto& name:object.getDynamicPropertyNames())if(!bindings.count(name))return false;
    return true;
}
}
std::vector<std::string> collectLayerTransferObjects(App::Document& doc,const std::vector<std::string>& selected,std::uint32_t scope) {
    auto* app=QCoreApplication::instance();
    if(!app || QThread::currentThread()!=app->thread() || App::GetApplication().getActiveDocument()!=&doc)throw std::runtime_error("Layer collection requires the active GUI project");
    const auto origins=nativeOriginHelpers(doc);std::vector<Fact> facts;std::size_t total=0;
    for(auto* object:doc.getObjects()) {
        const std::string id(object->getNameInDocument()),label(object->Label.getValue()),kind(object->getTypeId().getName());
        charge(total,sizeof(Fact)+id.size()+label.size()+kind.size());
        Fact fact;fact.id=id;fact.label=label;fact.nativeType=kind;
        auto* shape=dynamic_cast<Part::PropertyPartShape*>(object->getPropertyByName("Shape"));
        auto* mesh=object->getPropertyByName("Mesh");const bool hasCloud=object->getPropertyByName("OM9PointCloudSchema");
        if(isLayerStorageObject(object) || CoreNotes::isStorageObject(object))fact.role=6;
        else if(origins.count(object))fact.role=originMetadata(*object)?6:7;
        // Archive/definition/instance roles need a separately verified native
        // provenance graph. Markers are not proof; do not omit these records.
        else if(object->getPropertyByName("OM9ArchiveMode") || object->getPropertyByName("OM9DefinitionUUID") || object->getPropertyByName("OM9NewDefinitionUUID") || object->getPropertyByName("OM9InstanceMatrix") || object->getPropertyByName("OM9NewInstanceUUID"))fact.role=7;
        else if(shape && !shape->getValue().IsNull() && !mesh && !hasCloud)fact.role=derived(*object,"PartDesign::Body")?3:1;
        else if(mesh && derived(*object,"Mesh::Feature") && !shape && !hasCloud)fact.role=1;
        else if(hasCloud && cloudFields(*object) && !shape && !mesh)fact.role=1;
        else if((derived(*object,"App::DocumentObjectGroup") || derived(*object,"App::Part")) && !geometryProperties(*object))fact.role=2;
        if(fact.role==2 || fact.role==3) {
            auto* group=dynamic_cast<App::PropertyLinkList*>(object->getPropertyByName("Group"));
            if(!group)fact.role=7;
            else for(auto* child:group->getValues()) {
                if(!child || child->getDocument()!=&doc)throw std::runtime_error("Layer collection group contains missing or foreign objects");
                const std::string name=child->getNameInDocument();charge(total,sizeof(std::string)+sizeof(Om9LayerByteView)+name.size());fact.children.push_back(name);
            }
        }
        facts.push_back(std::move(fact));
    }
    std::vector<Om9LayerCollectionNodeView> views;views.reserve(facts.size());
    for(auto& fact:facts){fact.views.reserve(fact.children.size());for(const auto& name:fact.children)fact.views.push_back(bytes(name));views.push_back({bytes(fact.id),bytes(fact.label),bytes(fact.nativeType),fact.views.data(),fact.views.size(),fact.role,0});}
    std::vector<Om9LayerByteView> selection;selection.reserve(selected.size());for(const auto& name:selected)selection.push_back(bytes(name));
    CollectionHandle collection;requireStatus(om9_layer_collection_prepare(views.data(),views.size(),scope,selection.data(),selection.size(),&collection.id));
    std::size_t needed=0;auto status=om9_layer_collection_json(collection.id,nullptr,0,&needed);if(status!=15)requireStatus(status);
    std::string text(needed,'\0');requireStatus(om9_layer_collection_json(collection.id,reinterpret_cast<unsigned char*>(text.data()),text.size(),&needed));
    QJsonParseError error;auto result=QJsonDocument::fromJson(QByteArray::fromStdString(text),&error);
    if(error.error!=QJsonParseError::NoError || !result.isObject())throw std::runtime_error("Invalid owned layer collection result");
    const auto object=result.object();const auto unsupported=object.value("unsupported").toArray();
    if(!unsupported.isEmpty())throw std::runtime_error("Unsupported transfer inventory: "+QJsonDocument(unsupported).toJson(QJsonDocument::Compact).toStdString());
    std::vector<std::string> names;for(const auto& value:object.value("objects").toArray()){if(!value.isString())throw std::runtime_error("Invalid native collection binding");names.push_back(value.toString().toStdString());}
    // This revalidates native bindings and the complete canonical document;
    // registered metadata augmented with payload cannot evade preflight.
    layerDocumentExportSelection(doc,names);
    return names;
}
}
