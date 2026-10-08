// SPDX-License-Identifier: LGPL-2.1-or-later
#include "EditGeometry.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <algorithm>
#include <cmath>
#include <sstream>
#include <limits>
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
std::string signature(PyObject* p){Ref shape(PyObject_GetAttrString(p,"Shape")),brep(PyObject_CallMethod(shape.value,"exportBrepToString",nullptr));std::string result=text(brep.value);
    if(PyObject_HasAttrString(p,"getGlobalPlacement")){Ref placement(PyObject_CallMethod(p,"getGlobalPlacement",nullptr)),matrix(PyObject_CallMethod(placement.value,"toMatrix",nullptr));std::ostringstream values;values<<std::hexfloat;
        for(unsigned row=1;row<=4;++row)for(unsigned col=1;col<=4;++col){const auto key="A"+std::to_string(row)+std::to_string(col);Ref value(PyObject_GetAttrString(matrix.value,key.c_str()));const auto n=PyFloat_AsDouble(value.value);if(PyErr_Occurred()||!std::isfinite(n))throw std::runtime_error("Invalid global placement");values<<';'<<n;}result+=values.str();}return result;}
EditShapes elements(PyObject* p,const char* key){Ref list(PyObject_GetAttrString(p,key));EditShapes result;for(Py_ssize_t i=0;i<PySequence_Size(list.value);++i)result.push_back(owned(PySequence_GetItem(list.value,i)));return result;}
void append(PyObject* list,PyObject* p){if(PyList_Append(list,p)<0)throw std::runtime_error("Cannot collect native geometry");}
EditShape combined(const std::vector<EditInput>& inputs,std::size_t begin,std::size_t end){if(begin>=end)throw std::runtime_error("Select both Boolean input sets");auto result=owned(PyObject_CallMethod(inputs[begin].shape->value,"copy",nullptr));for(auto i=begin+1;i<end;++i)result=owned(PyObject_CallMethod(result->value,"fuse","O",inputs[i].shape->value));return result;}
const char* name(unsigned kind){const char* names[]={"","Join","Explode","Trim","BooleanDifference","BooleanIntersection","BooleanUnion","Boolean2Objects"};return names[kind];}
const char* feature(unsigned kind){const char* ids[]={"","OM9-TOP11-008","OM9-TOP11-005","OM9-TOP11-010","OM9-SOLID-001","OM9-SOLID-002","OM9-SOLID-003","OM9-SOLID-004"};return ids[kind];}
void valid(PyObject* shape){if(flag(shape,"isNull")||!flag(shape,"isValid"))throw std::runtime_error("Kernel produced empty or invalid geometry; inputs are unchanged");}
bool solidOnly(PyObject* shape){const auto type=attr(shape,"ShapeType");if(type=="Solid")return flag(shape,"isClosed");if(type!="Compound"&&type!="CompSolid")return false;
    Ref children(PyObject_CallMethod(shape,"childShapes",nullptr));const auto n=PySequence_Size(children.value);if(n<=0)return false;
    for(Py_ssize_t i=0;i<n;++i){Ref child(PySequence_GetItem(children.value,i));if(!solidOnly(child.value))return false;}return true;}
}
namespace OpenMatrix9Gui {
EditInput editInput(App::Document& doc,const std::string& name,unsigned kind){
    auto* o=doc.getObject(name.c_str());if(!o)throw std::runtime_error("Select an object in the active document");
    Ref object(o->getPyObject());if(o->isDerivedFrom(Base::Type::fromName("App::Link"))||!PyObject_HasAttrString(object.value,"Shape"))throw std::runtime_error("Select native curve/surface/solid objects; blocks, links and groups are unsupported");
    Ref original(PyObject_GetAttrString(object.value,"Shape"));auto shape=owned(PyObject_CallMethod(original.value,"copy",nullptr));valid(shape->value);
    const auto type=attr(shape->value,"ShapeType");
    if(kind>=4){if(!solidOnly(shape->value))throw std::runtime_error("Boolean supports closed native solids and solid-only compounds");}
    else if(kind==3){if(type!="Edge"&&type!="Wire"&&type!="Face"&&type!="Shell")throw std::runtime_error("Trim supports native curves and open surfaces/polysurfaces");}
    else if(kind==1){if(type!="Edge"&&type!="Wire"&&type!="Face"&&type!="Shell")throw std::runtime_error("Join supports curves or open surfaces, not solids/compounds");}
    else if(type!="Wire"&&type!="Shell"&&type!="Solid")throw std::runtime_error("Explode supports polycurves and polysurfaces; blocks are preserved");
    if(PyObject_HasAttrString(object.value,"getGlobalPlacement")){
        Ref global(PyObject_CallMethod(object.value,"getGlobalPlacement",nullptr)),local(PyObject_GetAttrString(object.value,"Placement")),inverse(PyObject_CallMethod(local.value,"inverse",nullptr)),parent(PyNumber_Multiply(global.value,inverse.value)),matrix(PyObject_CallMethod(parent.value,"toMatrix",nullptr));
        Ref moved(PyObject_CallMethod(shape->value,"transformShape","OO",matrix.value,Py_False));
    }
    return {name,signature(object.value),shape};
}
void verifyEditInputs(App::Document& doc,const std::vector<EditInput>& inputs){for(const auto& i:inputs){auto* o=doc.getObject(i.name.c_str());if(!o)throw std::runtime_error("An edit input was deleted; cancel and reselect");Ref object(o->getPyObject());if(signature(object.value)!=i.signature)throw std::runtime_error("An edit input changed; cancel and reselect");}}
EditShapes buildEdit(const std::vector<EditInput>& inputs,unsigned kind,std::size_t first,unsigned mode){
    // Spec: OM9-TOP11-005/008; OM9-SOLID-001..004. No interpreted command text.
    if(inputs.empty()||kind<1||kind>7)throw std::runtime_error("Select input objects");
    Ref part(PyImport_ImportModule("Part"));EditShapes output;
    if(kind==1){
        bool curves=count(inputs[0].shape->value,"Faces")==0;Ref list(PyList_New(0));
        for(const auto& i:inputs){if((count(i.shape->value,"Faces")==0)!=curves)throw std::runtime_error("Join curves and surfaces separately");for(auto& e:elements(i.shape->value,curves?"Edges":"Faces"))append(list.value,e->value);}
        if(curves){Ref sorted(PyObject_CallMethod(part.value,"sortEdges","Od",list.value,1e-7));if(PySequence_Size(sorted.value)!=1)throw std::runtime_error("Curves must meet at endpoints in one connected chain");Ref chain(PySequence_GetItem(sorted.value,0));output.push_back(owned(PyObject_CallMethod(part.value,"Wire","O",chain.value)));}
        else{auto shell=owned(PyObject_CallMethod(part.value,"makeShell","O",list.value));Ref sew(PyObject_CallMethod(shell->value,"sewShape",nullptr));if(attr(shell->value,"ShapeType")!="Shell"||count(shell->value,"Shells")!=1||count(shell->value,"Faces")!=PySequence_Size(list.value))throw std::runtime_error("Surfaces must share compatible edges to form one shell");output.push_back(shell);}
    }else if(kind==2){for(const auto& i:inputs){auto pieces=elements(i.shape->value,count(i.shape->value,"Faces")?"Faces":"Edges");if(pieces.size()<2)throw std::runtime_error("Each Explode input needs at least two components");output.insert(output.end(),pieces.begin(),pieces.end());}}
    else if(kind>=4){EditShape shape;
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
        valid(shape->value);output=elements(shape->value,"Solids");if(output.empty())throw std::runtime_error("Boolean has no solid result; inputs are unchanged");
    }else throw std::runtime_error("Trim requires a picked segment");
    for(const auto& shape:output)valid(shape->value);return output;
}
std::vector<EditShapes> splitEditCurves(const std::vector<EditInput>& inputs){
    if(inputs.size()<2)throw std::runtime_error("Trim needs at least two intersecting objects");
    Ref list(PyList_New(0));for(std::size_t i=1;i<inputs.size();++i)append(list.value,inputs[i].shape->value);
    Ref fuse(PyObject_CallMethod(inputs[0].shape->value,"generalFuse","Od",list.value,0.0)),mapping(PySequence_GetItem(fuse.value,1));
    std::vector<EditShapes> split;bool changed=false;
    for(std::size_t i=0;i<inputs.size();++i){const char* key=count(inputs[i].shape->value,"Faces")?"Faces":"Edges";Ref parts(PySequence_GetItem(mapping.value,i));EditShapes edges;for(Py_ssize_t j=0;j<PySequence_Size(parts.value);++j){Ref piece(PySequence_GetItem(parts.value,j));for(auto& edge:elements(piece.value,key)){bool duplicate=false;for(auto& existing:edges){Ref same(PyObject_CallMethod(edge->value,"isSame","O",existing->value));if(PyObject_IsTrue(same.value)==1){duplicate=true;break;}}if(!duplicate)edges.push_back(edge);}}
        changed|=edges.size()>std::size_t(count(inputs[i].shape->value,key));split.push_back(std::move(edges));}
    if(!changed)throw std::runtime_error("No true 3D intersections to trim; apparent intersections and ExtendLines are unsupported");return split;
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
        for(std::size_t i=0;i<shapes.size();++i)if(i!=picked&&!removed.contains(i))for(const auto& other:elements(shapes[i]->value,key)){Ref same(PyObject_CallMethod(boundary->value,"isSame","O",other->value));if(PyObject_IsTrue(same.value)==1)throw std::runtime_error("Pick inside a region, away from shared split boundaries");}
    }
}
void commitEdit(App::Document& doc,const std::vector<EditInput>& inputs,const EditShapes& shapes,unsigned kind,bool remove,unsigned mode,std::size_t first){
    verifyEditInputs(doc,inputs);for(const auto& s:shapes)valid(s->value);
    // Deleting dependent/parametric inputs would sever unrelated models. Retain them instead.
    std::vector<std::string> deleting;for(std::size_t index=0;index<inputs.size();++index)if(remove||(kind==4&&index<first))deleting.push_back(inputs[index].name);
    for(const auto& name:deleting){auto* o=doc.getObject(name.c_str());for(auto* dependent:o->getInList()){
        if(dependent->isDerivedFrom(Base::Type::fromName("App::Part"))||dependent->isDerivedFrom(Base::Type::fromName("App::DocumentObjectGroup")))continue;
        throw std::runtime_error("DeleteInput is unsafe for referenced objects; retain inputs or detach their dependents");
    }}
    doc.openTransaction(name(kind));
    try{Ref pyDoc(doc.getPyObject()),id(PyUnicode_FromString(feature(kind))),command(PyUnicode_FromString(name(kind))),sources(PyList_New(0));
        for(const auto& i:inputs){Ref n(PyUnicode_FromString(i.name.c_str()));append(sources.value,n.value);}
        for(const auto& shape:shapes){Ref o(PyObject_CallMethod(pyDoc.value,"addObject","ss","Part::Feature",name(kind)));set(o.value,"Shape",shape->value);property(o.value,"App::PropertyString","OM9FeatureId",id.value);property(o.value,"App::PropertyString","OM9Command",command.value);property(o.value,"App::PropertyStringList","SourceNames",sources.value);Ref m(PyLong_FromUnsignedLong(mode));property(o.value,"App::PropertyInteger","BooleanMode",m.value);}
        for(const auto& n:deleting)doc.removeObject(n.c_str());doc.recompute();doc.commitTransaction();
    }catch(...){doc.abortTransaction();throw;}
}
}
