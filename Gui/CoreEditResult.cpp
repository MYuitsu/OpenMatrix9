// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreEditResult.h"
#include "CorePreparedSnapshot.h"
#include "CorePreparedCad.h"
#include "CoreGeometryContract.h"
#include "CoreTopologySnapshot.h"
#include "CoreShapeValidation.h"
#include "CoreSourceAttributes.h"
#include <App/Document.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <BRepBuilderAPI_Copy.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <stdexcept>
namespace OpenMatrix9Gui {
namespace {
extern "C" std::size_t om9_prepared_result_limit();
const TopoDS_Shape& shape(PyObject* p){
    if(!p||!PyObject_TypeCheck(p,&Part::TopoShapePy::Type))throw std::runtime_error("UnsupportedRepresentation: Edit result requires native CAD ownership");
    auto* wrapper=static_cast<Part::TopoShapePy*>(p);
    if(!wrapper->Base::PyObjectBase::isValid())throw std::runtime_error("StaleGeometry: Edit result native owner expired");
    return wrapper->getTopoShapePtr()->getShape();
}
QJsonObject stamp(const ObjectStamp& s){return {{"document",QString::number(s.document)},{"object",QString::number(s.object)},
    {"generation",QString::number(s.generation)},{"revision",QString::number(s.revision)},{"epoch",QString::number(s.epoch)}};}
QJsonObject reference(const ObjectSnapshot& s){auto r=stamp(s.stamp);QJsonArray world,ancestors;
    for(double v:s.world)world.append(v);for(const auto& a:s.ancestors)ancestors.append(stamp(a));
    r["document_uid"]=QString::fromStdString(s.documentUid);r["object_name"]=QString::fromStdString(s.objectName);
    r["subelement"]=QString::fromStdString(s.subelement);r["unit_context_snapshot"]=QString::fromStdString(s.unitContext);
    r["world"]=world;r["ancestors"]=ancestors;return r;}
}
EditEvidence captureEditEvidence(App::Document& doc,const std::vector<EditInput>& inputs,unsigned kind,unsigned flags,
    unsigned mode,std::size_t first,const std::vector<std::string>& deleting,double tolerance,
    const ToleranceSnapshot* toleranceContext,const EditProjection* projection){
    if(kind<1||kind>7||inputs.empty())throw std::runtime_error("InvalidBasis: Edit result command or sources missing");
    static const char* ids[]={"","OM9-TOP11-008","OM9-TOP11-005","OM9-TOP11-010","OM9-SOLID-001","OM9-SOLID-002","OM9-SOLID-003","OM9-SOLID-004"};
    const char* operation=kind>=4?"Boolean":"NativeEdit";
    EditEvidence evidence;QJsonArray refs,snapshots,deleted;std::size_t bytes=0;
    for(const auto& input:inputs){verifyObject(doc,input.guard);auto source=captureObject(doc,input.name);
        if(evidence.sources.empty())evidence.units=source.unitContext;
        validateNativePartShape(input.shape->value,operation,input.name);const auto& native=shape(input.shape->value);
        const auto topology=topologySnapshot(native);const auto validation=shapeValidation(native,topology);
        auto* object=doc.getObject(input.name.c_str());if(!object)throw std::runtime_error("StaleGeometry: Edit source disappeared");
        const auto attributes=sourceAttributes(*object,source);
        const QJsonObject identity{{"document_name",doc.getName()},{"name",QString::fromStdString(input.name)}};
        const auto snapshot=preparedCadSnapshot(native,topology,validation,attributes,source.unitContext,identity,operation);
        const auto ref=reference(source);const auto size=static_cast<std::size_t>(QJsonDocument(snapshot).toJson(QJsonDocument::Compact).size())
            +static_cast<std::size_t>(QJsonDocument(ref).toJson(QJsonDocument::Compact).size());
        if(size>om9_prepared_result_limit()-bytes)throw std::runtime_error("InvalidBasis: Edit aggregate source snapshot byte budget");
        bytes+=size;refs.append(ref);snapshots.append(snapshot);evidence.sources.push_back(std::move(source));
    }
    for(const auto& name:deleting)deleted.append(QString::fromStdString(name));
    QJsonValue projected(QJsonValue::Null);
    if(kind==3&&(flags&2)&&projection){QJsonArray frame;
        for(const auto& row:{projection->eye,projection->right,projection->up,projection->forward})for(double value:row)frame.append(value);
        projected=QJsonObject{{"frame",frame},{"perspective",projection->perspective}};}
    evidence.context={{"schema_version",1},{"operation",operation},{"feature_id",ids[kind]},
        {"kind",int(kind)},{"flags",int(flags)},{"mode",int(kind==7?mode:0)},
        {"first_count",qint64(kind==7?1:first)},{"deleted_source_names",deleted},{"tolerance_mm",tolerance},
        {"tolerance_report",toleranceContext?QJsonValue(QString::fromStdString(toleranceContext->report)):QJsonValue(QJsonValue::Null)},
        {"projection",projected},{"source_refs",refs},{"source_snapshots",snapshots},{"source_snapshot_frame","document_world"},
        {"attribute_role","source_evidence_in_input_snapshots_output_attributes_unavailable"}};
    for(const auto& s:evidence.sources)verifyObject(doc,s);return evidence;
}
PreparedEditOutput prepareEditOutput(App::Document& doc,PyObject* output,const EditEvidence& evidence,const std::string& name,
    const Base::Placement& parent,std::size_t index,std::size_t count,std::size_t owner){
    for(const auto& s:evidence.sources)verifyObject(doc,s);
    const auto operation=evidence.context.value("operation").toString().toUtf8();validateNativePartShape(output,operation.constData(),name);
    BRepBuilderAPI_Copy copy(shape(output),true,false);
    if(!copy.IsDone()||copy.Shape().IsNull())throw std::runtime_error("KernelFailure: Edit private output copy failed");
    PreparedEditOutput prepared;prepared.shape=Part::TopoShape(copy.Shape());prepared.name=name;prepared.parent=parent;
    // New outputs have identity own Placement. Bake the rigid parent inverse
    // together with the existing root location, preserving analytic geometry.
    const auto embedded=prepared.shape.getTransform();prepared.shape.setTransform(Base::Matrix4D());
    prepared.shape.transformShape(parent.inverse().toMatrix()*embedded,true);
    prepared.rootFrame=prepared.shape.getTransform();
    const auto topology=topologySnapshot(prepared.shape.getShape());const auto validation=shapeValidation(prepared.shape.getShape(),topology);
    auto context=evidence.context;QJsonArray matrix;const auto frame=parent.toMatrix();
    for(unsigned row=0;row<4;++row)for(unsigned col=0;col<4;++col)matrix.append(frame[row][col]);
    context["output_parent_to_world"]=matrix;context["output_index"]=qint64(index);context["output_count"]=qint64(count);
    context["output_owner_index"]=context.value("kind").toInt()==2?QJsonValue(qint64(owner)):QJsonValue(QJsonValue::Null);
    const QJsonObject attributes{{"schema_version",1},{"availability","not_applicable_constructed_output"}};
    const QJsonObject identity{{"document_name",doc.getName()},{"name",QString::fromStdString(name)}};
    prepared.result=QJsonDocument(preparedCadResult(prepared.shape.getShape(),topology,validation,attributes,identity,evidence.units,context))
        .toJson(QJsonDocument::Compact).toStdString();
    // Native validation may update BRep checked flags. Freeze the accepted
    // master after the same observations used by the serialized result.
    prepared.masterDigest=preparedCadNativeMasterDigest(prepared.shape.getShape());
    for(const auto& s:evidence.sources)verifyObject(doc,s);return prepared;
}
}
