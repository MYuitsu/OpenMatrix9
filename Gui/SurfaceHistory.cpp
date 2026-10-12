// SPDX-License-Identifier: LGPL-2.1-or-later
#include "SurfaceHistory.h"
#include "HistoryFeature.h"
#include <App/Document.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Base/Exception.h>
#include <Base/Interpreter.h>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonParseError>
#include <algorithm>
#include <cmath>
#include <set>
#include <stdexcept>

namespace {
struct ChangeGuard {
    bool& flag;
    bool previous;
    explicit ChangeGuard(bool& value):flag(value),previous(value){flag=true;}
    ~ChangeGuard(){flag=previous;}
};
double number(const QJsonObject& obj,const char* name,double lo,double hi) {
    const auto value=obj.value(QLatin1String(name));
    if(!value.isDouble()||!std::isfinite(value.toDouble())||value.toDouble()<lo||value.toDouble()>hi)
        throw std::runtime_error(std::string("Invalid Surface History number: ")+name);
    return value.toDouble();
}
unsigned integer(const QJsonObject& obj,const char* name,unsigned lo,unsigned hi) {
    const double value=number(obj,name,lo,hi);
    if(std::floor(value)!=value)throw std::runtime_error(std::string("Invalid Surface History integer: ")+name);
    return static_cast<unsigned>(value);
}
bool boolean(const QJsonObject& obj,const char* name) {
    const auto value=obj.value(QLatin1String(name));
    if(!value.isBool())throw std::runtime_error(std::string("Invalid Surface History boolean: ")+name);
    return value.toBool();
}
void parse(App::Document& doc,const char* text,const App::PropertyLinkSubList& links,
           std::vector<OpenMatrix9Gui::SurfaceInput>& inputs,OpenMatrix9Gui::SurfaceOptions& options) {
    QJsonParseError error;
    const auto document=QJsonDocument::fromJson(QByteArray(text),&error);
    if(error.error!=QJsonParseError::NoError||!document.isObject())throw std::runtime_error("Malformed Surface History settings");
    const auto root=document.object();
    integer(root,"version",1,1);
    options.kind=integer(root,"kind",1,3);
    options.style=integer(root,"style",0,5);
    options.sectionMode=integer(root,"sectionMode",0,2);
    options.pointCount=integer(root,"pointCount",2,256);
    options.frenet=boolean(root,"frenet");
    options.closed=boolean(root,"closed");
    options.maintainHeight=boolean(root,"maintainHeight");
    options.preview=boolean(root,"preview");
    options.history=boolean(root,"history");
    options.historyCommand=boolean(root,"historyCommand");
    options.tolerance=number(root,"tolerance",1e-7,1e6);
    options.continuityA=integer(root,"continuityA",0,2);
    options.continuityB=integer(root,"continuityB",0,2);
    options.matchStart=boolean(root,"matchStart");
    options.matchEnd=boolean(root,"matchEnd");
    const auto slashes=root.value("slashes");
    if(!slashes.isArray()||slashes.toArray().size()>256)throw std::runtime_error("Invalid Surface History slash list");
    for(const auto value:slashes.toArray()){
        if(!value.isArray()||value.toArray().size()!=2)throw std::runtime_error("Invalid Surface History slash pair");
        const auto pair=value.toArray();
        for(const auto coordinate:pair)if(!coordinate.isDouble()||!std::isfinite(coordinate.toDouble())||coordinate.toDouble()<0||coordinate.toDouble()>1)
            throw std::runtime_error("Surface History slash fractions must be between 0 and 1");
        options.slashes.emplace_back(pair[0].toDouble(),pair[1].toDouble());
    }
    const auto records=root.value("inputs");
    if(!records.isArray()||records.toArray().size()>256)throw std::runtime_error("Invalid Surface History input records");
    const auto& objects=links.getValues();
    const auto& subs=links.getSubValues();
    if(objects.size()!=subs.size())throw std::runtime_error("Surface History source reference mismatch");
    std::size_t index=0;
    for(const auto record:records.toArray()){
        if(!record.isObject())throw std::runtime_error("Invalid Surface History input record");
        const auto obj=record.toObject();
        const auto count=integer(obj,"count",1,4096);
        const bool chain=boolean(obj,"chain");
        if(!chain&&count!=1)throw std::runtime_error("Surface History input grouping mismatch");
        if(count>objects.size()-std::min(index,objects.size()))throw std::runtime_error("A Surface History input was deleted or detached");
        OpenMatrix9Gui::SurfaceInput input;
        input.closed=boolean(obj,"closed");
        input.reverse=boolean(obj,"reverse");
        input.seam=number(obj,"seam",0,1);
        if(input.seam>=1)throw std::runtime_error("Surface History seam must be less than 1");
        for(unsigned i=0;i<count;++i,++index){
            auto* source=objects[index];
            if(!source||!source->isAttachedToDocument())throw std::runtime_error("A Surface History input was deleted");
            if(source->getDocument()!=&doc)throw std::runtime_error("Surface History inputs must belong to the same document");
            if(!source->isValid())throw std::runtime_error("A Surface History input has a recompute error");
            if(i==0){input.object=source->getNameInDocument();input.sub=subs[index];}
            if(chain)input.chain.emplace_back(source->getNameInDocument(),subs[index]);
        }
        inputs.push_back(std::move(input));
    }
    if(index!=objects.size())throw std::runtime_error("Surface History source reference count mismatch");
}
std::string pythonError() {
    PyObject *type=nullptr,*value=nullptr,*trace=nullptr;
    PyErr_Fetch(&type,&value,&trace);PyErr_NormalizeException(&type,&value,&trace);
    PyObject* message=value?PyObject_Str(value):nullptr;
    const char* text=message?PyUnicode_AsUTF8(message):nullptr;
    std::string result=text?text:"Surface History Part conversion failed";
    Py_XDECREF(message);Py_XDECREF(type);Py_XDECREF(value);Py_XDECREF(trace);PyErr_Clear();
    return result;
}
}

namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::SurfaceHistory,Part::Feature)

SurfaceHistory::SurfaceHistory() {
    ADD_PROPERTY_TYPE(SourceCurves,(nullptr),"OpenMatrix9",App::Prop_None,"Ordered native source curves and selected subelements");
    ADD_PROPERTY_TYPE(PlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"Native parent frame dependencies");
    ADD_PROPERTY_TYPE(SurfaceOptions,(""),"OpenMatrix9",App::Prop_None,"Versioned surface options and input grouping (JSON)");
    ADD_PROPERTY_TYPE(OM9FeatureId,(""),"OpenMatrix9",App::Prop_ReadOnly,"Exact OpenMatrix9 feature identifier");
    ADD_PROPERTY_TYPE(OM9Command,(""),"OpenMatrix9",App::Prop_ReadOnly,"Surface construction command");
    ADD_PROPERTY_TYPE(HistoryEnabled,(true),"OpenMatrix9",App::Prop_None,"Clear to detach this surface from its parents");
    ADD_PROPERTY_TYPE(UpdateHistory,(true),"OpenMatrix9",App::Prop_None,"Suspend or resume updates while retaining parent links");
}
SurfaceHistory::~SurfaceHistory(){disconnect();}
void SurfaceHistory::disconnect(){changedConnection.disconnect();}
void SurfaceHistory::onSettingDocument() {
    Part::Feature::onSettingDocument();
    disconnect();
    if(auto* doc=getDocument())changedConnection=doc->signalChangedObject.connect(
        [this](const App::DocumentObject& object,const App::Property& prop){
            if(changing||isRestoring()||isRemoving()||!HistoryEnabled.getValue()||!getDocument()
               ||getDocument()->testStatus(App::Document::Restoring)
               ||getDocument()->isPerformingTransaction()||&object==this)return;
            // Group membership can change without changing the child's Shape.
            if(object.getPropertyByName("Group")==&prop){refreshPlacements();touch();}
            const auto& parents=PlacementSources.getValues();
            if(std::find(parents.begin(),parents.end(),&object)!=parents.end()
               &&object.getPropertyByName("Placement")==&prop)touch();
            const auto& sources=SourceCurves.getValues();
            if(std::find(sources.begin(),sources.end(),&object)!=sources.end()){
                touch();
                if(auto* source=dynamic_cast<const Part::Feature*>(&object);
                   UpdateHistory.getValue()&&historyUpdatesEnabled(*getDocument())&&source&&&prop==&source->Shape&&source->Shape.getShape().isNull()){
                    clearShape();setError();
                }
            }
        });
}
void SurfaceHistory::unsetupObject(){disconnect();Part::Feature::unsetupObject();}
void SurfaceHistory::refreshPlacements() {
    std::vector<App::DocumentObject*> parents;
    std::set<App::DocumentObject*> seen;
    for(auto* source:SourceCurves.getValues()){
        if(!source)continue;
        for(auto* parent=App::GeoFeatureGroupExtension::getGroupOfObject(source);parent;
            parent=App::GeoFeatureGroupExtension::getGroupOfObject(parent)){
            if(!seen.insert(parent).second)break;
            parents.push_back(parent);
        }
    }
    if(parents!=PlacementSources.getValues()){
        ChangeGuard guard(changing);
        PlacementSources.setValues(parents);
    }
}
short SurfaceHistory::mustExecute() const {
    if(HistoryEnabled.getValue()&&UpdateHistory.getValue()&&isTouched())return 1;
    return Part::Feature::mustExecute();
}
void SurfaceHistory::clearShape(){ChangeGuard guard(changing);Shape.setValue(Part::TopoShape());}
App::DocumentObjectExecReturn* SurfaceHistory::execute() {
    if(!HistoryEnabled.getValue()||!UpdateHistory.getValue()||!historyUpdatesEnabled(*getDocument()))return Part::Feature::execute();
    try {
        refreshPlacements();
        std::vector<SurfaceInput> inputs;
        OpenMatrix9Gui::SurfaceOptions options;
        parse(*getDocument(),SurfaceOptions.getValue(),SourceCurves,inputs,options);
        Base::PyGILStateLocker lock;
        PyObject* result=buildSurface(*getDocument(),inputs,options);
        if(!result)throw std::runtime_error(pythonError());
        try {ChangeGuard guard(changing);Shape.setPyObject(result);}
        catch(...){Py_DECREF(result);throw;}
        Py_DECREF(result);
        const char* id=options.kind==1?(options.historyCommand?"OM9-SURFACE-002":"OM9-SURFACE-001"):
                       options.kind==2?(options.historyCommand?"OM9-SURFACE-004":"OM9-SURFACE-003"):"OM9-SURFACE-009";
        OM9FeatureId.setValue(id);
        OM9Command.setValue(options.kind==1?(options.historyCommand?"gvSweepHistory":"Sweep1"):
                            options.kind==2?(options.historyCommand?"gvSweep2History":"Sweep2"):"Loft");
        return Part::Feature::execute();
    }catch(const std::exception& error){clearShape();return new App::DocumentObjectExecReturn(error.what());}
     catch(const Base::Exception& error){clearShape();return new App::DocumentObjectExecReturn(error.what());}
     catch(...){clearShape();return new App::DocumentObjectExecReturn("Surface History recompute failed");}
}
void SurfaceHistory::onBeforeChange(const App::Property* prop) {
    if(!changing&&!isRecomputing()&&!isRestoring()&&getDocument()
       &&!getDocument()->testStatus(App::Document::Restoring)&&!getDocument()->isPerformingTransaction()
       &&(prop==&Shape||prop==&Placement||prop==&SurfaceOptions||prop==&SourceCurves)&&historyEditLocked(this))
        throw Base::RuntimeError("History Lock prevents child surface geometry edits; edit its parents or set Lock=No");
    if(prop==&SourceCurves&&!changing&&!isRestoring()&&getDocument()
       &&!getDocument()->testStatus(App::Document::Restoring)){
        previousSources=SourceCurves.getValues();previousSubs=SourceCurves.getSubValues();
    }
    Part::Feature::onBeforeChange(prop);
}
void SurfaceHistory::onChanged(const App::Property* prop) {
    if(!changing&&!isRestoring()&&getDocument()&&!getDocument()->testStatus(App::Document::Restoring)
       &&!getDocument()->isPerformingTransaction()){
        if(prop==&SourceCurves){
            bool cycle=false,foreign=false;
            for(auto* source:SourceCurves.getValues()){
                if(!source)continue;
                if(source->getDocument()!=getDocument()){foreign=true;break;}
                if(source==this){cycle=true;break;}
                try {
                    const auto ancestors=source->getOutListRecursive();
                    if(std::find(ancestors.begin(),ancestors.end(),this)!=ancestors.end()){cycle=true;break;}
                }catch(const Base::BadGraphError&){cycle=true;break;}
            }
            if(cycle||foreign){
                ChangeGuard guard(changing);
                SourceCurves.setValues(previousSources,previousSubs);
                throw Base::ValueError(foreign?"Surface History inputs must belong to the same document":
                                      "Surface History cannot reference itself or one of its descendants");
            }
            refreshPlacements();
        }
        if(((prop==&Shape||prop==&Placement)&&!isRecomputing())||(prop==&HistoryEnabled&&!HistoryEnabled.getValue())){
            if((prop==&Shape||prop==&Placement)&&hasRecordedHistory(this))warnBrokenHistory(this);
            ChangeGuard guard(changing);
            HistoryEnabled.setValue(false);
            SourceCurves.setValues(std::vector<App::DocumentObject*>{},std::vector<std::string>{});
            PlacementSources.setValues(std::vector<App::DocumentObject*>{});
        }
    }
    Part::Feature::onChanged(prop);
}
void SurfaceHistory::onLostLinkToObject(App::DocumentObject* object) {
    const auto& sources=SourceCurves.getValues();
    const bool lost=std::find(sources.begin(),sources.end(),object)!=sources.end();
    Part::Feature::onLostLinkToObject(object);
    if(lost&&HistoryEnabled.getValue()){clearShape();setError();touch();}
}
void SurfaceHistory::onDocumentRestored(){ChangeGuard guard(changing);Part::Feature::onDocumentRestored();refreshPlacements();}

