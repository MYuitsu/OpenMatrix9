// SPDX-License-Identifier: LGPL-2.1-or-later
// Native exception to Rust-first: FreeCAD document transactions, dynamic
// properties, membership and GUI view-provider reads/writes require host APIs.
// All independent identifiers/defaults/typed selection/output validation are Rust.
#include "CoreLayers.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/DocumentObjectGroup.h>
#include <App/PropertyStandard.h>
#include <Base/Color.h>
#include <Base/Console.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <algorithm>
#include <stdexcept>
#include <limits>
#include <utility>

// Kept local so this bridge remains independently integrable; declarations
// mirror Rust's scalar-only ABI with one bounded borrowed input slice.
extern "C" {
const char* om9_layer_name(int index);
double om9_layer_default_color(int index,std::size_t channel);
double om9_layer_root_color(std::size_t channel);
int om9_layer_output_status(int index,bool exists,bool locked,bool visible,double r,double g,double b);
int om9_layer_parse_selection(const unsigned char* text,std::size_t length);
}
namespace {
template<class T> T* property(App::DocumentObject* object,const char* name) {
    return object?dynamic_cast<T*>(object->getPropertyByName(name)):nullptr;
}
bool tag(App::DocumentObject* object,const char* key) {
    const auto* p=property<App::PropertyString>(object,key);
    return p&&std::string(p->getValue())=="1";
}
App::DocumentObject* settings(App::Document& document) {
    App::DocumentObject* result=nullptr;
    for(auto* object:document.getObjects())if(tag(object,"OM9LayerStateSchema")) {
        if(result)throw std::runtime_error("Duplicate OM9 layer settings");result=object;
    }
    return result;
}
App::DocumentObjectGroup* group(App::Document& document,int index) {
    App::DocumentObjectGroup* result=nullptr;
    for(auto* object:document.getObjects())if(tag(object,"OM9LayerSchema")) {
        const auto* p=property<App::PropertyInteger>(object,"OM9LayerIndex");
        if(!p||p->getValue()!=index)continue;
        auto* g=dynamic_cast<App::DocumentObjectGroup*>(object);
        if(!g||g->getTypeId()!=App::DocumentObjectGroup::getClassTypeId()||result)
            throw std::runtime_error("Invalid or duplicate OM9 layer group");
        // Layers stay at document root: no implicit App::Part placement/cycles.
        if(!g->getInList().empty())throw std::runtime_error("OM9 layer must be a root document group");
        result=g;
    }
    return result;
}
int activeIndex(App::Document& document) {
    auto* object=settings(document);if(!object)return 0;
    auto* p=property<App::PropertyInteger>(object,"OM9ActiveLayer");
    if(!p||p->getValue()<std::numeric_limits<int>::min()||p->getValue()>std::numeric_limits<int>::max())throw std::runtime_error("Invalid OM9 active layer setting");
    if(p->getValue()!=0&&!om9_layer_name(int(p->getValue())))throw std::runtime_error("Invalid OM9 active layer setting");
    return int(p->getValue());
}
Gui::ViewProviderDocumentObject* view(App::DocumentObject& object) {
    auto* gui=Gui::Application::Instance->getDocument(object.getDocument());
    return gui?dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(&object)):nullptr;
}
template<class T> T* add(App::DocumentObject& object,const char* type,const char* name) {
    auto* p=dynamic_cast<T*>(object.addDynamicProperty(type,name,"OpenMatrix9 Layers",nullptr,0,false,true));
    if(!p)throw std::runtime_error("Cannot create OM9 layer property");return p;
}
std::array<double,3> defaultColor(int index) {
    return {om9_layer_default_color(index,0),om9_layer_default_color(index,1),om9_layer_default_color(index,2)};
}
void requirePlan(int index,bool exists,bool locked,bool visible,const std::array<double,3>& color) {
    switch(om9_layer_output_status(index,exists,locked,visible,color[0],color[1],color[2])) {
    case 0:return;
    case 1:throw std::runtime_error("Invalid active layer index");
    case 2:throw std::runtime_error("Active layer is missing; select a layer again");
    case 3:throw std::runtime_error("Active layer is locked; unlock it or select another layer");
    default:throw std::runtime_error("Invalid active layer color");
    }
}
OpenMatrix9Gui::SidebarLayerState read(App::Document& document,int index) {
    const char* name=om9_layer_name(index);if(!name)throw std::runtime_error("Invalid layer slot");
    OpenMatrix9Gui::SidebarLayerState s;s.index=index;s.name=QString::fromUtf8(name);s.color=defaultColor(index);
    s.active=activeIndex(document)==index;
    if(auto* g=group(document,index)) {
        auto* lock=property<App::PropertyBool>(g,"OM9LayerLocked");
        auto* color=property<App::PropertyColor>(g,"OM9LayerColor");auto* provider=view(*g);
        if(!lock||!color||!provider)throw std::runtime_error("Incomplete native OM9 layer");
        const auto c=color->getValue();s.locked=lock->getValue();s.visible=provider->Visibility.getValue();s.color={c.r,c.g,c.b};
        // Lock is an allowed layer configuration; only output creation rejects it.
        requirePlan(index,true,false,s.visible,s.color);
    }
    return s;
}
App::DocumentObjectGroup* ensureGroup(App::Document& document,int index) {
    if(auto* existing=group(document,index))return existing;
    const char* name=om9_layer_name(index);if(!name)throw std::runtime_error("Invalid layer slot");
    const auto rgb=defaultColor(index);requirePlan(index,true,false,true,rgb);
    auto* g=dynamic_cast<App::DocumentObjectGroup*>(document.addObject("App::DocumentObjectGroup",("OM9Layer"+std::to_string(index)).c_str()));
    if(!g)throw std::runtime_error("Cannot create native OM9 layer");g->Label.setValue(name);
    add<App::PropertyString>(*g,"App::PropertyString","OM9LayerSchema")->setValue("1");
    add<App::PropertyInteger>(*g,"App::PropertyInteger","OM9LayerIndex")->setValue(index);
    add<App::PropertyBool>(*g,"App::PropertyBool","OM9LayerLocked")->setValue(false);
    add<App::PropertyColor>(*g,"App::PropertyColor","OM9LayerColor")->setValue(Base::Color(float(rgb[0]),float(rgb[1]),float(rgb[2])));
    if(auto* provider=view(*g))provider->Visibility.setValue(true);
    return g;
}
App::DocumentObject* ensureSettings(App::Document& document) {
    if(auto* existing=settings(document))return existing;
    auto* s=document.addObject("App::DocumentObject","OM9LayerSettings");if(!s)throw std::runtime_error("Cannot create layer settings");
    s->Label.setValue("OpenMatrix9 Layer Settings");
    add<App::PropertyString>(*s,"App::PropertyString","OM9LayerStateSchema")->setValue("1");
    add<App::PropertyInteger>(*s,"App::PropertyInteger","OM9ActiveLayer")->setValue(0);
    if(auto* provider=view(*s)){provider->ShowInTree.setValue(false);provider->Visibility.setValue(false);}
    return s;
}
bool mayEdit(App::Document* document) {return document&&Gui::Control().isAllowedAlterDocument(document);}
template<class F> bool transaction(const char* label,F&& change) {
    auto* doc=App::GetApplication().getActiveDocument();if(!mayEdit(doc))return false;
    bool opened=false;
    try {doc->openTransaction(std::string(label));opened=true;change(*doc);doc->recompute();doc->commitTransaction();return true;}
    catch(const std::exception& e){if(opened)doc->abortTransaction();Base::Console().error("OpenMatrix9 Layers: %s\n",e.what());return false;}
    catch(...){if(opened)doc->abortTransaction();Base::Console().error("OpenMatrix9 Layers: native operation failed\n");return false;}
}
void lineColor(App::DocumentObject& object,const std::array<double,3>& rgb,bool required) {
    auto* provider=view(object);
    auto* p=provider?dynamic_cast<App::PropertyColor*>(provider->getPropertyByName("LineColor")):nullptr;
    if(!p){if(required)throw std::runtime_error("Circle view provider has no line color");return;}
    const Base::Color color{float(rgb[0]),float(rgb[1]),float(rgb[2])};
    p->setValue(color);
    // Vertex outputs (including Ellipse foci) use the same layer swatch.
    if(auto* points=dynamic_cast<App::PropertyColor*>(provider->getPropertyByName("PointColor")))points->setValue(color);
}
}
namespace OpenMatrix9Gui {
bool CoreLayers::available() {return mayEdit(App::GetApplication().getActiveDocument());}
bool CoreLayers::select(int index) {
    if((index!=0&&!om9_layer_name(index))||!available())return false;
    try {auto& doc=*App::GetApplication().getActiveDocument();if(activeIndex(doc)==index&&(!index||group(doc,index))) {
        if(index)read(doc,index);return true;
    }}catch(const std::exception& e){Base::Console().error("OpenMatrix9 Layers: %s\n",e.what());return false;}
    return transaction("Select OM9 active layer",[index](App::Document& doc){
        if(index) {ensureGroup(doc,index);read(doc,index);}
        auto* s=ensureSettings(doc);auto* p=property<App::PropertyInteger>(s,"OM9ActiveLayer");
        if(!p)throw std::runtime_error("Invalid layer settings");p->setValue(index);
    });
}
bool CoreLayers::submit(const QString& text,QString& error) {
    error.clear();const auto bytes=text.toUtf8();const int index=om9_layer_parse_selection(reinterpret_cast<const unsigned char*>(bytes.constData()),std::size_t(bytes.size()));
    if(index==-2)return false;
    if(index<0)error="Use Layer=1..32, Layer=Gem 01, or Layer=None.";
    else if(!select(index))error="Layer selection failed; check document editability and layer metadata.";
    return true;
}
SidebarLayerState CoreLayers::state(int index) {
    auto* doc=App::GetApplication().getActiveDocument();if(doc)return read(*doc,index);
    SidebarLayerState s;s.index=index;s.name=QString::fromUtf8(om9_layer_name(index));s.color=defaultColor(index);return s;
}
bool CoreLayers::toggleLocked(int index) {
    if(!om9_layer_name(index))return false;
    return transaction("Toggle OM9 layer lock",[index](App::Document& doc){auto* g=ensureGroup(doc,index);read(doc,index);auto* p=property<App::PropertyBool>(g,"OM9LayerLocked");p->setValue(!p->getValue());});
}
bool CoreLayers::toggleVisible(int index) {
    if(!om9_layer_name(index))return false;
    return transaction("Toggle OM9 layer visibility",[index](App::Document& doc){
        auto* g=ensureGroup(doc,index);const bool visible=!read(doc,index).visible;view(*g)->Visibility.setValue(visible);
        // Explicit member writes cover empty/hidden group native behavior and
        // share the same Undo transaction as the layer view property.
        for(auto* member:g->getObjects())if(auto* provider=view(*member))provider->Visibility.setValue(visible);
    });
}
bool CoreLayers::setColor(int index,const std::array<double,3>& color) {
    if(!om9_layer_name(index)||om9_layer_output_status(index,true,false,true,color[0],color[1],color[2])!=0)return false;
    return transaction("Set OM9 layer color",[index,color](App::Document& doc){
        auto* g=ensureGroup(doc,index);read(doc,index);property<App::PropertyColor>(g,"OM9LayerColor")->setValue(Base::Color(float(color[0]),float(color[1]),float(color[2])));
        for(auto* member:g->getObjects())if(auto* p=property<App::PropertyInteger>(member,"OM9OutputLayerIndex");p&&p->getValue()==index)lineColor(*member,color,false);
    });
}
CircleLayerSnapshot CoreLayers::captureCircle(App::Document& doc) {
    CircleLayerSnapshot s;s.documentName=doc.getName();s.documentUid=doc.Uid.getValueStr();s.index=activeIndex(doc);
    s.color={om9_layer_root_color(0),om9_layer_root_color(1),om9_layer_root_color(2)};
    if(s.index){const auto state=read(doc,s.index);auto* g=group(doc,s.index);s.locked=state.locked;s.visible=state.visible;s.color=state.color;requirePlan(s.index,g!=nullptr,s.locked,s.visible,s.color);s.groupName=g->getNameInDocument();}
    return s;
}
void CoreLayers::applyCircle(App::Document& doc,App::DocumentObject& output,const CircleLayerSnapshot& captured) {
    if(captured.documentName!=doc.getName()||captured.documentUid!=doc.Uid.getValueStr()||output.getDocument()!=&doc)
        throw std::runtime_error("Circle layer document changed");
    const auto current=captureCircle(doc);
    if(current.index!=captured.index||current.groupName!=captured.groupName||current.visible!=captured.visible||current.color!=captured.color)
        throw std::runtime_error("Circle active layer changed before commit; retry");
    auto* provider=view(output);if(!provider)throw std::runtime_error("Circle view provider unavailable");
    if(current.index) {
        auto* g=group(doc,current.index);
        if(!g||&output==g||dynamic_cast<App::DocumentObjectGroup*>(&output)||App::GroupExtension::getGroupOfObject(&output))
            throw std::runtime_error("Circle output cannot be assigned to active layer");
        auto members=g->Group.getValues();members.push_back(&output);g->Group.setValues(std::move(members));
        add<App::PropertyInteger>(output,"App::PropertyInteger","OM9OutputLayerIndex")->setValue(current.index);
    }
    lineColor(output,current.color,true);provider->Visibility.setValue(current.visible);
}
}
