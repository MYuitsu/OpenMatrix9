// SPDX-License-Identifier: LGPL-2.1-or-later
#include "EditGeometry.h"
#include "CoreLayers.h"
#include "CoreGeometryContract.h"
#include "CoreUnits.h"
#include "CoreDiscreteSnapshot.h"
#include "CoreEditResult.h"
#include "CorePreparedSnapshot.h"
#include "NativeInputTree.h"
#include <App/PropertyGeo.h>
#include <App/ComplexGeoData.h>
#include <Mod/Mesh/App/Mesh.h>
#include <Mod/Part/App/TopoShape.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <BRepBuilderAPI_Sewing.hxx>
#include "RustBridge.h"
#include "EditSpecialTypes.h"
#include "HistoryFeature.h"
#include <App/Document.h>
#include <App/Application.h>
#include <App/GeoFeature.h>
#include <App/DocumentObject.h>
#include <App/DocumentObjectPy.h>
#include <Mod/Part/App/PartFeature.h>
#include <fastsignals/signal.h>
#include <QJsonDocument>
#include <algorithm>
#include <cmath>
#include <sstream>
#include <limits>
#include <map>
namespace {
using namespace OpenMatrix9Gui;
using Ref=CurvePyRef;
EditShape owned(PyObject* p){return std::make_shared<Ref>(p);}
std::string text(PyObject* p){const char* s=PyUnicode_AsUTF8(p);if(!s)throw std::runtime_error("Invalid native text");return s;}
std::string attr(PyObject* p,const char* key){Ref s(PyObject_GetAttrString(p,key));return text(s.value);}
bool flag(PyObject* p,const char* key){Ref b(PyObject_CallMethod(p,key,nullptr));int value=PyObject_IsTrue(b.value);if(value<0)throw std::runtime_error("Cannot inspect shape");return value==1;}
Py_ssize_t count(PyObject* p,const char* key){Ref list(PyObject_GetAttrString(p,key));return PySequence_Size(list.value);}
void set(PyObject* p,const char* key,PyObject* value){if(PyObject_SetAttrString(p,key,value)<0)throw std::runtime_error("Cannot assign edit output");}
void property(PyObject* p,const char* type,const char* key,PyObject* value){Ref a(PyObject_CallMethod(p,"addProperty","sss",type,key,"OpenMatrix9"));set(p,key,value);}
std::string signature(PyObject* p){if(specialExplodeSource(p))return specialExplodeSignature(p);const bool mesh=PyObject_HasAttrString(p,"Mesh");Ref shape(PyObject_GetAttrString(p,mesh?"Mesh":"Shape"));std::string result;
    if(mesh)result=editMeshSignature(shape.value);else{
        if(!PyObject_TypeCheck(shape.value,&Part::TopoShapePy::Type))throw std::runtime_error("Unsupported native source Shape owner");
        result=preparedCadNativeMasterDigest(static_cast<Part::TopoShapePy*>(shape.value)->getTopoShapePtr()->getShape()).toHex().toStdString();
    }
    if(PyObject_HasAttrString(p,"getGlobalPlacement")){Ref placement(PyObject_CallMethod(p,"getGlobalPlacement",nullptr)),matrix(PyObject_CallMethod(placement.value,"toMatrix",nullptr));std::ostringstream values;values<<std::hexfloat;
        for(unsigned row=1;row<=4;++row)for(unsigned col=1;col<=4;++col){const auto key="A"+std::to_string(row)+std::to_string(col);Ref value(PyObject_GetAttrString(matrix.value,key.c_str()));const auto n=PyFloat_AsDouble(value.value);if(PyErr_Occurred()||!std::isfinite(n))throw std::runtime_error("Invalid global placement");values<<';'<<n;}result+=values.str();}return result;}
EditShapes elements(PyObject* p,const char* key){Ref list(PyObject_GetAttrString(p,key));EditShapes result;for(Py_ssize_t i=0;i<PySequence_Size(list.value);++i)result.push_back(owned(PySequence_GetItem(list.value,i)));return result;}
void append(PyObject* list,PyObject* p){if(PyList_Append(list,p)<0)throw std::runtime_error("Cannot collect native geometry");}
EditShape combined(const std::vector<EditInput>& inputs,std::size_t begin,std::size_t end){if(begin>=end)throw std::runtime_error("Select both Boolean input sets");auto result=owned(PyObject_CallMethod(inputs[begin].shape->value,"copy",nullptr));for(auto i=begin+1;i<end;++i)result=owned(PyObject_CallMethod(result->value,"fuse","O",inputs[i].shape->value));return result;}
const char* name(unsigned kind){const char* names[]={"","Join","Explode","Trim","BooleanDifference","BooleanIntersection","BooleanUnion","Boolean2Objects"};return names[kind];}
const char* feature(unsigned kind){const char* ids[]={"","OM9-TOP11-008","OM9-TOP11-005","OM9-TOP11-010","OM9-SOLID-001","OM9-SOLID-002","OM9-SOLID-003","OM9-SOLID-004"};return ids[kind];}
extern "C" unsigned om9_native_edit_capability(unsigned,unsigned,unsigned,unsigned);
extern "C" unsigned om9_prepared_cad_budget(std::size_t,std::size_t,std::size_t,std::size_t);
extern "C" std::size_t om9_prepared_result_limit();
unsigned editFlags(unsigned kind){return (kind<=3||om9_edit_delete_input(-1)?4U:0U)|(om9_edit_option(1)?1U:0U)|(om9_edit_option(2)?2U:0U);}
unsigned representation(PyObject* shape){const auto type=attr(shape,"ShapeType");
    if(type=="Vertex")return 1;if(type=="Edge")return 2;if(type=="Wire")return 3;
    if(type=="Face")return 4;if(type=="Shell")return 5;if(type=="Solid")return 6;
    if(type=="Compound"||type=="CompSolid")return 7;return 0;
}
void capability(unsigned kind,unsigned input,unsigned flags,unsigned mode,const std::string& identity){
    if(kind==0)return;const auto status=om9_native_edit_capability(kind,input,flags,mode);
    if(status>=100)throw std::runtime_error(std::string(status==102?"InvalidBasis: ":"UnsupportedRepresentation: ")+(kind<=7?name(kind):"Edit")+" input "+identity+" rejected by shared operation/type/options contract ("+std::to_string(status)+")");
}
void preflight(App::DocumentObject& object,PyObject* original,bool mesh,unsigned kind,const std::string& identity){
    try{
        if(mesh){
            auto* property=object.getPropertyByName<App::PropertyComplexGeoData>("Mesh");
            const auto type=Base::Type::fromName("Mesh::PropertyMeshKernel");
            if(!property||type.isBad()||property->getTypeId()!=type)throw std::runtime_error("Unsupported native mesh property owner");
            const auto* owner=property->getComplexData();if(!owner)throw std::runtime_error("Missing native mesh owner");
            const auto nativeType=Base::Type::fromName("Mesh::MeshObject");
            if(nativeType.isBad()||owner->getTypeId()!=nativeType)throw std::runtime_error("Unsupported native mesh kernel owner");
            const auto& native=*static_cast<const Mesh::MeshObject*>(owner);meshInputBudget(native);meshSnapshot(native);
        }else{
            if(!PyObject_TypeCheck(original,&Part::TopoShapePy::Type))throw std::runtime_error("Unsupported native Part shape owner");
            auto* wrapper=static_cast<Part::TopoShapePy*>(original);
            if(!wrapper->Base::PyObjectBase::isValid())throw std::runtime_error("Native Part shape is no longer live");
            ThreeDm::InputTree::validate(wrapper->getTopoShapePtr()->getShape());
        }
    }catch(const ThreeDm::GeometryRepresentationUnavailable& e){throw std::runtime_error(std::string("UnsupportedRepresentation: ")+(kind&&kind<=7?name(kind):"Inspect")+" input "+identity+": "+e.what());}
     catch(const MeshInputFailure& e){const char* code=e.status==1?"UnsupportedRepresentation":e.status==3?"InvalidTopology":e.status==4?"NonFiniteGeometry":"InvalidBasis";
        throw std::runtime_error(std::string(code)+": "+(kind&&kind<=7?name(kind):"Inspect")+" input "+identity+": "+e.what());}
     catch(const std::exception& e){throw std::runtime_error(std::string("InvalidBasis: ")+(kind&&kind<=7?name(kind):"Inspect")+" input "+identity+": "+e.what());}
}
void valid(PyObject* shape){if(flag(shape,"isNull")||!flag(shape,"isValid"))throw std::runtime_error("Kernel produced empty or invalid geometry; inputs are unchanged");}
bool solidOnly(PyObject* shape){const auto type=attr(shape,"ShapeType");if(type=="Solid")return flag(shape,"isClosed");if(type!="Compound"&&type!="CompSolid")return false;
    Ref children(PyObject_CallMethod(shape,"childShapes",nullptr));const auto n=PySequence_Size(children.value);if(n<=0)return false;
    for(Py_ssize_t i=0;i<n;++i){Ref child(PySequence_GetItem(children.value,i));if(!solidOnly(child.value))return false;}return true;}
bool surfaceOnly(PyObject* shape){const auto type=attr(shape,"ShapeType");if(type=="Face"||type=="Shell")return true;if(type!="Compound")return false;Ref children(PyObject_CallMethod(shape,"childShapes",nullptr));if(PySequence_Size(children.value)<1)return false;for(Py_ssize_t i=0;i<PySequence_Size(children.value);++i){Ref child(PySequence_GetItem(children.value,i));if(!surfaceOnly(child.value))return false;}return true;}
// Native synchronous callbacks can run during any document write. Observe
// source/ancestor changes throughout publication, including deliberate History
// detachment and group membership updates. No callback exception or native
// pointer leaves this GUI-thread stack scope.
class EditPublicationGuard {
    App::Document& doc;const std::vector<EditInput>& inputs;std::string documentName,bad,operation;
    std::set<std::uint64_t> sources,parents,history,changedSources,changedParents,retired;
    std::uint64_t deleting=0,detaching=0,addingGroup=0;bool callbackFailure=false,recomputing=false;
    std::map<std::uint64_t,TopoDS_Shape> sourceShapes;
    struct GroupState {std::string name;std::vector<std::uint64_t> members;};
    std::map<std::uint64_t,GroupState> groups;
    fastsignals::scoped_connection changed,removed;
    static std::vector<std::uint64_t> members(const App::PropertyLinkList& property){
        const auto& values=property.getValues();if(om9_prepared_cad_budget(0,0,values.size(),0))throw std::runtime_error("InvalidBasis: Edit ancestor membership budget");
        std::vector<std::uint64_t> ids;ids.reserve(values.size());for(auto* o:values)ids.push_back(o?std::uint64_t(o->getID()):0);return ids;
    }
    static bool detachedValue(const App::Property& p){
        if(auto* value=dynamic_cast<const App::PropertyBool*>(&p))return !value->getValue();
        if(auto* value=dynamic_cast<const App::PropertyLink*>(&p))return !value->getValue();
        if(auto* value=dynamic_cast<const App::PropertyLinkSub*>(&p))return !value->getValue()&&value->getSubValues().empty();
        if(auto* value=dynamic_cast<const App::PropertyLinkList*>(&p))return value->getValues().empty();
        if(auto* value=dynamic_cast<const App::PropertyLinkSubList*>(&p))return value->getValues().empty();
        return false;
    }
public:
    EditPublicationGuard(App::Document& d,const std::vector<EditInput>& values,unsigned kind):doc(d),inputs(values),documentName(d.getName()),operation(name(kind)){
        for(const auto& i:inputs){sources.insert(i.guard.stamp.object);for(const auto& a:i.guard.ancestors)parents.insert(a.object);
            auto* object=d.getObject(i.name.c_str());
            if(hasRecordedHistory(object))history.insert(i.guard.stamp.object);
            if(!i.mesh&&i.components.empty())if(auto* native=dynamic_cast<Part::Feature*>(object))sourceShapes.emplace(i.guard.stamp.object,native->Shape.getValue());}
        std::size_t groupNodes=0;
        for(auto* o:d.getObjects())if(parents.contains(std::uint64_t(o->getID())))if(auto* p=o->getPropertyByName<App::PropertyLinkList>("Group")){
            if(p->getValues().size()>std::numeric_limits<std::size_t>::max()-groupNodes)throw std::runtime_error("InvalidBasis: Edit ancestor membership overflow");
            groupNodes+=p->getValues().size();if(om9_prepared_cad_budget(0,0,groupNodes,0))throw std::runtime_error("InvalidBasis: Edit aggregate ancestor membership budget");
            groups.emplace(std::uint64_t(o->getID()),GroupState{o->getNameInDocument(),members(*p)});}
        auto& app=App::GetApplication();
        changed=app.signalChangedObject.connect([this](const App::DocumentObject& o,const App::Property& p){
          try{
            if(o.getDocument()!=&doc)return;const auto id=std::uint64_t(o.getID());if(retired.contains(id)||id==deleting)return;
            const char* raw=o.getPropertyName(&p);const std::string key=raw?raw:"";
            if(sources.contains(id)){
                if(key=="_GroupTouched")return;
                // FreeCAD may reassign the same native master with a new root
                // Location datum during this command's recompute. Admit only
                // this exact cache/frame refresh; replacements and real edits
                // (including observer edits nested in recompute) remain stale.
                if(recomputing&&o.isRecomputing()&&key=="Shape"){
                    const auto original=sourceShapes.find(id);const auto* native=dynamic_cast<const Part::Feature*>(&o);
                    const auto input=std::find_if(inputs.begin(),inputs.end(),[id](const EditInput& i){return i.guard.stamp.object==id;});
                    if(original!=sourceShapes.end()&&native&&input!=inputs.end()
                        &&native->Shape.getValue().IsPartner(original->second)
                        &&native->Shape.getValue().Orientation()==original->second.Orientation()){
                        Ref py(const_cast<App::DocumentObject&>(o).getPyObject());
                        if(signature(py.value)==input->signature){changedSources.insert(id);return;}
                    }
                }
                static const std::set<std::string> detachFields{"Recorded","SourceCurve","Sources","Source","UnitSource","Parents","ParentRecord","PlacementSources","HistoryDirty","HistoryEnabled"};
                if(id==detaching&&history.contains(id)&&detachFields.contains(key)&&detachedValue(p)){changedSources.insert(id);return;}
                bad=o.getNameInDocument();
            }else if(parents.contains(id)){
                if(key=="_GroupTouched"||key=="Label"||key=="Label2"||key=="Visibility")return;
                if(key=="Group"&&(id==addingGroup||deleting)){
                    const auto expected=groups.find(id);const auto* value=dynamic_cast<const App::PropertyLinkList*>(&p);
                    if(expected!=groups.end()&&value&&members(*value)==expected->second.members){changedParents.insert(id);return;}}
                bad=o.getNameInDocument();
            }
          }catch(...){callbackFailure=true;}
        });
        removed=app.signalDeletedObject.connect([this](const App::DocumentObject& o){
          try{
            if(o.getDocument()!=&doc)return;const auto id=std::uint64_t(o.getID());
            if(id==deleting){retired.insert(id);return;}
            if(sources.contains(id)||parents.contains(id))bad=o.getNameInDocument();
          }catch(...){callbackFailure=true;}
        });
    }
    bool live()const{return App::GetApplication().getDocument(documentName.c_str())==&doc;}
    void recompute(){check();recomputing=true;try{doc.recompute();}catch(...){recomputing=false;throw;}recomputing=false;check();}
    void detach(App::DocumentObject& object,bool begin){
        if(begin){check();detaching=std::uint64_t(object.getID());}
        else{detaching=0;check();if(hasRecordedHistory(&object))throw std::runtime_error("StaleGeometry: "+operation+" History detach failed: "+std::string(object.getNameInDocument()));}
    }
    void addGroup(App::DocumentObject& group,App::DocumentObject& output){
        check();addingGroup=std::uint64_t(group.getID());auto found=groups.find(addingGroup);
        if(found!=groups.end()){auto* p=group.getPropertyByName<App::PropertyLinkList>("Group");if(!p||members(*p)!=found->second.members)throw std::runtime_error("StaleGeometry: "+operation+" group changed before publication: "+found->second.name);
            const auto id=std::uint64_t(output.getID());if(std::find(found->second.members.begin(),found->second.members.end(),id)==found->second.members.end())found->second.members.push_back(id);}
    }
    void endGroup(){addingGroup=0;check();}
    void remove(std::uint64_t id){deleting=id;if(id)for(auto& [key,state]:groups)state.members.erase(std::remove(state.members.begin(),state.members.end(),id),state.members.end());}
    void check(){
        if(!live())throw std::runtime_error("StaleGeometry: "+operation+" document closed during publication: "+documentName);
        if(callbackFailure)throw std::runtime_error("StaleGeometry: "+operation+" native callback observation failed: "+documentName);
        if(!bad.empty())throw std::runtime_error("StaleGeometry: "+operation+" source changed during publication: "+bad);
        for(const auto& i:inputs){if(retired.contains(i.guard.stamp.object))continue;
            const auto now=captureObject(doc,i.name);const auto& old=i.guard;const auto& a=old.stamp;const auto& b=now.stamp;
            if(a.document!=b.document||a.object!=b.object||a.generation!=b.generation||a.epoch!=b.epoch
                ||(a.revision!=b.revision&&!changedSources.contains(a.object))||old.world!=now.world
                ||old.unitContext!=now.unitContext||old.documentUid!=now.documentUid||old.ancestors.size()!=now.ancestors.size())
                throw std::runtime_error("StaleGeometry: "+operation+" source changed during publication: "+i.name);
            for(std::size_t j=0;j<old.ancestors.size();++j){const auto& p=old.ancestors[j];const auto& q=now.ancestors[j];
                if(p.document!=q.document||p.object!=q.object||p.generation!=q.generation||p.epoch!=q.epoch
                    ||(p.revision!=q.revision&&!changedParents.contains(p.object)))
                    throw std::runtime_error("StaleGeometry: "+operation+" source ancestor changed during publication: "+i.name);}
            auto* object=doc.getObject(i.name.c_str());if(!object||!CoreLayers::objectEditable(object))throw std::runtime_error("StaleGeometry: "+operation+" source missing or no longer editable: "+i.name);
            Ref py(object->getPyObject());if(signature(py.value)!=i.signature)throw std::runtime_error("StaleGeometry: "+operation+" source geometry changed during publication: "+i.name);
        }
        for(const auto& [id,state]:groups){auto* o=doc.getObject(state.name.c_str());auto* p=o?o->getPropertyByName<App::PropertyLinkList>("Group"):nullptr;
            if(!o||std::uint64_t(o->getID())!=id||!p||members(*p)!=state.members)throw std::runtime_error("StaleGeometry: "+operation+" ancestor membership changed: "+state.name);}
    }
};
}
namespace OpenMatrix9Gui {
void validateExplodeNativeLeaf(PyObject* object,const std::string& identity){
    if(!PyObject_TypeCheck(object,&App::DocumentObjectPy::Type))throw std::runtime_error("UnsupportedRepresentation: Explode input "+identity+" has no native document owner");
    auto* wrapper=static_cast<App::DocumentObjectPy*>(object);
    if(!wrapper->Base::PyObjectBase::isValid()||!wrapper->getDocumentObjectPtr())throw std::runtime_error("StaleGeometry: Explode input "+identity+" is no longer live");
    const bool mesh=PyObject_HasAttrString(object,"Mesh");
    if(!mesh&&!PyObject_HasAttrString(object,"Shape"))throw std::runtime_error("UnsupportedRepresentation: Explode input "+identity+" has no native geometry");
    Ref original(PyObject_GetAttrString(object,mesh?"Mesh":"Shape"));
    preflight(*wrapper->getDocumentObjectPtr(),original.value,mesh,2,identity);
}
void validateEditCapabilities(const std::vector<EditInput>& inputs,unsigned kind,unsigned mode){
    for(const auto& i:inputs)capability(kind,!i.components.empty()?13:i.mesh?8:representation(i.shape->value),editFlags(kind),mode,i.name);
}
static EditInput readInput(App::Document& doc,const std::string& name,unsigned kind,bool allowBlocks,const NativeShapeAdmission* admission){
    auto* o=doc.getObject(name.c_str());if(!o){
        if(admission)throw std::runtime_error(std::string("StaleGeometry: ")+admission->operation+" input "+name+" is absent from the active document");
        throw std::runtime_error("Select an object in the active document");
    }
    auto guard=captureObject(doc,name);
    Ref object(o->getPyObject());if(kind==2&&specialExplodeSource(object.value)){capability(kind,o->isDerivedFrom(Base::Type::fromName("App::Link"))?10:13,editFlags(kind),0,name);auto parts=specialExplodeComponents(object.value,allowBlocks);auto preview=specialExplodePreview(parts);return {name,signature(object.value),preview,false,parts,std::move(guard)};}
    const bool mesh=PyObject_HasAttrString(object.value,"Mesh");if(o->isDerivedFrom(Base::Type::fromName("App::Link"))||(!mesh&&!PyObject_HasAttrString(object.value,"Shape"))){
        if(admission)throw std::runtime_error(std::string("UnsupportedRepresentation: ")+admission->operation+" input "+name+" has no supported native shape owner");
        throw std::runtime_error("Select native curve/surface/solid/mesh objects; blocks require explicit CMD Explode");
    }
    if(kind==2){if(PyObject_HasAttrString(object.value,"String")&&PyObject_HasAttrString(object.value,"FontFile"))throw std::runtime_error("Font/text Explode requires a dedicated annotation adapter; source is preserved");
        if(PyObject_HasAttrString(object.value,"Proxy")){Ref proxy(PyObject_GetAttrString(object.value,"Proxy"));if(proxy.value!=Py_None&&PyObject_HasAttrString(proxy.value,"Type")){Ref type(PyObject_GetAttrString(proxy.value,"Type"));if(PyUnicode_Check(type.value)){const auto name=text(type.value);if(name=="Dimension"||name=="AngularDimension"||name=="Text"||name=="Label"||name=="ShapeString")throw std::runtime_error("Annotation Explode requires a dedicated adapter; source is preserved");}}}}
    Ref original(PyObject_GetAttrString(object.value,mesh?"Mesh":"Shape"));
    if(!mesh&&!PyObject_TypeCheck(original.value,&Part::TopoShapePy::Type))throw std::runtime_error(std::string("UnsupportedRepresentation: ")+(admission?admission->operation:"Edit")+" input "+name+" has no native Part shape owner");
    capability(kind,mesh?8:representation(original.value),editFlags(kind),kind==7?om9_edit_mode():0,name);
    // A consumer can reject a known unsupported mesh before native extraction.
    if(admission&&mesh)admission->check(original.value,true);
    if(admission&&!mesh)validateNativePartShape(original.value,admission->operation,name);
    else preflight(*o,original.value,mesh,kind,name);
    if(admission&&!mesh)admission->check(original.value,false);
    auto shape=owned(PyObject_CallMethod(original.value,"copy",nullptr));if(mesh){if(kind!=2&&kind<4)throw std::runtime_error("Mesh is supported by Explode and Boolean commands");validateEditMesh(shape->value,kind>=4);}else{valid(shape->value);
    const auto type=attr(shape->value,"ShapeType");
    if(kind>=4){if(!solidOnly(shape->value)&&!surfaceOnly(shape->value))throw std::runtime_error("Boolean supports closed solids or surface-only BRep inputs");}
    else if(kind==3){if(type!="Edge"&&type!="Wire"&&type!="Face"&&type!="Shell")throw std::runtime_error("Trim supports native curves and open surfaces/polysurfaces");}
    else if(kind==1){if(type!="Edge"&&type!="Wire"&&type!="Face"&&type!="Shell")throw std::runtime_error("Join supports curves or open surfaces, not solids/compounds");}
    else if(kind==2&&type!="Wire"&&type!="Shell"&&type!="Solid"&&type!="Compound"&&type!="CompSolid")throw std::runtime_error("Explode supports polycurves, polysurfaces and compound components; blocks are preserved");}
    if(PyObject_HasAttrString(object.value,"getGlobalPlacement")){
        Ref global(PyObject_CallMethod(object.value,"getGlobalPlacement",nullptr)),local(PyObject_GetAttrString(object.value,"Placement")),inverse(PyObject_CallMethod(local.value,"inverse",nullptr)),parent(PyNumber_Multiply(global.value,inverse.value)),matrix(PyObject_CallMethod(parent.value,"toMatrix",nullptr));
        if(mesh)set(shape->value,"Placement",global.value);else{Ref moved(PyObject_CallMethod(shape->value,"transformShape","OO",matrix.value,Py_False));}
    }
    return {name,signature(object.value),shape,mesh,{},std::move(guard)};
}
EditInput editInput(App::Document& doc,const std::string& name,unsigned kind,bool allowBlocks){if(!CoreLayers::objectEditable(doc.getObject(name.c_str())))throw std::runtime_error("Input is hidden, locked or not selectable");return readInput(doc,name,kind,allowBlocks,nullptr);}
EditInput nativeShapeInput(App::Document& doc,const std::string& name,const NativeShapeAdmission* admission){return readInput(doc,name,0,false,admission);}
void verifyEditInputs(App::Document& doc,const std::vector<EditInput>& inputs){for(const auto& i:inputs){verifyObject(doc,i.guard);auto* o=doc.getObject(i.name.c_str());if(!o)throw std::runtime_error("An edit input was deleted; cancel and reselect");if(CoreLayers::objectLocked(o))throw std::runtime_error("An edit input layer is locked; unlock and retry");Ref object(o->getPyObject());if(signature(object.value)!=i.signature)throw std::runtime_error("An edit input changed; cancel and reselect");}}
ToleranceSnapshot editJoinTolerance(const std::string& context,const std::vector<EditInput>& inputs,const std::optional<double>& overrideMm){
    if(inputs.empty())throw std::runtime_error("Select Join inputs before resolving tolerance");
    const bool surfaces=count(inputs.front().shape->value,"Faces")!=0;double bound=0;
    for(const auto& input:inputs){
        Ref box(PyObject_GetAttrString(input.shape->value,"BoundBox"));
        for(const char* key:{"XMin","XMax","YMin","YMax","ZMin","ZMax"}){
            Ref value(PyObject_GetAttrString(box.value,key));const double coordinate=PyFloat_AsDouble(value.value);
            if(PyErr_Occurred()||!std::isfinite(coordinate))throw std::runtime_error("Invalid Join input bounds");
            bound=std::max(bound,std::abs(coordinate));
        }
    }
    return resolveTolerance(context,surfaces?2U:1U,overrideMm,bound);
}
EditShapes buildEdit(const std::vector<EditInput>& inputs,unsigned kind,std::size_t first,unsigned mode,double tolerance){
    // Spec: OM9-TOP11-005/008; OM9-SOLID-001..004. No interpreted command text.
    if(inputs.empty()||kind<1||kind>7)throw std::runtime_error("Select input objects");
    validateEditCapabilities(inputs,kind,mode);
    for(const auto& i:inputs)if(kind!=2&&i.mesh!=inputs[0].mesh)throw std::runtime_error("Mesh and BRep input types cannot be mixed in one Edit operation");
    if(inputs[0].mesh&&kind>=4){auto converted=inputs;for(auto& i:converted){i.shape=meshEditSolid(i.shape->value);i.mesh=false;}auto solids=buildEdit(converted,kind,first,mode,tolerance);EditShapes meshes;for(auto& s:solids)meshes.push_back(meshEditResult(s->value));return meshes;}
    Ref part(PyImport_ImportModule("Part"));EditShapes output;
    if(kind==1){
        bool curves=count(inputs[0].shape->value,"Faces")==0;Ref list(PyList_New(0));
        for(const auto& i:inputs){if((count(i.shape->value,"Faces")==0)!=curves)throw std::runtime_error("Join curves and surfaces separately");for(auto& e:elements(i.shape->value,curves?"Edges":"Faces"))append(list.value,e->value);}
        if(curves)output.push_back(joinEditCurves(list.value,tolerance));
        else{
            if(!std::isfinite(tolerance)||tolerance<=0)throw std::runtime_error("Surface Join requires a positive finite tolerance");
            BRepBuilderAPI_Sewing sewing(tolerance);
            // OCCT may clamp small requests to its own resolution. Never report
            // a cutoff smaller than the one the backend will actually use.
            if(sewing.Tolerance()!=tolerance)throw std::runtime_error("Surface Join tolerance is below the kernel resolution; choose an explicit supported tolerance");
            for(Py_ssize_t index=0;index<PyList_Size(list.value);++index){
                auto* face=PyList_GetItem(list.value,index);
                if(!PyObject_TypeCheck(face,&Part::TopoShapePy::Type))throw std::runtime_error("Surface Join requires native faces");
                sewing.Add(static_cast<Part::TopoShapePy*>(face)->getTopoShapePtr()->getShape());
            }
            sewing.Perform();auto shell=owned(new Part::TopoShapePy(new Part::TopoShape(sewing.SewedShape())));
            if(attr(shell->value,"ShapeType")!="Shell"||count(shell->value,"Shells")!=1||count(shell->value,"Faces")!=PySequence_Size(list.value))throw std::runtime_error("Surfaces must share compatible edges within the effective Join tolerance");output.push_back(shell);
        }
    }else if(kind==2){for(const auto& i:inputs){EditShapes pieces;if(!i.components.empty())pieces=i.components;else if(i.mesh){Ref list(PyObject_CallMethod(i.shape->value,"getSeparateComponents",nullptr));for(Py_ssize_t j=0;j<PySequence_Size(list.value);++j)pieces.push_back(owned(PySequence_GetItem(list.value,j)));}else{const auto type=attr(i.shape->value,"ShapeType");if(type=="Compound"||type=="CompSolid"){Ref list(PyObject_CallMethod(i.shape->value,"childShapes",nullptr));for(Py_ssize_t j=0;j<PySequence_Size(list.value);++j)pieces.push_back(owned(PySequence_GetItem(list.value,j)));}else pieces=elements(i.shape->value,count(i.shape->value,"Faces")?"Faces":"Edges");}if(pieces.empty()||(i.components.empty()&&pieces.size()<2))throw std::runtime_error("Explode input needs separable components");output.insert(output.end(),pieces.begin(),pieces.end());}}
    else if(kind>=4){EditShape shape;const bool surfaces=surfaceOnly(inputs[0].shape->value);for(const auto& i:inputs)if(surfaceOnly(i.shape->value)!=surfaces)throw std::runtime_error("Select solid-only or surface-only Boolean sets");
        if(kind==6)shape=combined(inputs,0,inputs.size());
        else {if(kind==7){if(inputs.size()!=2)throw std::runtime_error("Boolean2Objects needs exactly two objects");first=1;}
            auto a=combined(inputs,0,first),b=combined(inputs,first,inputs.size());
            const auto operation=kind==7?mode:kind==4?1U:3U;
            if(operation==0)shape=owned(PyObject_CallMethod(a->value,"fuse","O",b->value));
            else if(operation==1)shape=owned(PyObject_CallMethod(a->value,"cut","O",b->value));
            else if(operation==2)shape=owned(PyObject_CallMethod(b->value,"cut","O",a->value));
            else if(operation==3)shape=owned(PyObject_CallMethod(a->value,"common","O",b->value));
            else{auto ab=owned(PyObject_CallMethod(a->value,"cut","O",b->value)),ba=owned(PyObject_CallMethod(b->value,"cut","O",a->value));shape=owned(PyObject_CallMethod(ab->value,"fuse","O",ba->value));}
        }
        valid(shape->value);if(surfaces){if(!surfaceOnly(shape->value))throw std::runtime_error("Surface Boolean has no surface-area result; inputs are unchanged");output.push_back(shape);}else{output=elements(shape->value,"Solids");if(output.empty())throw std::runtime_error("Boolean has no solid result; inputs are unchanged");}
    }else throw std::runtime_error("Trim requires a picked segment");
    for(const auto& shape:output)if(isExplodedText(shape->value))continue;else if(isEditMesh(shape->value))validateEditMesh(shape->value,false);else valid(shape->value);return output;
}
std::vector<EditShapes> splitEditCurves(const std::vector<EditInput>& inputs,bool extendLines){
    if(inputs.size()<2)throw std::runtime_error("Trim needs at least two intersecting objects");
    Ref list(PyList_New(0));for(std::size_t i=1;i<inputs.size();++i)append(list.value,inputs[i].shape->value);
    if(extendLines)for(auto& e:extendedEditLines(inputs))append(list.value,e->value);
    Ref fuse(PyObject_CallMethod(inputs[0].shape->value,"generalFuse","Od",list.value,0.0)),mapping(PySequence_GetItem(fuse.value,1));
    std::vector<EditShapes> split;bool changed=false;
    for(std::size_t i=0;i<inputs.size();++i){const char* key=count(inputs[i].shape->value,"Faces")?"Faces":"Edges";Ref parts(PySequence_GetItem(mapping.value,i));EditShapes edges;for(Py_ssize_t j=0;j<PySequence_Size(parts.value);++j){Ref piece(PySequence_GetItem(parts.value,j));for(auto& edge:elements(piece.value,key)){bool duplicate=false;for(auto& existing:edges){Ref same(PyObject_CallMethod(edge->value,"isSame","O",existing->value));if(PyObject_IsTrue(same.value)==1){duplicate=true;break;}}if(!duplicate)edges.push_back(edge);}}
        changed|=edges.size()>std::size_t(count(inputs[i].shape->value,key));split.push_back(std::move(edges));}
    if(!changed)throw std::runtime_error("No true 3D intersections to trim; try ExtendLines or ApparentIntersections for curves");return split;
}
std::size_t pickedEditSegment(const EditShapes& shapes,const std::set<std::size_t>& removed,const std::array<double,3>& point){
    Ref app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part")),p(PyObject_CallMethod(app.value,"Vector","ddd",point[0],point[1],point[2])),vertex(PyObject_CallMethod(part.value,"Vertex","O",p.value));
    double nearest=1e-4;std::size_t pick=shapes.size();bool ambiguous=false;
    for(std::size_t i=0;i<shapes.size();++i){if(removed.contains(i))continue;Ref d(PyObject_CallMethod(shapes[i]->value,"distToShape","O",vertex.value)),value(PySequence_GetItem(d.value,0));const auto distance=PyFloat_AsDouble(value.value);if(!std::isfinite(distance))throw std::runtime_error("Invalid pick distance");if(distance<nearest-1e-8){nearest=distance;pick=i;ambiguous=false;}else if(distance<=nearest+1e-8&&pick!=shapes.size()){ambiguous=true;}}
    if(pick==shapes.size()||ambiguous)throw std::runtime_error("Pick inside one remaining curve segment, away from intersections");return pick;
}
void verifyEditPickBoundary(const EditShapes& shapes,const std::set<std::size_t>& removed,std::size_t picked,const std::array<double,3>& point,double pixelUncertainty){
    Ref app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part")),p(PyObject_CallMethod(app.value,"Vector","ddd",point[0],point[1],point[2])),vertex(PyObject_CallMethod(part.value,"Vertex","O",p.value));
    // Coin picks carry float coordinates; include one ULP per coordinate when
    // identifying a shared boundary, without relaxing analytic CMD pick tolerance.
    double uncertaintySquared=0;
    for(double value:point){const float f=float(value);const double up=double(std::nextafter(f,std::numeric_limits<float>::infinity()))-f,down=double(f)-std::nextafter(f,-std::numeric_limits<float>::infinity());const double ulp=std::max(up,down);uncertaintySquared+=ulp*ulp;}
    const double tolerance=1e-4+std::sqrt(uncertaintySquared)+pixelUncertainty;
    const char* key=count(shapes.at(picked)->value,"Faces")?"Edges":"Vertexes";
    for(const auto& boundary:elements(shapes[picked]->value,key)){
        Ref distance(PyObject_CallMethod(boundary->value,"distToShape","O",vertex.value)),d(PySequence_GetItem(distance.value,0));if(PyFloat_AsDouble(d.value)>tolerance)continue;
        for(std::size_t i=0;i<shapes.size();++i)if(i!=picked&&!removed.contains(i))for(const auto& other:elements(shapes[i]->value,key)){Ref same(PyObject_CallMethod(boundary->value,"isSame","O",other->value));bool shared=PyObject_IsTrue(same.value)==1;
            if(!shared&&std::string(key)=="Vertexes"){Ref separation(PyObject_CallMethod(boundary->value,"distToShape","O",other->value)),d(PySequence_GetItem(separation.value,0));shared=PyFloat_AsDouble(d.value)<=1e-7;}
            if(shared)throw std::runtime_error("Pick inside a region, away from shared split boundaries");}
    }
}
void commitEdit(App::Document& doc,const std::vector<EditInput>& inputs,const EditShapes& shapes,unsigned kind,bool remove,unsigned mode,std::size_t first,const EditProjection* projection,const ToleranceSnapshot* toleranceContext){
    const double tolerance=toleranceContext?toleranceContext->values.effectiveMm:om9_edit_tolerance();
    if(toleranceContext&&unitContextSnapshot(doc)!=toleranceContext->context)throw std::runtime_error("Join unit/tolerance context changed; cancel and reselect");
    verifyEditInputs(doc,inputs);for(const auto& s:shapes)if(isExplodedText(s->value))continue;else if(isEditMesh(s->value))validateEditMesh(s->value,kind>=4);else valid(s->value);
    // Deleting dependent/parametric inputs would sever unrelated models. Retain them instead.
    std::vector<std::string> deleting;for(std::size_t index=0;index<inputs.size();++index)if(remove||(kind==4&&index<first))deleting.push_back(inputs[index].name);
    for(const auto& name:deleting){auto* o=doc.getObject(name.c_str());for(auto* dependent:o->getInList()){
        if(dependent->isDerivedFrom(Base::Type::fromName("App::Part"))||dependent->isDerivedFrom(Base::Type::fromName("App::DocumentObjectGroup"))||hasRecordedHistory(dependent))continue;
        throw std::runtime_error("DeleteInput is unsafe for referenced objects; retain inputs or detach their dependents");
    }}
    const auto layer=kind==2?CircleLayerSnapshot{}:CoreLayers::captureCircle(doc);
    // Resolve component ownership and groups before opening the transaction.
    // Special/mesh consumers retain their existing adapters until their result
    // snapshot contract is integrated; they must not masquerade as CAD evidence.
    std::vector<std::size_t> owners;if(kind==2)for(std::size_t i=0;i<inputs.size();++i){auto pieces=buildEdit({inputs[i]},2,0,0,tolerance);owners.insert(owners.end(),pieces.size(),i);}
    if(kind==2&&owners.size()!=shapes.size())throw std::runtime_error("InvalidTopology: Explode output/source count changed");
    std::vector<EditShapes> outputGroups(shapes.size());
    for(std::size_t index=0;kind==2&&index<shapes.size();++index){auto* input=doc.getObject(inputs[owners[index]].name.c_str());
        for(auto* dependent:input->getInList())if(dependent->isDerivedFrom(Base::Type::fromName("App::Part"))||dependent->isDerivedFrom(Base::Type::fromName("App::DocumentObjectGroup")))outputGroups[index].push_back(owned(dependent->getPyObject()));}
    const bool cad=std::all_of(inputs.begin(),inputs.end(),[](const auto& i){return !i.mesh&&i.components.empty();})
        &&std::all_of(shapes.begin(),shapes.end(),[](const auto& s){return !isExplodedText(s->value)&&!isEditMesh(s->value);});
    std::vector<PreparedEditOutput> prepared;
    if(cad){
        std::size_t nativeNodes=0,nativeBytes=0;
        auto account=[&](PyObject* value){auto* wrapper=static_cast<Part::TopoShapePy*>(value);
            const auto summary=ThreeDm::InputTree::validate(wrapper->getTopoShapePtr()->getShape());
            if(summary.nodes>std::numeric_limits<std::size_t>::max()-nativeNodes||summary.bytes>std::numeric_limits<std::size_t>::max()-nativeBytes)
                throw std::runtime_error("InvalidBasis: Edit aggregate native budget overflow");
            nativeNodes+=summary.nodes;nativeBytes+=summary.bytes;
            if(om9_prepared_cad_budget(0,0,nativeNodes,nativeBytes))throw std::runtime_error("InvalidBasis: Edit aggregate native source/output budget");};
        // Count all borrowed trees before any new private output copy.
        for(const auto& input:inputs)account(input.shape->value);for(const auto& output:shapes)account(output->value);
        const unsigned flags=(kind<=3||remove?4U:0U)|(kind==3&&om9_edit_option(1)?1U:0U)|(kind==3&&om9_edit_option(2)?2U:0U);
        const auto evidence=captureEditEvidence(doc,inputs,kind,flags,mode,first,deleting,tolerance,toleranceContext,projection);
        std::size_t metadataBytes=QJsonDocument(evidence.context).toJson(QJsonDocument::Compact).size();
        if(metadataBytes>om9_prepared_result_limit()||om9_prepared_cad_budget(0,metadataBytes,nativeNodes,nativeBytes))throw std::runtime_error("InvalidBasis: Edit aggregate source/result budget");
        std::set<std::string> reserved;
        for(std::size_t index=0;index<shapes.size();++index){
            std::string outputName;
            for(std::size_t serial=0;;++serial){auto proposed=std::string(name(kind));
                if(serial){const auto number=std::to_string(serial);proposed+=std::string(number.size()<3?3-number.size():0,'0')+number;}
                outputName=doc.getUniqueObjectName(proposed);if(reserved.insert(outputName).second)break;}
            Base::Placement parent;
            for(const auto& group:outputGroups[index])if(PyObject_HasAttrString(group->value,"getGlobalPlacement")){
                Ref frame(PyObject_CallMethod(group->value,"getGlobalPlacement",nullptr)),matrix(PyObject_CallMethod(frame.value,"toMatrix",nullptr));Base::Matrix4D native;
                for(unsigned r=1;r<=4;++r)for(unsigned c=1;c<=4;++c){Ref value(PyObject_GetAttrString(matrix.value,("A"+std::to_string(r)+std::to_string(c)).c_str()));native[r-1][c-1]=PyFloat_AsDouble(value.value);}
                if(PyErr_Occurred())throw std::runtime_error("InvalidTransform: Explode parent frame read failed");parent=Base::Placement(native);break;}
            auto output=prepareEditOutput(doc,shapes[index]->value,evidence,outputName,parent,index,shapes.size(),kind==2?owners[index]:0);
            if(output.result.size()>om9_prepared_result_limit()-metadataBytes)throw std::runtime_error("InvalidBasis: Edit aggregate source/result budget");
            metadataBytes+=output.result.size();if(om9_prepared_cad_budget(0,metadataBytes,nativeNodes,nativeBytes))throw std::runtime_error("InvalidBasis: Edit aggregate native/result budget");
            prepared.push_back(std::move(output));
        }
    }
    verifyEditInputs(doc,inputs);EditPublicationGuard publication(doc,inputs,kind);bool opened=false;
    std::vector<std::string> publishedNames;
    auto verifyPublished=[&](bool parents){publication.check();
        if(cad)for(std::size_t index=0;index<publishedNames.size();++index){const auto& output=prepared[index];auto* native=dynamic_cast<Part::Feature*>(doc.getObject(output.name.c_str()));
            auto* result=native?native->getPropertyByName<App::PropertyString>("OM9GeometryResult"):nullptr;
            if(!native||!result)
                throw std::runtime_error("StaleGeometry: Edit output/result missing during publication: "+output.name);
            // FreeCAD reconstructs Location during recompute. Compare its
            // numerical frame, not the identity of OCCT's Location datums.
            // Invalid Placement can remain in the property while native Shape
            // falls back to a finite transform; inspect both representations.
            const auto placement=native->Placement.getValue().toMatrix();
            for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)if(!std::isfinite(placement[r][c]))
                throw std::runtime_error("StaleGeometry: Edit output Placement frame is not finite during publication: "+output.name);
            const auto root=native->Shape.getShape().getTransform();
            for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)if(!std::isfinite(root[r][c])||!std::isfinite(output.rootFrame[r][c])||std::abs(root[r][c]-output.rootFrame[r][c])>32*std::numeric_limits<double>::epsilon()*std::max(1.,std::abs(output.rootFrame[r][c])))
                throw std::runtime_error("StaleGeometry: Edit output frame changed during publication: "+output.name);
            const bool metadata=output.result==result->getValue(),partner=native->Shape.getValue().IsPartner(output.shape.getShape()),
                orientation=native->Shape.getValue().Orientation()==output.shape.getShape().Orientation(),
                master=metadata&&partner&&orientation&&preparedCadNativeMasterDigest(native->Shape.getValue())==output.masterDigest;
            if(!metadata||!partner||!orientation||!master)
                throw std::runtime_error("StaleGeometry: Edit output/result changed during publication: "+output.name
                    +" (metadata="+std::to_string(metadata)+", partner="+std::to_string(partner)+", orientation="+std::to_string(orientation)+", master="+std::to_string(master)+")");
            const auto actual=(App::GeoFeature::getGlobalPlacement(native)*native->Placement.getValue().inverse()).toMatrix();const auto expected=output.parent.toMatrix();
            if(parents)for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)if(!std::isfinite(actual[r][c])||!std::isfinite(expected[r][c])||std::abs(actual[r][c]-expected[r][c])>32*std::numeric_limits<double>::epsilon()*std::max(1.,std::abs(expected[r][c])))
                throw std::runtime_error("StaleGeometry: Edit output parent changed during publication: "+output.name);
        }
    };
    try{if(!doc.openTransactionPreservingRedo(name(kind)))throw std::runtime_error("StaleGeometry: Edit transaction could not be opened");opened=true;publication.check();Ref pyDoc(doc.getPyObject()),id(PyUnicode_FromString(feature(kind))),command(PyUnicode_FromString(name(kind))),sources(PyList_New(0));
        for(const auto& i:inputs){Ref n(PyUnicode_FromString(i.name.c_str()));append(sources.value,n.value);}
        for(std::size_t index=0;index<shapes.size();++index){publication.check();auto shape=cad?owned(new Part::TopoShapePy(new Part::TopoShape(prepared[index].shape))):shapes[index];const bool mesh=isEditMesh(shape->value),textOutput=isExplodedText(shape->value);auto groups=outputGroups[index];Ref source(kind==2?doc.getObject(inputs[owners.at(index)].name.c_str())->getPyObject():Py_NewRef(Py_None));
            if(kind==2&&!cad){
                for(auto& group:groups)if(PyObject_HasAttrString(group->value,"getGlobalPlacement")){Ref frame(PyObject_CallMethod(group->value,"getGlobalPlacement",nullptr)),inverse(PyObject_CallMethod(frame.value,"inverse",nullptr)),matrix(PyObject_CallMethod(inverse.value,"toMatrix",nullptr));shape=owned(textOutput?PyDict_Copy(shape->value):PyObject_CallMethod(shape->value,"copy",nullptr));if(textOutput){transformExplodedText(shape->value,matrix.value);}else if(mesh){Ref old(PyObject_GetAttrString(shape->value,"Placement")),local(PyNumber_Multiply(inverse.value,old.value));set(shape->value,"Placement",local.value);}else{Ref transformed(PyObject_CallMethod(shape->value,"transformShape","OO",matrix.value,Py_False));}break;}
            }
            Ref o(textOutput?createExplodedText(pyDoc.value,shape->value):Ref(PyObject_CallMethod(pyDoc.value,"addObject","ss",mesh?"Mesh::Feature":"Part::Feature",cad?prepared[index].name.c_str():name(kind))));publication.check();if(!textOutput)set(o.value,mesh?"Mesh":"Shape",shape->value);property(o.value,"App::PropertyString","OM9FeatureId",id.value);property(o.value,"App::PropertyString","OM9Command",command.value);property(o.value,"App::PropertyStringList","SourceNames",sources.value);Ref m(PyLong_FromUnsignedLong(mode));property(o.value,"App::PropertyInteger","BooleanMode",m.value);
            if(cad){if(attr(o.value,"Name")!=prepared[index].name)throw std::runtime_error("StaleGeometry: Edit prepared output name changed");
                Ref result(PyUnicode_FromString(prepared[index].result.c_str()));property(o.value,"App::PropertyString","OM9GeometryResult",result.value);
                Ref readonly(PyObject_CallMethod(o.value,"setEditorMode","si","OM9GeometryResult",1));}
            if(kind==1){Ref t(PyFloat_FromDouble(tolerance));property(o.value,"App::PropertyFloat","JoinTolerance",t.value);
                if(toleranceContext){Ref report(PyUnicode_FromString(toleranceContext->report.c_str()));property(o.value,"App::PropertyString","OM9ToleranceReport",report.value);
                    auto* native=doc.getObject(attr(o.value,"Name").c_str());if(!native)throw std::runtime_error("Cannot resolve Join output for provenance");addUnitContextProvenance(*native,toleranceContext->context);}
            }
            if(kind==3){property(o.value,"App::PropertyBool","TrimExtendLines",om9_edit_option(1)?Py_True:Py_False);property(o.value,"App::PropertyBool","TrimApparentIntersections",om9_edit_option(2)?Py_True:Py_False);
                if(projection){Ref frame(PyList_New(0));for(const auto& row:{projection->eye,projection->right,projection->up,projection->forward})for(double value:row){Ref v(PyFloat_FromDouble(value));append(frame.value,v.value);}property(o.value,"App::PropertyFloatList","TrimProjectionFrame",frame.value);property(o.value,"App::PropertyBool","TrimPerspectiveProjection",projection->perspective?Py_True:Py_False);}}
            if(kind==2){publication.check();Ref sv(PyObject_GetAttrString(source.value,"ViewObject")),ov(PyObject_GetAttrString(o.value,"ViewObject"));for(const char* key:{"ShapeColor","LineColor","PointColor","Transparency","LineWidth","PointSize","DisplayMode","TextColor"})if(PyObject_HasAttrString(sv.value,key)&&PyObject_HasAttrString(ov.value,key)){Ref style(PyObject_GetAttrString(sv.value,key));if(std::string(key)=="DisplayMode"){Ref modes(PyObject_CallMethod(ov.value,"listDisplayModes",nullptr));if(PySequence_Contains(modes.value,style.value)!=1)continue;}set(ov.value,key,style.value);}applyExplodeMetadata(o.value,shapes[index]);}
            auto* native=doc.getObject(attr(o.value,"Name").c_str());if(!native)throw std::runtime_error("Edit output identity missing");
            publishedNames.push_back(native->getNameInDocument());
            publication.check();if(kind!=2&&layer.index)CoreLayers::applyCircle(doc,*native,layer);verifyPublished(false);
        }
        verifyPublished(false);breakHistoryForEdit(doc,deleting,[&](App::DocumentObject& o,bool begin){publication.detach(o,begin);});verifyPublished(false);
        if(kind==2)for(std::size_t index=0;index<shapes.size();++index){publication.check();auto* native=doc.getObject(publishedNames[index].c_str());if(!native)throw std::runtime_error("StaleGeometry: Explode output deleted during publication");Ref output(native->getPyObject());
            for(auto& group:outputGroups[index]){auto* parent=doc.getObject(attr(group->value,"Name").c_str());if(!parent)throw std::runtime_error("StaleGeometry: Explode parent missing");publication.addGroup(*parent,*native);Ref added(PyObject_CallMethod(group->value,"addObject","O",output.value));publication.endGroup();verifyPublished(false);}
            publication.check();auto* source=doc.getObject(inputs[owners.at(index)].name.c_str());if(!source)throw std::runtime_error("StaleGeometry: Explode layer source missing");CoreLayers::inheritLayer(*native,*source);verifyPublished(false);}
        for(const auto& n:deleting){verifyPublished(true);auto* source=doc.getObject(n.c_str());if(!source)throw std::runtime_error("StaleGeometry: Edit source missing before deletion: "+n);
            publication.remove(std::uint64_t(source->getID()));doc.removeObject(n.c_str());publication.remove(0);verifyPublished(true);}
        publication.recompute();verifyPublished(true);
        doc.commitTransaction();
    }catch(...){if(opened&&publication.live())doc.abortTransaction();throw;}
}
}
