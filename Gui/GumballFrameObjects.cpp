// SPDX-License-Identifier: LGPL-2.1-or-later
// Native document/viewport bridge. Frame admission and gestures remain Rust.
#include "GumballFrameObjects.h"
#include <Python.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/Expression.h>
#include <App/ObjectIdentifier.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/Selection/SoFCSelection.h>
#include <Inventor/nodes/SoCamera.h>
#include <Inventor/nodes/SoPerspectiveCamera.h>
#include <Inventor/nodes/SoOrthographicCamera.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoLineSet.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoDrawStyle.h>
#include <cmath>
#include <stdexcept>
#include <charconv>
#include <limits>
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::GumballCamera,App::GeoFeature)
PROPERTY_SOURCE(OpenMatrix9Gui::GumballLight,App::GeoFeature)
PROPERTY_SOURCE(OpenMatrix9Gui::ViewProviderGumballFrameObject,Gui::ViewProviderGeometryObject)
GumballCamera::GumballCamera(){ADD_PROPERTY(FocalDistance,(10.0));ADD_PROPERTY(HeightAngle,(45.0));ADD_PROPERTY(Height,(10.0));ADD_PROPERTY(Perspective,(true));ADD_PROPERTY(AspectRatio,(1.0));}
GumballLight::GumballLight(){ADD_PROPERTY(SizeU,(4.0));ADD_PROPERTY(SizeV,(4.0));ADD_PROPERTY(Reach,(5.0));ADD_PROPERTY(Power,(1.0));}
bool isGumballFrameObject(const App::DocumentObject& o){return dynamic_cast<const GumballCamera*>(&o)||dynamic_cast<const GumballLight*>(&o);}
namespace {
constexpr double pi=3.1415926535897932384626433832795;
std::vector<std::vector<Base::Vector3d>> localLines(const App::DocumentObject& object){
    double u=2,v=2,z=-5;
    if(auto* camera=dynamic_cast<const GumballCamera*>(&object)){
        z=-camera->FocalDistance.getValue();if(!std::isfinite(camera->HeightAngle.getValue())||camera->HeightAngle.getValue()<=0||camera->HeightAngle.getValue()>=179||!std::isfinite(camera->AspectRatio.getValue())||camera->AspectRatio.getValue()<=0)throw std::runtime_error("Camera angle/aspect ratio is invalid");v=camera->Perspective.getValue()?(-z)*std::tan(camera->HeightAngle.getValue()*pi/360):camera->Height.getValue()/2;u=v*camera->AspectRatio.getValue();
    }else if(auto* light=dynamic_cast<const GumballLight*>(&object)){if(!std::isfinite(light->Power.getValue())||light->Power.getValue()<0)throw std::runtime_error("Light power must be finite and nonnegative");u=light->SizeU.getValue()/2;v=light->SizeV.getValue()/2;z=-light->Reach.getValue();}
    else throw std::runtime_error("Unsupported native camera/light owner");
    if(!std::isfinite(u)||!std::isfinite(v)||!std::isfinite(z)||u<=0||v<=0||z>=0||u>1e9||v>1e9||-z>1e9)throw std::runtime_error("Camera/light display dimensions must be finite and positive");
    auto p=[](double x,double y,double z){return Base::Vector3d(x,y,z);};
    std::vector<std::vector<Base::Vector3d>> result={{p(-u,-v,z),p(u,-v,z),p(u,v,z),p(-u,v,z),p(-u,-v,z)},{p(0,0,0),p(0,0,z)}};
    for(const auto& point:std::vector<Base::Vector3d>{p(-u,-v,z),p(u,-v,z),p(u,v,z),p(-u,v,z)})result.push_back({p(0,0,0),point});
    result.push_back({p(-u,v,z),p(0,v+v/3,z),p(u,v,z)});return result;
}
PyObject* createAPI(PyObject*,PyObject* args){const char* kind=nullptr;if(!PyArg_ParseTuple(args,"s",&kind))return nullptr;try{return PyUnicode_FromString(createGumballFrameObject(kind).c_str());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}}
}
GumballFramePayload gumballFramePayload(const App::DocumentObject& object){GumballFramePayload data;const auto world=App::GeoFeature::getGlobalPlacement(&object).toMatrix();data.origin=world*Base::Vector3d();data.lines=localLines(object);for(auto& line:data.lines)for(auto& p:line)p=world*p;return data;}
std::optional<GumballParametricScale> planGumballFrameScale(App::DocumentObject& object,const double* affine,const double* world){
    if(!isGumballFrameObject(object))return {};
    if(!object.ExpressionEngine.getExpressions().empty())throw std::runtime_error("UnsupportedOwner: bound camera/light dimensions require explicit parameter editing");
    GumballParametricScale plan;const double previous[3]={1,1,1};if(om9_gumball_owner_scale(affine,world,previous,1,&plan.owner))throw std::runtime_error("UnsupportedOwner: camera/light Scale must preserve orthogonal positive axes");
    auto edit=[&](const char* key,double value){auto* property=object.getPropertyByName<App::PropertyFloat>(key);if(!property||!std::isfinite(value)||std::abs(value)>1e12)throw std::runtime_error("Invalid native frame dimension");if(property->getValue()==value)return;
        if(property->isReadOnly()||(property->getType()&(App::Prop_ReadOnly|App::Prop_Output))
            ||property->testStatus(App::Property::Immutable)||property->testStatus(App::Property::Output))
            throw std::runtime_error("UnsupportedOwner: camera/light dimension "+std::string(key)+" is read only");
        GumballParametricEdit change;change.key=key;change.before=property->getValue();change.value=value;plan.edits.push_back(std::move(change));};
    const auto* factors=plan.owner.factors;
    if(auto* light=dynamic_cast<GumballLight*>(&object)){edit("SizeU",light->SizeU.getValue()*factors[0]);edit("SizeV",light->SizeV.getValue()*factors[1]);edit("Reach",light->Reach.getValue()*factors[2]);}
    else if(auto* camera=dynamic_cast<GumballCamera*>(&object)){edit("FocalDistance",camera->FocalDistance.getValue()*factors[2]);edit("Height",camera->Height.getValue()*factors[1]);edit("AspectRatio",camera->AspectRatio.getValue()*factors[0]/factors[1]);edit("HeightAngle",std::atan(std::tan(camera->HeightAngle.getValue()*pi/360)*factors[1]/factors[2])*360/pi);}
    return plan;
}
void ViewProviderGumballFrameObject::attach(App::DocumentObject* object){
    Gui::ViewProviderGeometryObject::attach(object);symbol=new SoSeparator;
    auto* selection=new Gui::SoFCSelection;selection->documentName=object->getDocument()->getName();selection->objectName=object->getNameInDocument();selection->subElementName="";symbol->addChild(selection);
    addDisplayMaskMode(symbol,"Wireframe");updateData(&static_cast<App::GeoFeature*>(object)->Placement);
}
void ViewProviderGumballFrameObject::setDisplayMode(const char* mode){setDisplayMaskMode("Wireframe");Gui::ViewProviderGeometryObject::setDisplayMode(mode);}
void ViewProviderGumballFrameObject::updateData(const App::Property* property){
    Gui::ViewProviderGeometryObject::updateData(property);if(!symbol||!getObject())return;
    auto* selection=static_cast<Gui::SoFCSelection*>(symbol->getChild(0));selection->removeAllChildren();
    auto* color=new SoBaseColor;color->rgb.setValue(dynamic_cast<GumballCamera*>(getObject())?SbColor(0.2f,0.6f,0.9f):SbColor(1.0f,0.75f,0.15f));selection->addChild(color);
    auto* style=new SoDrawStyle;style->lineWidth=2;selection->addChild(style);
    auto* coordinates=new SoCoordinate3;auto* lines=new SoLineSet;selection->addChild(coordinates);selection->addChild(lines);
    try{int offset=0,index=0;for(const auto& line:localLines(*getObject())){lines->numVertices.set1Value(index++,int(line.size()));for(auto p:line)coordinates->point.set1Value(offset++,float(p.x),float(p.y),float(p.z));}}catch(...){coordinates->point.setNum(0);lines->numVertices.setNum(0);}
}
std::string createGumballFrameObject(const std::string& kind){
    auto* doc=App::GetApplication().getActiveDocument();if(!doc||!Gui::Application::Instance||!Gui::Control().isAllowedAlterDocument(doc))throw std::runtime_error("Open an editable document before creating a camera/light");
    if(kind!="Camera"&&kind!="Light")throw std::runtime_error("Frame object kind must be Camera or Light");
    doc->openTransaction(("Create gumball "+kind).c_str());try{
        auto* object=doc->addObject(("OpenMatrix9Gui::Gumball"+kind).c_str(),("Gumball"+kind).c_str());
        if(auto* camera=dynamic_cast<GumballCamera*>(object))if(auto* gui=Gui::Application::Instance->getDocument(doc))if(auto* view=dynamic_cast<Gui::View3DInventor*>(gui->getActiveView())){
            auto* node=view->getViewer()?view->getViewer()->getCamera():nullptr;if(!node)throw std::runtime_error("Active viewport has no native camera");auto p=node->position.getValue();auto q=node->orientation.getValue();const float* values=q.getValue();camera->Placement.setValue(Base::Placement(Base::Vector3d(p[0],p[1],p[2]),Base::Rotation(values[0],values[1],values[2],values[3])));camera->FocalDistance.setValue(node->focalDistance.getValue());
            if(auto* perspective=dynamic_cast<SoPerspectiveCamera*>(node))camera->HeightAngle.setValue(perspective->heightAngle.getValue()*180/pi);
            else if(auto* ortho=dynamic_cast<SoOrthographicCamera*>(node)){camera->Perspective.setValue(false);camera->Height.setValue(ortho->height.getValue());}
        }
        doc->recompute();std::string name=object->getNameInDocument();doc->commitTransaction();return name;
    }catch(...){doc->abortTransaction();throw;}
}
void initializeGumballFrameTypes(){GumballCamera::init();GumballLight::init();ViewProviderGumballFrameObject::init();}
void addGumballFrameObjectMethods(PyObject* module){static PyMethodDef methods[]={{"createGumballFrameObject",createAPI,METH_VARARGS,"Create a selectable native document Camera or Light from the viewport."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
}
