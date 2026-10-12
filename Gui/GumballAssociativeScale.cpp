// SPDX-License-Identifier: LGPL-2.1-or-later
// FreeCAD object/property ownership bridge. Rust validates exact affine policy.
#include "GumballAssociativeScale.h"
#include <string_view>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/Expression.h>
#include <App/GeoFeatureGroupExtension.h>
#include <App/GroupExtension.h>
#include <App/Link.h>
#include <App/ObjectIdentifier.h>
#include <App/PropertyLinks.h>
#include <App/PropertyExpressionEngine.h>
#include <App/PropertyStandard.h>
#include <Mod/Part/App/FeatureScale.h>
#include <Mod/Part/App/FeatureCompound.h>
#include <Mod/Part/App/PartFeature.h>
#include <algorithm>
#include <cmath>
#include <map>
#include <set>
#include <stdexcept>
#include <utility>

extern "C" unsigned om9_gumball_associative_scale(const double*,const double*,double*);
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::GumballScaleCompound,Part::Compound)
namespace {
[[noreturn]] void unsupported(const std::string& reason){
    throw std::runtime_error("UnsupportedOwner: associative Gumball Scale "+reason);
}
bool writable(const App::Property& property){
    return !property.isReadOnly()&&!(property.getType()&App::Prop_ReadOnly)
        &&!property.testStatus(App::Property::Immutable)
        &&!property.testStatus(App::Property::Output)&&!(property.getType()&App::Prop_Output);
}
unsigned long linkProtection(const App::Property& property){
    constexpr unsigned long mask=(1UL<<App::Property::ReadOnly)
        |(1UL<<App::Property::Immutable)|(1UL<<App::Property::Output)
        |(1UL<<App::Property::PropReadOnly)|(1UL<<App::Property::PropOutput);
    return property.getStatus()&mask;
}
struct XLinkDescriptorAccess:App::PropertyXLink {
    static const std::string& documentName(const App::PropertyXLink& value){
        // Native Copy stores docName but deliberately clears _pcLink/docInfo.
        // Protected member access through a standard base member pointer keeps
        // this read-only adapter independent of object layout or downcasts.
        return value.*(&XLinkDescriptorAccess::docName);
    }
};
std::string xlinkDocument(const App::PropertyXLink& value,const App::Document* ownerDocument){
    if(const auto* object=value.getValue())return object->getDocument()->getName();
    const auto& name=XLinkDescriptorAccess::documentName(value);
    if(!name.empty())return name;
    // CopyOnLinkReplace for XLinkSubList can copy an already-detached entry a
    // second time, leaving a local object name instead of its document name.
    if(!*value.getFilePath()&&!*value.getDocumentPath()&&*value.getObjectName()&&ownerDocument)
        return ownerDocument->getName();
    return {};
}
bool sameExternalLink(const App::PropertyXLink& left,const App::PropertyXLink& right,
    const App::Document* ownerDocument){
    const auto a=xlinkDocument(left,ownerDocument),b=xlinkDocument(right,ownerDocument);
    return a==b&&std::string(left.getObjectName())==right.getObjectName()
        &&left.getSubValues()==right.getSubValues()
        &&left.testFlag(App::PropertyLinkBase::LinkAllowPartial)==right.testFlag(App::PropertyLinkBase::LinkAllowPartial)
        // Unresolved entries retain their persisted file/object identity;
        // loaded entries are represented by the exact native document name.
        &&(!a.empty()||(std::string(left.getFilePath())==right.getFilePath()
            &&std::string(left.getDocumentPath())==right.getDocumentPath()));
}
bool sameStoredLinks(const App::PropertyLinkBase& left,const App::PropertyLinkBase& right,
    const App::Document* ownerDocument){
    // Stored subnames preserve duplicates, order, null entries and unresolved
    // links. CopyOnLinkReplace may omit its shadow-name cache; reading the
    // native stored vectors also avoids asserting on that incomplete cache.
    if(const auto* value=dynamic_cast<const App::PropertyXLinkSubList*>(&left)){
        const auto* expected=dynamic_cast<const App::PropertyXLinkSubList*>(&right);
        if(!expected)return false;
        const auto& a=value->getSubListValues();const auto& b=expected->getSubListValues();
        if(a.size()!=b.size())return false;
        auto second=b.begin();
        for(const auto& first:a)if(!sameExternalLink(first,*second++,ownerDocument))return false;
        return true;
    }
    if(const auto* value=dynamic_cast<const App::PropertyXLink*>(&left)){
        const auto* expected=dynamic_cast<const App::PropertyXLink*>(&right);
        return expected&&sameExternalLink(*value,*expected,ownerDocument);
    }
    if(const auto* value=dynamic_cast<const App::PropertyLinkSubList*>(&left)){
        const auto* expected=dynamic_cast<const App::PropertyLinkSubList*>(&right);
        return expected&&value->getValues()==expected->getValues()
            &&value->getSubValues()==expected->getSubValues();
    }
    if(const auto* value=dynamic_cast<const App::PropertyLinkSub*>(&left)){
        const auto* expected=dynamic_cast<const App::PropertyLinkSub*>(&right);
        return expected&&value->getValue()==expected->getValue()
            &&value->getSubValues()==expected->getSubValues();
    }
    if(const auto* value=dynamic_cast<const App::PropertyLinkList*>(&left)){
        const auto* expected=dynamic_cast<const App::PropertyLinkList*>(&right);
        return expected&&value->getValues()==expected->getValues();
    }
    if(const auto* value=dynamic_cast<const App::PropertyLink*>(&left)){
        const auto* expected=dynamic_cast<const App::PropertyLink*>(&right);
        return expected&&value->getValue()==expected->getValue();
    }
    return false; // Unknown link families have no proven exact-value policy.
}
bool originalTargetsRemain(const App::PropertyLinkBase& links,const GumballScaleLinkEdit& edit){
    std::vector<App::DocumentObject*> current;
    links.getLinks(current,true);
    for(const auto& original:edit.originalTargets){
        bool referenced=false;
        for(const auto* object:current)
            if(object->getDocument()->getName()==original.documentName&&object->getNameInDocument()==original.name)
                referenced=true;
        const auto externalReference=[&](const App::PropertyXLink& value){
            return xlinkDocument(value,edit.owner->getDocument())==original.documentName
                &&value.getObjectName()==original.name;
        };
        if(const auto* external=dynamic_cast<const App::PropertyXLinkSubList*>(&links))
            for(const auto& entry:external->getSubListValues())referenced=referenced||externalReference(entry);
        else if(const auto* external=dynamic_cast<const App::PropertyXLink*>(&links))referenced=referenced||externalReference(*external);
        if(!referenced)continue;
        const auto* document=App::GetApplication().getDocument(original.documentName.c_str());
        const auto* object=document?document->getObject(original.name.c_str()):nullptr;
        if(document!=original.document||object!=original.object||!object||object->getID()!=original.id)return false;
    }
    return true;
}
void preserveScalarExpressions(App::DocumentObject& owner,App::DocumentObject& source,
    const App::PropertyExpressionEngine& engine){
    for(const auto& [path,expression]:engine.getExpressions()){
        if(!expression)continue;
        const auto dependencies=expression->getDeps(App::Expression::DepAll);
        const auto incoming=dependencies.find(&source);
        if(incoming==dependencies.end())continue;
        const auto scalar=[](const App::Property* property){
            return property&&(dynamic_cast<const App::PropertyFloat*>(property)
                ||dynamic_cast<const App::PropertyInteger*>(property))
                &&!property->testStatus(App::Property::Output)&&!(property->getType()&App::Prop_Output);
        };
        if(!scalar(path.getProperty()))
            unsupported("expression governed geometry "+std::string(owner.getNameInDocument())+"."+path.toString());
        for(const auto& [key,identifiers]:incoming->second){
            if(!scalar(source.getPropertyByName(key.c_str())))
                unsupported("expression governed geometry "+std::string(owner.getNameInDocument())+" reads "+source.getNameInDocument()+"."+key);
        }
        // Scalar dimensions continue reading the untouched native definition.
        // App::Link results intentionally do not impersonate native Length,
        // Radius or other parameter properties in downstream expressions.
    }
}
bool incomingWritable(const App::DocumentObject& owner,const std::string& key,
    const App::Property& property){
    if(writable(property))return true;
    // SubShapeBinder deliberately makes Support read-only in the property
    // editor. Its native setLinks command still writes Support.setValues.
    // Admit that exact built-in input without clearing any property flags;
    // arbitrary read-only, immutable, static read-only and output fields stay
    // protected. Legacy ShapeBinder.Support is normally writable already.
    return std::string_view(owner.getTypeId().getName())=="PartDesign::SubShapeBinder"&&key=="Support"
        &&property.getContainer()==&owner
        &&std::string_view(property.getTypeId().getName())=="App::PropertyXLinkSubList"
        &&property.getType()==App::Prop_None
        &&!property.testStatus(App::Property::Immutable)
        &&!property.testStatus(App::Property::Output);
}
bool legacyShapeSupport(const App::DocumentObject& owner,const std::string& key,
    const App::Property& property){
    return std::string_view(owner.getTypeId().getName())=="PartDesign::ShapeBinder"&&key=="Support"
        &&property.getContainer()==&owner
        &&std::string_view(property.getTypeId().getName())=="App::PropertyLinkSubListGlobal";
}
std::string placementExpression(App::DocumentObject& object){
    const std::string name=object.getNameInDocument();
    if(!dynamic_cast<Part::Feature*>(&object))
        unsupported("requires a native Part shape owner or a rigid occurrence of one");
    if(!object.getPropertyByName<App::PropertyPlacement>("Placement"))
        unsupported("source has no native Placement property");
    return name+".Placement";
}
}
bool sameGumballIncomingValue(const App::Property& property,const GumballScaleLinkEdit& edit){
    const auto* links=dynamic_cast<const App::PropertyLinkBase*>(&property);
    const auto* expected=dynamic_cast<const App::PropertyLinkBase*>(edit.before.get());
    return links&&expected&&edit.owner&&property.getContainer()==edit.owner
        &&edit.owner->getPropertyByName(edit.key.c_str())==&property
        &&property.getTypeId().getName()==edit.propertyType&&links->getScope()==edit.scope
        &&property.getType()==edit.propertyFlags&&linkProtection(property)==edit.protection
        &&sameStoredLinks(*links,*expected,edit.owner->getDocument())&&originalTargetsRemain(*links,edit);
}
GumballAssociativeScalePlan planGumballAssociativeScale(
    App::DocumentObject& source,const double* affine,const double* frame,bool copy,
    const std::set<std::string>& retainedDefinitions){
    GumballAssociativeScalePlan plan;
    plan.source=&source;plan.copy=copy;
    if(!affine||!frame||om9_gumball_associative_scale(affine,frame,plan.factors.data()))
        unsupported("affine is singular, nonfinite, or is not diagonal in the Gumball frame");
    plan.normalizeOccurrence=dynamic_cast<App::Link*>(&source)!=nullptr;
    if(!plan.normalizeOccurrence)plan.sourcePlacementExpression=placementExpression(source);
    Base::Matrix4D matrix;
    for(unsigned row=0;row<3;++row){
        matrix[row][3]=frame[row];
        for(unsigned column=0;column<3;++column)matrix[row][column]=frame[3+3*column+row];
    }
    Base::Placement parent;
    auto* geoParent=App::GeoFeatureGroupExtension::getGroupOfObject(&source);
    if(geoParent){
        // A PartDesign feature cannot be replaced by an App::Link inside Body.
        // Scaling the Body's final result requires an external result owner.
        if(geoParent->isDerivedFrom(Base::Type::fromName("PartDesign::Body")))
            unsupported("requires the Body result rather than an internal PartDesign feature");
        parent=geoParent->getExtension<App::GeoFeatureGroupExtension>()->globalGroupPlacement();
    }
    plan.resultFrame=parent.inverse()*Base::Placement(matrix);
    plan.inputFrame=plan.resultFrame.inverse();
    const auto local=plan.resultFrame.toMatrix();
    for(unsigned row=0;row<4;++row)for(unsigned column=0;column<4;++column)
        if(!std::isfinite(local[row][column])||std::abs(local[row][column])>1e12)
            unsupported("local frame exceeds the finite transform budget");
    auto* document=source.getDocument();
    if(!document||!source.getNameInDocument())unsupported("source is detached");
    for(auto* owner:document->getObjects()){
        if(owner==&source)continue;
        auto* group=owner->getExtension<App::GroupExtension>();
        if(group&&group->hasObject(&source)){
            if(!writable(group->Group))unsupported("group membership is read only");
            if(!group->allowObject(&source))unsupported("source group has restricted membership");
            plan.groups.push_back(owner);
        }
        if(copy||retainedDefinitions.count(owner->getNameInDocument()))continue;
        std::map<std::string,App::Property*> properties;
        owner->getPropertyMap(properties);
        for(const auto& [key,property]:properties){
            if(const auto* expressions=dynamic_cast<const App::PropertyExpressionEngine*>(property)){
                preserveScalarExpressions(*owner,source,*expressions);continue;
            }
            auto* links=dynamic_cast<App::PropertyLinkBase*>(property);
            if(!links||links->getScope()==App::LinkScope::Hidden||key=="OM9GumballScaleSource"
                ||(group&&property==&group->Group))continue;
            std::vector<App::DocumentObject*> targets;
            std::vector<std::string> subs;
            links->getLinks(targets,true,&subs);
            if(std::find(targets.begin(),targets.end(),&source)==targets.end())continue;
            if(!incomingWritable(*owner,key,*links))unsupported("incoming dependency "+std::string(owner->getNameInDocument())+"."+key+" is read only");
            if(links->getScope()==App::LinkScope::Child)
                unsupported("incoming child ownership "+std::string(owner->getNameInDocument())+"."+key+" cannot accept a Scale result");
            if(owner->getExpression(App::ObjectIdentifier(owner,key)).expression)
                unsupported("incoming dependency "+std::string(owner->getNameInDocument())+"."+key+" is expression governed");
            plan.requiresShapeOwner=plan.requiresShapeOwner||legacyShapeSupport(*owner,key,*property);
            plan.incoming.push_back({owner,key,std::shared_ptr<App::Property>(property->Copy()),subs,
                std::string(property->getTypeId().getName()),links->getScope(),property->getType(),linkProtection(*property)});
            std::set<App::DocumentObject*> identified;
            for(auto* target:targets)if(identified.insert(target).second)
                plan.incoming.back().originalTargets.push_back({target,target->getDocument(),target->getDocument()->getName(),target->getNameInDocument(),target->getID()});
            if(!sameGumballIncomingValue(*property,plan.incoming.back()))
                unsupported("incoming dependency "+std::string(owner->getNameInDocument())+"."+key+" has an unsupported native link value");
        }
    }
    return plan;
}
GumballAssociativeScaleResult createGumballAssociativeScale(
    App::Document& document,const GumballAssociativeScalePlan& plan,
    const std::function<void(App::DocumentObject*,const char*,bool)>& observer){
    if(!plan.source||plan.source->getDocument()!=&document)unsupported("source became unavailable before publication");
    for(const auto& edit:plan.incoming){
        if(!edit.owner||edit.owner->getDocument()!=&document)unsupported("incoming dependency owner became unavailable");
        auto* property=edit.owner->getPropertyByName(edit.key.c_str());
        if(!property||!sameGumballIncomingValue(*property,edit))
            unsupported("incoming dependency changed before publication");
        if(!incomingWritable(*edit.owner,edit.key,*property))
            unsupported("incoming dependency became protected before publication");
    }
    GumballAssociativeScaleResult result;
    result.document=&document;result.documentName=document.getName();
    result.source=plan.source;result.groups=plan.groups;
    const std::string stem=plan.source->getNameInDocument();
    // These internal native Compounds default to the stock PartExt renderer;
    // this survives transaction replay and does not need a provider override. The
    // specialized Compound view provider hides its Links synchronously;
    // source/output visibility belongs to the caller's publication guard.
    auto* sourceBridge=plan.normalizeOccurrence?dynamic_cast<App::Link*>(
        document.addObject("App::Link",(stem+"ScaleSourceBridge").c_str())):nullptr;
    auto* normalizer=plan.normalizeOccurrence?dynamic_cast<Part::Compound*>(
        document.addObject("OpenMatrix9Gui::GumballScaleCompound",(stem+"ScaleGeometry").c_str())):nullptr;
    auto* input=dynamic_cast<App::Link*>(document.addObject("App::Link",(stem+"ScaleInput").c_str()));
    auto* scale=dynamic_cast<Part::Scale*>(document.addObject("Part::Scale",(stem+"Scale").c_str()));
    auto* output=dynamic_cast<App::Link*>(document.addObject("App::Link",(stem+"Scaled").c_str()));
    auto* shapeOwner=plan.requiresShapeOwner?dynamic_cast<Part::Compound*>(
        document.addObject("OpenMatrix9Gui::GumballScaleCompound",(stem+"ScaledShape").c_str())):nullptr;
    if(!input||!scale||!output||(plan.normalizeOccurrence&&(!sourceBridge||!normalizer))||(plan.requiresShapeOwner&&!shapeOwner))
        unsupported("native Compound, Link or Part::Scale type is unavailable");
    result.sourceBridge=sourceBridge;result.normalizer=normalizer;result.input=input;result.scale=scale;result.output=output;result.shapeOwner=shapeOwner;
    result.created={input,scale,output};
    if(normalizer)result.created.insert(result.created.begin(),normalizer);
    if(sourceBridge)result.created.insert(result.created.begin(),sourceBridge);
    if(shapeOwner)result.created.push_back(shapeOwner);
    auto identify=[&](App::DocumentObject* object){result.identities.push_back(
        {object,object->getNameInDocument(),std::string(object->getTypeId().getName()),object->getID()});};
    identify(plan.source);for(auto* object:result.created)identify(object);
    auto write=[&](App::DocumentObject* object,const char* key,const auto& mutate){
        if(observer)observer(object,key,true);
        mutate();
        if(observer)observer(object,key,false);
    };
    // App::Link.LinkedObject is natively Global. Keep the complete original
    // occurrence, including its Placement and scaled/nested geometry, while
    // giving Compound.Links a target in its own coordinate system. This also
    // keeps Alt copies independent of their source's original App::Part.
    if(sourceBridge){
        write(sourceBridge,"LinkedObject",[&]{sourceBridge->setLink(-1,plan.source);});
        write(sourceBridge,"LinkTransform",[&]{sourceBridge->getLinkTransformProperty()->setValue(true);});
        write(sourceBridge,"LinkPlacement",[&]{sourceBridge->getLinkPlacementProperty()->setValue(Base::Placement());});
    }
    if(normalizer)write(normalizer,"Links",[&]{normalizer->Links.setValues({sourceBridge});});
    write(input,"LinkedObject",[&]{input->setLink(-1,normalizer?normalizer:plan.source);});
    write(input,"LinkTransform",[&]{input->getLinkTransformProperty()->setValue(true);});
    write(input,"LinkPlacement",[&]{input->getLinkPlacementProperty()->setValue(plan.inputFrame);});
    write(scale,"Base",[&]{scale->Base.setValue(input);});
    write(scale,"Uniform",[&]{scale->Uniform.setValue(false);});
    write(scale,"XScale",[&]{scale->XScale.setValue(plan.factors[0]);});
    write(scale,"YScale",[&]{scale->YScale.setValue(plan.factors[1]);});
    write(scale,"ZScale",[&]{scale->ZScale.setValue(plan.factors[2]);});
    // A Compound has a stable identity root Location for every native Link
    // scale, reflection, nested occurrence and near-unit kernel transition.
    const std::string expression=std::string(input->getNameInDocument())+".LinkPlacement"+
        (normalizer?std::string():" * "+plan.sourcePlacementExpression);
    const auto placement=std::shared_ptr<App::Expression>(App::Expression::parse(scale,expression));
    result.placementExpression=placement->toString();
    write(scale,"ExpressionEngine",[&]{scale->setExpression(App::ObjectIdentifier(scale,std::string("Placement")),placement);});
    write(output,"LinkedObject",[&]{output->setLink(-1,scale);});
    write(output,"LinkTransform",[&]{output->getLinkTransformProperty()->setValue(true);});
    write(output,"LinkPlacement",[&]{output->getLinkPlacementProperty()->setValue(plan.resultFrame);});
    write(output,"OM9GumballScaleSource",[&]{
        auto* property=dynamic_cast<App::PropertyLinkGlobal*>(output->addDynamicProperty(
            "App::PropertyLinkGlobal","OM9GumballScaleSource","Gumball",
            "Original parametric definition of this associative Gumball Scale result"));
        if(!property)unsupported("cannot persist the Scale source association");
        property->setValue(plan.source);
    });
    // The legacy binder ignores App::Link targets. A standard Compound
    // embeds the complete post-frame result as native Part geometry while
    // retaining a one-way live dependency on the visible result.
    if(shapeOwner)write(shapeOwner,"Links",[&]{shapeOwner->Links.setValues({output});});
    for(auto* owner:plan.groups){
        auto* group=owner->getExtension<App::GroupExtension>();
        if(!group||!group->hasObject(plan.source)||!group->allowObject(output))
            unsupported("source group changed or cannot accept an associative result");
        auto members=group->Group.getValues();
        // Standard local-scope Scale.Base requires helpers in the same CS.
        if(owner->hasExtension(App::GeoFeatureGroupExtension::getExtensionClassTypeId())){
            if(sourceBridge)members.push_back(sourceBridge);
            if(normalizer)members.push_back(normalizer);
            if(shapeOwner)members.push_back(shapeOwner);
            members.push_back(input);members.push_back(scale);
        }
        members.push_back(output);
        write(owner,"Group",[&]{group->Group.setValues(members);});
    }
    for(const auto& edit:plan.incoming){
        auto* property=edit.owner->getPropertyByName(edit.key.c_str());
        auto* links=dynamic_cast<App::PropertyLinkBase*>(property);
        if(!links||!sameGumballIncomingValue(*property,edit))unsupported("incoming dependency changed before publication");
        if(!incomingWritable(*edit.owner,edit.key,*property))
            unsupported("incoming dependency became protected during publication");
        App::DocumentObject* target=legacyShapeSupport(*edit.owner,edit.key,*property)
            ?static_cast<App::DocumentObject*>(shapeOwner):static_cast<App::DocumentObject*>(output);
        if(!target)unsupported("native ShapeBinder result owner is unavailable");
        std::unique_ptr<App::Property> replacement(links->CopyOnLinkReplace(edit.owner,plan.source,target));
        if(!replacement)unsupported("incoming dependency cannot be rerouted to the native result");
        write(edit.owner,edit.key.c_str(),[&]{property->Paste(*replacement);});
        auto expected=edit;expected.before=std::shared_ptr<App::Property>(std::move(replacement));
        if(!sameGumballIncomingValue(*property,expected))
            unsupported("incoming dependency changed during publication");
        result.rewritten.push_back(edit);
    }
    verifyGumballAssociativeScale(plan,result);
    return result;
}
void verifyGumballAssociativeScale(const GumballAssociativeScalePlan& plan,
    const GumballAssociativeScaleResult& result){
    auto* document=App::GetApplication().getDocument(result.documentName.c_str());
    if(!document||document!=result.document||result.source!=plan.source
        ||result.identities.size()!=result.created.size()+1)
        unsupported("native graph document or source changed during publication");
    for(const auto& identity:result.identities){
        auto* object=document->getObject(identity.name.c_str());
        if(!object||object!=identity.object||object->getID()!=identity.id||object->getTypeId().getName()!=identity.type)
            unsupported("native graph object identity changed during publication");
    }
    auto* input=dynamic_cast<App::Link*>(result.input);
    auto* scale=dynamic_cast<Part::Scale*>(result.scale);
    auto* output=dynamic_cast<App::Link*>(result.output);
    auto* sourceBridge=dynamic_cast<App::Link*>(result.sourceBridge);
    auto* normalizer=dynamic_cast<Part::Compound*>(result.normalizer);
    auto* shapeOwner=dynamic_cast<Part::Compound*>(result.shapeOwner);
    if(!input||!scale||!output||bool(sourceBridge)!=plan.normalizeOccurrence
        ||bool(normalizer)!=plan.normalizeOccurrence||bool(shapeOwner)!=plan.requiresShapeOwner)
        unsupported("native graph types changed during publication");
    const auto actualPlacement=scale->getExpression(App::ObjectIdentifier(scale,std::string("Placement"))).expression;
    const auto* association=output->getPropertyByName<App::PropertyLinkGlobal>("OM9GumballScaleSource");
    const Base::Vector3d unit(1,1,1);
    if(input->getLinkedObject(false)!=(normalizer?normalizer:plan.source)
        ||!input->ExpressionEngine.getExpressions().empty()||!output->ExpressionEngine.getExpressions().empty()
        ||scale->ExpressionEngine.getExpressions().size()!=1
        ||!input->getLinkTransformValue()||!input->getLinkPlacementProperty()->getValue().isSame(plan.inputFrame)
        ||input->getScaleProperty()->getValue()!=1||input->getScaleVectorProperty()->getValue()!=unit
        ||scale->Base.getValue()!=input||scale->Uniform.getValue()
        ||scale->XScale.getValue()!=plan.factors[0]||scale->YScale.getValue()!=plan.factors[1]
        ||scale->ZScale.getValue()!=plan.factors[2]||!actualPlacement
        ||actualPlacement->toString()!=result.placementExpression||output->getLinkedObject(false)!=scale
        ||!output->getLinkTransformValue()||!output->getLinkPlacementProperty()->getValue().isSame(plan.resultFrame)
        ||output->getScaleProperty()->getValue()!=1||output->getScaleVectorProperty()->getValue()!=unit
        ||!association||association->getValue()!=plan.source
        ||(sourceBridge&&(sourceBridge->getLinkedObject(false)!=plan.source
            ||sourceBridge->getLinkedObjectProperty()->getScope()!=App::LinkScope::Global
            ||!sourceBridge->getLinkTransformValue()||!sourceBridge->getLinkPlacementProperty()->getValue().isSame(Base::Placement())
            ||sourceBridge->getScaleProperty()->getValue()!=1||sourceBridge->getScaleVectorProperty()->getValue()!=unit
            ||!sourceBridge->ExpressionEngine.getExpressions().empty()))
        ||(normalizer&&(normalizer->Links.getValues()!=std::vector<App::DocumentObject*>{sourceBridge}
            ||std::string(normalizer->getViewProviderNameStored())!="PartGui::ViewProviderPartExt"
            ||!normalizer->Placement.getValue().isSame(Base::Placement())||!normalizer->ExpressionEngine.getExpressions().empty()))
        ||(shapeOwner&&(shapeOwner->Links.getValues()!=std::vector<App::DocumentObject*>{output}
            ||std::string(shapeOwner->getViewProviderNameStored())!="PartGui::ViewProviderPartExt"
            ||!shapeOwner->Placement.getValue().isSame(Base::Placement())||!shapeOwner->ExpressionEngine.getExpressions().empty())))
        unsupported("native graph inputs changed during publication");
}
}
