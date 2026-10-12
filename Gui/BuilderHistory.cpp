// SPDX-License-Identifier: LGPL-2.1-or-later
// Native FreeCAD persistence/OCCT adapter. Rust owns recipe parsing and bounds.
#include "BuilderHistory.h"
#include "HistoryFeature.h"
#include "NativeInputTree.h"
#include "CoreObjects.h"
#include "CoreTopologySnapshot.h"
#include "CoreShapeValidation.h"
#include "CoreSourceAttributes.h"
#include "CorePreparedSnapshot.h"
#include "CorePreparedCad.h"
#include "CoreBuilderTransform.h"
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
#include <Standard_Failure.hxx>
#include <QApplication>
#include <QThread>
#include <QJsonArray>
#include <QJsonDocument>
#include <algorithm>
#include <memory>
#include <set>
#include <stdexcept>
#include <string>
#include <string_view>
extern "C" bool om9_builder_recipe_plan(const char*,const double*,const double*,bool,double*);
extern "C" bool om9_builder_shapes_compatible(const char*,const char*);
extern "C" bool om9_builder_feature_id_valid(const char*);
extern "C" bool om9_builder_replay_budget(std::size_t,std::size_t,std::size_t);
extern "C" bool om9_native_builder_counts(std::size_t,std::size_t,std::size_t);
extern "C" unsigned om9_native_builder_capability(unsigned,unsigned,const unsigned char*,std::size_t,const unsigned char*,std::size_t,std::size_t,std::size_t,std::size_t,unsigned,const double*,std::size_t);
extern "C" unsigned om9_native_cad_result_admit(unsigned,unsigned,unsigned);
extern "C" unsigned om9_builder_attributes_validate(const unsigned char*,std::size_t,std::size_t);
extern "C" std::size_t om9_prepared_result_limit();
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::BuilderRecord,App::DocumentObject)
PROPERTY_SOURCE(OpenMatrix9Gui::BuilderOutput,Part::Feature)
namespace {
struct Guard {bool& f;bool old;explicit Guard(bool& f):f(f),old(f){f=true;}~Guard(){f=old;}};
// Synchronous borrowed property/argument buffers. Neither callbacks nor pointers
// are retained by Rust; native geometry owners stay in the calling GUI scope.
struct BuilderRequest {
    std::string_view id,raw;
    bool scale;
    std::size_t records=1,templates=1,targets=1;
};
std::runtime_error failure(const char* code,const std::string& identity,const std::string& message){return std::runtime_error(std::string(code)+": Builder input "+identity+": "+message);}
std::string diagnostic(const std::string& message,const std::string& identity,const char* fallback="InvalidBasis"){
    for(const char* code:{"UnsupportedRepresentation","InvalidBasis","InvalidTrim","NonFiniteGeometry","StaleGeometry","UnsupportedTransform","KernelFailure","ConversionLoss","UnsupportedOperation"})
        if(message.starts_with(std::string(code)+":"))return message;
    return std::string(fallback)+": Builder input "+identity+": "+message;
}
BuilderRequest request(const BuilderRecord& r){return {r.OM9FeatureId.getStrValue(),r.Parameters.getStrValue(),r.ScaleToGem.getValue()};}
void admission(const BuilderRequest& r,unsigned representation,const std::string& identity,unsigned stage=2,const double* dimensions=nullptr){
    const auto status=om9_native_builder_capability(representation,stage,
        reinterpret_cast<const unsigned char*>(r.id.data()),r.id.size(),
        reinterpret_cast<const unsigned char*>(r.raw.data()),r.raw.size(),r.records,r.templates,r.targets,r.scale?1U:0U,dimensions,dimensions?6:0);
    if(status==4)return;
    throw failure(status==101?"UnsupportedRepresentation":status==103?"KernelFailure":status==100?"UnsupportedOperation":"InvalidBasis",identity,"unsupported or invalid stored affine-template request");
}
unsigned representation(const TopoDS_Shape& s){
    if(s.IsNull())return 0;
    switch(s.ShapeType()){
        case TopAbs_VERTEX:return 1;case TopAbs_EDGE:return 2;case TopAbs_WIRE:return 3;
        case TopAbs_FACE:return 4;case TopAbs_SHELL:return 5;case TopAbs_SOLID:return 6;
        case TopAbs_COMPOUND:case TopAbs_COMPSOLID:return 7;default:return 0;
    }
}
void borrowedShape(const Part::TopoShape& s,const std::string& identity){
    if(s.isNull())throw failure("StaleGeometry",identity,"native shape is null");
    try{ThreeDm::InputTree::validate(s.getShape());}
    catch(const ThreeDm::GeometryRepresentationUnavailable& e){throw failure("UnsupportedRepresentation",identity,e.what());}
    catch(const Standard_Failure& e){throw failure("KernelFailure",identity,e.GetMessageString()?e.GetMessageString():"native input inspection failed");}
    catch(const std::exception& e){throw failure("InvalidBasis",identity,e.what());}
}
std::string_view text(PyObject* value,std::size_t limit,const std::string& identity){
    if(!PyUnicode_Check(value)||PyUnicode_GetLength(value)<0||std::size_t(PyUnicode_GetLength(value))>limit)
        throw failure("InvalidBasis",identity,"bounded UTF-8 string required");
    Py_ssize_t length=0;const auto* bytes=PyUnicode_AsUTF8AndSize(value,&length);
    if(!bytes||length<0||std::size_t(length)>limit){PyErr_Clear();throw failure("InvalidBasis",identity,"UTF-8 byte count outside existing bounds");}
    return {bytes,std::size_t(length)};
}
std::vector<App::DocumentObject*> outputFrames(const App::DocumentObject* o){std::vector<App::DocumentObject*> out;std::set<App::DocumentObject*> seen;for(auto* p=App::GeoFeatureGroupExtension::getGroupOfObject(o);p;p=App::GeoFeatureGroupExtension::getGroupOfObject(p)){if(!seen.insert(p).second||out.size()>=64)throw std::runtime_error("Invalid Builder output placement hierarchy");out.push_back(p);}return out;}
bool frameIncludes(const std::vector<App::DocumentObject*>& frames,const App::DocumentObject& o){return std::find(frames.begin(),frames.end(),&o)!=frames.end();}
bool guarded(const App::DocumentObject* o){return o->isRestoring()||o->isRemoving()||!o->getDocument()||o->getDocument()->testStatus(App::Document::Restoring)||o->getDocument()->isPerformingTransaction();}
void live(const App::DocumentObject* o){if(!qApp||QThread::currentThread()!=qApp->thread())throw failure("KernelFailure","native call","GUI thread required");if(!o||!o->getDocument()||o->isRemoving()||!o->getDocument()->containsObject(o))throw failure("StaleGeometry","native object","object was deleted or is not in its document");}
void editable(App::DocumentObject* o){live(o);auto* doc=o->getDocument();auto* app=Gui::Application::Instance;auto* gui=app?app->activeDocument():nullptr;if(doc!=App::GetApplication().getActiveDocument()||!gui||gui->getInEdit()||gui->isAboutToClose()||!Gui::Control().isAllowedAlterDocument(doc))throw std::runtime_error("The active Builder project is not editable");}
Part::Feature* gemFeature(App::DocumentObject* gem,const BuilderRequest* r=nullptr){live(gem);const std::string identity=gem->getNameInDocument();auto* f=dynamic_cast<Part::Feature*>(gem);if(!f){if(r)admission(*r,gem->getPropertyByName("Mesh")?8U:gem->getPropertyByName("Points")?9U:gem->isDerivedFrom(Base::Type::fromName("App::Link"))?10U:gem->getPropertyByName("Group")?11U:0U,identity);throw failure("UnsupportedRepresentation",identity,"native Part geometry required");}const auto& shape=f->Shape.getShape();if(shape.isNull())throw failure("StaleGeometry",identity,"native shape is null");if(r)admission(*r,representation(shape.getShape()),identity);borrowedShape(shape,identity);if(!gem->isValid()||!shape.isValid())throw failure("InvalidBasis",identity,"native Part geometry is invalid");return f;}
std::string shapeTag(App::DocumentObject* gem){live(gem);auto* p=gem->getPropertyByName<App::PropertyString>("OM9GemShape");if(!p||!om9_builder_shapes_compatible(p->getValue(),p->getValue()))throw std::runtime_error("Builder gem requires explicit nonempty OM9GemShape");return p->getValue();}
Part::TopoShape copied(const Part::TopoShape& s,const std::string& identity="private shape"){borrowedShape(s,identity);try{return Part::TopoShape(BRepBuilderAPI_Copy(s.getShape(),true,false).Shape());}catch(const Standard_Failure& e){throw failure("KernelFailure",identity,e.GetMessageString()?e.GetMessageString():"native copy failed");}}
Part::TopoShape localGem(App::DocumentObject* gem,const BuilderRequest* r=nullptr){auto* f=gemFeature(gem,r);auto s=copied(f->Shape.getShape(),gem->getNameInDocument());s.transformShape(f->Placement.getValue().inverse().toMatrix(),true);return s;}
Base::Vector3d dimensions(App::DocumentObject* gem,const BuilderRequest* r=nullptr){auto b=localGem(gem,r).getBoundBoxOptimal();return Base::Vector3d(b.LengthX(),b.LengthY(),b.LengthZ());}
std::vector<Part::TopoShape> templates(const BuilderRecord&);
void plan(const BuilderRecord& r,double* out){auto policy=request(r);admission(policy,7,std::string(r.getNameInDocument())+".Parameters");policy.templates=templates(r).size();auto a=r.InitialDimensions.getValue(),b=dimensions(r.SourceGem.getValue(),&policy);double measured[6]={a.x,a.y,a.z,b.x,b.y,b.z};admission(policy,7,r.getNameInDocument(),3,measured);if(!om9_builder_recipe_plan(r.Parameters.getValue(),measured,measured+3,r.ScaleToGem.getValue(),out))throw failure("InvalidBasis",r.getNameInDocument(),"existing affine plan rejected admitted dimensions");}
std::vector<Part::TopoShape> templates(const BuilderRecord& r){
    auto policy=request(r);const std::string identity=std::string(r.getNameInDocument())+".Templates";const auto& shape=r.Templates.getShape();
    admission(policy,representation(shape.getShape()),identity);borrowedShape(shape,identity);
    if(shape.getShape().ShapeType()!=TopAbs_COMPOUND)throw failure("InvalidBasis",identity,"durable compound templates required");
    std::size_t count=0;for(TopoDS_Iterator it(shape.getShape());it.More();it.Next())if(!om9_native_builder_counts(1,++count,1))throw failure("InvalidBasis",identity,"template count outside existing bounds");
    policy.templates=count;admission(policy,7,identity);
    std::vector<Part::TopoShape> out;out.reserve(count);for(TopoDS_Iterator it(shape.getShape());it.More();it.Next())out.emplace_back(it.Value());return out;
}
App::DocumentObject* native(PyObject* p){if(!PyObject_TypeCheck(p,&App::DocumentObjectPy::Type))throw std::runtime_error("Expected native document object");auto* wrapper=static_cast<App::DocumentObjectPy*>(p);if(!wrapper->Base::PyObjectBase::isValid())throw std::runtime_error("Builder object has been deleted");auto* object=wrapper->getDocumentObjectPtr();live(object);return object;}
std::vector<App::DocumentObject*> sequence(PyObject* p){PyObject* list=PySequence_Fast(p,"Expected document object sequence");if(!list)throw std::runtime_error("Expected document object sequence");std::vector<App::DocumentObject*> out;try{const auto n=PySequence_Fast_GET_SIZE(list);if(n<1||n>1024)throw std::runtime_error("Builder object count outside bounds");std::set<App::DocumentObject*> seen;for(Py_ssize_t i=0;i<n;++i){auto* o=native(PySequence_Fast_GET_ITEM(list,i));if(!seen.insert(o).second)throw std::runtime_error("Duplicate Builder object");out.push_back(o);}Py_DECREF(list);return out;}catch(...){Py_DECREF(list);throw;}}
// FreeCAD getPyObject() returns a new reference. Keep that reference local
// until commit succeeds, including partially constructed result lists.
struct PyRelease {void operator()(PyObject* value) const {Py_XDECREF(value);}};
using PyOwner=std::unique_ptr<PyObject,PyRelease>;
PyOwner own(PyObject* value){if(!value)throw std::runtime_error("Python allocation failed");return PyOwner(value);}
PyObject* pyList(const std::vector<App::DocumentObject*>& objects){
    auto out=own(PyList_New(objects.size()));
    for(std::size_t i=0;i<objects.size();++i){auto item=own(objects[i]->getPyObject());PyList_SET_ITEM(out.get(),i,item.release());}
    return out.release();
}
// Freeze names before publication: a failed native close may delete outputs
// and invalidate pointers. No live object is queried while reporting failure.
std::string publicationIdentity(const std::vector<ObjectSnapshot>& inputs){
    std::string identity;
    for(const auto& input:inputs){if(!identity.empty())identity+=", ";identity+=input.objectName;}
    return identity;
}
void commitPublication(App::Document& doc,const char* operation,const std::string& identity){
    try{doc.commitTransaction();}
    catch(const Base::Exception& e){throw failure("KernelFailure",identity,std::string(operation)+": "+e.what());}
    catch(const std::exception& e){throw failure("KernelFailure",identity,std::string(operation)+": "+e.what());}
    catch(...){throw failure("KernelFailure",identity,std::string(operation)+": native commit failed");}
}
// The native exception owns OCCT shapes. Rust remains the admission/validation
// authority; no shape or document address is passed to or retained by Rust.
template<class F> auto outputBoundary(const std::string& identity,F&& native)->decltype(native()){
    try{return native();}
    catch(const ThreeDm::GeometryRepresentationUnavailable& e){throw failure("UnsupportedRepresentation",identity,e.what());}
    catch(const Standard_Failure& e){throw failure("KernelFailure",identity,e.GetMessageString()?e.GetMessageString():"native output operation failed");}
    catch(const std::exception& e){throw std::runtime_error(diagnostic(e.what(),identity));}
}
BuilderTrace solvedWorld(const Part::TopoShape& part,const double* values,
                           const Base::Placement& frame,const std::string& identity){
    return outputBoundary(identity,[&]{
    borrowedShape(part,identity);
    auto value=builderWorldTrace(part,values,frame);
    borrowedShape(value.shape,identity);
    if(!value.shape.isValid())throw failure("InvalidBasis",identity,"Builder evaluator returned invalid native shape");
    return value;
    });
}
BuilderTrace localOutput(const BuilderTrace& world,const Base::Placement& global,
                           const Base::Placement& own,const std::string& identity){
    return outputBoundary(identity,[&]{
    auto value=builderLocalTrace(world,global,own);
    borrowedShape(value.shape,identity);
    const auto topology=topologySnapshot(value.shape.getShape());
    const auto validation=shapeValidation(value.shape.getShape(),topology);
    const auto tri=[](const QJsonValue& v){return v.isBool()?(v.toBool()?2U:1U):0U;};
    const auto status=om9_native_cad_result_admit(static_cast<unsigned>(value.shape.getShape().ShapeType()),
        tri(validation.value("native_valid")),tri(validation.value("valid_solid")));
    if(status)throw failure(status==1?"UnsupportedRepresentation":status==7?"KernelFailure":"InvalidTrim",
        identity,"solved native output failed shared topology/solid validation");
    return value;
    });
}
BuilderOutput* ownedSlot(BuilderRecord& r,const std::string& name,unsigned slot){
    auto* object=name.empty()?nullptr:r.getDocument()->getObject(name.c_str());
    if(!object)return nullptr;
    auto* b=dynamic_cast<BuilderOutput*>(object);
    if(!b||!(b->OriginRecordIdentity.getValue()==r.RecordIdentity.getValue())||
       b->TemplateIndex.getValue()!=int(slot)||(b->Recorded.getValue()&&b->ParentRecord.getValue()!=&r))
        throw failure("StaleGeometry",std::string(r.getNameInDocument())+".OutputNames["+std::to_string(slot)+"]",
                      "Builder output slot name occupied by detached or unrelated object");
    return b;
}
struct PreparedOutputs {
    std::vector<Part::TopoShape> local;
    std::vector<std::string> names;
    std::vector<std::string> results;
};
QJsonArray metadata(std::string_view raw,std::size_t count,const std::string& identity){
    const auto status=om9_builder_attributes_validate(reinterpret_cast<const unsigned char*>(raw.data()),raw.size(),count);
    if(status)throw failure("InvalidBasis",identity,"invalid or oversized persisted template attributes");
    if(raw.empty())return {}; // Genuine legacy recipe: no seed metadata was stored.
    return QJsonDocument::fromJson(QByteArray(raw.data(),raw.size())).array();
}
QJsonArray matrix(const Base::Matrix4D& frame){QJsonArray values;for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)values.append(frame[r][c]);return values;}
QJsonObject stamp(const ObjectStamp& s){
    return {{"document",QString::number(s.document)},{"object",QString::number(s.object)},
        {"generation",QString::number(s.generation)},{"revision",QString::number(s.revision)},{"epoch",QString::number(s.epoch)}};
}
struct EvidenceInputs {
    BuilderRequest policy;
    ObjectSnapshot source;
    std::string documentName;
    Base::Vector3d initial,current;
    unsigned sourceType;
    QJsonArray attributes;
};
EvidenceInputs evidence(const BuilderRecord& r){
    const auto count=templates(r).size();
    const auto attributes=metadata(r.TemplateAttributes.getStrValue(),count,r.getNameInDocument());
    const auto source=captureObject(*r.getDocument(),r.SourceGem.getValue()->getNameInDocument());
    auto policy=request(r);policy.templates=count;
    return {policy,source,r.getDocument()->getName(),r.InitialDimensions.getValue(),dimensions(r.SourceGem.getValue(),&policy),
        static_cast<unsigned>(gemFeature(r.SourceGem.getValue(),&policy)->Shape.getShape().getShape().ShapeType()),attributes};
}
std::string resultEvidence(const BuilderTrace& trace,const EvidenceInputs& input,unsigned slot,unsigned templateType,
                          const double* affine,const Base::Placement& global,const Base::Placement& own){
    auto source=stamp(input.source.stamp);QJsonArray world,ancestors,values;
    for(double v:input.source.world)world.append(v);
    for(const auto& s:input.source.ancestors)ancestors.append(stamp(s));
    for(unsigned i=0;i<6;++i)values.append(affine[i]);
    source["document_uid"]=QString::fromStdString(input.source.documentUid);source["object_name"]=QString::fromStdString(input.source.objectName);
    source["unit_context_snapshot"]=QString::fromStdString(input.source.unitContext);source["world"]=world;source["ancestors"]=ancestors;
    const auto dimensionsJson=[](const Base::Vector3d& d){return QJsonArray{d.x,d.y,d.z};};
    const auto bytes=[](std::string_view s){return QString::fromUtf8(s.data(),s.size());};
    QJsonObject context{{"schema_version",1},{"operation","NativeBuilder"},{"feature_id",bytes(input.policy.id)},
        {"recipe",bytes(input.policy.raw)},{"scale_to_gem",input.policy.scale},{"template_slot",int(slot)},
        {"template_count",int(input.policy.templates)},{"initial_dimensions",dimensionsJson(input.initial)},
        {"current_dimensions",dimensionsJson(input.current)},{"affine",values},{"source_native_shape_type",int(input.sourceType)},
        {"template_native_shape_type",int(templateType)},{"source_ref",source},
        {"output_parent_to_world",matrix((global*own.inverse()).toMatrix())},{"attribute_role","immutable_template_source_evidence"}};
    const auto attributes=input.attributes.isEmpty()?QJsonObject{{"schema_version",1},{"availability","unavailable_legacy_record"}}:input.attributes[int(slot)].toObject();
    auto* doc=App::GetApplication().getDocument(input.documentName.c_str());
    if(!doc)throw failure("StaleGeometry",input.source.objectName,"source document disappeared");
    QJsonObject identity{{"document_name",doc->getName()},{"name",QString::fromStdString(input.source.objectName)}};
    const auto inputTopology=topologySnapshot(trace.source.getShape());const auto inputValidation=shapeValidation(trace.source.getShape(),inputTopology);
    const auto inputSnapshot=preparedCadSnapshot(trace.source.getShape(),inputTopology,inputValidation,attributes,input.source.unitContext,identity,"NativeBuilder");
    Base::Matrix4D sourceWorldMatrix;for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)sourceWorldMatrix[r][c]=input.source.world[4*r+c];
    context["native_transform_observations"]=builderTraceObservations(trace,inputSnapshot,affine,Base::Placement(sourceWorldMatrix),global*own.inverse());
    const auto& shape=trace.shape;
    const auto topology=topologySnapshot(shape.getShape());const auto validation=shapeValidation(shape.getShape(),topology);
    const auto result=preparedCadResult(shape.getShape(),topology,validation,attributes,identity,input.source.unitContext,context);
    verifyObject(*doc,input.source);
    return QJsonDocument(result).toJson(QJsonDocument::Compact).toStdString();
}
PreparedOutputs prepareOutputs(const Part::TopoShape& compound,const double* values,
                               const Base::Placement& frame,const std::string& identity,
                               const EvidenceInputs& input,BuilderRecord* record=nullptr){
    PreparedOutputs prepared;
    for(TopoDS_Iterator it(compound.getShape());it.More();it.Next()){
        const auto slot=prepared.names.size();
        if(!om9_native_builder_counts(1,slot+1,1))throw failure("InvalidBasis",identity,"output count outside existing bounds");
        const auto& names=record?record->OutputNames.getValues():prepared.names;
        prepared.names.push_back(record&&slot<names.size()?names[slot]:std::string());
    }
    // Check every durable slot before any solver or first missing output write.
    std::vector<BuilderOutput*> existing;existing.reserve(prepared.names.size());
    for(unsigned slot=0;slot<prepared.names.size();++slot)
        existing.push_back(record?ownedSlot(*record,prepared.names[slot],slot):nullptr);
    prepared.local.reserve(prepared.names.size());
    unsigned slot=0;std::size_t resultBytes=0;
    for(TopoDS_Iterator it(compound.getShape());it.More();it.Next(),++slot){
        const auto context=identity+".Output["+std::to_string(slot)+"]";
        const auto world=solvedWorld(Part::TopoShape(it.Value()),values,frame,context);
        auto* b=existing[slot];
        const auto trace=localOutput(world,b?App::GeoFeature::getGlobalPlacement(b):Base::Placement(),
                                            b?b->Placement.getValue():Base::Placement(),context);
        prepared.local.push_back(trace.shape);
        auto result=resultEvidence(trace,input,slot,it.Value().ShapeType(),values,
            b?App::GeoFeature::getGlobalPlacement(b):Base::Placement(),b?b->Placement.getValue():Base::Placement());
        if(result.size()>om9_prepared_result_limit()-resultBytes)throw failure("InvalidBasis",identity,"aggregate output evidence exceeds Rust result budget");
        resultBytes+=result.size();prepared.results.push_back(std::move(result));
    }
    return prepared;
}
struct PublicationScope {
    App::Document& document;
    std::vector<std::string> names;
    void stage(BuilderOutput& output,const Part::TopoShape& shape,const std::string& result){
        names.emplace_back(output.getNameInDocument());output.stagePreparedShape(shape,result);
    }
    void recompute(const std::vector<BuilderRecord*>& records){
        // Recomputing the whole document also touches unrelated dirty Part
        // inputs, invalidating the captured revisions and changing sources in
        // this transaction. Publish only these native records and their slots.
        for(auto* r:records)if(!document.recomputeFeature(r,false))
            throw failure("KernelFailure",r->getNameInDocument(),"prepared record publication failed");
        for(const auto& name:names){
            auto* b=dynamic_cast<BuilderOutput*>(document.getObject(name.c_str()));
            if(!b||!document.recomputeFeature(b,false))
                throw failure("KernelFailure",name,"prepared output publication failed");
        }
    }
    ~PublicationScope(){for(const auto& name:names)if(auto* b=dynamic_cast<BuilderOutput*>(document.getObject(name.c_str())))b->clearPreparedShape();}
};
std::vector<App::DocumentObject*> restore(BuilderRecord* r,const PreparedOutputs& prepared,PublicationScope& publication){
    auto names=prepared.names;std::vector<App::DocumentObject*> out;out.reserve(names.size());
    for(unsigned i=0;i<names.size();++i){
        if(auto* b=ownedSlot(*r,names[i],i)){
            // Existing detached/frozen output slots remain unchanged. A recorded
            // output that recomputes consumes its already solved local shape.
            if(b->Recorded.getValue())publication.stage(*b,prepared.local[i],prepared.results[i]);
            out.push_back(b);continue;
        }
        auto* b=dynamic_cast<BuilderOutput*>(r->getDocument()->addObject("OpenMatrix9Gui::BuilderOutput","BuilderOutput"));
        publication.stage(*b,prepared.local[i],prepared.results[i]);b->initialize(r,i,prepared.local[i],prepared.results[i]);
        names[i]=b->getNameInDocument();out.push_back(b);
    }
    r->OutputNames.setValues(names);return out;
}
BuilderRecord* create(App::DocumentObject* gem,const std::string& id,const std::string& raw,const Part::TopoShape& shape,Base::Vector3d initial,bool scale,const std::string& attributes){auto& doc=*gem->getDocument();auto* r=dynamic_cast<BuilderRecord*>(doc.addObject("OpenMatrix9Gui::BuilderRecord","BuilderRecord"));r->SourceGem.setValue(gem);r->OM9FeatureId.setValue(id);r->Parameters.setValue(raw);r->GemShapeTag.setValue(shapeTag(gem));r->InitialDimensions.setValue(initial);r->InitialFrame.setValue(App::GeoFeature::getGlobalPlacement(gem));r->Templates.setValue(shape);r->TemplateAttributes.setValue(attributes);r->ScaleToGem.setValue(scale);r->Recorded.setValue(historyRecordingEnabled(doc));return r;}
ObjectSnapshot capture(App::DocumentObject& object){return captureObject(*object.getDocument(),object.getNameInDocument());}
void verify(App::Document& doc,const std::vector<ObjectSnapshot>& inputs){
    for(const auto& input:inputs)try{verifyObject(doc,input);}
    catch(const std::exception& e){throw failure("StaleGeometry",input.objectName,e.what());}
}
template<class F> PyObject* api(F&& f){try{return f();}catch(const PreparedCadFailure& e){setPreparedCadError(e);}catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,diagnostic(e.what(),"API","KernelFailure").c_str());}catch(const std::exception& e){PyErr_SetString(PyExc_ValueError,diagnostic(e.what(),"API").c_str());}catch(...){PyErr_SetString(PyExc_RuntimeError,"KernelFailure: Builder input API: native operation failed");}return nullptr;}
}
BuilderRecord::BuilderRecord(){
    ADD_PROPERTY_TYPE(SourceGem,(nullptr),"OpenMatrix9",App::Prop_ReadOnly,"Explicit source gem; no inferred ancestry");
    ADD_PROPERTY_TYPE(PlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"Native frame dependencies");
    ADD_PROPERTY(OM9FeatureId,(""));ADD_PROPERTY(Parameters,(""));ADD_PROPERTY(GemShapeTag,(""));ADD_PROPERTY(OutputNames,(std::vector<std::string>()));
    ADD_PROPERTY(InitialDimensions,(Base::Vector3d(1,1,1)));ADD_PROPERTY(InitialFrame,(Base::Placement()));ADD_PROPERTY(Templates,(Part::TopoShape()));
    ADD_PROPERTY_TYPE(TemplateAttributes,(""),"OpenMatrix9",App::Prop_ReadOnly,"Immutable seed identity and native appearance captured with template slots; empty only for legacy recipes");
    ADD_PROPERTY(Recorded,(false));ADD_PROPERTY(ScaleToGem,(false));ADD_PROPERTY_TYPE(RecordIdentity,(Base::Uuid()),"OpenMatrix9",App::Prop_ReadOnly,"Durable record identity without output reverse links");
    OM9FeatureId.setReadOnly(true);GemShapeTag.setReadOnly(true);InitialDimensions.setReadOnly(true);InitialFrame.setReadOnly(true);Templates.setReadOnly(true);OutputNames.setReadOnly(true);
}
BuilderRecord::~BuilderRecord(){changed.disconnect();}
void BuilderRecord::refreshPlacements(){std::vector<App::DocumentObject*> frames;std::set<App::DocumentObject*> seen;for(auto* p=SourceGem.getValue()?App::GeoFeatureGroupExtension::getGroupOfObject(SourceGem.getValue()):nullptr;p;p=App::GeoFeatureGroupExtension::getGroupOfObject(p)){if(!seen.insert(p).second)throw std::runtime_error("Cyclic gem frames");if(frames.size()>=64)throw std::runtime_error("Gem frame depth exceeded");frames.push_back(p);}if(frames!=PlacementSources.getValues()){Guard g(writing);PlacementSources.setValues(frames);}}
void BuilderRecord::onSettingDocument(){App::DocumentObject::onSettingDocument();changed.disconnect();if(auto* doc=getDocument())changed=doc->signalChangedObject.connect([this](const App::DocumentObject& o,const App::Property& p){if(writing||guarded(this)||!Recorded.getValue()||&o==this)return;const auto& frames=PlacementSources.getValues();if(&o==SourceGem.getValue()&&historyUpdatesEnabled(*getDocument())){if(auto* f=dynamic_cast<const Part::Feature*>(&o);f&&f->Shape.getShape().isNull()){setError();for(auto* child:getInList())if(auto* b=dynamic_cast<BuilderOutput*>(child))b->invalidate();}}if(&o==SourceGem.getValue()||o.getPropertyByName("Group")==&p||(o.getPropertyByName("Placement")==&p&&std::find(frames.begin(),frames.end(),&o)!=frames.end())){refreshPlacements();touch();}});}
void BuilderRecord::unsetupObject(){changed.disconnect();App::DocumentObject::unsetupObject();}
void BuilderRecord::onDocumentRestored(){Guard g(writing);App::DocumentObject::onDocumentRestored();refreshPlacements();}
void BuilderRecord::onUndoRedoFinished(){App::DocumentObject::onUndoRedoFinished();refreshPlacements();if(Recorded.getValue())touch();}
void BuilderRecord::onChanged(const App::Property* p){App::DocumentObject::onChanged(p);if(writing||guarded(this))return;if(p==&SourceGem)refreshPlacements();if(p==&Parameters||p==&ScaleToGem||p==&SourceGem||p==&TemplateAttributes)touch();}
void BuilderRecord::onBeforeChange(const App::Property* p){if(!writing&&!guarded(this)&&p==&SourceGem&&SourceGem.getValue()&&!SourceGem.getValue()->isRemoving())throw Base::RuntimeError("Builder source gem is immutable; create another record to change it");App::DocumentObject::onBeforeChange(p);}
void BuilderRecord::onLostLinkToObject(App::DocumentObject* o){const bool lost=SourceGem.getValue()==o;App::DocumentObject::onLostLinkToObject(o);if(lost&&Recorded.getValue()&&getDocument()&&historyUpdatesEnabled(*getDocument())){setError();touch();for(auto* child:getInList())if(auto* b=dynamic_cast<BuilderOutput*>(child))b->invalidate();}}
short BuilderRecord::mustExecute()const{return Recorded.getValue()&&isTouched()?1:App::DocumentObject::mustExecute();}
void BuilderRecord::validate()const{live(this);auto policy=request(*this);admission(policy,7,std::string(getNameInDocument())+".Parameters");auto* gem=SourceGem.getValue();gemFeature(gem,&policy);if(gem==this||gem->getDocument()!=getDocument()||shapeTag(gem)!=GemShapeTag.getValue())throw failure("StaleGeometry",getNameInDocument(),"source gem or explicit shape tag changed");const auto count=templates(*this).size();metadata(TemplateAttributes.getStrValue(),count,getNameInDocument());double values[6];plan(*this,values);}
Part::TopoShape BuilderRecord::evaluate(unsigned slot)const{validate();auto parts=templates(*this);if(slot>=parts.size())throw failure("InvalidBasis",getNameInDocument(),"Invalid Builder output template slot");double values[6];plan(*this,values);return solvedWorld(parts[slot],values,App::GeoFeature::getGlobalPlacement(SourceGem.getValue()),std::string(getNameInDocument())+".Output["+std::to_string(slot)+"]").shape;}
App::DocumentObjectExecReturn* BuilderRecord::execute(){if(!Recorded.getValue()||!historyUpdatesEnabled(*getDocument()))return App::DocumentObject::StdReturn;try{refreshPlacements();validate();return App::DocumentObject::StdReturn;}catch(const std::exception& e){for(auto* o:getInList())if(auto* b=dynamic_cast<BuilderOutput*>(o))b->invalidate();return new App::DocumentObjectExecReturn(diagnostic(e.what(),getNameInDocument()),this);}catch(...){for(auto* o:getInList())if(auto* b=dynamic_cast<BuilderOutput*>(o))b->invalidate();return new App::DocumentObjectExecReturn(diagnostic("native record failed",getNameInDocument(),"KernelFailure"),this);}}
void BuilderRecord::detach(){Guard g(writing);Recorded.setValue(false);PlacementSources.setValues({});}
BuilderOutput::BuilderOutput(){ADD_PROPERTY_TYPE(OriginRecordIdentity,(Base::Uuid()),"OpenMatrix9",App::Prop_ReadOnly,"Durable recipe slot identity even for unrecorded outputs");ADD_PROPERTY_TYPE(ParentRecord,(nullptr),"OpenMatrix9",App::Prop_ReadOnly,"Durable recipe parent");ADD_PROPERTY_TYPE(PlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"Native output ancestor frames without container cycles");ADD_PROPERTY_TYPE(TemplateIndex,(0),"OpenMatrix9",App::Prop_ReadOnly,"Template slot");ADD_PROPERTY(Recorded,(false));ADD_PROPERTY_TYPE(OM9GeometryResult,(""),"OpenMatrix9",App::Prop_ReadOnly,"Immutable native operation evidence for the published Shape");}
BuilderOutput::~BuilderOutput(){changed.disconnect();beforeChanged.disconnect();}
void BuilderOutput::refreshPlacements(){auto frames=outputFrames(this);Guard g(writing);if(frames!=PlacementSources.getValues())PlacementSources.setValues(frames);}
void BuilderOutput::setWorldShape(){
    auto* record=dynamic_cast<BuilderRecord*>(ParentRecord.getValue());
    if(!record)throw failure("StaleGeometry",getNameInDocument(),"recipe parent missing");
    record->validate();const auto input=evidence(*record);double affine[6];plan(*record,affine);
    const auto parts=templates(*record);const auto slot=static_cast<unsigned>(TemplateIndex.getValue());
    if(slot>=parts.size())throw failure("InvalidBasis",getNameInDocument(),"template slot out of range");
    const auto global=App::GeoFeature::getGlobalPlacement(this),own=Placement.getValue();
    const auto world=solvedWorld(parts[slot],affine,App::GeoFeature::getGlobalPlacement(record->SourceGem.getValue()),getNameInDocument());
    const auto local=localOutput(world,global,own,getNameInDocument());
    const auto result=resultEvidence(local,input,slot,parts[slot].getShape().ShapeType(),affine,global,own);
    Shape.setValue(local.shape);OM9GeometryResult.setValue(result);
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
void BuilderOutput::stagePreparedShape(const Part::TopoShape& shape,const std::string& result){preparedShape=Prepared{shape,result};}
void BuilderOutput::clearPreparedShape(){preparedShape.reset();}
void BuilderOutput::initialize(BuilderRecord* r,unsigned slot,const Part::TopoShape& preparedLocal,const std::string& result){Guard g(writing);ParentRecord.setValue(r);OriginRecordIdentity.setValue(r->RecordIdentity.getValue());TemplateIndex.setValue(slot);Shape.setValue(preparedLocal);OM9GeometryResult.setValue(result);Recorded.setValue(r->Recorded.getValue()&&historyRecordingEnabled(*getDocument()));if(!Recorded.getValue())ParentRecord.setValue(nullptr);refreshPlacements();}
short BuilderOutput::mustExecute()const{return Recorded.getValue()&&isTouched()?1:Part::Feature::mustExecute();}
App::DocumentObjectExecReturn* BuilderOutput::execute(){if(!Recorded.getValue()||!historyUpdatesEnabled(*getDocument()))return Part::Feature::execute();try{auto* r=dynamic_cast<BuilderRecord*>(ParentRecord.getValue());if(!r||r->getDocument()!=getDocument()||!r->isValid())throw failure("StaleGeometry",getNameInDocument(),"recipe parent missing or invalid");if(!r->Recorded.getValue())return Part::Feature::execute();Guard g(writing);refreshPlacements();if(preparedShape){Shape.setValue(preparedShape->shape);OM9GeometryResult.setValue(preparedShape->result);preparedShape.reset();}else setWorldShape();return Part::Feature::execute();}catch(const std::exception& e){preparedShape.reset();Guard g(writing);Shape.setValue(Part::TopoShape());OM9GeometryResult.setValue("");return new App::DocumentObjectExecReturn(diagnostic(e.what(),getNameInDocument()),this);}catch(...){preparedShape.reset();Guard g(writing);Shape.setValue(Part::TopoShape());OM9GeometryResult.setValue("");return new App::DocumentObjectExecReturn(diagnostic("native recompute failed",getNameInDocument(),"KernelFailure"),this);}}
void BuilderOutput::onBeforeChange(const App::Property* p){if(!writing&&!guarded(this)&&!isRecomputing()&&(p==&Shape||p==&Placement||p==&ParentRecord||p==&TemplateIndex)&&historyEditLocked(this))throw Base::RuntimeError("History Lock prevents Builder output edits");Part::Feature::onBeforeChange(p);}
void BuilderOutput::onChanged(const App::Property* p){if(!writing&&(p==&Shape||p==&Placement||p==&ParentRecord||p==&TemplateIndex))preparedShape.reset();if(!writing&&!guarded(this)&&!isRecomputing()&&Recorded.getValue()&&(p==&Shape||p==&Placement||p==&ParentRecord||p==&TemplateIndex)){warnBrokenHistory(this);detach();}Guard g(writing);Part::Feature::onChanged(p);}
void BuilderOutput::onUndoRedoFinished(){Part::Feature::onUndoRedoFinished();refreshPlacements();if(Recorded.getValue())touch();}
void BuilderOutput::detach(){Guard g(writing);Recorded.setValue(false);ParentRecord.setValue(nullptr);PlacementSources.setValues({});}
void BuilderOutput::invalidate(){preparedShape.reset();Guard g(writing);Shape.setValue(Part::TopoShape());OM9GeometryResult.setValue("");setError();touch();}
void initializeBuilderHistoryTypes(){BuilderRecord::init();BuilderOutput::init();}
bool builderHistoryRecorded(App::DocumentObject* o){if(auto* r=dynamic_cast<BuilderRecord*>(o))return r->Recorded.getValue()&&r->SourceGem.getValue();if(auto* b=dynamic_cast<BuilderOutput*>(o))return b->Recorded.getValue()&&b->ParentRecord.getValue();return false;}
std::vector<App::DocumentObject*> builderHistoryParents(App::DocumentObject* o){if(auto* r=dynamic_cast<BuilderRecord*>(o))return r->SourceGem.getValue()?std::vector<App::DocumentObject*>{r->SourceGem.getValue()}:std::vector<App::DocumentObject*>{};if(auto* b=dynamic_cast<BuilderOutput*>(o))return b->ParentRecord.getValue()?std::vector<App::DocumentObject*>{b->ParentRecord.getValue()}:std::vector<App::DocumentObject*>{};return {};}
bool isBuilderStorageObject(const App::DocumentObject* o){return dynamic_cast<const BuilderRecord*>(o)!=nullptr;}
bool detachBuilderHistory(App::DocumentObject* o){if(auto* r=dynamic_cast<BuilderRecord*>(o)){r->detach();return true;}if(auto* b=dynamic_cast<BuilderOutput*>(o)){b->detach();return true;}return false;}
static PyObject* createAPI(PyObject*,PyObject* args){return api([&]()->PyObject*{
    PyObject *gemPy,*seedsPy,*idPy,*rawPy;int scale=0;
    if(!PyArg_ParseTuple(args,"OOOO|p",&gemPy,&idPy,&rawPy,&seedsPy,&scale))return nullptr;
    auto* gem=native(gemPy);editable(gem);const std::string identity=gem->getNameInDocument();
    BuilderRequest policy{text(idPy,24,identity+".FeatureId"),text(rawPy,1048576,identity+".Recipe"),scale!=0};
    admission(policy,7,identity+".Recipe");gemFeature(gem,&policy);shapeTag(gem);auto seeds=sequence(seedsPy);policy.templates=seeds.size();
    gemFeature(gem,&policy);
    for(auto* seed:seeds){if(seed==gem||seed->getDocument()!=gem->getDocument())throw failure("StaleGeometry",seed->getNameInDocument(),"separate same-document Part seed required");gemFeature(seed,&policy);}
    std::vector<ObjectSnapshot> inputs{capture(*gem)};
    for(auto* seed:seeds)inputs.push_back(capture(*seed));
    QJsonArray attributes;std::size_t attributeBytes=2;
    for(std::size_t i=0;i<seeds.size();++i){
        const auto row=sourceAttributes(*seeds[i],inputs[i+1]);
        const auto bytes=QJsonDocument(row).toJson(QJsonDocument::Compact).size();
        if(std::size_t(bytes)+1>om9_prepared_result_limit()-attributeBytes)throw failure("InvalidBasis",identity,"aggregate template metadata exceeds result budget");
        attributeBytes+=std::size_t(bytes)+1;attributes.append(row);
    }
    const auto attributesRaw=QJsonDocument(attributes).toJson(QJsonDocument::Compact).toStdString();
    metadata(attributesRaw,seeds.size(),identity);
    auto initial=dimensions(gem,&policy);double measured[6]={initial.x,initial.y,initial.z,initial.x,initial.y,initial.z},result[6];
    admission(policy,representation(gemFeature(gem,&policy)->Shape.getShape().getShape()),identity,3,measured);
    if(!om9_builder_recipe_plan(policy.raw.data(),measured,measured+3,scale!=0,result))throw failure("InvalidBasis",identity,"existing affine plan rejected admitted dimensions");
    TopoDS_Compound compound;BRep_Builder builder;builder.MakeCompound(compound);
    for(auto* seed:seeds){auto* f=gemFeature(seed,&policy);auto s=copied(f->Shape.getShape(),seed->getNameInDocument());const auto parent=App::GeoFeature::getGlobalPlacement(seed)*f->Placement.getValue().inverse();s.transformShape((App::GeoFeature::getGlobalPlacement(gem).inverse()*parent).toMatrix(),true);builder.Add(compound,s.getShape());}
    Part::TopoShape stored(compound);borrowedShape(stored,identity+".Templates");
    const EvidenceInputs input{policy,inputs[0],gem->getDocument()->getName(),initial,initial,
        static_cast<unsigned>(gemFeature(gem,&policy)->Shape.getShape().getShape().ShapeType()),attributes};
    const auto prepared=prepareOutputs(stored,result,App::GeoFeature::getGlobalPlacement(gem),identity,input);
    auto& doc=*gem->getDocument();verify(doc,inputs);editable(gem);
    const auto commitIdentity=publicationIdentity(inputs);
    PublicationScope publication{doc,{}};
    if(!doc.openTransactionPreservingRedo("Record Builder History"))throw failure("StaleGeometry",identity,"Builder publication transaction could not be opened");
    try{verify(doc,inputs);auto* r=create(gem,std::string(policy.id),std::string(policy.raw),stored,initial,scale!=0,attributesRaw);restore(r,prepared,publication);verify(doc,inputs);publication.recompute({r});verify(doc,inputs);auto py=own(r->getPyObject());commitPublication(doc,"Record Builder History",commitIdentity);return py.release();}catch(...){doc.abortTransaction();throw;}
});}
static PyObject* restoreAPI(PyObject*,PyObject* args){return api([&]()->PyObject*{
    PyObject* py;if(!PyArg_ParseTuple(args,"O",&py))return nullptr;
    auto* r=dynamic_cast<BuilderRecord*>(native(py));if(!r)throw std::runtime_error("Expected native BuilderRecord");
    editable(r);r->validate();auto& doc=*r->getDocument();
    std::vector<ObjectSnapshot> inputs{capture(*r),capture(*r->SourceGem.getValue())};
    double values[6];plan(*r,values);
    const auto prepared=prepareOutputs(r->Templates.getShape(),values,
        App::GeoFeature::getGlobalPlacement(r->SourceGem.getValue()),r->getNameInDocument(),evidence(*r),r);
    for(unsigned i=0;i<prepared.names.size();++i)if(auto* b=ownedSlot(*r,prepared.names[i],i))inputs.push_back(capture(*b));
    verify(doc,inputs);editable(r);
    // OutputNames and output Shapes are planned writes. Source snapshot remains
    // valid throughout publication; the immutable recipe was checked above.
    const std::vector<ObjectSnapshot> source{inputs[1]};
    const auto commitIdentity=publicationIdentity(inputs);
    PublicationScope publication{doc,{}};
    if(!doc.openTransactionPreservingRedo("Restore Builder Outputs"))throw failure("StaleGeometry",r->getNameInDocument(),"Builder publication transaction could not be opened");
    try{verify(doc,inputs);auto objects=restore(r,prepared,publication);verify(doc,source);publication.recompute({r});verify(doc,source);auto out=own(pyList(objects));commitPublication(doc,"Restore Builder Outputs",commitIdentity);return out.release();}catch(...){doc.abortTransaction();throw;}
});}
static PyObject* matchAPI(PyObject*,PyObject* args){return api([&]()->PyObject*{
    PyObject *sourcePy,*targetsPy;if(!PyArg_ParseTuple(args,"OO",&sourcePy,&targetsPy))return nullptr;
    auto* source=native(sourcePy);editable(source);gemFeature(source);auto tag=shapeTag(source);auto targets=sequence(targetsPy);auto& doc=*source->getDocument();
    std::vector<BuilderRecord*> records;std::size_t templateCount=0;
    for(auto* o:doc.getObjects())if(auto* r=dynamic_cast<BuilderRecord*>(o);r&&r->SourceGem.getValue()==source){
        r->validate();const auto count=templates(*r).size();if(!om9_builder_replay_budget(records.size()+1,templateCount+count,targets.size()))throw failure("InvalidBasis",r->getNameInDocument(),"aggregate record/output budget exceeded");templateCount+=count;records.push_back(r);
    }
    if(records.empty())throw failure("InvalidBasis",source->getNameInDocument(),"source gem has no supported Builder records");
    std::vector<ObjectSnapshot> inputs{capture(*source)};
    for(auto* old:records)inputs.push_back(capture(*old));
    for(auto* target:targets)inputs.push_back(capture(*target));
    struct PreparedRecord {Part::TopoShape templates;PreparedOutputs outputs;};
    std::vector<PreparedRecord> prepared;prepared.reserve(records.size()*targets.size());std::size_t resultBytes=0;
    for(auto* target:targets){
        if(target==source||target->getDocument()!=&doc||!om9_builder_shapes_compatible(tag.c_str(),shapeTag(target).c_str()))throw failure("StaleGeometry",target->getNameInDocument(),"distinct same-shape gem in this document required");
        for(auto* r:records){auto policy=request(*r);policy.records=records.size();policy.templates=templateCount;policy.targets=targets.size();auto* f=gemFeature(target,&policy);auto d=dimensions(target,&policy),initial=r->InitialDimensions.getValue();double measured[6]={initial.x,initial.y,initial.z,d.x,d.y,d.z},p[6];admission(policy,representation(f->Shape.getShape().getShape()),target->getNameInDocument(),3,measured);if(!om9_builder_recipe_plan(r->Parameters.getValue(),measured,measured+3,r->ScaleToGem.getValue(),p))throw failure("InvalidBasis",target->getNameInDocument(),"existing affine plan rejected target dimensions");auto owned=copied(r->Templates.getShape(),r->getNameInDocument());auto perRecord=policy;perRecord.records=1;perRecord.templates=templates(*r).size();perRecord.targets=1;
            const EvidenceInputs input{perRecord,capture(*target),doc.getName(),initial,d,
                static_cast<unsigned>(f->Shape.getShape().getShape().ShapeType()),metadata(r->TemplateAttributes.getStrValue(),perRecord.templates,r->getNameInDocument())};
            auto outputs=prepareOutputs(owned,p,App::GeoFeature::getGlobalPlacement(target),target->getNameInDocument(),input);
            for(const auto& result:outputs.results){if(result.size()>om9_prepared_result_limit()-resultBytes)throw failure("InvalidBasis",target->getNameInDocument(),"aggregate Match evidence exceeds Rust result budget");resultBytes+=result.size();}
            prepared.push_back({std::move(owned),std::move(outputs)});}
    }
    verify(doc,inputs);editable(source);const auto commitIdentity=publicationIdentity(inputs);PublicationScope publication{doc,{}};
    if(!doc.openTransactionPreservingRedo("Match Builder Attributes"))throw failure("StaleGeometry",source->getNameInDocument(),"Builder publication transaction could not be opened");
    try{verify(doc,inputs);std::vector<App::DocumentObject*> made;std::vector<BuilderRecord*> newRecords;std::size_t index=0;for(auto* target:targets)for(auto* old:records){const auto& staged=prepared[index++];auto* r=create(target,old->OM9FeatureId.getStrValue(),old->Parameters.getStrValue(),staged.templates,old->InitialDimensions.getValue(),old->ScaleToGem.getValue(),old->TemplateAttributes.getStrValue());restore(r,staged.outputs,publication);made.push_back(r);newRecords.push_back(r);}verify(doc,inputs);publication.recompute(newRecords);verify(doc,inputs);auto py=own(pyList(made));commitPublication(doc,"Match Builder Attributes",commitIdentity);return py.release();}catch(...){doc.abortTransaction();throw;}
});}
}
void AddBuilderHistoryMethods(PyObject* module){static PyMethodDef methods[]={{"createBuilderRecord",OpenMatrix9Gui::createAPI,METH_VARARGS,"Copy seed geometry into a durable gem-local affine template recipe."},{"restoreBuilderOutputs",OpenMatrix9Gui::restoreAPI,METH_VARARGS,"Explicitly recreate missing native outputs from a durable recipe."},{"matchBuilderAttributes",OpenMatrix9Gui::matchAPI,METH_VARARGS,"Copy all supported recipes to explicitly compatible same-shape gems atomically."},{nullptr,nullptr,0,nullptr}};if(PyModule_AddFunctions(module,methods)<0)throw Base::RuntimeError("Cannot register Builder History API");}
