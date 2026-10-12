// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreDefinitionSnapshot.h"
#include "CoreObjects.h"
#include "CoreSourceAttributes.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/GeoFeatureGroupExtension.h>
#include <App/GroupExtension.h>
#include <App/Link.h>
#include <Mod/Part/App/PartFeature.h>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonParseError>
#include <algorithm>
#include <charconv>
#include <cstdint>
#include <stdexcept>
#include <unordered_set>
#include <vector>
namespace OpenMatrix9Gui {
namespace {
using Matrix=std::array<double,16>;
constexpr Matrix identity{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1};
constexpr std::size_t rootIndex=static_cast<std::size_t>(-1);
struct FrameInput {Matrix parent,path,source,location;};
struct FrameSummary {std::uint32_t status,reserved;Matrix world;std::int32_t sign;std::uint32_t reserved2;};
struct Node {ObjectStamp stamp;std::size_t parent;std::uint32_t representation,reserved;Matrix world;};
struct JsonSummary {std::uint32_t status,reserved;std::size_t needed;};
struct DescriptorInput {ObjectStamp stamp;std::uint32_t legacy_kind,representation,known,values;double domains[4];std::uint32_t domain_count,form,facts_scope,domain_scope;};
struct DescriptorSummary {std::uint32_t status,schema_version,representation,solid,known,values,form,domain_count;double domains[4];std::uint32_t facts_scope,domain_scope;};
static_assert(sizeof(FrameInput)==512&&sizeof(FrameSummary)==144&&alignof(FrameSummary)==8&&offsetof(FrameSummary,world)==8);
static_assert(sizeof(Node)==184&&alignof(Node)==8&&offsetof(Node,world)==56&&offsetof(Node,parent)==40);
static_assert(sizeof(JsonSummary)==16&&sizeof(DescriptorInput)==104&&sizeof(DescriptorSummary)==72);
extern "C" FrameSummary om9_geometry_frame(FrameInput);
extern "C" DescriptorSummary om9_geometry_descriptor(DescriptorInput);
extern "C" std::uint32_t om9_instance_graph_counts(std::size_t,std::size_t);
extern "C" JsonSummary om9_instance_graph_json(const Node*,std::size_t,std::uint8_t*,std::size_t);
extern "C" std::uint32_t om9_support_payload(std::size_t,std::size_t);
struct ArrayOccurrence {std::uint64_t index;std::uint32_t element_present,reserved;ObjectStamp element,definition;Matrix world;};
static_assert(sizeof(ArrayOccurrence)==224&&alignof(ArrayOccurrence)==8&&offsetof(ArrayOccurrence,world)==96);
extern "C" std::uint32_t om9_array_occurrences_counts(std::size_t,std::size_t);
extern "C" JsonSummary om9_array_occurrences_json(const ArrayOccurrence*,std::size_t,std::size_t,std::uint8_t*,std::size_t);
struct ReferenceStep {std::uint32_t start,len,kind,index;};
struct ReferenceSummary {std::uint32_t status,count,selector_start,selector_len;};
struct ArrayAliasSummary {std::uint32_t status,matched,index,reserved;};
static_assert(sizeof(ReferenceStep)==16&&alignof(ReferenceStep)==4&&sizeof(ReferenceSummary)==16&&sizeof(ArrayAliasSummary)==16);
extern "C" ReferenceSummary om9_reference_path_parse(const std::uint8_t*,std::size_t,ReferenceStep*,std::size_t);
extern "C" ArrayAliasSummary om9_reference_array_index(const std::uint8_t*,std::size_t,const std::uint8_t*,std::size_t,std::uint32_t);
ArrayAliasSummary arrayIndex(App::DocumentObject& object,const std::string& token,bool collapsed=true) {
    const std::string owner=object.getNameInDocument();
    const auto result=om9_reference_array_index(reinterpret_cast<const std::uint8_t*>(owner.data()),owner.size(),reinterpret_cast<const std::uint8_t*>(token.data()),token.size(),static_cast<std::uint32_t>(collapsed));
    if(result.status||result.reserved||result.matched>1)throw std::runtime_error("Invalid bounded native array element selector");return result;
}
QJsonValue unknown(){return QJsonValue(QJsonValue::Null);}
QString text(const std::string& s){return QString::fromUtf8(s.data(),static_cast<int>(s.size()));}
QJsonArray matrix(const Matrix& m){QJsonArray rows;for(int r=0;r<4;++r)rows.append(QJsonArray{m[r*4],m[r*4+1],m[r*4+2],m[r*4+3]});return rows;}
Matrix nativeMatrix(const Base::Matrix4D& m){Matrix out{};for(int r=0;r<4;++r)for(int c=0;c<4;++c)out[r*4+c]=m[r][c];return out;}
Matrix composed(Matrix parent,Matrix path,Matrix source=identity,Matrix location=identity) {
    const auto result=om9_geometry_frame(FrameInput{parent,path,source,location});
    if(result.status)throw std::runtime_error("Unsupported native definition transform (status "+std::to_string(result.status)+")");
    return result.world;
}
Matrix selfPlacement(App::DocumentObject& object) {
    Base::Matrix4D m;
    if(object.getSubObject("",nullptr,&m,true)!=&object)throw std::runtime_error("Unsupported native definition object frame");
    return nativeMatrix(m);
}
QJsonObject stamp(const ObjectStamp& s){return {{"document",QString::number(s.document)},{"object",QString::number(s.object)},{"generation",QString::number(s.generation)},{"revision",QString::number(s.revision)},{"epoch",QString::number(s.epoch)}};}
App::GroupExtension* group(App::DocumentObject& object){return object.getExtensionByType<App::GroupExtension>(true);}
bool groupDefinition(App::DocumentObject& object){
    const auto derived=[&](const char* name){const auto type=Base::Type::fromName(name);return !type.isBad()&&object.getTypeId().isDerivedFrom(type);};
    return !dynamic_cast<Part::Feature*>(&object)&&!derived("Mesh::Feature")&&!derived("Points::Feature")&&group(object);
}
struct Capture {
    struct Guard {App::Document* doc;ObjectSnapshot snapshot;};
    DefinitionReaders readers;
    std::vector<Node> nodes;
    std::vector<Guard> guards;
    std::unordered_set<App::DocumentObject*> guarded,active;
    std::size_t payload=0;
    ObjectSnapshot remember(App::DocumentObject& object,const std::string& sub={}) {
        auto* doc=object.getDocument();
        if(!doc||doc->testStatus(App::Document::Restoring)||!object.isAttachedToDocument()||object.isRemoving())throw std::runtime_error("Definition owner is missing or restoring");
        if(!guarded.contains(&object)&&om9_instance_graph_counts(guards.size()+1,1))throw std::runtime_error("Definition source guard budget exceeded");
        auto snapshot=captureObject(*doc,object.getNameInDocument(),sub);
        if(guarded.insert(&object).second)guards.push_back({doc,snapshot});
        return snapshot;
    }
    std::size_t reserve(const ObjectSnapshot& s,std::size_t parent,std::size_t depth,std::uint32_t representation,Matrix world) {
        if(om9_instance_graph_counts(nodes.size()+1,depth))throw std::runtime_error("Definition occurrence count/depth budget exceeded");
        const auto index=nodes.size();nodes.push_back(Node{s.stamp,parent,representation,0,world});return index;
    }
    void accountBytes(std::size_t bytes) {
        if(om9_support_payload(payload,bytes))throw std::runtime_error("Definition serialized payload budget exceeded");
        payload+=bytes;
    }
    void account(const QJsonObject& header,std::size_t members=0) {
        const auto bytes=QJsonDocument(header).toJson(QJsonDocument::Compact).size();
        if(bytes<0)throw std::runtime_error("Invalid definition serialized payload size");
        accountBytes(static_cast<std::size_t>(bytes));
        if(members>1)accountBytes(members-1); // Commas between child JSON values.
    }
    QJsonObject graph(std::size_t copies) {
        const auto query=om9_instance_graph_json(nodes.data(),nodes.size(),nullptr,0);
        if(query.status)throw std::runtime_error("Rust rejected owned definition graph");
        for(std::size_t i=0;i<copies;++i){accountBytes(sizeof(",\"definition_graph\":")-1);accountBytes(query.needed);}
        QByteArray raw(static_cast<int>(query.needed),'\0');
        const auto copied=om9_instance_graph_json(nodes.data(),nodes.size(),reinterpret_cast<std::uint8_t*>(raw.data()),query.needed);
        if(copied.status||copied.needed!=query.needed)throw std::runtime_error("Rust could not copy definition graph atomically");
        QJsonParseError error;const auto parsed=QJsonDocument::fromJson(raw,&error);
        if(error.error!=QJsonParseError::NoError||!parsed.isObject())throw std::runtime_error("Invalid Rust definition graph JSON");
        // Qt writes floating point values with its own JSON spelling. The Rust
        // byte count alone is not a bound on the graph embedded by Qt.
        const auto embeddedBytes=parsed.toJson(QJsonDocument::Compact).size();
        if(embeddedBytes<0)throw std::runtime_error("Invalid embedded definition graph size");
        if(static_cast<std::size_t>(embeddedBytes)>query.needed)
            for(std::size_t i=0;i<copies;++i)accountBytes(static_cast<std::size_t>(embeddedBytes)-query.needed);
        return parsed.object();
    }
};
QJsonObject container(App::DocumentObject& object,const ObjectSnapshot& s,std::uint32_t representation,Matrix world) {
    DescriptorInput input{};input.stamp=s.stamp;input.legacy_kind=9;input.representation=representation;
    const auto facts=om9_geometry_descriptor(input);
    if(facts.status||facts.representation!=representation||facts.solid!=0)throw std::runtime_error("Rust rejected definition container representation");
    const auto checked=om9_geometry_frame(FrameInput{identity,world,identity,identity});
    if(checked.status)throw std::runtime_error("Rust rejected definition container frame");
    return {{"schema_version",static_cast<int>(facts.schema_version)},{"representation",representation==10?"instance":"group"},{"legacy_kind",9},
        {"stamp",stamp(s.stamp)},{"object_name",text(s.objectName)},{"document_uid",text(s.documentUid)},{"subelement",text(s.subelement)},
        {"unit_context_snapshot",text(s.unitContext)},{"world_placement_provenance",matrix(s.world)},
        {"local_to_world",matrix(world)},{"world_orientation_sign",checked.sign},{"coordinates",representation==10?"definition_object_local":"object_local"},
        {"form",unknown()},{"facts_scope",unknown()},{"domain_scope",unknown()},{"domains",unknown()},{"basis",unknown()},{"basis_frame",unknown()},
        {"basis_status",QJsonObject{{"state","uninspected"},{"reason","definition_members_are_separate"}}},{"brep_tool_location",unknown()},{"restriction_facts",unknown()},
        {"facts",QJsonObject{{"closed",unknown()},{"periodic",unknown()},{"rational",unknown()}}},
        {"topology",QJsonObject{{"solid",unknown()},{"solid_evidence",unknown()},{"closed",unknown()},{"manifold",unknown()},{"solid_orientation",unknown()},{"faces",unknown()},{"free_boundary_components",unknown()}}},
        {"source_attributes",sourceAttributes(object,s)},{"preservation",QJsonObject{{"locus",unknown()},{"parameterization",unknown()},{"topology",unknown()},{"parameter_map",unknown()}}}};
}
// Rust owns bounded UTF-8 path syntax. Native membership/linkage is preflighted
// one step at a time before getSubObject/getLinkedObject may recurse.
struct Reference {App::DocumentObject* object;std::string prefix,selector;};
Reference resolveReference(Capture& capture,App::DocumentObject& root,const std::string& path,std::size_t& depth,std::unordered_set<App::DocumentObject*>& activeLinks) {
    const auto* bytes=reinterpret_cast<const std::uint8_t*>(path.data());
    const auto query=om9_reference_path_parse(bytes,path.size(),nullptr,0);
    if(query.status||query.count>64||query.selector_start>path.size()||query.selector_len!=path.size()-query.selector_start)
        throw std::runtime_error("Invalid definition reference path");
    std::vector<ReferenceStep> steps(query.count);
    const auto parsed=om9_reference_path_parse(bytes,path.size(),steps.data(),steps.size());
    if(parsed.status||parsed.count!=query.count||parsed.selector_start!=query.selector_start||parsed.selector_len!=query.selector_len)
        throw std::runtime_error("Invalid Rust reference path output");
    Reference out{&root,"",""};capture.remember(root);bool virtualDefinition=false;
    for(const auto& step:steps) {
        if(++depth>64)throw std::runtime_error("Invalid or excessively nested definition reference");
        if(step.start>path.size()||step.len>path.size()-step.start||!step.len||step.kind>2)throw std::runtime_error("Invalid Rust reference step");
        const auto token=path.substr(step.start,step.len);
        const auto matches=[&](App::DocumentObject& object){return step.kind==1?object.Label.getStrValue()==token.substr(1):token==object.getNameInDocument();};
        bool linked=virtualDefinition;virtualDefinition=false;std::vector<App::DocumentObject*> chain;
        struct LinkGuard {std::unordered_set<App::DocumentObject*>& active;std::vector<App::DocumentObject*>& chain;~LinkGuard(){for(auto* object:chain)active.erase(object);}} guard{activeLinks,chain};
        while(out.object->isLink()) {
            if(linked&&matches(*out.object))break;
            if(++depth>64)throw std::runtime_error("Invalid or excessively nested definition reference");
            if(!activeLinks.insert(out.object).second)throw std::runtime_error("Cyclic native definition reference");chain.push_back(out.object);capture.remember(*out.object);
            auto* link=out.object->getExtensionByType<App::LinkBaseExtension>(true);
            if(!link||link->_getElementCountValue()!=0)throw std::runtime_error("Unsupported native reference linkage");
            if(std::any_of(link->getSubElements().begin(),link->getSubElements().end(),[](const auto& s){return !s.empty();}))
                throw std::runtime_error("Unsupported configured subelement reference traversal");
            auto* target=link->getLink();if(!target)throw std::runtime_error("Missing native reference definition");
            const auto resolved=resolveReference(capture,*target,link->getSubName()?link->getSubName():"",depth,activeLinks);
            if(!resolved.selector.empty())throw std::runtime_error("Invalid native reference member object path");
            out.object=resolved.object;linked=true;
        }
        if(linked&&matches(*out.object)) {
            // FreeCAD accepts an explicit linked-definition name at each hop,
            // including intermediate links and linked arrays.
            if(auto* ext=group(*out.object)) {
                const auto& members=ext->getObjects();
                if(om9_instance_graph_counts(members.size()+1,depth))throw std::runtime_error("Definition reference member budget exceeded");
                for(auto* member:members){if(!member)throw std::runtime_error("Missing native definition member");capture.remember(*member);
                    if(matches(*member))throw std::runtime_error("Ambiguous explicit definition member reference");}
            }
        } else if(auto* array=out.object->getExtensionByType<App::LinkBaseExtension>(true);array&&array->_getElementCountValue()!=0) {
            const int count=array->_getElementCountValue();
            if(count<=0||om9_array_occurrences_counts(static_cast<std::size_t>(count),static_cast<std::size_t>(count)))throw std::runtime_error("Reference array element/count budget exceeded");
            const auto& elements=array->_getElementListValue();const bool expanded=array->_getShowElementValue();
            if((expanded&&elements.size()!=static_cast<std::size_t>(count))||(!expanded&&!elements.empty()))throw std::runtime_error("Invalid native reference array ownership/count");
            std::size_t index=step.index;
            if(expanded&&step.kind!=2) {
                bool found=false;
                for(std::size_t i=0;i<elements.size();++i){auto* member=elements[i];if(!member)throw std::runtime_error("Missing native reference array element");capture.remember(*member);
                    if(matches(*member)){if(found)throw std::runtime_error("Ambiguous native reference array element");found=true;index=i;}}
                if(!found)throw std::runtime_error("Missing native reference array element");
            }
            if(index>=static_cast<std::size_t>(count))throw std::runtime_error("Native reference array index is out of range");
            if(expanded){out.object=elements[index];if(!out.object)throw std::runtime_error("Missing native reference array element");capture.remember(*out.object);}
            else {
                const auto alias=arrayIndex(*out.object,token);
                if(std::any_of(array->getSubElements().begin(),array->getSubElements().end(),[](const auto& s){return !s.empty();}))
                    throw std::runtime_error("Unsupported configured virtual array reference traversal");
                auto* target=array->getLink();if(!target)throw std::runtime_error("Missing native reference array definition");
                const auto resolved=resolveReference(capture,*target,array->getSubName()?array->getSubName():"",depth,activeLinks);
                if(!resolved.selector.empty())throw std::runtime_error("Invalid native reference array member path");out.object=resolved.object;
                if(alias.matched&&alias.index<static_cast<std::size_t>(count)) {
                    virtualDefinition=true;
                } else if(matches(*out.object))virtualDefinition=true;
                else {
                    // Native collapsed arrays also accept an unindexed member
                    // path through occurrence0 without consuming that member.
                    const auto member=resolveReference(capture,*out.object,token+'.',depth,activeLinks);
                    out.object=member.object;
                }
            }
        } else {
            auto* ext=group(*out.object);if(!ext)throw std::runtime_error("Unsupported definition reference container");
            const auto& members=ext->getObjects();
            if(om9_instance_graph_counts(members.size()+1,depth))throw std::runtime_error("Definition reference member budget exceeded");
            App::DocumentObject* child=nullptr;
            for(auto* member:members){if(!member)throw std::runtime_error("Missing native definition member");capture.remember(*member);
                if(matches(*member)){if(child)throw std::runtime_error("Ambiguous definition member reference");child=member;}}
            if(!child)throw std::runtime_error("Missing native definition member reference");out.object=child;capture.remember(*child);
        }
        out.prefix+=token+'.';
    }
    out.selector=path.substr(query.selector_start);
    return out;
}
Reference reference(Capture& capture,App::DocumentObject& root,const std::string& path,std::size_t depth) {
    std::unordered_set<App::DocumentObject*> activeLinks;return resolveReference(capture,root,path,depth,activeLinks);
}
std::string definitionSelection(Capture& capture,App::DocumentObject& target,const std::string& selection,std::size_t depth) {
    const auto parsed=om9_reference_path_parse(reinterpret_cast<const std::uint8_t*>(selection.data()),selection.size(),nullptr,0);
    if(parsed.status||parsed.count>64)throw std::runtime_error("Invalid linked definition selection");
    if(!parsed.count)return selection;
    const auto dot=selection.find('.');const auto token=selection.substr(0,dot);
    const auto matches=[&](App::DocumentObject& object){return token[0]=='$'?object.Label.getStrValue()==token.substr(1):token==object.getNameInDocument();};
    if(!matches(target))return selection;
    capture.remember(target);
    if(auto* ext=group(target)) {
        const auto& members=ext->getObjects();
        if(om9_instance_graph_counts(members.size()+1,depth))throw std::runtime_error("Definition selection member budget exceeded");
        for(auto* member:members){if(!member)throw std::runtime_error("Missing native definition member");capture.remember(*member);
            if(matches(*member))throw std::runtime_error("Ambiguous explicit definition member selection");}
    }
    return selection.substr(dot+1);
}
QJsonObject captureAt(Capture&,App::DocumentObject&,const std::string&,Matrix,std::size_t,std::size_t);
QJsonObject selectedCollection(Capture& capture,App::DocumentObject& object,const std::vector<std::string>& selectors,Matrix objectWorld,std::size_t parent,std::size_t depth) {
    const auto source=capture.remember(object);const auto index=capture.reserve(source,parent,depth,11,objectWorld);
    auto result=container(object,source,11,objectWorld);QJsonArray members;
    result["geometry_data"]=QJsonObject{{"kind","subelement_collection"},{"schema_version",1},{"members",members}};
    result["geometry_data_status"]="copied_validated_subelement_collection";
    capture.account(result,selectors.size());
    for(const auto& selector:selectors)members.append(captureAt(capture,object,selector,composed(objectWorld,identity,selfPlacement(object),identity),index,depth+1));
    result["geometry_data"]=QJsonObject{{"kind","subelement_collection"},{"schema_version",1},{"members",members}};
    result["geometry_data_status"]="copied_validated_subelement_collection";return result;
}
QJsonObject captureArray(Capture& capture,App::DocumentObject& object,App::LinkBaseExtension& ext,const ObjectSnapshot& source,const std::string& selection,Matrix parentWorld,std::size_t parent,std::size_t depth) {
    const int nativeCount=ext._getElementCountValue();
    if(nativeCount<=0||om9_array_occurrences_counts(static_cast<std::size_t>(nativeCount),static_cast<std::size_t>(nativeCount)))
        throw std::runtime_error("Instance array element/count budget exceeded");
    const auto total=static_cast<std::size_t>(nativeCount);
    // Every selected occurrence needs an instance and at least one definition
    // node. Reject against the existing graph before staging native paths.
    const auto selectedCount=selection.empty()?total:std::size_t{1};
    if(om9_instance_graph_counts(capture.nodes.size()+1+2*selectedCount,depth+2))
        throw std::runtime_error("Instance array occurrence/depth budget exceeded");
    const bool expanded=ext._getShowElementValue();
    const auto& elements=ext._getElementListValue();
    if((expanded&&elements.size()!=total)||(!expanded&&!elements.empty()))throw std::runtime_error("Invalid native array element ownership/count");
    // No label callbacks or unchecked native decimal parser. The array count
    // gate precedes even inspecting the native element list or reference path.
    for(auto* element:elements){if(!element||!element->isAttachedToDocument()||element->isRemoving())throw std::runtime_error("Missing native array element owner");capture.remember(*element);}
    std::vector<std::size_t> indices;std::string selected;
    if(selection.empty()){indices.reserve(total);for(std::size_t i=0;i<total;++i)indices.push_back(i);}
    else {
        const auto dot=selection.find('.');const auto token=selection.substr(0,dot);selected=dot==std::string::npos?"":selection.substr(dot+1);
        std::size_t index=0;const auto alias=arrayIndex(object,token,!expanded);
        if(token.empty())throw std::runtime_error("Invalid native array element selector");
        if(alias.matched&&(alias.index<total||arrayIndex(object,token,false).matched))index=alias.index;
        else if(!expanded) {
            auto* raw=ext.getLink();if(!raw)throw std::runtime_error("Native array definition is missing");
            const auto target=reference(capture,*raw,ext.getSubName()?ext.getSubName():"",depth+1);
            if(!target.selector.empty())throw std::runtime_error("Invalid native array member object path");
            const bool matches=token[0]=='$'?target.object->Label.getStrValue()==token.substr(1):token==target.object->getNameInDocument();
            if(!matches){const auto member=reference(capture,*target.object,token+'.',depth+1);(void)member;selected=token+'.'+selected;}
        } else {
            bool found=false;index=0;
            for(std::size_t i=0;i<elements.size();++i){const bool match=token[0]=='$'?elements[i]->Label.getStrValue()==token.substr(1):token==elements[i]->getNameInDocument();
                if(match){if(found)throw std::runtime_error("Ambiguous native array element selector");found=true;index=i;}}
            if(!found)throw std::runtime_error("Missing native array element selector");
        }
        if(index>=total)throw std::runtime_error("Native array element selector is out of range");indices.push_back(index);
        const auto syntax=om9_reference_path_parse(reinterpret_cast<const std::uint8_t*>(selection.data()),selection.size(),nullptr,0);
        if(syntax.status)throw std::runtime_error("Invalid native array selection path");
        // Collapsed occurrence-only selection is checked by the canonical
        // numeric path below and must retain configured topology collections.
        if(expanded||!selected.empty()) {
            const auto original=reference(capture,object,selection,depth);
            Base::Matrix4D originalPath;
            if(object.getSubObject(original.prefix.c_str(),nullptr,&originalPath,true)!=original.object)
                throw std::runtime_error("Native array selection identity changed or is suppressed");
        }
    }
    const auto world=composed(parentWorld,selfPlacement(object));const auto root=capture.reserve(source,parent,depth,11,world);
    auto result=container(object,source,11,world);
    struct Pending {std::size_t index;App::DocumentObject* element;App::DocumentObject* target;Matrix world,elementParent;std::vector<std::string> selectors;ObjectSnapshot owner,definition;std::string linkedPath;};
    std::vector<Pending> pending;std::vector<ArrayOccurrence> rows;pending.reserve(indices.size());rows.reserve(indices.size());
    for(const auto index:indices) {
        auto* element=expanded?elements[index]:nullptr;
        auto* linkExt=element?element->getExtensionByType<App::LinkBaseExtension>(true):&ext;
        if(!linkExt||(element&&linkExt->_getElementCountValue()!=0))throw std::runtime_error("Unsupported native array element linkage");
        auto* raw=linkExt->getLink();if(!raw)throw std::runtime_error("Native array definition is missing");
        const std::string linkedPath=linkExt->getSubName()?linkExt->getSubName():"";
        const auto target=reference(capture,*raw,linkedPath,depth+1);
        if(!target.selector.empty()||capture.active.contains(target.object))throw std::runtime_error("Cyclic or invalid native array definition reference");
        std::vector<std::string> selectors;const auto& nativeSelectors=linkExt->getSubElements();
        if(om9_instance_graph_counts(nativeSelectors.size()+1,depth+1))throw std::runtime_error("Array subelement count/depth budget exceeded");
        for(const auto& sub:nativeSelectors)if(!sub.empty())selectors.push_back(sub);
        const auto nativeSelection=definitionSelection(capture,*target.object,selected,depth+1);
        if(!selected.empty()){if(!selectors.empty())throw std::runtime_error("Unsupported overlapping native array subelement selection");selectors.push_back(nativeSelection);}
        Base::Matrix4D nativePath;const auto path=std::to_string(index)+'.';
        auto* resolved=object.getSubObject(path.c_str(),nullptr,&nativePath,true);
        if(resolved!=(element?element:target.object))throw std::runtime_error("Native array element identity changed or is suppressed");
        auto occurrenceWorld=composed(parentWorld,nativeMatrix(nativePath));Matrix elementParent=identity;
        if(element) {
            Base::Matrix4D linkedPath;
            if(element->getLinkedObject(false,&linkedPath,false)!=target.object)throw std::runtime_error("Native array element definition identity changed");
            elementParent=composed(occurrenceWorld,identity,selfPlacement(*element),identity);
            occurrenceWorld=composed(occurrenceWorld,nativeMatrix(linkedPath));
        }
        auto owner=element?capture.remember(*element):source;
        if(!element)owner.subelement=path;
        const auto definition=capture.remember(*target.object,selectors.size()==1?selectors[0]:"");
        rows.push_back({index,static_cast<std::uint32_t>(element!=nullptr),0,element?owner.stamp:ObjectStamp{},definition.stamp,occurrenceWorld});
        pending.push_back({index,element,target.object,occurrenceWorld,elementParent,std::move(selectors),std::move(owner),definition,linkedPath});
    }
    const auto query=om9_array_occurrences_json(rows.data(),rows.size(),total,nullptr,0);
    if(query.status||query.reserved||!query.needed||om9_support_payload(capture.payload,query.needed))throw std::runtime_error("Rust rejected native array occurrence metadata/budget");
    QByteArray raw(static_cast<qsizetype>(query.needed),'\0');const auto copied=om9_array_occurrences_json(rows.data(),rows.size(),total,reinterpret_cast<std::uint8_t*>(raw.data()),query.needed);
    if(copied.status||copied.reserved||copied.needed!=query.needed)throw std::runtime_error("Rust rejected native array occurrence output");
    QJsonParseError error;const auto metadata=QJsonDocument::fromJson(raw,&error);
    if(error.error!=QJsonParseError::NoError||!metadata.isObject())throw std::runtime_error("Invalid Rust array occurrence JSON");
    QJsonArray members;
    result["geometry_data"]=QJsonObject{{"kind","instance_array"},{"schema_version",1},{"array_occurrences",metadata.object()},{"members",members}};
    result["geometry_data_status"]="copied_validated_instance_array";capture.account(result,pending.size());
    for(const auto& entry:pending) {
        QJsonObject member;
        if(entry.element)member=captureAt(capture,*entry.element,selected,entry.elementParent,root,depth+1);
        else {
            const auto index=capture.reserve(entry.owner,root,depth+1,10,entry.world);
            member=container(object,entry.owner,10,entry.world);
            const QJsonObject ref{{"stamp",stamp(entry.definition.stamp)},{"document_uid",text(entry.definition.documentUid)},{"object_name",text(entry.definition.objectName)},{"subelement",text(entry.definition.subelement)}};
            member["geometry_data"]=QJsonObject{{"kind","instance"},{"schema_version",1},{"definition_ref",ref},{"definition_object_to_world",matrix(entry.world)},{"linked_member_path",text(entry.linkedPath)},{"definition_geometry",QJsonObject{}}};
            member["geometry_data_status"]="copied_validated_instance_definition";capture.account(member);
            QJsonObject child;
            if(entry.selectors.size()>1)child=selectedCollection(capture,*entry.target,entry.selectors,entry.world,index,depth+2);
            else child=captureAt(capture,*entry.target,entry.selectors.empty()?"":entry.selectors[0],composed(entry.world,identity,selfPlacement(*entry.target),identity),index,depth+2);
            const QJsonObject actual{{"stamp",child["stamp"]},{"document_uid",child["document_uid"]},{"object_name",child["object_name"]},{"subelement",child["subelement"]}};
            if(actual!=ref)throw std::runtime_error("Native array definition identity changed during capture");
            auto data=member["geometry_data"].toObject();data["definition_geometry"]=child;member["geometry_data"]=data;
        }
        // Index/path is represented by the Rust-owned metadata above; the
        // ordered member position is sufficient and adds no unaccounted bytes.
        members.append(member);
    }
    auto data=result["geometry_data"].toObject();data["members"]=members;result["geometry_data"]=data;return result;
}
QJsonObject captureAt(Capture& capture,App::DocumentObject& object,const std::string& selection,Matrix parentWorld,std::size_t parent,std::size_t depth) {
    if(!selection.empty()&&object.isDerivedFrom<App::LinkElement>()) {
        const auto* suppressed=object.getExtensionByType<App::SuppressibleExtension>(true);
        if(suppressed&&suppressed->Suppressed.getValue())throw std::runtime_error("Native suppressed element definition traversal is unsupported");
    }
    if(!capture.active.insert(&object).second)throw std::runtime_error("Cyclic instance/group definition graph");
    struct ActiveGuard {Capture& capture;App::DocumentObject* object;~ActiveGuard(){capture.active.erase(object);}} active{capture,&object};
    const auto source=capture.remember(object,selection);
    if(auto* array=object.getExtensionByType<App::LinkBaseExtension>(true);array&&array->_getElementCountValue()!=0)
        return captureArray(capture,object,*array,source,selection,parentWorld,parent,depth);
    if(object.isLink()) {
        const auto index=capture.reserve(source,parent,depth,10,identity);
        auto* ext=object.getExtensionByType<App::LinkBaseExtension>();
        if(!ext||ext->_getElementCountValue()!=0)throw std::runtime_error("Unsupported instance array definition snapshot");
        auto* raw=ext->getLink();if(!raw)throw std::runtime_error("Instance definition is missing");
        const auto path=ext->getSubName()?std::string(ext->getSubName()):std::string();
        const auto target=reference(capture,*raw,path,depth);
        if(!target.selector.empty())throw std::runtime_error("Invalid native member object path");
        if(capture.active.contains(target.object))throw std::runtime_error("Cyclic nested instance definition");
        std::vector<std::string> selectors;
        const auto& nativeSelectors=ext->getSubElements();
        if(om9_instance_graph_counts(nativeSelectors.size()+1,depth))throw std::runtime_error("Instance subelement count budget exceeded");
        for(const auto& sub:nativeSelectors)if(!sub.empty())selectors.push_back(sub);
        if(!selection.empty()) {if(!selectors.empty())throw std::runtime_error("Unsupported overlapping nested subelement selection");selectors.push_back(definitionSelection(capture,*target.object,selection,depth));}
        Base::Matrix4D nativePath;
        if(object.getLinkedObject(false,&nativePath,true)!=target.object)throw std::runtime_error("Native instance member identity does not match preflight");
        const auto world=composed(parentWorld,nativeMatrix(nativePath));capture.nodes[index].world=world;
        const auto targetParent=composed(world,identity,selfPlacement(*target.object),identity);
        auto result=container(object,source,10,world);
        const auto targetSource=capture.remember(*target.object,selectors.size()==1?selectors[0]:"");
        const QJsonObject expectedRef{{"stamp",stamp(targetSource.stamp)},{"document_uid",text(targetSource.documentUid)},
            {"object_name",text(targetSource.objectName)},{"subelement",text(targetSource.subelement)}};
        result["geometry_data"]=QJsonObject{{"kind","instance"},{"schema_version",1},{"definition_ref",expectedRef},
            {"definition_object_to_world",matrix(world)},{"linked_member_path",text(path)},{"definition_geometry",QJsonObject{}}};
        result["geometry_data_status"]="copied_validated_instance_definition";
        // Count the container before descending. The empty child placeholder
        // conservatively adds two bytes; every child's JSON is counted separately.
        capture.account(result);
        QJsonObject child;
        if(selectors.size()>1)child=selectedCollection(capture,*target.object,selectors,world,index,depth+1);
        else child=captureAt(capture,*target.object,selectors.empty()?"":selectors[0],targetParent,index,depth+1);
        const QJsonObject actualRef{{"stamp",child["stamp"]},{"document_uid",child["document_uid"]},{"object_name",child["object_name"]},{"subelement",child["subelement"]}};
        if(actualRef!=expectedRef)throw std::runtime_error("Native definition identity changed during capture");
        result["geometry_data"]=QJsonObject{{"kind","instance"},{"schema_version",1},
            {"definition_ref",QJsonObject{{"stamp",child["stamp"]},{"document_uid",child["document_uid"]},{"object_name",child["object_name"]},{"subelement",child["subelement"]}}},
            {"definition_object_to_world",matrix(world)},{"linked_member_path",text(path)},{"definition_geometry",child}};
        result["geometry_data_status"]="copied_validated_instance_definition";return result;
    }
    if(groupDefinition(object)) {
        const auto self=selfPlacement(object),world=composed(parentWorld,self);
        const auto index=capture.reserve(source,parent,depth,11,world);
        auto result=container(object,source,11,world);QJsonArray members;
        result["geometry_data"]=QJsonObject{{"kind",selection.empty()?"group_definition":"group_reference"},{"schema_version",1},{"members",members}};
        result["geometry_data_status"]="copied_validated_group_definition";
        capture.account(result,selection.empty()?group(object)->getObjects().size():1);
        if(!selection.empty()) {
            const auto selected=reference(capture,object,selection,depth);Base::Matrix4D nativePath;
            if(object.getSubObject(selected.prefix.c_str(),nullptr,&nativePath,true)!=selected.object)throw std::runtime_error("Native group member identity does not match preflight");
            const auto childWorld=composed(world,identity,self,nativeMatrix(nativePath));
            const auto childParent=composed(childWorld,identity,selfPlacement(*selected.object),identity);
            members.append(captureAt(capture,*selected.object,selected.selector,childParent,index,depth+1));
        } else {
            const auto& children=group(object)->getObjects();
            if(om9_instance_graph_counts(capture.nodes.size()+children.size(),depth))throw std::runtime_error("Group member count budget exceeded");
            for(auto* child:children){if(!child)throw std::runtime_error("Missing native group member");capture.remember(*child);
                Base::Matrix4D nativePath;const std::string sub=std::string(child->getNameInDocument())+'.';
                if(object.getSubObject(sub.c_str(),nullptr,&nativePath,true)!=child)throw std::runtime_error("Native group member identity does not match stored membership");
                const auto childWorld=composed(world,identity,self,nativeMatrix(nativePath));
                members.append(captureAt(capture,*child,"",composed(childWorld,identity,selfPlacement(*child),identity),index,depth+1));
            }
        }
        result["geometry_data"]=QJsonObject{{"kind",selection.empty()?"group_definition":"group_reference"},{"schema_version",1},{"members",members}};
        result["geometry_data_status"]="copied_validated_group_definition";return result;
    }
    const auto derived=[&](const char* name){const auto type=Base::Type::fromName(name);return !type.isBad()&&object.getTypeId().isDerivedFrom(type);};
    if(!dynamic_cast<Part::Feature*>(&object)&&!derived("Mesh::Feature")&&!derived("Points::Feature"))throw std::runtime_error("Unsupported native definition leaf representation");
    const auto world=composed(parentWorld,selfPlacement(object));
    const auto index=capture.reserve(source,parent,depth,1,world);
    auto result=capture.readers.leaf(*object.getDocument(),object.getNameInDocument(),selection.c_str());
    capture.readers.reframe(result,object,world);
    const auto rep=result["representation"].toString();
    const std::vector<QString> types{"unknown","point","curve","polycurve","trimmed_face","shell","solid","brep","mesh","point_cloud"};
    const auto at=std::find(types.begin(),types.end(),rep);
    if(at==types.end()||at==types.begin())throw std::runtime_error("Unsupported empty/native definition geometry");
    capture.nodes[index].representation=static_cast<std::uint32_t>(at-types.begin());
    // Leaf graph frame uses the same support/object coordinates as its payload.
    const auto rows=result["local_to_world"].toArray();Matrix leafWorld{};
    for(int r=0;r<4;++r){const auto row=rows[r].toArray();for(int c=0;c<4;++c)leafWorld[r*4+c]=row[c].toDouble();}
    capture.nodes[index].world=leafWorld;capture.account(result);return result;
}
}
QJsonObject definitionSnapshot(App::DocumentObject& object,const ObjectSnapshot& source,DefinitionReaders readers) {
    Capture capture{readers};
    auto* parent=App::GeoFeatureGroupExtension::getGroupOfObject(&object);
    const auto parentWorld=parent?nativeMatrix(parent->getExtensionByType<App::GeoFeatureGroupExtension>()->globalGroupPlacement().toMatrix()):identity;
    auto result=captureAt(capture,object,source.subelement,parentWorld,rootIndex,1);
    auto data=result["geometry_data"].toObject();
    const bool childGroup=data.value("definition_geometry").toObject().value("representation")=="group";
    const auto graph=capture.graph(result["representation"]=="group"||childGroup?2:1);
    result["definition_graph"]=graph;
    if(result["representation"]=="group")data["definition_graph"]=graph;
    else if(childGroup){
        auto child=data["definition_geometry"].toObject();auto childData=child["geometry_data"].toObject();childData["definition_graph"]=graph;child["geometry_data"]=childData;data["definition_geometry"]=child;
    }
    result["geometry_data"]=data;
    const auto raw=QJsonDocument(result).toJson(QJsonDocument::Compact);
    if(raw.size()<0||static_cast<std::size_t>(raw.size())>capture.payload||om9_support_payload(0,static_cast<std::size_t>(raw.size())))throw std::runtime_error("Complete definition snapshot exceeds serialized budget (actual "+std::to_string(raw.size())+", accounted "+std::to_string(capture.payload)+")");
    for(const auto& guard:capture.guards)verifyObject(*guard.doc,guard.snapshot);
    verifyObject(*object.getDocument(),source);return result;
}
}
