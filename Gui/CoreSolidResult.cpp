// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreSolidResult.h"
#include "CorePreparedSnapshot.h"
#include "CoreTopologySnapshot.h"
#include "CoreShapeValidation.h"
#include "CoreSourceAttributes.h"
#include "CoreDiscreteSnapshot.h"
#include "CoreGeometryContract.h"
#include "NativeFitPoints.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/GeoFeature.h>
#include <App/PropertyGeo.h>
#include <Gui/Selection/Selection.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <QCryptographicHash>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonParseError>
#include <stdexcept>
#include <cmath>
#include <cstdint>
namespace OpenMatrix9Gui {
namespace {
extern "C" std::size_t om9_prepared_result_limit();
extern "C" std::uint32_t om9_solid_source_budget_json(const std::uint8_t*,std::size_t);
const TopoDS_Shape& shape(PyObject* p){
    auto* wrapper=static_cast<Part::TopoShapePy*>(p);
    if(!p||!PyObject_TypeCheck(p,&Part::TopoShapePy::Type)||!wrapper->Base::PyObjectBase::isValid()||!wrapper->getTopoShapePtr())
        throw std::runtime_error("StaleGeometry: Solid native shape owner unavailable");
    return wrapper->getTopoShapePtr()->getShape();
}
QJsonObject stamp(const ObjectStamp& s){return {{"document",QString::number(s.document)},{"object",QString::number(s.object)},
    {"generation",QString::number(s.generation)},{"revision",QString::number(s.revision)},{"epoch",QString::number(s.epoch)}};}
QJsonArray matrix(const Base::Matrix4D& frame){QJsonArray result;for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)result.append(frame[r][c]);return result;}
QJsonObject reference(const SolidSourceEvidence& evidence){
    const auto& s=evidence.source;auto r=stamp(s.stamp);QJsonArray world,ancestors;
    for(double v:s.world)world.append(v);for(const auto& a:s.ancestors)ancestors.append(stamp(a));
    r["document_uid"]=QString::fromStdString(s.documentUid);r["object_name"]=QString::fromStdString(s.objectName);
    r["subelement"]=QString::fromStdString(s.subelement);r["unit_context_snapshot"]=QString::fromStdString(s.unitContext);
    r["world"]=world;r["ancestors"]=ancestors;r["role"]=QString::fromStdString(evidence.role);
    r["role_index"]=qint64(evidence.index);r["point_begin"]=qint64(evidence.begin);r["point_end"]=qint64(evidence.end);
    r["snapshot_frame"]=QString::fromStdString(evidence.frame);return r;
}
QJsonObject cad(App::Document& doc,PyObject* owner,const ObjectSnapshot& source){
    validateNativePartShape(owner,"Sphere",source.objectName);
    const auto& native=shape(owner);const auto topology=topologySnapshot(native);const auto validation=shapeValidation(native,topology);
    auto* object=doc.getObject(source.objectName.c_str());if(!object)throw std::runtime_error("StaleGeometry: Sphere input "+source.objectName+": source disappeared");
    const auto attributes=sourceAttributes(*object,source);
    const QJsonObject identity{{"document_name",doc.getName()},{"name",QString::fromStdString(source.objectName)}};
    return preparedCadSnapshot(native,topology,validation,attributes,source.unitContext,identity,"ConstructSolid");
}
void budget(std::size_t& bytes,const QJsonObject& value){
    const auto added=static_cast<std::size_t>(QJsonDocument(value).toJson(QJsonDocument::Compact).size());
    if(added>om9_prepared_result_limit()-bytes)throw std::runtime_error("InvalidBasis: Solid aggregate source snapshot byte budget");bytes+=added;
}
void appendOwnedSource(QJsonArray& refs,QJsonArray& snapshots,std::size_t& bytes,const SolidSourceEvidence& input){
    auto ref=reference(input);int slot=0;
    // The admission gate and completed result must use the same exact sharing.
    while(slot<snapshots.size()&&snapshots.at(slot).toObject()!=input.snapshot)++slot;
    ref["input_snapshot_index"]=slot+1;budget(bytes,ref);
    if(slot==snapshots.size()){budget(bytes,input.snapshot);snapshots.append(input.snapshot);}
    refs.append(ref);
}
QByteArray contentDigest(App::Document& doc,const SolidSourceEvidence& input){
    auto* object=doc.getObject(input.source.objectName.c_str());
    if(!object)throw std::runtime_error("source disappeared");
    if(object->getPropertyByName("Shape")){
        auto* property=object->getPropertyByName<App::PropertyComplexGeoData>("Shape");
        if(!property||property->getTypeId()!=Base::Type::fromName("Part::PropertyPartShape")||!property->getComplexData())
            throw std::runtime_error("native shape owner unavailable");
        const auto& native=static_cast<const Part::TopoShape*>(property->getComplexData())->getShape();
        // The shared master digest deliberately normalizes root Location. Bind
        // its numerical frame separately: native setTransform need not signal
        // a Placement/property revision change.
        QJsonArray root;const auto& transform=native.Location().Transformation();
        for(int row=1;row<=3;++row)for(int column=1;column<=4;++column){
            const auto value=transform.Value(row,column);
            if(!std::isfinite(value))throw std::runtime_error("non-finite native shape root frame");
            root.append(value);
        }
        return QCryptographicHash::hash(preparedCadNativeMasterDigest(native)+QJsonDocument(root).toJson(QJsonDocument::Compact),QCryptographicHash::Sha256);
    }
    // Full bounded native XYZ/faces/location, rather than selected points only.
    const auto captured=discreteSnapshot(*object,object->getPropertyByName("Mesh")!=nullptr);
    return QCryptographicHash::hash(QJsonDocument(captured).toJson(QJsonDocument::Compact),QCryptographicHash::Sha256);
}
}
void verifySolidFitSources(App::Document& doc,const std::vector<SolidSourceEvidence>& inputs){
    for(const auto& input:inputs)try{
        verifyObject(doc,input.source);
        if(contentDigest(doc,input)!=input.contentDigest)throw std::runtime_error("native source content changed");
        verifyObject(doc,input.source);
    }
    catch(const std::exception& e){throw std::runtime_error("StaleGeometry: Sphere input "+input.source.objectName+(input.source.subelement.empty()?"":"."+input.source.subelement)+": "+e.what());}
}
void validateSolidFitBudget(const std::vector<SolidSourceEvidence>& existing,const std::vector<SolidSourceEvidence>& incoming){
    try{
    std::size_t bytes=0;
    const auto account=[&](const auto& inputs){for(const auto& input:inputs){
        budget(bytes,reference(input));budget(bytes,input.snapshot);
        const auto retained=sizeof(SolidSourceEvidence)+input.points.size()*sizeof(SolidPoint)+static_cast<std::size_t>(input.contentDigest.size());
        if(retained>om9_prepared_result_limit()-bytes)throw std::runtime_error("InvalidBasis: Solid aggregate retained FitPoints byte budget");
        bytes+=retained;
    }};
    account(existing);account(incoming);
    QJsonArray refs,snapshots;std::size_t packetBytes=0;
    for(const auto& inputs:{&existing,&incoming})for(const auto& input:*inputs)appendOwnedSource(refs,snapshots,packetBytes,input);
    const auto raw=QJsonDocument(QJsonObject{{"source_refs",refs},{"source_snapshots",snapshots}}).toJson(QJsonDocument::Compact);
    const auto status=om9_solid_source_budget_json(reinterpret_cast<const std::uint8_t*>(raw.constData()),raw.size());
    if(status)throw std::runtime_error(std::string(status==7?"KernelFailure: ":"InvalidBasis: ")+"Solid aggregate source JSON node/depth/byte budget");
    }catch(const std::exception& error){
        std::string identity;const auto& inputs=incoming.empty()?existing:incoming;
        for(const auto& input:inputs){if(!identity.empty())identity+=", ";identity+=input.source.objectName;if(!input.source.subelement.empty())identity+="."+input.source.subelement;}
        throw std::runtime_error(std::string(error.what())+": Sphere input "+(identity.empty()?"selected FitPoints":identity));
    }
}
std::vector<SolidSourceEvidence> captureSolidFitSelection(App::Document& doc,std::size_t used){
    std::vector<SolidSourceEvidence> result;std::size_t count=used,bytes=0;
    for(const auto& selected:Gui::Selection().getSelection(doc.getName())){
        const std::string name=selected.FeatName,sub=selected.SubName?selected.SubName:"";
        SolidSourceEvidence input;input.source=captureObject(doc,name,sub);
        input.points=nativeFitPoints(doc,name,sub,false,count,"Sphere FitPoints",false,true);
        if(input.points.empty())continue;
        input.contentDigest=contentDigest(doc,input);
        auto* object=doc.getObject(name.c_str());if(!object)throw std::runtime_error("StaleGeometry: Sphere input "+name+": source disappeared");
        if(object->getPropertyByName("Shape")){
            auto native=nativeShapeInput(doc,name);verifyObject(doc,input.source);
            input.snapshot=cad(doc,native.shape->value,input.source);input.frame="document_world";
        }else{
            const bool mesh=object->getPropertyByName("Mesh")!=nullptr;
            input.snapshot=discreteSnapshot(*object,mesh);
            input.snapshot["schema_version"]=1;input.snapshot["ownership"]=mesh?"private_native_copy":"private_plain_copy";
            input.snapshot["unit_context_snapshot"]=QString::fromStdString(input.source.unitContext);
            input.snapshot["source_attributes"]=sourceAttributes(*object,input.source);input.snapshot["topology_data"]=QJsonObject();
            Base::Placement own;
            if(auto* property=object->getPropertyByName<App::PropertyPlacement>("Placement"))own=property->getValue();
            const auto parent=App::GeoFeature::getGlobalPlacement(object)*own.inverse();
            input.snapshot["native_parent_to_world"]=matrix(parent.toMatrix());input.frame="native_local_with_locations";
        }
        QJsonArray points;for(const auto& p:input.points)points.append(QJsonArray{p[0],p[1],p[2]});
        input.snapshot["selected_points_world"]=points;input.role="fit_points";input.begin=count;count+=input.points.size();input.end=count;
        verifyObject(doc,input.source);budget(bytes,input.snapshot);result.push_back(std::move(input));
    }
    verifySolidFitSources(doc,result);validateSolidFitBudget({},result);return result;
}
SolidEvidence captureSolidEvidence(App::Document& doc,const std::map<std::size_t,SolidCurveReference>& tangent,
    const std::optional<SolidCurveReference>& path,const std::vector<SolidSourceEvidence>& fit,
    const std::string& construction,const std::string& units){
    verifySolidReferences(doc,tangent,path);verifySolidFitSources(doc,fit);
    QJsonParseError error;const auto parsed=QJsonDocument::fromJson(QByteArray::fromStdString(construction),&error);
    if(error.error!=QJsonParseError::NoError||!parsed.isObject())throw std::runtime_error("InvalidBasis: Solid owned construction recipe is invalid");
    SolidEvidence evidence;evidence.units=units;QJsonArray refs,snapshots;std::size_t bytes=0;
    auto append=[&](const SolidSourceEvidence& input){
        verifyObject(doc,input.source);appendOwnedSource(refs,snapshots,bytes,input);evidence.sources.push_back(input.source);
    };
    auto curve=[&](const SolidCurveReference& native,const char* role,std::size_t index){
        SolidSourceEvidence input;input.source=native.input.guard;input.source.subelement=native.sub;
        input.role=role;input.index=index;input.frame="document_world";input.snapshot=cad(doc,native.edge->value,input.source);append(input);
    };
    if(path)curve(*path,"path",0);for(const auto& [index,input]:tangent)curve(input,"tangent",index);
    for(std::size_t index=0;index<fit.size();++index){auto input=fit[index];input.index=index;append(input);}
    const auto recipe=parsed.object();evidence.context={{"schema_version",1},{"operation","ConstructSolid"},
        {"feature_id",recipe.value("feature_id")},{"construction",recipe},{"source_refs",refs},{"source_snapshots",snapshots},
        {"source_snapshot_indexing","input_snapshots_zero_based_including_numeric_input"},
        {"attribute_role","source_evidence_in_input_snapshots_output_attributes_unavailable"},
        {"output_parent_to_world",QJsonArray{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1}}};
    budget(bytes,recipe);for(const auto& s:evidence.sources)verifyObject(doc,s);return evidence;
}
std::string solidGeometryResult(App::Document& doc,PyObject* output,const SolidEvidence& evidence,const std::string& name){
    for(const auto& source:evidence.sources)verifyObject(doc,source);
    validateNativePartShape(output,"ConstructSolid",name);const auto& native=shape(output);
    const auto topology=topologySnapshot(native);const auto validation=shapeValidation(native,topology);
    const QJsonObject attributes{{"schema_version",1},{"availability","not_applicable_constructed_output"}};
    const QJsonObject identity{{"document_name",doc.getName()},{"name",QString::fromStdString(name)}};
    const auto result=preparedCadResult(native,topology,validation,attributes,identity,evidence.units,evidence.context);
    for(const auto& source:evidence.sources)verifyObject(doc,source);
    return QJsonDocument(result).toJson(QJsonDocument::Compact).toStdString();
}
}