std::string serializeSurfaceHistoryOptions(const std::vector<SurfaceInput>& inputs,const OpenMatrix9Gui::SurfaceOptions& options) {
    if(!std::isfinite(options.tolerance)||options.tolerance<1e-7||options.tolerance>1e6)
        throw std::runtime_error("Surface History tolerance must be finite and between 1e-7 and 1e6 mm");
    for(const auto& input:inputs)if(!std::isfinite(input.seam)||input.seam<0||input.seam>=1)
        throw std::runtime_error("Surface History seam must be finite and between 0 (inclusive) and 1");
    for(const auto& [a,b]:options.slashes)if(!std::isfinite(a)||!std::isfinite(b)||a<0||a>1||b<0||b>1)
        throw std::runtime_error("Surface History slash fractions must be finite and between 0 and 1");
    QJsonObject root{{"version",1},{"kind",int(options.kind)},{"style",int(options.style)},
        {"frenet",options.frenet},{"closed",options.closed},{"maintainHeight",options.maintainHeight},
        {"preview",options.preview},{"sectionMode",int(options.sectionMode)},{"pointCount",int(options.pointCount)},
        {"tolerance",options.tolerance},{"continuityA",int(options.continuityA)},{"continuityB",int(options.continuityB)},
        {"matchStart",options.matchStart},{"matchEnd",options.matchEnd},{"history",options.history},{"historyCommand",options.historyCommand}};
    QJsonArray records,slashes;
    for(const auto& input:inputs)records.append(QJsonObject{{"count",int(input.chain.empty()?1:input.chain.size())},
        {"chain",!input.chain.empty()},{"closed",input.closed},{"reverse",input.reverse},{"seam",input.seam}});
    for(const auto& [a,b]:options.slashes)slashes.append(QJsonArray{a,b});
    root.insert("inputs",records);root.insert("slashes",slashes);
    return QJsonDocument(root).toJson(QJsonDocument::Compact).toStdString();
}
void SurfaceHistory::initialize(PyObject* shape,const std::vector<SurfaceInput>& inputs,const OpenMatrix9Gui::SurfaceOptions& options) {
    std::vector<App::DocumentObject*> objects;
    std::vector<std::string> subs;
    for(const auto& input:inputs){
        const auto references=input.chain.empty()?std::vector<std::pair<std::string,std::string>>{{input.object,input.sub}}:input.chain;
        for(const auto& [name,sub]:references){
            auto* source=getDocument()->getObject(name.c_str());
            if(!source)throw std::runtime_error("A Surface History source was deleted");
            objects.push_back(source);subs.push_back(sub);
        }
    }
    // Validate the schema using exactly the references that will be persisted.
    SourceCurves.setValues(objects,subs);
    SurfaceOptions.setValue(serializeSurfaceHistoryOptions(inputs,options));
    std::vector<SurfaceInput> parsedInputs;
    OpenMatrix9Gui::SurfaceOptions parsedOptions;
    parse(*getDocument(),SurfaceOptions.getValue(),SourceCurves,parsedInputs,parsedOptions);
    {ChangeGuard guard(changing);Shape.setPyObject(shape);HistoryEnabled.setValue(true);UpdateHistory.setValue(true);}
    refreshPlacements();
    OM9FeatureId.setValue(options.kind==1?(options.historyCommand?"OM9-SURFACE-002":"OM9-SURFACE-001"):
                         options.kind==2?(options.historyCommand?"OM9-SURFACE-004":"OM9-SURFACE-003"):"OM9-SURFACE-009");
    OM9Command.setValue(options.kind==1?(options.historyCommand?"gvSweepHistory":"Sweep1"):
                        options.kind==2?(options.historyCommand?"gvSweep2History":"Sweep2"):"Loft");
    if(!historyRecordingEnabled(*getDocument()))HistoryEnabled.setValue(false);
}
SurfaceHistory* createSurfaceHistory(App::Document& doc,PyObject* shape,const std::vector<SurfaceInput>& inputs,const SurfaceOptions& options) {
    auto* object=doc.addObject("OpenMatrix9Gui::SurfaceHistory",options.kind==1?"Sweep1":options.kind==2?"Sweep2":"Loft");
    if(!object)throw std::runtime_error("Cannot create native Surface History feature");
    auto* result=dynamic_cast<SurfaceHistory*>(object);
    const std::string name=object->getNameInDocument();
    try {if(!result)throw std::runtime_error("Surface History native type mismatch");result->initialize(shape,inputs,options);return result;}
    catch(...){doc.removeObject(name.c_str());throw;}
}
}
