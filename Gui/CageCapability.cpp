// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CageCapability.h"
#include "CoreGeometryContract.h"
#include "CoreDiscreteSnapshot.h"
#include "CurveGeometry.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyGeo.h>
#include <App/ComplexGeoData.h>
#include <Mod/Mesh/App/Mesh.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <Mod/Part/App/TopoShape.h>
#include <QApplication>
#include <QThread>
#include <stdexcept>
extern "C" unsigned om9_native_cage_capability(unsigned,OpenMatrix9Gui::NativeCageRequest);
extern "C" unsigned om9_cage_session_capability(const void*,unsigned,unsigned,unsigned);
namespace OpenMatrix9Gui {
namespace {
void decision(unsigned status,const std::string& operation,const std::string& identity){
    if(status==7)return;
    const char* code=status==101?"UnsupportedRepresentation":status==102?"InvalidBasis":status==103?"KernelFailure":"UnsupportedOperation";
    throw std::runtime_error(std::string(code)+": "+operation+" input "+identity+": shared Cage request rejected ("+std::to_string(status)+")");
}
}
void validateNativeCageCapability(unsigned representation,const NativeCageRequest& request,
    const std::string& operation,const std::string& identity){decision(om9_native_cage_capability(representation,request),operation,identity);}
void validateCageSessionCapability(const void* session,unsigned representation,unsigned stage,unsigned role,
    const std::string& operation,const std::string& identity){decision(om9_cage_session_capability(session,representation,stage,role),operation,identity);}
unsigned validateCageGeometryInput(App::DocumentObject& object,const std::string& operation,const std::function<void(unsigned)>& admission){
    const std::string context=operation+" input "+object.getNameInDocument()+": ";
    if(!qApp||QThread::currentThread()!=qApp->thread())throw std::runtime_error("KernelFailure: "+context+"native input requires the GUI thread");
    if(object.isRemoving()||!object.getDocument()||!object.getDocument()->containsObject(&object))throw std::runtime_error("StaleGeometry: "+context+"native owner is no longer live");
    if(object.isDerivedFrom(Base::Type::fromName("App::Link"))){admission(10);throw std::runtime_error("UnsupportedRepresentation: "+context+"linked geometry requires a separate adapter");}
    if(auto* property=object.getPropertyByName<App::PropertyComplexGeoData>("Mesh")){
        admission(8);
        try {
            auto type=Base::Type::fromName("Mesh::PropertyMeshKernel");
            if(type.isBad()||property->getTypeId()!=type)throw MeshInputFailure(1,"Unsupported native mesh property owner");
            auto* owner=property->getComplexData();auto nativeType=Base::Type::fromName("Mesh::MeshObject");
            if(!owner||nativeType.isBad()||owner->getTypeId()!=nativeType)throw MeshInputFailure(1,"Unsupported native mesh kernel owner");
            const auto& native=*static_cast<const Mesh::MeshObject*>(owner);
            meshInputBudget(native);meshSnapshot(native);return 8;
        }catch(const MeshInputFailure& error){
            const char* code=error.status==1?"UnsupportedRepresentation":error.status==3?"InvalidTopology":error.status==4?"NonFiniteGeometry":"InvalidBasis";
            throw std::runtime_error(std::string(code)+": "+context+error.what());
        }catch(const std::exception& error){throw std::runtime_error("InvalidBasis: "+context+error.what());}
    }
    CurvePyRef pyObject(object.getPyObject());
    if(!object.getPropertyByName("Shape")){admission(0);throw std::runtime_error("UnsupportedRepresentation: "+context+"no supported native geometry owner");}
    CurvePyRef shape(PyObject_GetAttrString(pyObject.value,"Shape"));
    if(!shape.value||!PyObject_TypeCheck(shape.value,&Part::TopoShapePy::Type))throw std::runtime_error("UnsupportedRepresentation: "+context+"no native Part shape owner");
    auto* wrapper=static_cast<Part::TopoShapePy*>(shape.value);
    if(!wrapper->Base::PyObjectBase::isValid()||!wrapper->getTopoShapePtr())throw std::runtime_error("StaleGeometry: "+context+"native shape is no longer live");
    const auto& native=wrapper->getTopoShapePtr()->getShape();
    if(native.IsNull())throw std::runtime_error("InvalidTopology: "+context+"Empty captive shape");
    unsigned representation=0;
    switch(native.ShapeType()){
        case TopAbs_VERTEX:representation=1;break;case TopAbs_EDGE:representation=2;break;case TopAbs_WIRE:representation=3;break;
        case TopAbs_FACE:representation=4;break;case TopAbs_SHELL:representation=5;break;case TopAbs_SOLID:representation=6;break;
        case TopAbs_COMPOUND:case TopAbs_COMPSOLID:representation=7;break;default:break;
    }
    admission(representation);validateNativePartShape(shape.value,operation,object.getNameInDocument());return representation;
}
}
