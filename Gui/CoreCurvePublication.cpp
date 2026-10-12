// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreCurvePublication.h"
#include "CurveGeometry.h"
#include "CorePreparedSnapshot.h"
#include "CoreUnits.h"
#include "RustBridge.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/GeoFeature.h>
#include <App/PropertyStandard.h>
#include <Base/Exception.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Mod/Part/App/PartFeature.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <QJsonDocument>
#include <QJsonArray>
#include <set>
#include <algorithm>
namespace OpenMatrix9Gui {
namespace {
std::array<double,16> matrix(const Base::Matrix4D& m){std::array<double,16> a;for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)a[r*4+c]=m[r][c];return a;}
void frame(const std::array<double,16>& expected,const std::array<double,16>& actual,const std::string& command,const std::string& name){
    // Reuse the existing stateless Rust native-publication frame validator.
    Om9SolidPublicationFrame input{};std::copy(expected.begin(),expected.end(),input.expected);std::copy(actual.begin(),actual.end(),input.actual);
    const auto status=om9_solid_publication_frame(input);
    if(status)throw std::runtime_error(std::string(status==1?"NonFiniteGeometry: ":status==2?"UnsupportedRepresentation: ":"StaleGeometry: ")+command+" output "+name+": native publication frame changed or invalid");
}
}
CurvePublication::CurvePublication(App::Document& doc,const std::string& command,const std::string& construction,const std::string& units,const CircleLayerSnapshot& layer)
 :identity(&doc),documentName(doc.getName()),documentUid(doc.Uid.getValueStr()),command(command),units(units),generation(om9_curve_generation()),revision(om9_curve_input_revision()),context(om9_circle_input_context_key()),layer(layer){
    check();
    const auto recipe=QJsonDocument::fromJson(QByteArray::fromStdString(construction)).object();
    if(recipe.value("generation").toString().toStdString()!=std::to_string(generation)||recipe.value("command").toString().toStdString()!=command)
        throw std::runtime_error("StaleGeometry: "+command+" input construction: session changed before publication");
    std::set<std::pair<std::string,std::string>> captured;
    for(const auto& value:recipe.value("source_refs").toArray()){
        const auto ref=value.toObject();const auto name=ref.value("object_name").toString().toStdString(),sub=ref.value("subelement").toString().toStdString();
        if(captured.emplace(name,sub).second)sources.push_back(captureCurveNativeSource(doc,name,sub,{},command));
    }
    check();
}
App::Document& CurvePublication::document()const{
    auto* doc=App::GetApplication().getDocument(documentName.c_str());
    if(!doc||doc!=identity||documentUid!=doc->Uid.getValueStr())throw std::runtime_error("StaleGeometry: "+command+" input construction: document closed or replaced");
    return *doc;
}
void CurvePublication::check()const{
    auto& doc=document();
    if(opened&&(doc.getBookedTransactionID()!=transactionId||!transactionLocked||!doc.isTransactionLocked()))
        throw std::runtime_error("StaleGeometry: "+command+" input construction: native publication booking changed or unlocked");
    auto* gui=Gui::Application::Instance->getDocument(&doc);
    if(generation!=om9_curve_generation()||revision!=om9_curve_input_revision()||context!=om9_circle_input_context_key()
        ||App::GetApplication().getActiveDocument()!=&doc||!gui||gui->getInEdit()||!Gui::Control().isAllowedAlterDocument(&doc)||Gui::Application::Instance->isClosing())
        throw std::runtime_error("StaleGeometry: "+command+" input construction: session/document/units changed during publication");
    CircleLayerSnapshot current;
    try {
        if(unitContextSnapshot(doc)!=units)
            throw std::runtime_error("document units changed during publication");
        current=CoreLayers::captureCircle(doc);
    }
    catch(const Base::Exception& error){throw std::runtime_error("StaleGeometry: "+command+" input construction: native units/layer context: "+error.what());}
    catch(const std::exception& error){throw std::runtime_error("StaleGeometry: "+command+" input construction: native units/layer context: "+error.what());}
    if(current.documentUid!=layer.documentUid||current.index!=layer.index||current.groupName!=layer.groupName||current.locked!=layer.locked||current.visible!=layer.visible||current.color!=layer.color)
        throw std::runtime_error("StaleGeometry: "+command+" input construction: output layer changed during publication");
    for(const auto& source:sources)verifyCurveNativeSource(doc,*source,command);
    for(const auto& output:outputs)if(output.id>=0)checkOutput(output);
}
void CurvePublication::open(){
    check();auto& doc=document();transactionId=doc.openTransactionPreservingRedo(command);
    if(!transactionId)throw std::runtime_error("StaleGeometry: "+command+" input construction: native publication transaction unavailable");
    opened=true;
    // The existing host permits materializing/writing this same booking while
    // locked, but refuses native commit/abort/open of a replacement booking.
    // No callback may retire the journal before the final validation barrier.
    doc.lockTransaction();transactionLocked=true;check();
}
void CurvePublication::prepare(PyObject* expectedShape,const std::string& name,const QJsonObject& metadata){
    check();if(opened)throw std::runtime_error("KernelFailure: "+command+" output "+name+": expected values must precede publication");
    const auto& shape=*static_cast<Part::TopoShapePy*>(expectedShape)->getTopoShapePtr();
    outputs.push_back({name,-1,preparedCadNativeMasterDigest(shape.getShape()),int(shape.getShape().Orientation()),matrix(shape.getTransform()),metadata});
}
void CurvePublication::watch(PyObject* object,const std::string& name){
    check();auto& doc=document();auto* native=dynamic_cast<Part::Feature*>(doc.getObject(name.c_str()));
    if(!native)throw std::runtime_error("StaleGeometry: "+command+" output "+name+": native identity missing");
    CurvePyRef wrapper(native->getPyObject());
    if(wrapper.value!=object)throw std::runtime_error("StaleGeometry: "+command+" output "+name+": native identity replaced");
    auto output=std::find_if(outputs.begin(),outputs.end(),[&](const auto& o){return o.name==name&&o.id<0;});
    if(output==outputs.end())throw std::runtime_error("KernelFailure: "+command+" output "+name+": expected output identity missing");
    output->id=native->getID();check();
}
void CurvePublication::checkOutput(const Output& output)const{
    auto& doc=document();auto* native=dynamic_cast<Part::Feature*>(doc.getObject(output.name.c_str()));
    const auto stale="StaleGeometry: "+command+" output "+output.name+": ";
    if(!native||native->getID()!=output.id||CoreLayers::objectLocked(native))throw std::runtime_error(stale+"identity or lock permission changed");
    if(native->getInList().size()!=(layer.index?1u:0u))
        throw std::runtime_error(stale+"required output layer ownership missing or changed");
    for(auto* owner:native->getInList())if(!CoreLayers::isLayerOwner(owner,*native)||!layer.index||layer.groupName!=owner->getNameInDocument())
        throw std::runtime_error(stale+"output ownership changed during publication");
    const auto* assigned=dynamic_cast<App::PropertyInteger*>(native->getPropertyByName("OM9OutputLayerIndex"));
    if(layer.index?(!assigned||assigned->getValue()!=layer.index):(assigned&&assigned->getValue()!=0))
        throw std::runtime_error(stale+"output layer index missing or changed");
    if(layer.index)for(const auto* key:{"OM9LayerExplicitVisible","OM9LayerExplicitSelectable"}){
        const auto* property=dynamic_cast<App::PropertyBool*>(native->getPropertyByName(key));
        if(!property||!property->getValue())throw std::runtime_error(stale+"explicit output permission changed: "+key);
    }
    auto* gui=Gui::Application::Instance->getDocument(&doc);
    auto* provider=gui?dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(native)):nullptr;
    const auto* selectable=provider?dynamic_cast<App::PropertyBool*>(provider->getPropertyByName("Selectable")):nullptr;
    const auto* color=provider?dynamic_cast<App::PropertyColor*>(provider->getPropertyByName("LineColor")):nullptr;
    // These values are assigned by the existing native output bridge. Visibility
    // masks display; the layer contract keeps Selectable true on an off layer.
    const auto expectedColor=(layer.index||command=="Circle"||command=="Ellipse")?layer.color:std::array<double,3>{0.,130./255.,85./255.};
    if(!provider||provider->Visibility.getValue()!=layer.visible||!selectable||!selectable->getValue()
        ||!color||color->getValue().r!=float(expectedColor[0])||color->getValue().g!=float(expectedColor[1])||color->getValue().b!=float(expectedColor[2]))
        throw std::runtime_error(stale+"output visibility/color/Selectable changed during publication");
    frame(output.frame,matrix(native->Placement.getValue().toMatrix()),command,output.name+" local");
    frame(output.frame,matrix(App::GeoFeature::getGlobalPlacement(native).toMatrix()),command,output.name+" world");
    frame(output.frame,matrix(native->Shape.getShape().getTransform()),command,output.name+" root");
    const auto& shape=native->Shape.getValue();
    if(int(shape.Orientation())!=output.orientation||preparedCadNativeMasterDigest(shape)!=output.digest)
        throw std::runtime_error(stale+"geometry/orientation changed during publication");
    for(auto i=output.metadata.begin();i!=output.metadata.end();++i){
        const auto key=i.key().toStdString();auto* property=native->getPropertyByName(key.c_str());bool same=false;
        if(i.value().isString()){auto* p=dynamic_cast<App::PropertyString*>(property);same=p&&i.value().toString().toStdString()==p->getValue();}
        else if(i.value().isBool()){auto* p=dynamic_cast<App::PropertyBool*>(property);same=p&&i.value().toBool()==p->getValue();}
        else if(i.value().isDouble()){auto* p=dynamic_cast<App::PropertyFloat*>(property);same=p&&i.value().toDouble()==p->getValue();}
        if(!same)throw std::runtime_error(stale+"metadata missing or changed: "+key);
    }
}
void CurvePublication::commit(){
    check();if(std::any_of(outputs.begin(),outputs.end(),[](const auto& o){return o.id<0;}))throw std::runtime_error("StaleGeometry: "+command+" input construction: prepared output missing before commit");
    try{
        auto& doc=document();doc.unlockTransaction();transactionLocked=false;
        if(!App::GetApplication().commitTransaction(transactionId))
            throw std::runtime_error("native publication transaction commit rejected");
        opened=false;transactionId=0;
    }
    catch(const Base::Exception& e){throw std::runtime_error("KernelFailure: "+command+" input construction: "+e.what());}
    catch(const std::exception& e){throw std::runtime_error("KernelFailure: "+command+" input construction: "+e.what());}
}
void CurvePublication::abort()noexcept{
    if(!opened)return;
    try{
        auto& doc=document();
        if(transactionLocked){doc.unlockTransaction();transactionLocked=false;}
        // Never abort a replacement booking, even after an unexpected native
        // close. The ID passed to the host is always the original owned ID.
        if(doc.getBookedTransactionID()==transactionId)
            App::GetApplication().abortTransaction(transactionId);
    }catch(...){}
    opened=false;transactionId=0;transactionLocked=false;
}
}
