// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreCurveNativeInput.h"
#include "CorePreparedSnapshot.h"
#include "CoreTopologySnapshot.h"
#include "CoreShapeValidation.h"
#include "CoreSourceAttributes.h"
#include "CoreDiscreteSnapshot.h"
#include "CoreGeometryContract.h"
#include "CoreUnits.h"
#include "RustBridge.h"
#include <App/Document.h>
#include <App/GeoFeature.h>
#include <App/PropertyGeo.h>
#include <Base/Exception.h>
#include <Mod/Part/App/TopoShape.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <QApplication>
#include <QThread>
#include <QCryptographicHash>
#include <QJsonArray>
#include <QJsonDocument>
#include <Standard_Failure.hxx>
#include <stdexcept>
namespace OpenMatrix9Gui {
namespace {
extern "C" std::size_t om9_prepared_result_limit();
void thread(){if(!qApp||QThread::currentThread()!=qApp->thread()||!PyGILState_Check())throw std::runtime_error("KernelFailure: Curve native capture requires GUI thread/GIL");}
QJsonArray matrix(const Base::Matrix4D& m){QJsonArray a;for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)a.append(m[r][c]);return a;}
QJsonArray points(const std::vector<SolidPoint>& values){QJsonArray a;for(const auto& p:values)a.append(QJsonArray{p[0],p[1],p[2]});return a;}
QJsonArray point(const SolidPoint& p){return {p[0],p[1],p[2]};}
QJsonObject stamp(const ObjectStamp& s){return {{"document",QString::number(s.document)},{"object",QString::number(s.object)},{"generation",QString::number(s.generation)},{"revision",QString::number(s.revision)},{"epoch",QString::number(s.epoch)}};}
QByteArray digest(App::Document& doc,const ObjectSnapshot& guard){
    auto* object=doc.getObject(guard.objectName.c_str());if(!object)throw std::runtime_error("source disappeared");
    QByteArray geometry;
    if(auto* property=object->getPropertyByName<App::PropertyComplexGeoData>("Shape")){
        if(property->getTypeId()!=Base::Type::fromName("Part::PropertyPartShape")||!property->getComplexData())throw std::runtime_error("native shape owner unavailable");
        const auto& native=static_cast<const Part::TopoShape*>(property->getComplexData())->getShape();
        QJsonArray root;const auto& tr=native.Location().Transformation();for(int r=1;r<=3;++r)for(int c=1;c<=4;++c)root.append(tr.Value(r,c));
        geometry=preparedCadNativeMasterDigest(native)+QJsonDocument(QJsonObject{{"location",root},{"orientation",static_cast<int>(native.Orientation())}}).toJson(QJsonDocument::Compact);
    }else geometry=QJsonDocument(discreteSnapshot(*object,object->getPropertyByName("Mesh")!=nullptr)).toJson(QJsonDocument::Compact);
    return QCryptographicHash::hash(geometry+QJsonDocument(sourceAttributes(*object,guard)).toJson(QJsonDocument::Compact),QCryptographicHash::Sha256);
}
QJsonObject reference(const CurveNativeBinding& binding){
    const auto& s=binding.source->guard;auto r=stamp(s.stamp);QJsonArray world,ancestors;for(double v:s.world)world.append(v);for(const auto& a:s.ancestors)ancestors.append(stamp(a));
    r["document_uid"]=QString::fromStdString(s.documentUid);r["object_name"]=QString::fromStdString(s.objectName);r["subelement"]=QString::fromStdString(s.subelement);
    r["unit_context_snapshot"]=QString::fromStdString(s.unitContext);r["world"]=world;r["ancestors"]=ancestors;r["snapshot_frame"]=QString::fromStdString(binding.source->frame);
    r["role"]=QString::fromStdString(binding.role);r["role_index"]=qint64(binding.index);r["point_begin"]=qint64(binding.begin);r["point_end"]=qint64(binding.end);return r;
}
}
CurveNativeAdmission curveNativeAdmission(App::Document& doc,const char* command){
    thread();return {om9_curve_generation(),om9_curve_input_revision(),om9_circle_input_context_key(),command,unitContextSnapshot(doc)};
}
QJsonObject curveNativeSourceReference(const CurveNativeBinding& binding){return reference(binding);}
void verifyCurveNativeSource(App::Document& doc,const CurveNativeSource& input,const std::string& command){
    thread();try{verifyObject(doc,input.guard);if(digest(doc,input.guard)!=input.contentDigest)throw std::runtime_error("native geometry/frame/attributes changed");verifyObject(doc,input.guard);}
    catch(const std::exception& e){throw std::runtime_error("StaleGeometry: "+command+" input "+input.guard.objectName+(input.guard.subelement.empty()?"":"."+input.guard.subelement)+": "+e.what());}
}
std::shared_ptr<const CurveNativeSource> captureCurveNativeSource(App::Document& doc,const std::string& name,const std::string& sub,const std::vector<SolidPoint>& selected,const std::string& command){
    const auto context=command+" input "+name+(sub.empty()?"":"."+sub)+": ";
    const char* failure="StaleGeometry";
    try {
    thread();CurveNativeSource input;input.guard=captureObject(doc,name,sub);
    failure="InvalidBasis";input.contentDigest=digest(doc,input.guard);
    auto* object=doc.getObject(name.c_str());if(!object)throw std::runtime_error("StaleGeometry: "+command+" input "+name+": source disappeared");
    if(object->getPropertyByName("Shape")){
        // Full parent in document world, independently of the picked Edge/Vertex.
        auto parent=nativeShapeInput(doc,name);validateNativePartShape(parent.shape->value,command,name);
        const auto& native=static_cast<Part::TopoShapePy*>(parent.shape->value)->getTopoShapePtr()->getShape();
        failure="InvalidTrimTopology";
        const auto topology=topologySnapshot(native),validation=shapeValidation(native,topology);
        failure="InvalidBasis";const auto attributes=sourceAttributes(*object,input.guard);
        input.snapshot=preparedCadSnapshot(native,topology,validation,attributes,input.guard.unitContext,
            {{"document_name",doc.getName()},{"name",QString::fromStdString(name)}},"AcceptCurveNativeInput");input.frame="document_world";
    }else{
        const bool mesh=object->getPropertyByName("Mesh")!=nullptr;input.snapshot=discreteSnapshot(*object,mesh);
        input.snapshot["schema_version"]=1;input.snapshot["ownership"]=mesh?"private_native_copy":"private_plain_copy";
        input.snapshot["source_attributes"]=sourceAttributes(*object,input.guard);input.snapshot["unit_context_snapshot"]=QString::fromStdString(input.guard.unitContext);input.snapshot["topology_data"]=QJsonObject();
        auto data=input.snapshot["geometry_data"].toObject();data["representation"]=data["kind"];input.snapshot["geometry_data"]=data;
        Base::Placement own;if(auto* property=object->getPropertyByName<App::PropertyPlacement>("Placement"))own=property->getValue();
        input.snapshot["native_parent_to_world"]=matrix((App::GeoFeature::getGlobalPlacement(object)*own.inverse()).toMatrix());input.frame="native_local_with_locations";
    }
    input.snapshot["selected_points_world"]=points(selected);
    if(static_cast<std::size_t>(QJsonDocument(input.snapshot).toJson(QJsonDocument::Compact).size())>om9_prepared_result_limit())throw std::runtime_error("InvalidBasis: "+command+" input "+name+": source byte budget exceeded");
    verifyCurveNativeSource(doc,input,command);return std::make_shared<const CurveNativeSource>(std::move(input));
    }catch(const Standard_Failure& error){
        throw std::runtime_error("KernelFailure: "+context+(error.GetMessageString()?error.GetMessageString():"native capture failed"));
    }catch(const Base::Exception& error){
        throw std::runtime_error("KernelFailure: "+context+error.what());
    }catch(const std::exception& error){
        std::string detail=error.what();
        // Native helpers predate typed command diagnostics. Preserve a helper's
        // explicit type; otherwise use its capture stage, with budgets treated
        // as input admission failures rather than topology or kernel failures.
        for(const char* type:{"UnsupportedOperation","UnsupportedRepresentation","InvalidBasis","InvalidTrimTopology","NonFiniteGeometry","StaleGeometry","SingularTransform","KernelFailure"}){
            const std::string prefix=std::string(type)+": ";
            if(detail.compare(0,prefix.size(),prefix)==0)throw std::runtime_error(prefix+context+detail.substr(prefix.size()));
        }
        if(detail.find("budget")!=std::string::npos)failure="InvalidBasis";
        throw std::runtime_error(std::string(failure)+": "+context+detail);
    }
}
unsigned acceptCurveNativeInput(App::Document& doc,const CurveNativeAdmission& admission,const char* action,unsigned kind,
    const SolidPoint& p,const SolidPoint& normal,const std::vector<SolidPoint>& values,const QString& text,const std::vector<CurveNativeBinding>& bindings,std::optional<double> sourceCurveFraction){
    thread();QJsonArray refs,snapshots;
    for(const auto& binding:bindings){
        verifyCurveNativeSource(doc,*binding.source,admission.command);auto r=reference(binding);int slot=0;
        while(slot<snapshots.size()&&snapshots.at(slot).toObject()!=binding.source->snapshot)++slot;
        r["input_snapshot_index"]=slot+1;if(slot==snapshots.size())snapshots.append(binding.source->snapshot);refs.append(r);
    }
    QJsonObject request{{"schema_version",1},{"operation","AcceptCurveNativeInput"},{"command",QString::fromStdString(admission.command)},
        {"generation",QString::number(admission.generation)},{"admission_revision",QString::number(admission.revision)},{"context_before",QString::number(admission.context)},
        {"unit_context_snapshot",QString::fromStdString(admission.units)},{"action",action},{"reference_kind",int(kind)},
        {"point_world",point(p)},{"normal",point(normal)},{"points_world",points(values)},{"text",text.isNull()?QJsonValue():QJsonValue(text)},
        {"source_refs",refs},{"source_snapshots",snapshots}};
    if(sourceCurveFraction)request["source_curve_fraction"]=*sourceCurveFraction;
    const auto bytes=QJsonDocument(request).toJson(QJsonDocument::Compact);
    if(static_cast<std::size_t>(bytes.size())>om9_prepared_result_limit())throw std::runtime_error("InvalidBasis: "+admission.command+" native input byte budget exceeded");
    for(const auto& binding:bindings)verifyCurveNativeSource(doc,*binding.source,admission.command);
    return om9_curve_native_input_json(reinterpret_cast<const unsigned char*>(bytes.constData()),bytes.size());
}
unsigned acceptCircleNativeSolution(App::Document& doc,const CurveNativeAdmission& admission,const std::array<double,7>& plan,const QJsonArray& contacts){
    thread();if(admission.command!="Circle"||unitContextSnapshot(doc)!=admission.units)throw std::runtime_error("StaleGeometry: Circle native solve context changed");
    const QJsonObject request{{"schema_version",1},{"operation","AcceptCurveNativeInput"},{"command","Circle"},
        {"generation",QString::number(admission.generation)},{"admission_revision",QString::number(admission.revision)},{"context_before",QString::number(admission.context)},
        {"unit_context_snapshot",QString::fromStdString(admission.units)},{"action","tangent_solution"},{"reference_kind",0},
        {"point_world",QJsonArray{plan[0],plan[1],plan[2]}},{"normal",QJsonArray{plan[3],plan[4],plan[5]}},{"points_world",QJsonArray()},
        {"text",QJsonValue()},{"source_refs",QJsonArray()},{"source_snapshots",QJsonArray()},{"solution_radius",plan[6]},
        {"resolved_contact_fractions",contacts},{"resolved_contact_indexing","current_constraint_points_zero_based"}};
    const auto bytes=QJsonDocument(request).toJson(QJsonDocument::Compact);
    return om9_curve_native_input_json(reinterpret_cast<const unsigned char*>(bytes.constData()),bytes.size());
}
}
