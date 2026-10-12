// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreSurfaceResult.h"
#include "SurfaceHistory.h"
#include "NativeSurfaceHistoryRequest.h"
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
#include <memory>
#include <stdexcept>
namespace OpenMatrix9Gui {
namespace {
extern "C" std::size_t om9_prepared_result_limit();
void decref(PyObject* p){Py_XDECREF(p);}
using Owner=std::unique_ptr<PyObject,decltype(&decref)>;
Owner own(PyObject* p){if(!p){PyErr_Clear();throw std::runtime_error("KernelFailure: Surface result native capture failed");}return Owner(p,decref);}
const TopoDS_Shape& shape(PyObject* p){
    if(!p||!PyObject_TypeCheck(p,&Part::TopoShapePy::Type))throw std::runtime_error("UnsupportedRepresentation: Surface result requires a native Part shape");
    auto* wrapper=static_cast<Part::TopoShapePy*>(p);
    if(!wrapper->Base::PyObjectBase::isValid())throw std::runtime_error("StaleGeometry: Surface result shape owner expired");
    return wrapper->getTopoShapePtr()->getShape();
}
QJsonObject stamp(const ObjectStamp& s){return {{"document",QString::number(s.document)},{"object",QString::number(s.object)},
    {"generation",QString::number(s.generation)},{"revision",QString::number(s.revision)},{"epoch",QString::number(s.epoch)}};}
QJsonObject reference(const ObjectSnapshot& s){
    auto r=stamp(s.stamp);QJsonArray world,ancestors;
    for(double v:s.world)world.append(v);for(const auto& a:s.ancestors)ancestors.append(stamp(a));
    r["document_uid"]=QString::fromStdString(s.documentUid);r["object_name"]=QString::fromStdString(s.objectName);
    r["subelement"]=QString::fromStdString(s.subelement);r["unit_context_snapshot"]=QString::fromStdString(s.unitContext);
    r["world"]=world;r["ancestors"]=ancestors;return r;
}
const char* feature(const SurfaceOptions& o){return o.kind==1?(o.historyCommand?"OM9-SURFACE-002":"OM9-SURFACE-001"):
    o.kind==2?(o.historyCommand?"OM9-SURFACE-004":"OM9-SURFACE-003"):"OM9-SURFACE-009";}
}
SurfaceEvidence captureSurfaceEvidence(App::Document& doc,const std::vector<SurfaceInput>& inputs,const SurfaceOptions& options){
    SurfaceEvidence evidence;evidence.recipe=serializeSurfaceHistoryOptions(inputs,options);evidence.operation=surfaceOperation(options.kind);
    std::size_t count=0;for(const auto& input:inputs){const auto n=input.chain.empty()?1:input.chain.size();
        if(n>1048576-count)throw std::runtime_error("InvalidBasis: Surface source reference budget");count+=n;}
    NativeSavedSurfaceRequest saved{};
    if(om9_surface_history_decode(reinterpret_cast<const unsigned char*>(evidence.recipe.data()),evidence.recipe.size(),count,&saved))
        throw std::runtime_error(std::string("InvalidBasis: ")+surfaceOperation(options.kind)+" result recipe rejected");
    evidence.inputs=inputs;QJsonArray refs,snapshots;std::size_t bytes=0;unsigned railClosed=0;
    const char* operation=surfaceOperation(options.kind);
    for(auto& input:evidence.inputs){
        for(const auto& guard:input.guards)verifyObject(doc,guard);
        const auto names=input.chain.empty()?std::vector<std::pair<std::string,std::string>>{{input.object,input.sub}}:input.chain;
        for(const auto& [name,sub]:names){
            auto source=captureObject(doc,name,sub);input.guards.push_back(source);
            if(evidence.sources.empty())evidence.units=source.unitContext;
            auto wire=own(surfaceWire(doc,SurfaceInput{name,sub},operation));const auto& native=shape(wire.get());
            const auto topology=topologySnapshot(native);const auto validation=shapeValidation(native,topology);
            auto* object=doc.getObject(name.c_str());if(!object)throw std::runtime_error("StaleGeometry: Surface source disappeared");
            const auto attributes=sourceAttributes(*object,source);
            const QJsonObject identity{{"document_name",doc.getName()},{"name",QString::fromStdString(name)}};
            const auto snapshot=preparedCadSnapshot(native,topology,validation,attributes,source.unitContext,identity,"EditSurface");
            const auto ref=reference(source);
            const auto size=static_cast<std::size_t>(QJsonDocument(snapshot).toJson(QJsonDocument::Compact).size())
                +static_cast<std::size_t>(QJsonDocument(ref).toJson(QJsonDocument::Compact).size());
            if(size>om9_prepared_result_limit()-bytes)throw std::runtime_error("InvalidBasis: Surface aggregate snapshot byte budget");
            bytes+=size;refs.append(ref);snapshots.append(snapshot);evidence.sources.push_back(std::move(source));
        }
    }
    const unsigned rails=options.kind==1?1:options.kind==2?2:0;
    for(unsigned i=0;i<rails&&i<evidence.inputs.size();++i){
        auto wire=own(surfaceWire(doc,evidence.inputs[i],operation));auto closed=own(PyObject_CallMethod(wire.get(),"isClosed",nullptr));
        const int value=PyObject_IsTrue(closed.get());if(value<0){PyErr_Clear();throw std::runtime_error("KernelFailure: Surface rail closure read failed");}
        if(value)railClosed|=1U<<i;
    }
    evidence.context={{"schema_version",1},{"operation","EditSurface"},{"feature_id",feature(options)},
        {"recipe",QString::fromStdString(evidence.recipe)},{"rail_closed",int(railClosed)},
        {"source_refs",refs},{"source_snapshots",snapshots},{"source_snapshot_frame","document_world"},
        {"attribute_role","source_evidence_in_input_snapshots_output_attributes_unavailable"}};
    for(const auto& s:evidence.sources)verifyObject(doc,s);return evidence;
}
PreparedSurfaceOutput prepareSurfaceOutput(App::Document& doc,PyObject* output,const SurfaceEvidence& evidence,
    const std::string& outputName,const Base::Placement& parent,const Base::Placement& own){
    for(const auto& s:evidence.sources)verifyObject(doc,s);
    validateNativePartShape(output,"EditSurface",outputName);
    const auto& native=shape(output);BRepBuilderAPI_Copy copy(native,true,false);
    if(!copy.IsDone()||copy.Shape().IsNull())throw std::runtime_error("KernelFailure: Surface private output copy failed");
    PreparedSurfaceOutput prepared;prepared.shape=Part::TopoShape(copy.Shape());prepared.sources=evidence.sources;prepared.recipe=evidence.recipe;prepared.outputName=outputName;prepared.operation=evidence.operation;
    // The Surface solver returns document-world geometry. Preserve the embedded
    // root frame before baking the rigid global inverse with FreeCAD's existing
    // copy transform. Generalized transforms would convert analytic surfaces to
    // NURBS even for an identity frame. Part::Feature reinstates own Placement
    // during recompute; serialize the exact shape carrying that root location.
    const auto embedded=prepared.shape.getTransform();prepared.shape.setTransform(Base::Matrix4D());
    prepared.shape.transformShape((parent*own).inverse().toMatrix()*embedded,true);
    prepared.shape.setTransform(own.toMatrix());
    const auto topology=topologySnapshot(prepared.shape.getShape());const auto validation=shapeValidation(prepared.shape.getShape(),topology);
    auto context=evidence.context;QJsonArray matrix;const auto frame=parent.toMatrix();
    for(unsigned row=0;row<4;++row)for(unsigned col=0;col<4;++col)matrix.append(frame[row][col]);context["output_parent_to_world"]=matrix;
    const QJsonObject attributes{{"schema_version",1},{"availability","not_applicable_constructed_output"}};
    const QJsonObject identity{{"document_name",doc.getName()},{"name",QString::fromStdString(outputName)}};
    const auto result=preparedCadResult(prepared.shape.getShape(),topology,validation,attributes,identity,evidence.units,context);
    prepared.result=QJsonDocument(result).toJson(QJsonDocument::Compact).toStdString();verifySurfaceOutput(doc,prepared);return prepared;
}
void verifySurfaceOutput(App::Document& doc,const PreparedSurfaceOutput& output){
    for(const auto& source:output.sources)try{verifyObject(doc,source);}
    catch(const std::exception& error){throw std::runtime_error("StaleGeometry: "+output.operation+" input "+source.objectName+(source.subelement.empty()?"":"."+source.subelement)+": "+error.what());}
}
}
