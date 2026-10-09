// SPDX-License-Identifier: LGPL-2.1-or-later
#include "HistoryFeature.h"
#include "EditGeometry.h"
#include "SurfaceHistory.h"
#include "CircleHistory.h"
#include "BuilderHistory.h"
#include "CageFeature.h"
#include <App/Document.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Base/Console.h>
#include <Base/Exception.h>
#include <Base/Interpreter.h>
#include <Gui/Application.h>
#include <Gui/MainWindow.h>
#include <QMessageBox>
#include <set>
#include <cmath>
#include <cstdint>
#include <limits>
#include <algorithm>
#include <map>
extern "C" bool om9_history_can_update(bool,bool);
extern "C" bool om9_history_can_edit(bool,bool);
extern "C" std::size_t om9_history_schedule(const std::uint64_t*,const std::uint64_t*,std::size_t,const std::uint64_t*,std::size_t,bool,bool,std::uint64_t*,std::size_t);
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::HistorySettings,App::DocumentObject)
PROPERTY_SOURCE(OpenMatrix9Gui::HistoryJoin,Part::Feature)
namespace {
struct Writing {bool& flag;bool old;explicit Writing(bool& f):flag(f),old(f){flag=true;}~Writing(){flag=old;}};
std::vector<App::DocumentObject*> historyParents(App::DocumentObject* object){
    if(builderHistoryRecorded(object))return builderHistoryParents(object);
    if(cageHistoryRecorded(object))return cageHistoryParents(object);
    if(auto* j=dynamic_cast<HistoryJoin*>(object))return j->Parents.getValues();
    if(auto* s=dynamic_cast<SurfaceHistory*>(object))return s->SourceCurves.getValues();
    if(auto* c=dynamic_cast<CircleHistory*>(object)){auto parents=c->Sources.getValues();if(c->SourceCurve.getValue())parents.push_back(c->SourceCurve.getValue());return parents;}
    return {};
}
bool guarded(App::DocumentObject* o){return o->isRestoring()||o->isRemoving()||!o->getDocument()||o->getDocument()->testStatus(App::Document::Restoring)||o->getDocument()->isPerformingTransaction();}
void warn(App::DocumentObject* o){
    if(auto* s=historySettings(*o->getDocument(),false);s&&!s->BrokenHistoryWarning.getValue())return;
    Base::Console().warning("History detached from {}. Undo restores the relationship.\n",o->getNameInDocument());
    if(Gui::Application::Instance){
        // Nonblocking: property callbacks must finish before processing another edit.
        auto* box=new QMessageBox(QMessageBox::Warning,"Broken History",QString("History detached from %1. Undo restores the relationship.").arg(QString::fromUtf8(o->getNameInDocument())),QMessageBox::Ok,Gui::getMainWindow());
        box->setObjectName("OM9BrokenHistoryWarning");box->setAttribute(Qt::WA_DeleteOnClose);box->setWindowModality(Qt::NonModal);box->show();
    }
}
void resume(App::Document& doc){
    const auto objects=doc.getObjects();std::vector<std::uint64_t> parents,children,changed,result(objects.size());
    std::map<std::uint64_t,App::DocumentObject*> byId;
    for(auto* o:objects){const auto child=std::uint64_t(o->getID());changed.push_back(child);byId.emplace(child,o);
        if(hasRecordedHistory(o)){std::set<std::uint64_t> seen;for(auto* p:historyParents(o)){
            if(!p||p->getDocument()!=&doc||!doc.containsObject(p))throw Base::RuntimeError("History dependency belongs to a missing or different document");
            const auto parent=std::uint64_t(p->getID());if(seen.insert(parent).second){parents.push_back(parent);children.push_back(child);}}}}
    auto count=om9_history_schedule(parents.data(),children.data(),parents.size(),changed.data(),changed.size(),true,true,result.data(),result.size());
    if(count==std::numeric_limits<std::size_t>::max()||count>result.size())throw Base::RuntimeError("Invalid History dependency graph");
    for(std::size_t i=0;i<count;++i)byId.at(result[i])->touch();
}
}
HistorySettings::HistorySettings(){
    ADD_PROPERTY(Record,(true));ADD_PROPERTY(Update,(true));ADD_PROPERTY(Lock,(false));ADD_PROPERTY(BrokenHistoryWarning,(true));
}
HistorySettings* historySettings(App::Document& d,bool create){
    for(auto* o:d.getObjects())if(auto* s=dynamic_cast<HistorySettings*>(o))return s;
    if(!create)return nullptr;
    return dynamic_cast<HistorySettings*>(d.addObject("OpenMatrix9Gui::HistorySettings","OM9HistorySettings"));
}
void HistorySettings::onChanged(const App::Property* p){
    App::DocumentObject::onChanged(p);
    if(getDocument()&&getDocument()->isPerformingTransaction())setStatus(App::ObjectStatus::PendingTransactionUpdate,true);
    if(guarded(this))return;
    if((p==&Record||p==&Update)&&om9_history_can_update(Record.getValue(),Update.getValue()))resume(*getDocument());
}
void HistorySettings::onUndoRedoFinished(){App::DocumentObject::onUndoRedoFinished();if(getDocument()&&om9_history_can_update(Record.getValue(),Update.getValue()))resume(*getDocument());}
HistoryJoin::HistoryJoin(){
    ADD_PROPERTY(Parents,(nullptr));Parents.setSize(0);
    ADD_PROPERTY_TYPE(PlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"Native parent frame dependencies without container cycles");
    ADD_PROPERTY(Tolerance,(1e-7));ADD_PROPERTY(Recorded,(false));
    ADD_PROPERTY_TYPE(HistoryDirty,(false),"OpenMatrix9",App::Prop_ReadOnly,"Output is stale or failed and must be recomputed from its History parents");
    ADD_PROPERTY(OM9FeatureId,("OM9-TOOLS-017"));OM9FeatureId.setReadOnly(true);
    Parents.setReadOnly(true);Recorded.setReadOnly(true);
}
HistoryJoin::~HistoryJoin(){changedConnection.disconnect();}
void HistoryJoin::onSettingDocument(){
    Part::Feature::onSettingDocument();changedConnection.disconnect();
    if(auto* doc=getDocument())changedConnection=doc->signalChangedObject.connect([this](const App::DocumentObject& object,const App::Property& prop){
        if(writing||isRestoring()||isRemoving()||!getDocument()||getDocument()->testStatus(App::Document::Restoring)||!Recorded.getValue()||&object==this)return;
        const bool membership=object.getPropertyByName("Group")==&prop;
        const auto& frames=PlacementSources.getValues();
        const bool placement=object.getPropertyByName("Placement")==&prop&&std::find(frames.begin(),frames.end(),&object)!=frames.end();
        if(getDocument()->isPerformingTransaction()){if(membership||placement)setStatus(App::ObjectStatus::PendingTransactionUpdate,true);return;}
        if(membership){refreshPlacements();Writing internal(writing);HistoryDirty.setValue(true);touch();}else if(placement){Writing internal(writing);HistoryDirty.setValue(true);touch();}
        const auto& sources=Parents.getValues();
        if(std::find(sources.begin(),sources.end(),&object)!=sources.end()){
            {Writing internal(writing);HistoryDirty.setValue(true);}
            touch();
            if(auto* source=dynamic_cast<const Part::Feature*>(&object);source&&&prop==&source->Shape&&source->Shape.getShape().isNull()&&historyUpdatesEnabled(*getDocument())){
                // FreeCAD may skip descendants after a parent execution error.
                // Propagate explicit invalidation before that skip leaves stale output.
                Writing internal(writing);Shape.setValue(Part::TopoShape());setError();
            }
        }
    });
}
void HistoryJoin::unsetupObject(){changedConnection.disconnect();Part::Feature::unsetupObject();}
void HistoryJoin::refreshPlacements(){
    std::vector<App::DocumentObject*> frames;std::set<App::DocumentObject*> seen;
    for(auto* parent:Parents.getValues())if(parent)for(auto* group=App::GeoFeatureGroupExtension::getGroupOfObject(parent);group;group=App::GeoFeatureGroupExtension::getGroupOfObject(group)){
        if(!seen.insert(group).second)break;frames.push_back(group);
    }
    if(frames!=PlacementSources.getValues()){Writing internal(writing);PlacementSources.setValues(frames);}
}
void HistoryJoin::onDocumentRestored(){Writing internal(writing);Part::Feature::onDocumentRestored();refreshPlacements();}
void HistoryJoin::onUndoRedoFinished(){Part::Feature::onUndoRedoFinished();refreshPlacements();if(Recorded.getValue())touch();}
void initializeHistoryTypes(){Base::PyGILStateLocker lock;CurvePyRef part(PyImport_ImportModule("Part"));HistorySettings::init();HistoryJoin::init();CircleHistory::init();}
short HistoryJoin::mustExecute()const{return Recorded.getValue()&&(Parents.isTouched()||Tolerance.isTouched())?1:Part::Feature::mustExecute();}
App::DocumentObjectExecReturn* HistoryJoin::execute(){
    if(!Recorded.getValue())return App::DocumentObject::StdReturn;
    auto* s=historySettings(*getDocument(),false);
    if(s&&!om9_history_can_update(s->Record.getValue(),s->Update.getValue()))return App::DocumentObject::StdReturn;
    try{
        Base::PyGILStateLocker lock;Writing internal(writing);
        HistoryDirty.setValue(true);
        if(Parents.getValues().size()<2)throw std::runtime_error("Join History requires its original two or more parents; the record is unresolved");
        if(!std::isfinite(Tolerance.getValue())||Tolerance.getValue()<=0)throw std::runtime_error("Join History tolerance must be finite and positive");
        std::vector<EditInput> inputs;
        for(auto* o:Parents.getValues()){
            if(!o||o==this||o->getDocument()!=getDocument()||!getDocument()->containsObject(o))throw std::runtime_error("Join History parent is missing or invalid");
            if(!o->isValid())throw std::runtime_error("Join History parent has a recompute error");
            inputs.push_back(editInput(*getDocument(),o->getNameInDocument(),1));
        }
        auto result=buildEdit(inputs,1,inputs.size(),0,Tolerance.getValue());
        if(result.size()!=1)throw std::runtime_error("Join History parents must form a single connected curve");
        // Geometry is built in world coordinates. Internal writes never detach.
        Placement.setValue(Base::Placement());Shape.setPyObject(result.front()->value);
        HistoryDirty.setValue(false);
        return Part::Feature::execute();
    }catch(const Base::Exception& e){Writing internal(writing);Shape.setValue(Part::TopoShape());return new App::DocumentObjectExecReturn(e.what(),this);}
    catch(const std::exception& e){Writing internal(writing);Shape.setValue(Part::TopoShape());return new App::DocumentObjectExecReturn(e.what(),this);}
    catch(...){Writing internal(writing);Shape.setValue(Part::TopoShape());return new App::DocumentObjectExecReturn("Join History recompute failed",this);}
}
bool hasRecordedHistory(App::DocumentObject* o){
    if(builderHistoryRecorded(o)||cageHistoryRecorded(o))return true;
    if(auto* c=dynamic_cast<CircleHistory*>(o))return c->Recorded.getValue()&&(c->SourceCurve.getValue()||!c->Recipe.getValues().empty());
    if(auto* j=dynamic_cast<HistoryJoin*>(o))return j->Recorded.getValue();
    if(auto* s=dynamic_cast<SurfaceHistory*>(o))return s->HistoryEnabled.getValue()&&!s->SourceCurves.getValues().empty();
    return false;
}
bool historyRecordingEnabled(App::Document& d){auto* s=historySettings(d,false);return !s||s->Record.getValue();}
bool historyUpdatesEnabled(App::Document& d){auto* s=historySettings(d,false);return !s||om9_history_can_update(s->Record.getValue(),s->Update.getValue());}
void warnBrokenHistory(App::DocumentObject* o){if(o&&o->getDocument())warn(o);}
bool historyEditLocked(App::DocumentObject* o){if(!o||!o->getDocument())return false;auto* s=historySettings(*o->getDocument(),false);return s&&!om9_history_can_edit(s->Lock.getValue(),hasRecordedHistory(o));}
void HistoryJoin::onBeforeChange(const App::Property* p){
    if(!writing&&!guarded(this)&&(p==&Shape||p==&Placement||p==&Tolerance)&&historyEditLocked(this))throw Base::RuntimeError("History Lock prevents child geometry edits; edit its parents or set Lock=No");
    Part::Feature::onBeforeChange(p);
}
void HistoryJoin::onChanged(const App::Property* p){
    // Part::Feature synchronizes Shape and Placement; guard both during that callback.
    if(!writing&&!guarded(this)&&(p==&Shape||p==&Placement||p==&Parents)&&Recorded.getValue()){warn(this);detach();}
    if(getDocument()&&getDocument()->isPerformingTransaction())setStatus(App::ObjectStatus::PendingTransactionUpdate,true);
    const bool dirty=!writing&&!guarded(this)&&Recorded.getValue()&&(p==&Tolerance||p==&Recorded);
    Writing internal(writing);Part::Feature::onChanged(p);if(dirty)HistoryDirty.setValue(true);if(p==&Parents)refreshPlacements();
}
void HistoryJoin::detach(){Writing internal(writing);Parents.setValues({});PlacementSources.setValues({});Recorded.setValue(false);HistoryDirty.setValue(false);}
void HistoryJoin::setInitialShape(PyObject* shape){Writing internal(writing);Shape.setPyObject(shape);}
void detachObjectHistory(App::DocumentObject* o){if(detachBuilderHistory(o))return;if(cageHistoryRecorded(o)){detachCageHistory(o);return;}if(auto* c=dynamic_cast<CircleHistory*>(o))c->detach();else if(auto* j=dynamic_cast<HistoryJoin*>(o))j->detach();else if(auto* s=dynamic_cast<SurfaceHistory*>(o))s->HistoryEnabled.setValue(false);}
bool hasHistoryDependents(App::DocumentObject* o){
    if(!o||!o->getDocument())return false;
    for(auto* candidate:o->getDocument()->getObjects())if(hasRecordedHistory(candidate))
        for(auto* parent:historyParents(candidate))if(parent==o)return true;
    return false;
}
void breakHistoryForEdit(App::Document& d,const std::vector<std::string>& names){
    std::set<App::DocumentObject*> targets;for(const auto& name:names)if(auto* o=d.getObject(name.c_str()))targets.insert(o);
    for(auto* o:targets)if(historyEditLocked(o))throw std::runtime_error("History Lock prevents this child geometry edit");
    std::set<App::DocumentObject*> broken;
    for(auto* o:d.getObjects())if(hasRecordedHistory(o)){
        if(targets.contains(o))broken.insert(o);
        for(auto* p:historyParents(o))if(targets.contains(p))broken.insert(o);
    }
    for(auto* o:broken){warn(o);detachObjectHistory(o);}
}
HistoryJoin* createHistoryJoin(App::Document& d,const std::vector<std::string>& names,double tolerance){
    if(names.size()<2||!std::isfinite(tolerance)||tolerance<=0)throw std::runtime_error("Join History requires two or more separate curves and a positive tolerance");
    Base::PyGILStateLocker lock;std::vector<EditInput> inputs;std::vector<App::DocumentObject*> parents;std::set<std::string> seen;
    for(const auto& name:names){if(!seen.insert(name).second)throw std::runtime_error("Join History input curves must be distinct");inputs.push_back(editInput(d,name,1));parents.push_back(d.getObject(name.c_str()));}
    // The ordinary Join adapter supports surfaces too; Join History is curves only.
    for(const auto& input:inputs){CurvePyRef faces(PyObject_GetAttrString(input.shape->value,"Faces"));if(PySequence_Size(faces.value)!=0)throw std::runtime_error("Join History accepts curves only");}
    auto result=buildEdit(inputs,1,inputs.size(),0,tolerance);
    if(result.size()!=1)throw std::runtime_error("Join History requires a single connected result");
    d.openTransaction("Join History");
    try{
        auto* s=historySettings(d);auto* j=dynamic_cast<HistoryJoin*>(d.addObject("OpenMatrix9Gui::HistoryJoin","JoinHistory"));
        j->Tolerance.setValue(tolerance);j->setInitialShape(result.front()->value);
        if(s->Record.getValue()){j->Parents.setValues(parents);j->Recorded.setValue(true);}
        d.recompute();if(j->isError())throw std::runtime_error("Join History recompute failed");
        d.commitTransaction();return j;
    }catch(...){d.abortTransaction();throw;}
}
}
