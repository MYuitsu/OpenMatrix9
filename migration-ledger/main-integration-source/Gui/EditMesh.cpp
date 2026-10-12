// SPDX-License-Identifier: LGPL-2.1-or-later
#include "EditGeometry.h"
#include <sstream>
#include <cmath>
#include <vector>
namespace {
using namespace OpenMatrix9Gui;
using Ref=CurvePyRef;
double number(PyObject* o,const char* key){Ref v(PyObject_GetAttrString(o,key));double n=PyFloat_AsDouble(v.value);if(PyErr_Occurred()||!std::isfinite(n))throw std::runtime_error("Invalid mesh coordinate or volume");return n;}
EditShape owned(PyObject* p){return std::make_shared<Ref>(p);}
bool flag(PyObject* o,const char* key){Ref b(PyObject_CallMethod(o,key,nullptr));return PyObject_IsTrue(b.value)==1;}
}
namespace OpenMatrix9Gui {
bool isEditMesh(PyObject* object){return PyObject_HasAttrString(object,"Topology");}
void validateEditMesh(PyObject* mesh,bool closed){
    if(number(mesh,"CountFacets")<1)throw std::runtime_error("Mesh has no facets");
    if(closed&&(!flag(mesh,"isSolid")||number(mesh,"Volume")<=0))throw std::runtime_error("Mesh Boolean requires a closed consistently oriented mesh with positive volume");
}
std::string editMeshSignature(PyObject* mesh){
    Ref topology(PyObject_GetAttrString(mesh,"Topology")),points(PySequence_GetItem(topology.value,0)),faces(PySequence_GetItem(topology.value,1));std::ostringstream out;out<<std::hexfloat;
    for(Py_ssize_t i=0;i<PySequence_Size(points.value);++i){Ref p(PySequence_GetItem(points.value,i));out<<';'<<number(p.value,"x")<<','<<number(p.value,"y")<<','<<number(p.value,"z");}
    for(Py_ssize_t i=0;i<PySequence_Size(faces.value);++i){Ref f(PySequence_GetItem(faces.value,i));out<<'|';for(Py_ssize_t j=0;j<PySequence_Size(f.value);++j){Ref index(PySequence_GetItem(f.value,j));out<<PyLong_AsLong(index.value)<<',';}}
    return out.str();
}
EditShape meshEditSolid(PyObject* mesh){
    // The native Mesh set-operation backend failed closedness/volume fixtures.
    // This separate polyhedral route retains each triangular input facet.
    validateEditMesh(mesh,true);Ref part(PyImport_ImportModule("Part")),topology(PyObject_GetAttrString(mesh,"Topology")),shape(PyObject_CallMethod(part.value,"Shape",nullptr));
    Ref made(PyObject_CallMethod(shape.value,"makeShapeFromMesh","Od",topology.value,1e-7)),shells(PyObject_GetAttrString(shape.value,"Shells")),solids(PyList_New(0));double volume=0;
    EditShapes boundaries;std::vector<double> volumes;
    for(Py_ssize_t i=0;i<PySequence_Size(shells.value);++i){Ref shell(PySequence_GetItem(shells.value,i));auto solid=owned(PyObject_CallMethod(part.value,"makeSolid","O",shell.value));if(!flag(solid->value,"isValid")||!flag(solid->value,"isClosed")||number(solid->value,"Volume")<=0)throw std::runtime_error("Mesh facets do not define valid positive closed polyhedra");volumes.push_back(number(solid->value,"Volume"));boundaries.push_back(solid);}
    // makeSolid orients every shell outward. Recover nesting before combining,
    // otherwise a cavity is filled when a Boolean mesh result is reused.
    std::vector<int> parent(boundaries.size(),-1);
    for(std::size_t i=0;i<boundaries.size();++i)for(std::size_t j=i+1;j<boundaries.size();++j){Ref common(PyObject_CallMethod(boundaries[i]->value,"common","O",boundaries[j]->value));const double overlap=number(common.value,"Volume"),epsilon=std::max(1e-6,std::min(volumes[i],volumes[j])*1e-7);if(overlap<=epsilon)continue;
        std::size_t inner=volumes[i]<volumes[j]?i:j,outer=inner==i?j:i;
        if(std::abs(overlap-volumes[inner])>epsilon||std::abs(volumes[inner]-volumes[outer])<=epsilon)throw std::runtime_error("Mesh shells overlap without unambiguous containment");
        if(parent[inner]<0||volumes[outer]<volumes[parent[inner]])parent[inner]=int(outer);
    }
    for(std::size_t i=0;i<boundaries.size();++i){unsigned depth=0;for(int p=parent[i];p>=0;p=parent[p])++depth;if(depth%2)continue;auto solid=boundaries[i];for(std::size_t j=0;j<boundaries.size();++j)if(parent[j]==int(i))solid=owned(PyObject_CallMethod(solid->value,"cut","O",boundaries[j]->value));
        if(!flag(solid->value,"isValid")||!flag(solid->value,"isClosed")||number(solid->value,"Volume")<=0)throw std::runtime_error("Mesh shell nesting does not define valid closed polyhedra");volume+=number(solid->value,"Volume");if(PyList_Append(solids.value,solid->value)<0)throw std::runtime_error("Cannot collect mesh polyhedra");}
    if(PyList_Size(solids.value)<1||std::abs(volume-number(mesh,"Volume"))>std::max(1e-3,std::abs(volume)*1e-5))throw std::runtime_error("Mesh/polyhedral volume mismatch; inputs unchanged");
    if(PyList_Size(solids.value)==1)return owned(PySequence_GetItem(solids.value,0));return owned(PyObject_CallMethod(part.value,"makeCompound","O",solids.value));
}
EditShape meshEditResult(PyObject* shape){
    Ref module(PyImport_ImportModule("MeshPart"));auto mesh=owned(PyObject_CallMethod(module.value,"meshFromShape","OddO",shape,1e-5,0.5,Py_False));validateEditMesh(mesh->value,true);
    double volume=number(shape,"Volume");if(std::abs(number(mesh->value,"Volume")-volume)>std::max(1e-3,std::abs(volume)*1e-5))throw std::runtime_error("Boolean mesh output does not preserve polyhedral volume");return mesh;
}
}
