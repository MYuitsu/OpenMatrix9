// SPDX-License-Identifier: LGPL-2.1-or-later
#include "Phase3Inputs.h"
#include "CurveGeometry.h"
#include "LayerDocumentAdapter.h"
#include <Mod/Part/App/TopoShape.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <Mod/Part/App/PartFeature.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/Property.h>
#include <Base/Interpreter.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <Gui/Selection/Selection.h>
#include <QCryptographicHash>
#include <cmath>
#include <sstream>
#include <stdexcept>
namespace OpenMatrix9Gui {
CurvePyRef publishedSurfaceSplineShape(){return CurvePyRef(new Part::TopoShapePy(new Part::TopoShape(publishedSplineShape())));}
std::vector<std::array<double,3>> sampleCurve(PyObject* shape,std::size_t count){
    if(!PyObject_TypeCheck(shape,&Part::TopoShapePy::Type))throw std::runtime_error("Select native CAD curves");
    return sampleCurve(static_cast<Part::TopoShapePy*>(shape)->getTopoShapePtr()->getShape(),count);
}
struct Phase3SnapshotState {
    std::string name,sub;std::uint64_t handle=0;
    ~Phase3SnapshotState(){om9_phase3_request_cancel(handle);}
};
namespace {
using Ref=CurvePyRef;
std::string text(PyObject* p){const char* s=PyUnicode_AsUTF8(p);if(!s){PyErr_Clear();throw std::runtime_error("Invalid native geometry data");}return s;}
bool flag(PyObject* p,const char* method){Ref result(PyObject_CallMethod(p,method,nullptr));int v=PyObject_IsTrue(result.value);if(v<0){PyErr_Clear();throw std::runtime_error("Cannot inspect native geometry");}return v==1;}
std::uint32_t count(PyObject* p,const char* attr){Ref result(PyObject_GetAttrString(p,attr));auto n=PySequence_Size(result.value);if(n<0||n>UINT32_MAX)throw std::runtime_error("Invalid native topology size");return std::uint32_t(n);}
bool solidsOnly(PyObject* p){Ref type(PyObject_GetAttrString(p,"ShapeType"));auto name=text(type.value);if(name=="Solid")return flag(p,"isClosed");if(name!="Compound"&&name!="CompSolid")return false;Ref children(PyObject_CallMethod(p,"childShapes",nullptr));auto n=PySequence_Size(children.value);if(n<1)return false;for(Py_ssize_t i=0;i<n;++i){Ref child(PySequence_GetItem(children.value,i));if(!solidsOnly(child.value))return false;}return true;}
struct Current {std::string document,identity,signature;Om9Phase3Snapshot view()const{return {reinterpret_cast<const std::uint8_t*>(identity.data()),identity.size(),reinterpret_cast<const std::uint8_t*>(signature.data()),signature.size()};}};
Current current(App::Document& doc,const std::string& name,const std::string& sub){
    if(om9ReadOnlyFile(doc))phase3Require(8);
    auto* object=doc.getObject(name.c_str());if(!object||object->getLinkedObject(true)!=object)throw std::runtime_error("Select an owning native CAD object");
    auto* property=object->getPropertyByName("Shape");
    if(object->getPropertyByName("OM9SourceUUID")||object->isReadOnly("Shape")||(property&&property->isReadOnly()))phase3Require(6);
    if(object->getPropertyByName("Mesh")||object->getPropertyByName("Points")||object->getPropertyByName("OM9CloudPoints")||!property)phase3Require(1);
    // Canonical eligibility is checked before BRep serialization or sampling;
    // native Selectable/Visibility projections cannot override Rust policy.
    const auto layerGeneration=layerMutationGeneration(doc,{name},1);
    Ref py(object->getPyObject());
    Ref shape(PyObject_GetAttrString(py.value,"Shape")),brep(PyObject_CallMethod(shape.value,"exportBrepToString",nullptr));
    std::string bytes=text(brep.value);if(PyObject_HasAttrString(py.value,"getGlobalPlacement")){
        Ref place(PyObject_CallMethod(py.value,"getGlobalPlacement",nullptr)),matrix(PyObject_CallMethod(place.value,"toMatrix",nullptr));std::ostringstream values;values<<std::hexfloat;
        for(unsigned row=1;row<=4;++row)for(unsigned col=1;col<=4;++col){auto key="A"+std::to_string(row)+std::to_string(col);Ref value(PyObject_GetAttrString(matrix.value,key.c_str()));double n=PyFloat_AsDouble(value.value);if(PyErr_Occurred()||!std::isfinite(n))throw std::runtime_error("Invalid world placement");values<<';'<<n;}bytes+=values.str();
    }
    bytes+=";layer-generation="+std::to_string(layerGeneration);
    return {doc.Uid.getValueStr(),name+"."+sub+":"+std::to_string(object->getID()),QCryptographicHash::hash(QByteArray::fromStdString(bytes),QCryptographicHash::Sha256).toHex().toStdString()};
}
}
void phase3Require(std::uint32_t reason){if(reason){std::uint8_t message[512]={};om9_phase3_message(reason,message,sizeof(message));throw std::runtime_error(reinterpret_cast<const char*>(message));}}
void phase3Validate(unsigned operation,const std::vector<Om9Phase3Facts>& facts,bool complete){phase3Require(om9_phase3_capability(operation,facts.data(),facts.size(),complete));}
Phase3Snapshot capturePhase3Object(App::Document& doc,const std::string& name,const std::string& sub){
    auto value=current(doc,name,sub);auto view=value.view();auto state=std::make_shared<Phase3SnapshotState>();state->name=name;state->sub=sub;
    state->handle=om9_phase3_request_begin(reinterpret_cast<const std::uint8_t*>(value.document.data()),value.document.size(),&view,1);if(!state->handle)phase3Require(10);return state;
}
void verifyPhase3Object(App::Document& doc,const Phase3Snapshot& state){
    if(!state)phase3Require(9);auto* gui=Gui::Application::Instance->activeDocument();
    bool available=&doc==App::GetApplication().getActiveDocument()&&!om9ReadOnlyFile(doc)&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&!doc.testStatus(App::Document::Restoring)&&om9AlterDocument(&doc);
    if(!available)phase3Require(8);auto value=current(doc,state->name,state->sub);auto view=value.view();phase3Require(om9_phase3_request_validate(state->handle,reinterpret_cast<const std::uint8_t*>(value.document.data()),value.document.size(),&view,1,available));
}
Om9Phase3Facts phase3ShapeFacts(PyObject* shape,bool protectedInput){
    Ref type(PyObject_GetAttrString(shape,"ShapeType"));auto name=text(type.value);std::uint32_t kind=0,components=1;
    if(name=="Edge"||name=="Wire"){kind=2;components=count(shape,"Edges");}
    else if(name=="Face")kind=3;else if(name=="Shell"){kind=4;components=count(shape,"Faces");}
    else if(name=="Solid"){kind=5;components=count(shape,"Faces");}
    else if(name=="Compound"||name=="CompSolid"){kind=6;Ref children(PyObject_CallMethod(shape,"childShapes",nullptr));components=std::uint32_t(PySequence_Size(children.value));}
    return {kind,std::uint8_t(!flag(shape,"isNull")&&flag(shape,"isValid")),std::uint8_t((kind==5||kind==6)?solidsOnly(shape):flag(shape,"isClosed")),std::uint8_t(protectedInput),0,components};
}
bool phase3SurfaceJoinSelection(){
    // Availability inspects only native metadata; never enumerates mesh/cloud points.
    Base::PyGILStateLocker lock;auto* doc=App::GetApplication().getActiveDocument();if(!doc)return false;
    for(const auto& sel:Gui::Selection().getSelection(doc->getName())){auto* object=doc->getObject(sel.FeatName);if(!object)continue;auto* part=dynamic_cast<Part::Feature*>(object);std::uint32_t kind=0;
        if(part&&!object->getPropertyByName("Mesh")&&!object->getPropertyByName("OM9CloudPoints")){const auto& shape=part->Shape.getValue();if(!shape.IsNull()){switch(shape.ShapeType()){case TopAbs_EDGE:case TopAbs_WIRE:kind=2;break;case TopAbs_FACE:kind=3;break;case TopAbs_SHELL:kind=4;break;case TopAbs_SOLID:kind=5;break;case TopAbs_COMPOUND:case TopAbs_COMPSOLID:kind=6;break;default:break;}}}
        if(om9_phase3_join_route(kind))return true;}
    return false;
}
}
