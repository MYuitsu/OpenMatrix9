#include "CoreSnapGeometry.h"
#include <App/DocumentObject.h>
#include <App/PropertyGeo.h>
#include <App/GeoFeature.h>
#include <Base/Interpreter.h>
#include <memory>
#include <cmath>
extern "C" unsigned om9_modeling_capabilities(unsigned kind);
namespace {
using PyRef=std::unique_ptr<PyObject,decltype(&Py_DecRef)>;
PyRef attr(PyObject* object,const char* name){return PyRef(PyObject_GetAttrString(object,name),&Py_DecRef);}
}
namespace OpenMatrix9Gui {
static std::vector<Base::Vector3d> candidates(const App::DocumentObject* object,unsigned int mode){
    const bool midpoint=mode==4;
    std::vector<Base::Vector3d> result;
    if(!object)return result;
    if(auto* kind=object->getPropertyByName<App::PropertyInteger>("OM9GeometryKind"))if(!(om9_modeling_capabilities(kind->getValue())&2u))return result;
    Base::PyGILStateLocker lock;
    PyObject* raw=nullptr;
    Base::Matrix4D transform;
    if(!object->getSubObject("",&raw,&transform,true)){Py_XDECREF(raw);return result;}
    PyRef shape(raw,&Py_DecRef);if(!shape)return result;
    // Resolve geometry through the native object API, including App::Link.
    // Standalone vertices belong to Point Snap, not End Snap.
    auto edges=attr(shape.get(),"Edges");
    if(!edges){PyErr_Clear();return result;}
    const auto edgeCount=PySequence_Size(edges.get());
    if(edgeCount<0||(mode==8?edgeCount!=0:edgeCount==0)){if(PyErr_Occurred())PyErr_Clear();return result;}
    // getSubObject already includes the object's local placement. Add only
    // the parent transform so nested geometry is not placed twice.
    Base::Placement parentTransform;
    if(auto* local=object->getPropertyByName<App::PropertyPlacement>("Placement"))
        parentTransform=App::GeoFeature::getGlobalPlacement(object)*local->getValue().inverse();
    auto vertices=attr(shape.get(),midpoint?"Edges":"Vertexes");
    if(!vertices){PyErr_Clear();return result;}
    PyRef sequence(PySequence_Fast(vertices.get(),"Expected native shape vertices"),&Py_DecRef);
    if(!sequence){PyErr_Clear();return result;}
    const auto count=PySequence_Fast_GET_SIZE(sequence.get());result.reserve(static_cast<std::size_t>(count));
    const char* axes[]={"x","y","z"};
    for(Py_ssize_t i=0;i<count;++i){
        auto* item=PySequence_Fast_GET_ITEM(sequence.get(),i);
        PyRef point(nullptr,&Py_DecRef);
        if(midpoint){
            auto length=attr(item,"Length");
            if(!length){PyErr_Clear();continue;}
            const auto distance=PyFloat_AsDouble(length.get())/2.;
            if(PyErr_Occurred()){PyErr_Clear();continue;}
            if(!std::isfinite(distance)||distance<=0)continue;
            PyRef parameter(PyObject_CallMethod(item,"getParameterByLength","d",distance),&Py_DecRef);
            if(!parameter){PyErr_Clear();continue;}
            point=PyRef(PyObject_CallMethod(item,"valueAt","O",parameter.get()),&Py_DecRef);
        }else point=attr(item,"Point");
        if(!point){PyErr_Clear();continue;}
        Base::Vector3d value;bool valid=true;
        for(int axis=0;axis<3;++axis){
            auto component=attr(point.get(),axes[axis]);if(!component){PyErr_Clear();valid=false;break;}
            value[axis]=PyFloat_AsDouble(component.get());
            if(PyErr_Occurred()){PyErr_Clear();valid=false;break;}
            if(!std::isfinite(value[axis])||std::abs(value[axis])>1e9){valid=false;break;}
        }
        if(valid){Base::Vector3d world;parentTransform.multVec(value,world);result.push_back(world);}
    }
    if(midpoint){
        auto faces=attr(shape.get(),"Faces");
        PyRef faceList(faces?PySequence_Fast(faces.get(),"Expected native faces"):nullptr,&Py_DecRef);
        if(!faceList){PyErr_Clear();return result;}
        for(Py_ssize_t i=0;i<PySequence_Fast_GET_SIZE(faceList.get());++i){
            auto* face=PySequence_Fast_GET_ITEM(faceList.get(),i);
            auto range=attr(face,"ParameterRange");
            double bounds[4];
            if(!range||!PyArg_ParseTuple(range.get(),"dddd",&bounds[0],&bounds[1],&bounds[2],&bounds[3])){PyErr_Clear();continue;}
            bool finite=true;for(double bound:bounds)finite=finite&&std::isfinite(bound);
            if(!finite)continue;
            PyRef point(PyObject_CallMethod(face,"valueAt","dd",bounds[0]/2.+bounds[1]/2.,bounds[2]/2.+bounds[3]/2.),&Py_DecRef);
            if(!point){PyErr_Clear();continue;}
            PyRef inside(PyObject_CallMethod(face,"isInside","OdO",point.get(),1e-7,Py_True),&Py_DecRef);
            if(!inside){PyErr_Clear();continue;}
            if(PyObject_IsTrue(inside.get())!=1)continue;
            Base::Vector3d value;bool valid=true;
            for(int axis=0;axis<3;++axis){
                auto component=attr(point.get(),axes[axis]);
                if(!component){PyErr_Clear();valid=false;break;}
                value[axis]=PyFloat_AsDouble(component.get());
                if(PyErr_Occurred()){PyErr_Clear();valid=false;break;}
                if(!std::isfinite(value[axis])||std::abs(value[axis])>1e9){valid=false;break;}
            }
            if(valid){Base::Vector3d world;parentTransform.multVec(value,world);result.push_back(world);}
        }
    }
    return result;
}
std::vector<Base::Vector3d> endCandidates(const App::DocumentObject* object){return candidates(object,2);}
std::vector<Base::Vector3d> midCandidates(const App::DocumentObject* object){return candidates(object,4);}
std::vector<Base::Vector3d> pointCandidates(const App::DocumentObject* object){return candidates(object,8);}
}
