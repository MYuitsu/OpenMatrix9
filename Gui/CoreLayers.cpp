// SPDX-License-Identifier: LGPL-2.1-or-later
// Native exception to Rust-first: FreeCAD document transactions, dynamic
// properties, membership and GUI view-provider reads/writes require host APIs.
// All independent identifiers/defaults/typed selection/output validation are Rust.
#include "CoreLayers.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/DocumentObjectGroup.h>
#include <App/GeoFeatureGroupExtension.h>
#include <App/PropertyStandard.h>
#include <Base/Color.h>
#include <Base/Console.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Gui/Selection/Selection.h>
#include <algorithm>
#include <stdexcept>
#include <limits>
#include <utility>
#include <set>

// Kept local so this bridge remains independently integrable; declarations
// mirror Rust's scalar-only ABI with one bounded borrowed input slice.
extern "C" {
const char* om9_layer_name(int index);
double om9_layer_default_color(int index,std::size_t channel);
double om9_layer_root_color(std::size_t channel);
int om9_layer_output_status(int index,bool exists,bool locked,bool visible,double r,double g,double b);
int om9_layer_parse_selection(const unsigned char* text,std::size_t length);
bool om9_layer_effective_visible(bool layer,bool explicitVisible,bool hidden);
bool om9_layer_effective_selectable(bool visible,bool explicitSelectable,bool hidden,bool locked);
bool om9_layer_selectable_permission(bool explicitSelectable,bool locked);
bool om9_layer_allows_action(int action,bool visible,bool locked,bool targetLocked);
bool om9_layer_valid_name(int index,const unsigned char* text,std::size_t length);
int om9_layer_group_target(unsigned int visible,unsigned int members);
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
        s.objectId=g->getID();
        s.groupName=g->getNameInDocument();
        auto* lock=property<App::PropertyBool>(g,"OM9LayerLocked");
        auto* color=property<App::PropertyColor>(g,"OM9LayerColor");auto* provider=view(*g);
        if(!lock||!color||!provider)throw std::runtime_error("Incomplete native OM9 layer");
        const auto c=color->getValue();s.locked=lock->getValue();s.visible=provider->Visibility.getValue();s.color={c.r,c.g,c.b};
        s.name=QString::fromUtf8(g->Label.getValue());
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
    const Base::Color color{float(rgb[0]),float(rgb[1]),float(rgb[2])};
    bool applied=false;if(provider)for(const char* key:{"LineColor","PointColor","ShapeColor"})
        if(auto* p=dynamic_cast<App::PropertyColor*>(provider->getPropertyByName(key))){p->setValue(color);applied=true;}
    if(required&&!applied)throw std::runtime_error("Output view provider has no layer color");
}
App::PropertyBool* boolean(App::DocumentObject& object,const char* name,bool initial) {
    if(auto* p=property<App::PropertyBool>(&object,name))return p;
    auto* p=add<App::PropertyBool>(object,"App::PropertyBool",name);p->setValue(initial);return p;
}
bool flag(App::DocumentObject& object,const char* name,bool fallback=false) {
    if(auto* p=property<App::PropertyBool>(&object,name))return p->getValue();return fallback;
}
int memberIndex(App::DocumentObject& object) {
    for(auto* parent:object.getInList())if(tag(parent,"OM9LayerSchema")) {
        auto* group=dynamic_cast<App::DocumentObjectGroup*>(parent);
        auto* index=property<App::PropertyInteger>(parent,"OM9LayerIndex");
        if(group&&index&&group->hasObject(&object))return int(index->getValue());
    }
    return 0;
}
bool drawable(App::DocumentObject& object) {
    return !dynamic_cast<App::DocumentObjectGroup*>(&object)&&view(object)
        &&(object.getPropertyByName("Shape")||object.getPropertyByName("Mesh")||object.getPropertyByName("Points"));
}
void rememberExplicit(App::DocumentObject& object,bool layerVisible,bool locked) {
    auto* provider=view(object);if(!provider)return;
    auto* visible=boolean(object,"OM9LayerExplicitVisible",provider->Visibility.getValue());
    auto* selectable=dynamic_cast<App::PropertyBool*>(provider->getPropertyByName("Selectable"));
    auto* stored=boolean(object,"OM9LayerExplicitSelectable",selectable?selectable->getValue():false);
    if(layerVisible&&!flag(object,"OM9ObjectHidden")) {
        visible->setValue(provider->Visibility.getValue());
        if(!locked&&selectable)stored->setValue(selectable->getValue());
    }
}
void syncMember(App::DocumentObject& object,bool layerVisible,bool locked) {
    auto* provider=view(object);if(!provider)return;
    const bool hidden=flag(object,"OM9ObjectHidden");
    const bool visible=om9_layer_effective_visible(layerVisible,flag(object,"OM9LayerExplicitVisible",provider->Visibility.getValue()),hidden);
    provider->Visibility.setValue(visible);
    if(auto* p=dynamic_cast<App::PropertyBool*>(provider->getPropertyByName("Selectable")))
        // Visibility already excludes native picks. Only lock masks the user's
        // Selectable preference, so normal tree Show can restore picking.
        p->setValue(om9_layer_selectable_permission(flag(object,"OM9LayerExplicitSelectable",p->getValue()),locked));
}
std::vector<std::string> selectedNames(App::Document& document) {
    std::set<std::string> names;
    for(const auto& sel:Gui::Selection().getCompleteSelection()) {
        if(sel.pDoc!=&document)throw std::runtime_error("Layer action requires objects from the active document only");
        if(sel.FeatName)names.insert(sel.FeatName);
    }
    return {names.begin(),names.end()};
}
bool sourceLocked(App::DocumentObject& object) {
    const int index=memberIndex(object);return index&&read(*object.getDocument(),index).locked;
}
void setVisible(App::Document& doc,int index,bool visible) {
    auto* g=ensureGroup(doc,index);const auto prior=read(doc,index);
    for(auto* member:g->getObjects())rememberExplicit(*member,prior.visible,prior.locked);
    view(*g)->Visibility.setValue(visible);
    for(auto* member:g->getObjects()){syncMember(*member,visible,prior.locked);if(!visible)Gui::Selection().rmvSelection(doc.getName(),member->getNameInDocument());}
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
    error.clear();const auto bytes=text.toUtf8();int index=om9_layer_parse_selection(reinterpret_cast<const unsigned char*>(bytes.constData()),std::size_t(bytes.size()));
    if(index==-2)return false;
    if(index<0&&bytes.size()<=256) {
        auto name=text.trimmed().mid(5).trimmed();if(name.startsWith('='))name=name.mid(1).trimmed();
        if(name.size()>=2&&name.startsWith('"')&&name.endsWith('"'))name=name.mid(1,name.size()-2);
        try {for(int slot=1;slot<=33;++slot)if(state(slot).name.compare(name,Qt::CaseInsensitive)==0){index=slot;break;}}
        catch(const std::exception& e){error=QString::fromUtf8(e.what());return true;}
    }
    if(index<0)error="Use Layer=1..33, a layer name, or Layer=None.";
    else if(!select(index))error="Layer selection failed; check document editability and layer metadata.";
    return true;
}
SidebarLayerState CoreLayers::state(int index) {
    auto* doc=App::GetApplication().getActiveDocument();if(doc)return read(*doc,index);
    SidebarLayerState s;s.index=index;s.name=QString::fromUtf8(om9_layer_name(index));s.color=defaultColor(index);return s;
}
bool CoreLayers::toggleLocked(int index) {
    if(!om9_layer_name(index))return false;
    return transaction("Toggle OM9 layer lock",[index](App::Document& doc){
        auto* g=ensureGroup(doc,index);const auto prior=read(doc,index);
        for(auto* member:g->getObjects())rememberExplicit(*member,prior.visible,prior.locked);
        auto* p=property<App::PropertyBool>(g,"OM9LayerLocked");p->setValue(!prior.locked);
        for(auto* member:g->getObjects()){
            syncMember(*member,prior.visible,!prior.locked);
            if(!prior.locked)Gui::Selection().rmvSelection(doc.getName(),member->getNameInDocument());
        }
    });
}
bool CoreLayers::toggleVisible(int index) {
    if(!om9_layer_name(index))return false;
    return transaction("Toggle OM9 layer visibility",[index](App::Document& doc){
        setVisible(doc,index,!read(doc,index).visible);
    });
}
bool CoreLayers::setName(int index,const QString& name) {
    const auto trimmed=name.trimmed();const auto bytes=trimmed.toUtf8();
    if(!om9_layer_valid_name(index,reinterpret_cast<const unsigned char*>(bytes.constData()),bytes.size()))return false;
    return transaction("Rename OM9 layer",[index,trimmed](App::Document& doc){
        for(int slot=1;slot<=33;++slot)if(slot!=index&&(read(doc,slot).name.compare(trimmed,Qt::CaseInsensitive)==0||QString::fromUtf8(om9_layer_name(slot)).compare(trimmed,Qt::CaseInsensitive)==0))
            throw std::runtime_error("Layer name is already in use or reserved");
        ensureGroup(doc,index)->Label.setValue(trimmed.toUtf8().constData());
    });
}
bool CoreLayers::assignSelected(int index) {
    if(!om9_layer_name(index)||!available())return false;
    auto* document=App::GetApplication().getActiveDocument();std::vector<std::string> names;
    try {names=selectedNames(*document);}catch(const std::exception& e){Base::Console().error("OpenMatrix9 Layers: %s\n",e.what());return false;}
    if(names.empty())return false;
    return transaction("Assign selection to OM9 layer",[index,names](App::Document& doc){
        const auto target=read(doc,index);
        // Validate the entire batch before creating a group or changing membership.
        for(const auto& name:names){auto* object=doc.getObject(name.c_str());
            if(!object||!drawable(*object)||!om9_layer_allows_action(1,view(*object)->Visibility.getValue(),sourceLocked(*object),target.locked))throw std::runtime_error("Invalid, missing or locked layer assignment");
            if(auto* owner=App::GroupExtension::getGroupOfObject(object);owner&&!tag(owner,"OM9LayerSchema"))throw std::runtime_error("Structural group members require an explicit placement-preserving move");
            if(App::GeoFeatureGroupExtension::getGroupOfObject(object))throw std::runtime_error("Placed group members require an explicit placement-preserving move");
        }
        auto* destination=ensureGroup(doc,index);
        for(const auto& name:names){auto* object=doc.getObject(name.c_str());const int previous=memberIndex(*object);const auto prior=previous?read(doc,previous):OpenMatrix9Gui::SidebarLayerState{};
            rememberExplicit(*object,previous?prior.visible:true,previous?prior.locked:false);
            if(auto* owner=dynamic_cast<App::DocumentObjectGroup*>(App::GroupExtension::getGroupOfObject(object));owner&&owner!=destination)owner->removeObject(object);
            destination->addObject(object);
            auto* p=property<App::PropertyInteger>(object,"OM9OutputLayerIndex");if(!p)p=add<App::PropertyInteger>(*object,"App::PropertyInteger","OM9OutputLayerIndex");p->setValue(index);
            lineColor(*object,target.color,false);syncMember(*object,target.visible,target.locked);
        }
    });
}
bool CoreLayers::objectEditable(App::DocumentObject* object) {
    if(!object||tag(object,"OM9LayerSchema")||tag(object,"OM9LayerStateSchema"))return false;
    try {
        auto* provider=view(*object);if(!provider)return false;
        const int index=memberIndex(*object);const auto s=index?read(*object->getDocument(),index):SidebarLayerState{};
        const bool visible=provider->Visibility.getValue()&&(!index||s.visible)&&!flag(*object,"OM9ObjectHidden");
        return om9_layer_effective_selectable(visible,provider->isSelectable(),false,index&&s.locked);
    }
    catch(...){return false;}
}
bool CoreLayers::objectLocked(App::DocumentObject* object) {
    if(!object)return true;try{return tag(object,"OM9LayerSchema")?flag(*object,"OM9LayerLocked"):sourceLocked(*object);}catch(...){return true;}
}
bool CoreLayers::isLayerOwner(App::DocumentObject* owner,App::DocumentObject& member) {
    if(!owner||owner->getDocument()!=member.getDocument()||!tag(owner,"OM9LayerSchema"))return false;
    const auto* index=property<App::PropertyInteger>(owner,"OM9LayerIndex");
    if(!index||index->getValue()<1||index->getValue()>33)return false;
    auto* g=group(*member.getDocument(),int(index->getValue()));
    return g==owner&&g->hasObject(&member);
}
bool CoreLayers::selectionEditable() {
    auto* doc=App::GetApplication().getActiveDocument();if(!doc)return false;
    for(const auto& sel:Gui::Selection().getCompleteSelection())if(sel.pDoc!=doc||!sel.FeatName||!objectEditable(doc->getObject(sel.FeatName)))return false;
    return true;
}
void CoreLayers::inheritLayer(App::DocumentObject& output,App::DocumentObject& source) {
    const int index=memberIndex(source);if(!index)return;
    if(source.getDocument()!=output.getDocument())throw std::runtime_error("Output layer document changed");
    auto& doc=*source.getDocument();const auto state=read(doc,index);
    requirePlan(index,true,state.locked,state.visible,state.color);
    auto* g=group(doc,index);if(!g||!g->hasObject(&output))throw std::runtime_error("Explode output layer ownership changed");
    boolean(output,"OM9LayerExplicitVisible",true)->setValue(true);boolean(output,"OM9LayerExplicitSelectable",true)->setValue(true);
    auto* p=property<App::PropertyInteger>(&output,"OM9OutputLayerIndex");if(!p)p=add<App::PropertyInteger>(output,"App::PropertyInteger","OM9OutputLayerIndex");p->setValue(index);
    lineColor(output,state.color,false);syncMember(output,state.visible,state.locked);
}
bool CoreLayers::selectMembers(int index) {
    auto* doc=App::GetApplication().getActiveDocument();if(!doc||!Gui::Control().isAllowedAlterSelection(doc)||!om9_layer_name(index))return false;
    try {std::vector<App::DocumentObject*> members;if(auto* g=group(*doc,index))for(auto* object:g->getObjects())if(objectEditable(object))members.push_back(object);
        Gui::Selection().clearSelection(doc->getName());Gui::Selection().setSelection(doc->getName(),members);return true;
    }catch(...){return false;}
}
bool CoreLayers::toggleGroupVisible(int first,int count) {
    if(first<1||count<1||count>4||first+count>33)return false;
    return transaction("Toggle OM9 layer group",[first,count](App::Document& doc){
        unsigned int visible=0,mask=0;for(int i=first;i<first+count;++i){mask|=1U<<(i-1);if(read(doc,i).visible)visible|=1U<<(i-1);}
        const bool target=om9_layer_group_target(visible,mask)==1;for(int i=first;i<first+count;++i)setVisible(doc,i,target);
    });
}
bool CoreLayers::hideSelected(bool reverse) {
    auto* document=App::GetApplication().getActiveDocument();if(!document||!available())return false;
    std::vector<std::string> selected;try{selected=selectedNames(*document);}catch(...){return false;}
    if(selected.empty())return false;
    return transaction(reverse?"Reverse Hide OM9 objects":"Hide OM9 objects",[selected,reverse](App::Document& doc){
        const std::set<std::string> names(selected.begin(),selected.end());
        for(auto* object:doc.getObjects())if(drawable(*object)&&view(*object)->Visibility.getValue()&&((names.count(object->getNameInDocument())!=0)!=reverse)){
            const int index=memberIndex(*object);const bool locked=index&&read(doc,index).locked;
            rememberExplicit(*object,true,locked);boolean(*object,"OM9ObjectHidden",false)->setValue(true);syncMember(*object,true,locked);
            Gui::Selection().rmvSelection(doc.getName(),object->getNameInDocument());
        }
    });
}
std::vector<std::pair<std::string,QString>> CoreLayers::hiddenObjects() {
    std::vector<std::pair<std::string,QString>> result;auto* doc=App::GetApplication().getActiveDocument();if(!doc)return result;
    for(auto* object:doc->getObjects())if(drawable(*object)&&flag(*object,"OM9ObjectHidden"))result.emplace_back(object->getNameInDocument(),QString::fromUtf8(object->Label.getValue()));
    return result;
}
bool CoreLayers::showHidden(const std::vector<std::string>& names,bool only) {
    if(only&&names.empty())return false;
    return transaction("Show OM9 hidden objects",[names,only](App::Document& doc){
        const std::set<std::string> wanted(names.begin(),names.end());
        if(only)for(const auto& name:names){auto* object=doc.getObject(name.c_str());if(!object||!drawable(*object)||!flag(*object,"OM9ObjectHidden"))throw std::runtime_error("Hidden object changed; reopen Show selection");}
        for(auto* object:doc.getObjects())if(drawable(*object)&&flag(*object,"OM9ObjectHidden")&&(!only||wanted.count(object->getNameInDocument()))){
            property<App::PropertyBool>(object,"OM9ObjectHidden")->setValue(false);const int index=memberIndex(*object);
            const auto state=index?read(doc,index):OpenMatrix9Gui::SidebarLayerState{};syncMember(*object,index?state.visible:true,index?state.locked:false);
        }
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
        boolean(output,"OM9LayerExplicitVisible",true)->setValue(true);
        boolean(output,"OM9LayerExplicitSelectable",true)->setValue(true);
        auto members=g->Group.getValues();members.push_back(&output);g->Group.setValues(std::move(members));
        add<App::PropertyInteger>(output,"App::PropertyInteger","OM9OutputLayerIndex")->setValue(current.index);
    }
    lineColor(output,current.color,true);provider->Visibility.setValue(current.visible);
    if(current.index)syncMember(output,current.visible,current.locked);
}
}
