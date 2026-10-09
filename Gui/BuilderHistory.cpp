// SPDX-License-Identifier: LGPL-2.1-or-later
// Native FreeCAD persistence/OCCT adapter. Rust owns recipe parsing and bounds.
#include "BuilderHistory.h"
#include "HistoryFeature.h"
#include <App/Document.h>
#include <App/Application.h>
#include <App/DocumentObjectPy.h>
#include <App/GeoFeature.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Base/Exception.h>
#include <Base/Interpreter.h>
#include <Base/Uuid.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Iterator.hxx>
#include <algorithm>
#include <set>
#include <stdexcept>
#include <string>
extern "C" bool om9_builder_recipe_plan(const char*,const double*,const double*,bool,double*);
extern "C" bool om9_builder_shapes_compatible(const char*,const char*);
extern "C" bool om9_builder_feature_id_valid(const char*);
extern "C" bool om9_builder_replay_budget(std::size_t,std::size_t,std::size_t);
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::BuilderRecord,App::DocumentObject)
PROPERTY_SOURCE(OpenMatrix9Gui::BuilderOutput,Part::Feature)
namespace {
struct Guard {bool& f;bool old;explicit Guard(bool& f):f(f),old(f){f=true;}~Guard(){f=old;}};
std::vector<App::DocumentObject*> outputFrames(const App::DocumentObject* o){std::vector<App::DocumentObject*> out;std::set<App::DocumentObject*> seen;for(auto* p=App::GeoFeatureGroupExtension::getGroupOfObject(o);p;p=App::GeoFeatureGroupExtension::getGroupOfObject(p)){if(!seen.insert(p).second||out.size()>=64)throw std::runtime_error("Invalid Builder output placement hierarchy");out.push_back(p);}return out;}
bool frameIncludes(const std::vector<App::DocumentObject*>& frames,const App::DocumentObject& o){return std::find(frames.begin(),frames.end(),&o)!=frames.end();}
bool guarded(const App::DocumentObject* o){return o->isRestoring()||o->isRemoving()||!o->getDocument()||o->getDocument()->testStatus(App::Document::Restoring)||o->getDocument()->isPerformingTransaction();}
void live(const App::DocumentObject* o){if(!o||!o->getDocument()||o->isRemoving()||!o->getDocument()->containsObject(o))throw std::runtime_error("Builder object was deleted or is not in its document");}
void editable(App::DocumentObject* o){live(o);auto* doc=o->getDocument();auto* app=Gui::Application::Instance;auto* gui=app?app->activeDocument():nullptr;if(doc!=App::GetApplication().getActiveDocument()||!gui||gui->getInEdit()||gui->isAboutToClose()||!Gui::Control().isAllowedAlterDocument(doc))throw std::runtime_error("The active Builder project is not editable");}
Part::Feature* gemFeature(App::DocumentObject* gem){live(gem);auto* f=dynamic_cast<Part::Feature*>(gem);if(!f||!gem->getDocument()||!gem->isValid()||f->Shape.getShape().isNull()||!f->Shape.getShape().isValid())throw std::runtime_error("Builder gem must have valid native Part geometry");return f;}
std::string shapeTag(App::DocumentObject* gem){live(gem);auto* p=gem->getPropertyByName<App::PropertyString>("OM9GemShape");if(!p||!om9_builder_shapes_compatible(p->getValue(),p->getValue()))throw std::runtime_error("Builder gem requires explicit nonempty OM9GemShape");return p->getValue();}
Part::TopoShape copied(const Part::TopoShape& s){return Part::TopoShape(BRepBuilderAPI_Copy(s.getShape(),true,false).Shape());}
Part::TopoShape localGem(App::DocumentObject* gem){auto* f=gemFeature(gem);auto s=copied(f->Shape.getShape());s.transformShape(f->Placement.getValue().inverse().toMatrix(),true);return s;}
Base::Vector3d dimensions(App::DocumentObject* gem){auto b=localGem(gem).getBoundBoxOptimal();return Base::Vector3d(b.LengthX(),b.LengthY(),b.LengthZ());}
void plan(const BuilderRecord& r,double* out){auto a=r.InitialDimensions.getValue(),b=dimensions(r.SourceGem.getValue());double initial[3]={a.x,a.y,a.z},current[3]={b.x,b.y,b.z};if(!om9_builder_recipe_plan(r.Parameters.getValue(),initial,current,r.ScaleToGem.getValue(),out))throw std::runtime_error("Unsupported or invalid Builder recipe/schema/evaluator/dimensions");}
std::vector<Part::TopoShape> templates(const BuilderRecord& r){std::vector<Part::TopoShape> out;for(TopoDS_Iterator it(r.Templates.getShape().getShape());it.More();it.Next()){if(out.size()>=1024)throw std::runtime_error("Builder template budget exceeded");out.emplace_back(it.Value());}if(out.empty())throw std::runtime_error("Builder record has no templates");return out;}
App::DocumentObject* native(PyObject* p){if(!PyObject_TypeCheck(p,&App::DocumentObjectPy::Type))throw std::runtime_error("Expected native document object");auto* wrapper=static_cast<App::DocumentObjectPy*>(p);if(!wrapper->Base::PyObjectBase::isValid())throw std::runtime_error("Builder object has been deleted");auto* object=wrapper->getDocumentObjectPtr();live(object);return object;}
std::vector<App::DocumentObject*> sequence(PyObject* p){PyObject* list=PySequence_Fast(p,"Expected document object sequence");if(!list)throw std::runtime_error("Expected document object sequence");std::vector<App::DocumentObject*> out;try{const auto n=PySequence_Fast_GET_SIZE(list);if(n<1||n>1024)throw std::runtime_error("Builder object count outside bounds");std::set<App::DocumentObject*> seen;for(Py_ssize_t i=0;i<n;++i){auto* o=native(PySequence_Fast_GET_ITEM(list,i));if(!seen.insert(o).second)throw std::runtime_error("Duplicate Builder object");out.push_back(o);}Py_DECREF(list);return out;}catch(...){Py_DECREF(list);throw;}}
PyObject* pyList(const std::vector<App::DocumentObject*>& objects){auto* out=PyList_New(objects.size());if(!out)throw std::runtime_error("Python allocation failed");for(std::size_t i=0;i<objects.size();++i)PyList_SET_ITEM(out,i,objects[i]->getPyObject());return out;}
std::vector<App::DocumentObject*> restore(BuilderRecord* r){r->validate();auto parts=templates(*r);auto names=r->OutputNames.getValues();names.resize(parts.size());std::vector<App::DocumentObject*> out;for(unsigned i=0;i<parts.size();++i){auto* existing=names[i].empty()?nullptr:r->getDocument()->getObject(names[i].c_str());if(existing){auto* b=dynamic_cast<BuilderOutput*>(existing);if(!b||!(b->OriginRecordIdentity.getValue()==r->RecordIdentity.getValue())||b->TemplateIndex.getValue()!=int(i)||(b->Recorded.getValue()&&b->ParentRecord.getValue()!=r))throw std::runtime_error("Builder output slot name occupied by detached or unrelated object");out.push_back(b);continue;}auto* b=dynamic_cast<BuilderOutput*>(r->getDocument()->addObject("OpenMatrix9Gui::BuilderOutput","BuilderOutput"));b->initialize(r,i);names[i]=b->getNameInDocument();out.push_back(b);}r->OutputNames.setValues(names);return out;}
BuilderRecord* create(App::DocumentObject* gem,const std::string& id,const std::string& raw,const Part::TopoShape& shape,Base::Vector3d initial,bool scale){auto& doc=*gem->getDocument();auto* r=dynamic_cast<BuilderRecord*>(doc.addObject("OpenMatrix9Gui::BuilderRecord","BuilderRecord"));r->SourceGem.setValue(gem);r->OM9FeatureId.setValue(id);r->Parameters.setValue(raw);r->GemShapeTag.setValue(shapeTag(gem));r->InitialDimensions.setValue(initial);r->InitialFrame.setValue(App::GeoFeature::getGlobalPlacement(gem));r->Templates.setValue(shape);r->ScaleToGem.setValue(scale);r->Recorded.setValue(historyRecordingEnabled(doc));r->validate();return r;}
template<class F> PyObject* api(F&& f){try{return f();}catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}catch(const std::exception& e){PyErr_SetString(PyExc_ValueError,e.what());}catch(...){PyErr_SetString(PyExc_RuntimeError,"Native Builder operation failed");}return nullptr;}
}
BuilderRecord::BuilderRecord(){
    ADD_PROPERTY_TYPE(SourceGem,(nullptr),"OpenMatrix9",App::Prop_ReadOnly,"Explicit source gem; no inferred ancestry");
    ADD_PROPERTY_TYPE(PlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"Native frame dependencies");
    ADD_PROPERTY(OM9FeatureId,(""));ADD_PROPERTY(Parameters,(""));ADD_PROPERTY(GemShapeTag,(""));ADD_PROPERTY(OutputNames,(std::vector<std::string>()));
    ADD_PROPERTY(InitialDimensions,(Base::Vector3d(1,1,1)));ADD_PROPERTY(InitialFrame,(Base::Placement()));ADD_PROPERTY(Templates,(Part::TopoShape()));
    ADD_PROPERTY(Recorded,(false));ADD_PROPERTY(ScaleToGem,(false));ADD_PROPERTY_TYPE(RecordIdentity,(Base::Uuid()),"OpenMatrix9",App::Prop_ReadOnly,"Durable record identity without output reverse links");
    OM9FeatureId.setReadOnly(true);GemShapeTag.setReadOnly(true);InitialDimensions.setReadOnly(true);InitialFrame.setReadOnly(true);Templates.setReadOnly(true);OutputNames.setReadOnly(true);
}
BuilderRecord::~BuilderRecord(){changed.disconnect();}
void BuilderRecord::refreshPlacements(){std::vector<App::DocumentObject*> frames;std::set<App::DocumentObject*> seen;for(auto* p=SourceGem.getValue()?App::GeoFeatureGroupExtension::getGroupOfObject(SourceGem.getValue()):nullptr;p;p=App::GeoFeatureGroupExtension::getGroupOfObject(p)){if(!seen.insert(p).second)throw std::runtime_error("Cyclic gem frames");if(frames.size()>=64)throw std::runtime_error("Gem frame depth exceeded");frames.push_back(p);}if(frames!=PlacementSources.getValues()){Guard g(writing);PlacementSources.setValues(frames);}}
void BuilderRecord::onSettingDocument(){App::DocumentObject::onSettingDocument();changed.disconnect();if(auto* doc=getDocument())changed=doc->signalChangedObject.connect([this](const App::DocumentObject& o,const App::Property& p){if(writing||guarded(this)||!Recorded.getValue()||&o==this)return;const auto& frames=PlacementSources.getValues();if(&o==SourceGem.getValue()&&historyUpdatesEnabled(*getDocument())){if(auto* f=dynamic_cast<const Part::Feature*>(&o);f&&f->Shape.getShape().isNull()){setError();for(auto* child:getInList())if(auto* b=dynamic_cast<BuilderOutput*>(child))b->invalidate();}}if(&o==SourceGem.getValue()||o.getPropertyByName("Group")==&p||(o.getPropertyByName("Placement")==&p&&std::find(frames.begin(),frames.end(),&o)!=frames.end())){refreshPlacements();touch();}});}
void BuilderRecord::unsetupObject(){changed.disconnect();App::DocumentObject::unsetupObject();}
void BuilderRecord::onDocumentRestored(){Guard g(writing);App::DocumentObject::onDocumentRestored();refreshPlacements();}
void BuilderRecord::onUndoRedoFinished(){App::DocumentObject::onUndoRedoFinished();refreshPlacements();if(Recorded.getValue())touch();}
void BuilderRecord::onChanged(const App::Property* p){App::DocumentObject::onChanged(p);if(writing||guarded(this))return;if(p==&SourceGem)refreshPlacements();if(p==&Parameters||p==&ScaleToGem||p==&SourceGem)touch();}
void BuilderRecord::onBeforeChange(const App::Property* p){if(!writing&&!guarded(this)&&p==&SourceGem&&SourceGem.getValue()&&!SourceGem.getValue()->isRemoving())throw Base::RuntimeError("Builder source gem is immutable; create another record to change it");App::DocumentObject::onBeforeChange(p);}
void BuilderRecord::onLostLinkToObject(App::DocumentObject* o){const bool lost=SourceGem.getValue()==o;App::DocumentObject::onLostLinkToObject(o);if(lost&&Recorded.getValue()&&getDocument()&&historyUpdatesEnabled(*getDocument())){setError();touch();for(auto* child:getInList())if(auto* b=dynamic_cast<BuilderOutput*>(child))b->invalidate();}}
short BuilderRecord::mustExecute()const{return Recorded.getValue()&&isTouched()?1:App::DocumentObject::mustExecute();}
void BuilderRecord::validate()const{auto* gem=SourceGem.getValue();gemFeature(gem);if(gem==this||gem->getDocument()!=getDocument()||shapeTag(gem)!=GemShapeTag.getValue())throw std::runtime_error("Invalid source gem or explicit shape tag changed");if(!om9_builder_feature_id_valid(OM9FeatureId.getValue()))throw std::runtime_error("Explicit feature ID required");double values[6];plan(*this,values);templates(*this);}
Part::TopoShape BuilderRecord::evaluate(unsigned slot)const{validate();auto parts=templates(*this);if(slot>=parts.size())throw std::runtime_error("Invalid Builder output template slot");double values[6];plan(*this,values);Base::Matrix4D affine;affine[0][0]=values[0];affine[1][1]=values[1];affine[2][2]=values[2];affine[0][3]=values[3];affine[1][3]=values[4];affine[2][3]=values[5];auto result=copied(parts[slot]);result.transformGeometry(affine);result.transformShape(App::GeoFeature::getGlobalPlacement(SourceGem.getValue()).toMatrix(),true);if(result.isNull()||!result.isValid())throw std::runtime_error("Builder evaluator returned null shape");return result;}
App::DocumentObjectExecReturn* BuilderRecord::execute(){if(!Recorded.getValue()||!historyUpdatesEnabled(*getDocument()))return App::DocumentObject::StdReturn;try{refreshPlacements();validate();return App::DocumentObject::StdReturn;}catch(const std::exception& e){for(auto* o:getInList())if(auto* b=dynamic_cast<BuilderOutput*>(o))b->invalidate();return new App::DocumentObjectExecReturn(e.what(),this);}catch(...){for(auto* o:getInList())if(auto* b=dynamic_cast<BuilderOutput*>(o))b->invalidate();return new App::DocumentObjectExecReturn("Invalid native Builder record",this);}}
void BuilderRecord::detach(){Guard g(writing);Recorded.setValue(false);PlacementSources.setValues({});}
BuilderOutput::BuilderOutput(){ADD_PROPERTY_TYPE(OriginRecordIdentity,(Base::Uuid()),"OpenMatrix9",App::Prop_ReadOnly,"Durable recipe slot identity even for unrecorded outputs");ADD_PROPERTY_TYPE(ParentRecord,(nullptr),"OpenMatrix9",App::Prop_ReadOnly,"Durable recipe parent");ADD_PROPERTY_TYPE(PlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"Native output ancestor frames without container cycles");ADD_PROPERTY_TYPE(TemplateIndex,(0),"OpenMatrix9",App::Prop_ReadOnly,"Template slot");ADD_PROPERTY(Recorded,(false));}
BuilderOutput::~BuilderOutput(){changed.disconnect();beforeChanged.disconnect();}
void BuilderOutput::refreshPlacements(){auto frames=outputFrames(this);Guard g(writing);if(frames!=PlacementSources.getValues())PlacementSources.setValues(frames);}
void BuilderOutput::setWorldShape(const Part::TopoShape& world){
    auto value=copied(world);
    // GTransform retains the input Location even when copying. Normalize it
    // first and compose it into the mapping before replacing Location below.
    const auto location=value.getTransform();
    value.setTransform(Base::Matrix4D());
    const auto bare=value;
    value.makeGTransform(bare,App::GeoFeature::getGlobalPlacement(this).inverse().toMatrix()*location,nullptr,true);
    // Part::Feature synchronizes Shape's location with its own Placement.
    // Bake the recipe into object-local coordinates, then apply that placement once.
    value.setTransform(Placement.getValue().toMatrix());
    Shape.setValue(value);
}
void BuilderOutput::onSettingDocument(){Part::Feature::onSettingDocument();changed.disconnect();beforeChanged.disconnect();if(auto* doc=getDocument()){
    beforeChanged=doc->signalBeforeChangeObject.connect([this](const App::DocumentObject& o,const App::Property& p){
        if(writing||guarded(this)||!Recorded.getValue()||o.isRecomputing())return;
        auto* record=dynamic_cast<BuilderRecord*>(ParentRecord.getValue());
        const bool sourceFrame=record&&frameIncludes(record->PlacementSources.getValues(),o);
        if(o.getPropertyByName("Placement")==&p&&frameIncludes(PlacementSources.getValues(),o)&&!sourceFrame&&historyEditLocked(this))throw Base::RuntimeError("History Lock prevents Builder output ancestor edits");
    });
    changed=doc->signalChangedObject.connect([this](const App::DocumentObject& o,const App::Property& p){
        if(writing||guarded(this)||!Recorded.getValue()||&o==this)return;
        if(o.getPropertyByName("Group")==&p){refreshPlacements();touch();return;}
        if(o.getPropertyByName("Placement")!=&p||!frameIncludes(PlacementSources.getValues(),o)||o.isRecomputing())return;
        auto* record=dynamic_cast<BuilderRecord*>(ParentRecord.getValue());
        if(record&&frameIncludes(record->PlacementSources.getValues(),o))touch();
        else {warnBrokenHistory(this);detach();}
    });
}}
void BuilderOutput::unsetupObject(){changed.disconnect();beforeChanged.disconnect();Part::Feature::unsetupObject();}
void BuilderOutput::onDocumentRestored(){Guard g(writing);Part::Feature::onDocumentRestored();refreshPlacements();}
void BuilderOutput::initialize(BuilderRecord* r,unsigned slot){Guard g(writing);ParentRecord.setValue(r);OriginRecordIdentity.setValue(r->RecordIdentity.getValue());TemplateIndex.setValue(slot);setWorldShape(r->evaluate(slot));Recorded.setValue(r->Recorded.getValue()&&historyRecordingEnabled(*getDocument()));if(!Recorded.getValue())ParentRecord.setValue(nullptr);refreshPlacements();}
short BuilderOutput::mustExecute()const{return Recorded.getValue()&&isTouched()?1:Part::Feature::mustExecute();}
App::DocumentObjectExecReturn* BuilderOutput::execute(){if(!Recorded.getValue()||!historyUpdatesEnabled(*getDocument()))return Part::Feature::execute();try{auto* r=dynamic_cast<BuilderRecord*>(ParentRecord.getValue());if(!r||r->getDocument()!=getDocument()||!r->isValid())throw std::runtime_error("Builder recipe parent missing or invalid");if(!r->Recorded.getValue())return Part::Feature::execute();Guard g(writing);refreshPlacements();setWorldShape(r->evaluate(TemplateIndex.getValue()));return Part::Feature::execute();}catch(const std::exception& e){Guard g(writing);Shape.setValue(Part::TopoShape());return new App::DocumentObjectExecReturn(e.what(),this);}catch(...){Guard g(writing);Shape.setValue(Part::TopoShape());return new App::DocumentObjectExecReturn("Builder recompute failed",this);}}
void BuilderOutput::onBeforeChange(const App::Property* p){if(!writing&&!guarded(this)&&!isRecomputing()&&(p==&Shape||p==&Placement||p==&ParentRecord||p==&TemplateIndex)&&historyEditLocked(this))throw Base::RuntimeError("History Lock prevents Builder output edits");Part::Feature::onBeforeChange(p);}
void BuilderOutput::onChanged(const App::Property* p){if(!writing&&!guarded(this)&&!isRecomputing()&&Recorded.getValue()&&(p==&Shape||p==&Placement||p==&ParentRecord||p==&TemplateIndex)){warnBrokenHistory(this);detach();}Guard g(writing);Part::Feature::onChanged(p);}
void BuilderOutput::onUndoRedoFinished(){Part::Feature::onUndoRedoFinished();refreshPlacements();if(Recorded.getValue())touch();}
void BuilderOutput::detach(){Guard g(writing);Recorded.setValue(false);ParentRecord.setValue(nullptr);PlacementSources.setValues({});}
void BuilderOutput::invalidate(){Guard g(writing);Shape.setValue(Part::TopoShape());setError();touch();}
void initializeBuilderHistoryTypes(){BuilderRecord::init();BuilderOutput::init();}
bool builderHistoryRecorded(App::DocumentObject* o){if(auto* r=dynamic_cast<BuilderRecord*>(o))return r->Recorded.getValue()&&r->SourceGem.getValue();if(auto* b=dynamic_cast<BuilderOutput*>(o))return b->Recorded.getValue()&&b->ParentRecord.getValue();return false;}
std::vector<App::DocumentObject*> builderHistoryParents(App::DocumentObject* o){if(auto* r=dynamic_cast<BuilderRecord*>(o))return r->SourceGem.getValue()?std::vector<App::DocumentObject*>{r->SourceGem.getValue()}:std::vector<App::DocumentObject*>{};if(auto* b=dynamic_cast<BuilderOutput*>(o))return b->ParentRecord.getValue()?std::vector<App::DocumentObject*>{b->ParentRecord.getValue()}:std::vector<App::DocumentObject*>{};return {};}
bool isBuilderStorageObject(const App::DocumentObject* o){return dynamic_cast<const BuilderRecord*>(o)!=nullptr;}
bool detachBuilderHistory(App::DocumentObject* o){if(auto* r=dynamic_cast<BuilderRecord*>(o)){r->detach();return true;}if(auto* b=dynamic_cast<BuilderOutput*>(o)){b->detach();return true;}return false;}
static PyObject* createAPI(PyObject*,PyObject* args){return api([&]()->PyObject*{PyObject *gemPy,*seedsPy;const char *id,*raw;int scale=0;if(!PyArg_ParseTuple(args,"OssO|p",&gemPy,&id,&raw,&seedsPy,&scale))return nullptr;auto* gem=native(gemPy);editable(gem);if(!om9_builder_feature_id_valid(id))throw std::runtime_error("Exact OM9 feature ID required");gemFeature(gem);shapeTag(gem);auto seeds=sequence(seedsPy);auto initial=dimensions(gem);double a[3]={initial.x,initial.y,initial.z},result[6];if(!om9_builder_recipe_plan(raw,a,a,scale,result))throw std::runtime_error("Invalid Builder recipe");TopoDS_Compound compound;BRep_Builder builder;builder.MakeCompound(compound);for(auto* seed:seeds){if(seed==gem||seed->getDocument()!=gem->getDocument())throw std::runtime_error("Builder seeds must be separate same-document Part objects");auto* f=gemFeature(seed);auto s=copied(f->Shape.getShape());const auto parent=App::GeoFeature::getGlobalPlacement(seed)*f->Placement.getValue().inverse();s.transformShape((App::GeoFeature::getGlobalPlacement(gem).inverse()*parent).toMatrix(),true);builder.Add(compound,s.getShape());}auto& doc=*gem->getDocument();doc.openTransaction("Record Builder History");try{auto* r=create(gem,id,raw,Part::TopoShape(compound),initial,scale);restore(r);doc.recompute();auto* py=r->getPyObject();doc.commitTransaction();return py;}catch(...){doc.abortTransaction();throw;}});}
static PyObject* restoreAPI(PyObject*,PyObject* args){return api([&]()->PyObject*{PyObject* py;if(!PyArg_ParseTuple(args,"O",&py))return nullptr;auto* r=dynamic_cast<BuilderRecord*>(native(py));if(!r)throw std::runtime_error("Expected native BuilderRecord");editable(r);auto& doc=*r->getDocument();doc.openTransaction("Restore Builder Outputs");try{auto objects=restore(r);doc.recompute();auto* out=pyList(objects);doc.commitTransaction();return out;}catch(...){doc.abortTransaction();throw;}});}
static PyObject* matchAPI(PyObject*,PyObject* args){return api([&]()->PyObject*{PyObject *sourcePy,*targetsPy;if(!PyArg_ParseTuple(args,"OO",&sourcePy,&targetsPy))return nullptr;auto* source=native(sourcePy);editable(source);gemFeature(source);auto tag=shapeTag(source);auto targets=sequence(targetsPy);auto& doc=*source->getDocument();std::vector<BuilderRecord*> records;std::size_t templateCount=0;for(auto* o:doc.getObjects())if(auto* r=dynamic_cast<BuilderRecord*>(o);r&&r->SourceGem.getValue()==source){r->validate();const auto count=templates(*r).size();if(!om9_builder_replay_budget(records.size()+1,templateCount+count,targets.size()))throw std::runtime_error("Match Builder aggregate record/output budget exceeded");templateCount+=count;records.push_back(r);}if(records.empty())throw std::runtime_error("Source gem has no supported Builder records");for(auto* target:targets){gemFeature(target);if(target==source||target->getDocument()!=&doc||!om9_builder_shapes_compatible(tag.c_str(),shapeTag(target).c_str()))throw std::runtime_error("Match Attributes requires distinct same-shape gems in one document");auto d=dimensions(target);double current[3]={d.x,d.y,d.z},p[6];for(auto* r:records){auto initial=r->InitialDimensions.getValue();double a[3]={initial.x,initial.y,initial.z};if(!om9_builder_recipe_plan(r->Parameters.getValue(),a,current,r->ScaleToGem.getValue(),p))throw std::runtime_error("Target gem dimensions invalid");}}doc.openTransaction("Match Builder Attributes");try{std::vector<App::DocumentObject*> made;for(auto* target:targets)for(auto* old:records){auto* r=create(target,old->OM9FeatureId.getStrValue(),old->Parameters.getStrValue(),copied(old->Templates.getShape()),old->InitialDimensions.getValue(),old->ScaleToGem.getValue());restore(r);made.push_back(r);}doc.recompute();auto* py=pyList(made);doc.commitTransaction();return py;}catch(...){doc.abortTransaction();throw;}});}
}
void AddBuilderHistoryMethods(PyObject* module){static PyMethodDef methods[]={{"createBuilderRecord",OpenMatrix9Gui::createAPI,METH_VARARGS,"Copy seed geometry into a durable gem-local affine template recipe."},{"restoreBuilderOutputs",OpenMatrix9Gui::restoreAPI,METH_VARARGS,"Explicitly recreate missing native outputs from a durable recipe."},{"matchBuilderAttributes",OpenMatrix9Gui::matchAPI,METH_VARARGS,"Copy all supported recipes to explicitly compatible same-shape gems atomically."},{nullptr,nullptr,0,nullptr}};if(PyModule_AddFunctions(module,methods)<0)throw Base::RuntimeError("Cannot register Builder History API");}
