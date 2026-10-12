// SPDX-License-Identifier: LGPL-2.1-or-later
// OM9-CURVE-005 analytic AroundCurve dependency adapter; Rust validates the plan.
#include "CircleHistory.h"
#include "HistoryFeature.h"
#include "RustBridge.h"
#include "CircleTangent.h"
#include "CoreCurveNativeInput.h"
#include "CorePreparedSnapshot.h"
#include "CorePreparedCad.h"
#include "CoreTopologySnapshot.h"
#include "CoreShapeValidation.h"
#include "CoreUnits.h"
#include "CoreGeometryContract.h"
#include "CurveGeometry.h"
#include <App/Document.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Base/Exception.h>
#include <Base/Console.h>
#include <Base/Interpreter.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <QJsonDocument>
#include <QJsonArray>
#include <algorithm>
#include <set>
#include <stdexcept>
#include <cstdio>
extern "C" unsigned om9_circle_history_capability(const double*,std::size_t,unsigned,std::size_t);
extern "C" unsigned om9_fit_point_valid(unsigned,double,double,double);
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::CircleHistory,Part::Feature)
namespace {
void reportReplayDisposal(const App::Document::PropertyReplayTransaction& replay,bool restored)noexcept{
    if(replay.disposalVerified())return;
    try{
        if(restored)Base::Console().warning("Circle History restored; disposal diagnostic: {}\n",replay.disposalDiagnostic());
        else Base::Console().warning("Circle History accepted; disposal diagnostic: {}\n",replay.disposalDiagnostic());
    }catch(...){
        // Logging observers and formatting can throw. Acceptance/restoration
        // already completed; diagnostics must never request another rollback.
        std::fputs(restored?"Circle History restored; disposal diagnostic: ":"Circle History accepted; disposal diagnostic: ",stderr);
        std::fputs(replay.disposalDiagnostic(),stderr);std::fputc('\n',stderr);
    }
}
struct Guard {bool& flag;bool old;explicit Guard(bool& b):flag(b),old(b){flag=true;}~Guard(){flag=old;}};
bool guarded(const CircleHistory* o){return o->isRestoring()||o->isRemoving()||!o->getDocument()||o->getDocument()->testStatus(App::Document::Restoring)||o->getDocument()->isPerformingTransaction();}
void historyAdmission(const double* config,std::size_t size,std::size_t count,unsigned representation,const std::string& identity){
    const auto status=om9_circle_history_capability(config,size,representation,count);
    if(status==4)return;
    const char* code=status==101?"UnsupportedRepresentation":status==103?"KernelFailure":status==100?"UnsupportedOperation":"InvalidBasis";
    throw std::runtime_error(std::string(code)+": Circle History input "+identity+": stored request rejected ("+std::to_string(status)+")");
}
std::string historyError(const std::string& message,const char* name){
    for(const char* code:{"UnsupportedRepresentation","InvalidBasis","InvalidTrim","NonFiniteGeometry","StaleGeometry","UnsupportedTransform","KernelFailure","ConversionLoss"})
        if(message.starts_with(std::string(code)+":"))return std::string(code)+": Circle History input "+name+": "+message;
    return std::string("InvalidBasis: Circle History input ")+name+": "+message;
}
std::array<double,24> legacyAroundRecipe(double radius){
    // Existing legacy AroundCurve stores mm radius and a native path, without
    // a24-value recipe. Materialize only scalar options; do not rewrite storage.
    return {2,4,0,radius,0,0,0,0,0,1,0,0,0,1,0,0,0,1,0,3,8,0,-1,0};
}
std::string stringValue(const App::DocumentObject& object,const char* name){
    auto* value=object.getPropertyByName<App::PropertyString>(name);return value?value->getValue():"";
}
struct ReplayEvidence{
    std::vector<std::shared_ptr<const CurveNativeSource>> sources;
    QJsonArray refs,snapshots;
};
ReplayEvidence captureReplaySources(CircleHistory& child){
    ReplayEvidence evidence;auto& doc=*child.getDocument();
    auto append=[&](App::DocumentObject* object,const std::string& sub,const char* role,std::size_t index,unsigned representation,const std::vector<SolidPoint>& selected){
        if(!object||object==&child||object->getDocument()!=&doc)throw std::runtime_error("StaleGeometry: Circle History source missing or belongs to another document");
        auto source=captureCurveNativeSource(doc,object->getNameInDocument(),sub,selected,"Circle History");
        int slot=0;while(slot<evidence.snapshots.size()&&evidence.snapshots[slot].toObject()!=source->snapshot)++slot;
        auto ref=curveNativeSourceReference({source,role,index,0,0});ref["input_snapshot_index"]=slot+1;
        ref["selected_representation"]=int(representation);
        if(source->frame=="native_local_with_locations"){
            QJsonArray indices;for(std::size_t i=0;i<selected.size();++i)indices.append(qint64(i));ref["selected_point_indices"]=indices;
        }
        if(slot==evidence.snapshots.size())evidence.snapshots.append(source->snapshot);
        evidence.refs.append(ref);evidence.sources.push_back(std::move(source));
    };
    const auto& objects=child.Sources.getValues();const auto& subs=child.Sources.getSubValues();
    const auto& roles=child.SourceRoles.getValues();const auto& config=child.Recipe.getValues();
    if(objects.size()!=subs.size()||objects.size()>1024||roles.size()!=objects.size()*3)throw std::runtime_error("InvalidBasis: Circle History source metadata/budget");
    for(std::size_t i=0;i<objects.size();++i){
        if(!objects[i])throw std::runtime_error("StaleGeometry: Circle History source missing");
        unsigned representation=0;std::vector<SolidPoint> selected;
        auto admit=[&](unsigned value){historyAdmission(config.data(),config.size(),child.Points.getValues().size(),value,objects[i]->getNameInDocument());representation=value;};
        if(roles[i*3]==2){const auto identity=std::string(objects[i]->getNameInDocument())+"."+subs[i];solidCurveReference(doc,identity,"Circle History",false,admit);}
        else{
            selected=circleHistorySourcePoints(doc,objects[i]->getNameInDocument(),subs[i],0,"Circle History",false,admit);
            if(roles[i*3]==3){const auto count=roles[i*3+2];if(count<0||std::size_t(count)>selected.size())throw std::runtime_error("InvalidBasis: Circle History source prefix");selected.resize(std::size_t(count));}
        }
        append(objects[i],subs[i],"history_source",i,representation,selected);
    }
    if(auto* path=child.SourceCurve.getValue()){
        const auto& subs=child.SourceCurve.getSubValues();if(subs.size()!=1)throw std::runtime_error("InvalidBasis: Circle History path metadata");
        unsigned representation=0;const auto identity=std::string(path->getNameInDocument())+"."+subs[0];
        solidCurveReference(doc,identity,"Circle History",false,[&](unsigned value){representation=value;});
        append(path,subs[0],"history_path",objects.size(),representation,{});
    }
    return evidence;
}
void verifyReplaySources(App::Document& doc,const ReplayEvidence& evidence){
    for(const auto& source:evidence.sources)verifyCurveNativeSource(doc,*source,"Circle History");
}
// Native property identities and values stay on the GUI thread. Rust receives
// owned geometry; this barrier binds the values it validated to each setter.
struct ReplayState{
    CircleHistory& child;
    const std::string units,construction;
    const bool recorded;
    const std::vector<double> recipe,parameters;
    const std::vector<Base::Vector3d> points;
    const std::vector<long> roles;
    const std::vector<App::DocumentObject*> sources;
    const std::vector<std::string> subs,pathSubs;
    App::DocumentObject* const path;
    const double fraction;
    double radius,fit,approx;
    std::string result,unitProperty;
    QByteArray master;
    Base::Placement placement;
    explicit ReplayState(CircleHistory& c):child(c),units(unitContextSnapshot(*c.getDocument())),construction(stringValue(c,"OM9CurveConstruction")),
        recorded(c.Recorded.getValue()),recipe(c.Recipe.getValues()),parameters(c.SourceParameters.getValues()),points(c.Points.getValues()),roles(c.SourceRoles.getValues()),
        sources(c.Sources.getValues()),subs(c.Sources.getSubValues()),pathSubs(c.SourceCurve.getSubValues()),path(c.SourceCurve.getValue()),fraction(c.PathFraction.getValue()),
        radius(c.Radius.getValue()),fit(c.FitDeviation.getValue()),approx(c.ApproxDeviation.getValue()),result(stringValue(c,"OM9GeometryResult")),unitProperty(stringValue(c,"OM9UnitContext")),
        master(c.Shape.getShape().isNull()?QByteArray():preparedCadNativeMasterDigest(c.Shape.getShape().getShape())),placement(c.Placement.getValue()){}
    void verify()const{
        if(!child.getDocument()||unitContextSnapshot(*child.getDocument())!=units||stringValue(child,"OM9CurveConstruction")!=construction
            ||child.Recorded.getValue()!=recorded||child.Recipe.getValues()!=recipe||child.Points.getValues()!=points||child.SourceRoles.getValues()!=roles
            ||child.SourceParameters.getValues()!=parameters||child.Sources.getValues()!=sources||child.Sources.getSubValues()!=subs
            ||child.SourceCurve.getValue()!=path||child.SourceCurve.getSubValues()!=pathSubs||child.PathFraction.getValue()!=fraction
            ||child.Radius.getValue()!=radius||child.FitDeviation.getValue()!=fit||child.ApproxDeviation.getValue()!=approx
            ||stringValue(child,"OM9GeometryResult")!=result||stringValue(child,"OM9UnitContext")!=unitProperty||child.Placement.getValue()!=placement
            ||(child.Shape.getShape().isNull()?QByteArray():preparedCadNativeMasterDigest(child.Shape.getShape().getShape()))!=master)
            throw std::runtime_error("StaleGeometry: Circle History recipe, units or paired output changed during replay");
    }
};
std::string replayResult(CircleHistory& child,PyObject* shape,const std::vector<double>& config,
    const std::vector<SolidPoint>& points,const double* nativePlan,const QJsonArray& contacts,const double* output,const ReplayEvidence& evidence,bool legacy,const ReplayState& state){
    state.verify();
    auto& doc=*child.getDocument();verifyReplaySources(doc,evidence);validateNativePartShape(shape,"Circle History",child.getNameInDocument());
    const auto& native=static_cast<Part::TopoShapePy*>(shape)->getTopoShapePtr()->getShape();
    const auto topology=topologySnapshot(native),validation=shapeValidation(native,topology);
    QJsonArray recipe,xyz,nativeRecipe,recordedPoints,roles,parameters,nativeInput;
    for(double n:config)recipe.append(n);for(const auto& p:points)xyz.append(QJsonArray{p[0],p[1],p[2]});
    for(double n:child.Recipe.getValues())nativeRecipe.append(n);
    for(const auto& p:child.Points.getValues())recordedPoints.append(QJsonArray{p.x,p.y,p.z});
    for(long n:child.SourceRoles.getValues())roles.append(qint64(n));for(double n:child.SourceParameters.getValues())parameters.append(n);
    if(nativePlan)for(unsigned i=0;i<7;++i)nativeInput.append(nativePlan[i]);
    const QJsonObject plan{{"center",QJsonArray{output[0],output[1],output[2]}},{"normal",QJsonArray{output[3],output[4],output[5]}},
        {"radius",output[6]},{"fit_deviation",output[7]},{"approx_deviation",output[8]}};
    const QJsonObject context{{"schema_version",1},{"operation","ReplayCircleHistory"},{"feature_id","OM9-CURVE-005"},
        {"storage",legacy?"legacy_around_curve":"recipe_v2"},{"recipe",recipe},{"points_world",xyz},{"native_plan",nativePlan?QJsonValue(nativeInput):QJsonValue()},
        {"resolved_plan",plan},{"recorded_recipe",nativeRecipe},{"recorded_points_world",recordedPoints},{"recorded_source_roles",roles},{"recorded_source_parameters",parameters},
        {"resolved_contact_fractions",config[1]==6.?QJsonValue(contacts):QJsonValue()},
        {"resolved_contact_indexing",config[1]==6.?"current_constraint_points_zero_based":"not_applicable"},
        {"path_fraction",child.PathFraction.getValue()},{"original_construction",QString::fromStdString(stringValue(child,"OM9CurveConstruction"))},
        {"source_refs",evidence.refs},{"source_snapshots",evidence.snapshots},{"source_snapshot_indexing","input_snapshots_zero_based_including_numeric_input"},
        {"output_parent_to_world",QJsonArray{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1}},
        {"attribute_role","source_evidence_in_input_snapshots_output_attributes_unavailable"}};
    const QJsonObject attributes{{"schema_version",1},{"availability","not_applicable_constructed_output"}};
    const QJsonObject identity{{"document_name",doc.getName()},{"name",child.getNameInDocument()}};
    const auto result=preparedCadResult(native,topology,validation,attributes,identity,state.units,context);
    verifyReplaySources(doc,evidence);state.verify();return QJsonDocument(result).toJson(QJsonDocument::Compact).toStdString();
}
void setReplayResult(CircleHistory& child,PyObject* shape,const std::string& result,const ReplayEvidence& evidence,ReplayState& state){
    auto& doc=*child.getDocument();
    auto verify=[&]{state.verify();verifyReplaySources(doc,evidence);};verify();
    state.placement=Base::Placement();child.Placement.setValue(state.placement);verify();
    const auto& native=static_cast<Part::TopoShapePy*>(shape)->getTopoShapePtr()->getShape();state.master=preparedCadNativeMasterDigest(native);
    child.Shape.setPyObject(shape);verify();
    CurvePyRef object(child.getPyObject());state.result=result;persistCurveGeometryResult(object.value,result);verify();
    if(!PyObject_HasAttrString(object.value,"OM9UnitContext")){CurvePyRef property(PyObject_CallMethod(object.value,"addProperty","sss","App::PropertyString","OM9UnitContext","OpenMatrix9"));}
    state.unitProperty=state.units;
    if(auto* property=child.getPropertyByName<App::PropertyString>("OM9UnitContext"))property->setValue(state.units);
    verify();
}
}
CircleHistory::CircleHistory(){
    ADD_PROPERTY_TYPE(SourceCurve,(nullptr),"OpenMatrix9",App::Prop_ReadOnly,"Recorded native bounded edge");
    ADD_PROPERTY_TYPE(PlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"World placement dependencies");
    ADD_PROPERTY_TYPE(PathFraction,(0.),"OpenMatrix9",App::Prop_ReadOnly,"Normalized edge parameter, not arclength");
    ADD_PROPERTY_TYPE(Radius,(1.),"OpenMatrix9",App::Prop_ReadOnly,"Recorded analytic radius in mm");
    ADD_PROPERTY_TYPE(Recorded,(false),"OpenMatrix9",App::Prop_None,"Clear to detach and keep current geometry");
    ADD_PROPERTY_TYPE(Recipe,(std::vector<double>{}),"OpenMatrix9",App::Prop_ReadOnly,"Versioned owned Circle parameter recipe");
    ADD_PROPERTY_TYPE(Points,(std::vector<Base::Vector3d>{}),"OpenMatrix9",App::Prop_ReadOnly,"Recorded world point snapshots");
    ADD_PROPERTY_TYPE(Sources,(nullptr),"OpenMatrix9",App::Prop_ReadOnly,"Native point, FitPoints and tangent source links");
    ADD_PROPERTY_TYPE(SourceRoles,(std::vector<long>{}),"OpenMatrix9",App::Prop_ReadOnly,"Source role, point offset and original point count triples");
    ADD_PROPERTY_TYPE(SourceParameters,(std::vector<double>{}),"OpenMatrix9",App::Prop_ReadOnly,"Recorded normalized tangent pick parameters");
    ADD_PROPERTY_TYPE(FitDeviation,(0.),"OpenMatrix9",App::Prop_ReadOnly,"Recomputed maximum spatial fit deviation in mm");
    ADD_PROPERTY_TYPE(ApproxDeviation,(0.),"OpenMatrix9",App::Prop_ReadOnly,"Recomputed maximum deformable approximation deviation in mm");
}
CircleHistory::~CircleHistory(){disconnect();}
void CircleHistory::disconnect(){changed.disconnect();undone.disconnect();redone.disconnect();}
void CircleHistory::onSettingDocument(){
    Part::Feature::onSettingDocument();disconnect();
    if(auto* d=getDocument()){
    auto resume=[this](const App::Document&){if(Recorded.getValue()){refreshPlacements();touch();}};
    undone=d->signalUndo.connect(resume);redone=d->signalRedo.connect(resume);
    changed=d->signalChangedObject.connect([this](const App::DocumentObject& o,const App::Property& p){
        if(changing||guarded(this)||!Recorded.getValue()||&o==this)return;
        if(o.getPropertyByName("Group")==&p){refreshPlacements();touch();}
        const auto& parents=PlacementSources.getValues();
        if(std::find(parents.begin(),parents.end(),&o)!=parents.end()&&o.getPropertyByName("Placement")==&p)touch();
        const auto& inputs=Sources.getValues();
        if(SourceCurve.getValue()==&o||std::find(inputs.begin(),inputs.end(),&o)!=inputs.end()){touch();if(auto* source=dynamic_cast<const Part::Feature*>(&o);source&&&p==&source->Shape&&source->Shape.getShape().isNull()&&historyUpdatesEnabled(*getDocument())){clearShape();setError();}}
    });}
}
void CircleHistory::unsetupObject(){disconnect();Part::Feature::unsetupObject();}
void CircleHistory::refreshPlacements(){
    std::vector<App::DocumentObject*> parents;std::set<App::DocumentObject*> seen;
    auto inputs=Sources.getValues();if(SourceCurve.getValue())inputs.push_back(SourceCurve.getValue());
    for(auto* input:inputs)for(auto* p=input?App::GeoFeatureGroupExtension::getGroupOfObject(input):nullptr;p;p=App::GeoFeatureGroupExtension::getGroupOfObject(p)){if(!seen.insert(p).second)break;parents.push_back(p);}
    if(parents!=PlacementSources.getValues()){Guard g(changing);PlacementSources.setValues(parents);}
}
short CircleHistory::mustExecute()const {return Recorded.getValue()&&isTouched()?1:Part::Feature::mustExecute();}
void CircleHistory::clearShape(){Guard g(changing);Shape.setValue(Part::TopoShape());if(auto* result=getPropertyByName<App::PropertyString>("OM9GeometryResult"))result->setValue("");}
App::DocumentObjectExecReturn* CircleHistory::execute(){
    // The command publication already owns and checks the initial constructor
    // result. Do not replace its frozen metadata during that first recompute.
    if(initialResultPending){initialResultPending=false;return Part::Feature::execute();}
    if(!Recorded.getValue()||!historyUpdatesEnabled(*getDocument()))return Part::Feature::execute();
    Base::PyGILStateLocker replayLock;
    std::unique_ptr<App::Document::PropertyReplayTransaction> replay;
    auto rejected=[&](const std::string& message)->App::DocumentObjectExecReturn*{
        if(!replay)return new App::DocumentObjectExecReturn(historyError(message+"; native savepoint unavailable",getNameInDocument()),this);
        // A blocked observer can leave a Python callback error, which the
        // metadata bridge reports as KernelFailure. Preserve the earlier
        // native scope verdict instead of losing StaleGeometry in that wrapper.
        std::string cause=message;
        try{replay->verify();}catch(const Base::Exception& e){cause=e.what();}
        if(PyErr_Occurred())PyErr_Clear();
        Guard g(changing);
        if(!replay->rollback())return new App::DocumentObjectExecReturn(std::string("KernelFailure: Circle History checked rollback incomplete: ")+cause,this);
        try{
            clearShape();
            const auto* result=getPropertyByName<App::PropertyString>("OM9GeometryResult");
            if(!Shape.getShape().isNull()||(result&&std::string(result->getValue()).size())||!replay->completeErrorState())
                return new App::DocumentObjectExecReturn("KernelFailure: Circle History protected error cleanup incomplete",this);
            reportReplayDisposal(*replay,true);
        }catch(const Base::Exception& e){return new App::DocumentObjectExecReturn(std::string("KernelFailure: Circle History error cleanup: ")+e.what(),this);}
         catch(const std::exception& e){return new App::DocumentObjectExecReturn(std::string("KernelFailure: Circle History error cleanup: ")+e.what(),this);}
        return new App::DocumentObjectExecReturn(historyError(cause,getNameInDocument()),this);
    };
    try{
        replay=std::make_unique<App::Document::PropertyReplayTransaction>(*getDocument());
        ReplayState state(*this);
        if(!Recipe.getValues().empty()){
            refreshPlacements();Base::PyGILStateLocker lock;const auto& stored=Recipe.getValues();const auto& storedPoints=Points.getValues();
            historyAdmission(stored.data(),stored.size(),storedPoints.size(),1,std::string(getNameInDocument())+".Recipe");
            const auto evidence=captureReplaySources(*this);
            for(const auto& p:storedPoints){const auto status=om9_fit_point_valid(1,p.x,p.y,p.z);if(status)throw std::runtime_error(std::string(status==4?"NonFiniteGeometry":status==5?"KernelFailure":"InvalidBasis")+": Circle History input "+getNameInDocument()+".Points: invalid stored world point");}
            auto config=stored;std::vector<SolidPoint> points;points.reserve(storedPoints.size());for(const auto& p:storedPoints)points.push_back({p.x,p.y,p.z});
            const auto& objects=Sources.getValues();const auto& subs=Sources.getSubValues();const auto& roles=SourceRoles.getValues();const auto& parameters=SourceParameters.getValues();
            if(objects.size()!=subs.size()||roles.size()!=objects.size()*3||parameters.size()!=objects.size())throw std::runtime_error("Circle History source was deleted or metadata is corrupt");
            if(points.empty()||points.size()>1024)throw std::runtime_error("Circle History recorded point count is invalid");
            std::map<std::size_t,SolidCurveReference> tangentRefs;std::vector<std::pair<std::size_t,SolidPoint>> directions,radii;
            std::vector<double> constraints(points.size()*4);for(std::size_t i=0;i<points.size();++i)constraints[i*4+3]=1.;
            // Replace FitPoints spans in original offset order; whole-object
            // topology edits can change sample count without retaining stale points.
            struct Replacement{std::size_t index,count;std::vector<SolidPoint> points;};std::vector<Replacement> replacements;std::size_t replacedPointCount=0;
            for(std::size_t i=0;i<objects.size();++i){
                auto* source=objects[i];if(!source||source==this||source->getDocument()!=getDocument()||!source->isValid())throw std::runtime_error("Circle History source is missing or invalid");
                const auto role=roles[i*3],offset=roles[i*3+1],count=roles[i*3+2];if(offset<0||count<0||std::size_t(offset)>points.size()||std::size_t(count)>points.size()-std::size_t(offset))throw std::runtime_error("Circle History source span is invalid");
                if(role==2){
                    if(config[1]!=6.||count!=1)throw std::runtime_error("Invalid Circle History tangent source");const std::string identity=std::string(source->getNameInDocument())+"."+subs[i];auto ref=solidCurveReference(*getDocument(),identity,"Circle History",false,[&](unsigned representation){historyAdmission(config.data(),config.size(),points.size(),representation,identity);});
                    points[std::size_t(offset)]=solidOnCurve(ref,points[std::size_t(offset)],parameters[i]).first;constraints[std::size_t(offset)*4+3]=0.;tangentRefs.emplace(std::size_t(offset),std::move(ref));
                }else{
                    if(role!=-1&&role!=4&&role!=0&&role!=1&&role!=3)throw std::runtime_error("Unknown Circle History source role");
                    const std::string identity=std::string(source->getNameInDocument())+(subs[i].empty()?"":"."+subs[i]);
                    auto current=circleHistorySourcePoints(*getDocument(),source->getNameInDocument(),subs[i],0,"Circle History",false,[&](unsigned representation){historyAdmission(config.data(),config.size(),points.size(),representation,identity);});
                    if(role==-1){if(count!=0||current.size()!=1||offset!=0)throw std::runtime_error("Circle History direction source is invalid");directions.emplace_back(std::size_t(offset),current.front());}
                    else if(role==4){if(count!=0||current.size()!=1||(config[1]!=2.&&config[1]!=6.))throw std::runtime_error("Circle History radius endpoint source is invalid");radii.emplace_back(std::size_t(offset),current.front());}
                    else if(role==0){if(count!=1||current.size()!=1)throw std::runtime_error("Circle History vertex source is invalid");points[std::size_t(offset)]=current.front();}
                    else if(role==1||role==3){if(config[1]!=5.)throw std::runtime_error("Circle History fit source has wrong mode");if(role==3){if(current.size()<std::size_t(count))throw std::runtime_error("Circle History fit prefix source lost points");current.resize(std::size_t(count));}if(current.size()>1024-replacedPointCount)throw std::runtime_error("Circle History fit source points exceed 1024");replacedPointCount+=current.size();replacements.push_back({std::size_t(offset),std::size_t(count),std::move(current)});}
                    else throw std::runtime_error("Unknown Circle History source role");
                }
            }
            std::sort(replacements.begin(),replacements.end(),[](const auto& a,const auto& b){return a.index<b.index;});
            if(!replacements.empty()){
                std::vector<SolidPoint> rebuilt;std::size_t cursor=0;for(const auto& r:replacements){if(r.index<cursor)throw std::runtime_error("Overlapping Circle History fit source spans");if(r.index-cursor+r.points.size()>1024-rebuilt.size())throw std::runtime_error("Circle History fit point count exceeds 1024");rebuilt.insert(rebuilt.end(),points.begin()+cursor,points.begin()+r.index);rebuilt.insert(rebuilt.end(),r.points.begin(),r.points.end());cursor=r.index+r.count;}if(points.size()-cursor>1024-rebuilt.size())throw std::runtime_error("Circle History fit point count exceeds 1024");rebuilt.insert(rebuilt.end(),points.begin()+cursor,points.end());points=std::move(rebuilt);
            }
            for(const auto& [index,endpoint]:directions){if(index>=points.size())throw std::runtime_error("Circle History direction needs its center");config[5]=1.;for(unsigned j=0;j<3;++j)config[6+j]=endpoint[j]-points[index][j];}
            for(const auto& [index,endpoint]:radii){if(index>=points.size())throw std::runtime_error("Circle History radius endpoint needs its anchor");config[4]=om9_circle_history_radius(points[index].data(),endpoint.data());}
            historyAdmission(config.data(),config.size(),points.size(),1,getNameInDocument());
            double native[7];const double* nativePlan=nullptr;QJsonArray contacts;
            if(config[1]==4.){
                auto* source=SourceCurve.getValue();const auto& pathSubs=SourceCurve.getSubValues();if(!source||source==this||source->getDocument()!=getDocument()||!source->isValid()||pathSubs.size()!=1)throw std::runtime_error("Circle History path is missing or invalid");
                const std::string identity=std::string(source->getNameInDocument())+"."+pathSubs[0];auto ref=solidCurveReference(*getDocument(),identity,"Circle History",false,[&](unsigned representation){historyAdmission(config.data(),config.size(),points.size(),representation,identity);});const auto [p,n]=solidOnCurve(ref,{0,0,0},PathFraction.getValue());for(unsigned i=0;i<3;++i){native[i]=p[i];native[i+3]=n[i];}native[6]=1e-7;nativePlan=native;
            }else if(config[1]==6.){
                if(points.size()>3)throw std::runtime_error("Circle History tangent count exceeds three");for(std::size_t i=0;i<points.size();++i)for(unsigned j=0;j<3;++j)constraints[i*4+j]=points[i][j];
                double frame[13];for(unsigned i=0;i<3;++i)frame[i]=points.at(0)[i];for(unsigned i=0;i<9;++i)frame[i+3]=config[i+9];frame[12]=config[4];
                std::array<double,3> fractions{};
                const auto solved=circleSolveNative(tangentRefs,constraints.data(),points.size(),frame,int(config[22]),config[21]!=0.,config[23]!=0.,&fractions);std::copy(solved.begin(),solved.end(),native);nativePlan=native;
                for(std::size_t i=0;i<points.size();++i)contacts.append(tangentRefs.find(i)!=tangentRefs.end()?QJsonValue(fractions[i]):QJsonValue());
            }
            std::vector<double> flat;for(const auto& p:points)flat.insert(flat.end(),p.begin(),p.end());double output[4096];
            if(!om9_circle_history_replay(config.data(),flat.data(),points.size(),nativePlan,output))throw std::runtime_error("Circle History replay rejected invalid source geometry or parameters");
            auto shape=circleHistoryShape(output);const auto result=replayResult(*this,shape.value,config,points,nativePlan,contacts,output,evidence,false,state);
            {Guard g(changing);state.radius=output[6];Radius.setValue(output[6]);state.verify();verifyReplaySources(*getDocument(),evidence);
                state.fit=output[7];FitDeviation.setValue(output[7]);state.verify();verifyReplaySources(*getDocument(),evidence);
                state.approx=output[8];ApproxDeviation.setValue(output[8]);state.verify();verifyReplaySources(*getDocument(),evidence);
                setReplayResult(*this,shape.value,result,evidence,state);}
            replay->verify();auto* execution=Part::Feature::execute();
            if(!replay->commit())throw std::runtime_error("StaleGeometry: Circle History native savepoint commit rejected");
            reportReplayDisposal(*replay,false);
            return execution;
        }
        auto* source=SourceCurve.getValue();const auto& subs=SourceCurve.getSubValues();
        if(!source||source==this||source->getDocument()!=getDocument()||!source->isValid()||subs.size()!=1)throw std::runtime_error("Circle History source is missing or invalid");
        refreshPlacements();Base::PyGILStateLocker lock;
        const auto evidence=captureReplaySources(*this);
        const auto legacy=legacyAroundRecipe(Radius.getValue());const std::string identity=std::string(source->getNameInDocument())+"."+subs[0];
        historyAdmission(legacy.data(),legacy.size(),1,1,getNameInDocument());
        auto ref=solidCurveReference(*getDocument(),identity,"Circle History",false,[&](unsigned representation){historyAdmission(legacy.data(),legacy.size(),1,representation,identity);});
        const auto [p,n]=solidOnCurve(ref,{0,0,0},PathFraction.getValue());
        double input[7]={p[0],p[1],p[2],n[0],n[1],n[2],Radius.getValue()},plan[7];
        if(!om9_circle_validate_plan(input,plan))throw std::runtime_error("Circle History exceeds validated analytic bounds");
        CurvePyRef app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part"));
        CurvePyRef center(PyObject_CallMethod(app.value,"Vector","ddd",plan[0],plan[1],plan[2])),normal(PyObject_CallMethod(app.value,"Vector","ddd",plan[3],plan[4],plan[5]));
        CurvePyRef edge(PyObject_CallMethod(part.value,"makeCircle","dOO",plan[6],center.value,normal.value)),list(PyList_New(1));PyList_SET_ITEM(list.value,0,Py_NewRef(edge.value));
        CurvePyRef shape(PyObject_CallMethod(part.value,"Wire","O",list.value)),valid(PyObject_CallMethod(shape.value,"isValid",nullptr));if(PyObject_IsTrue(valid.value)!=1)throw std::runtime_error("Circle History created invalid geometry");
        const std::vector<double> config(legacy.begin(),legacy.end());const std::vector<SolidPoint> points{p};
        double replayInput[7]={p[0],p[1],p[2],n[0],n[1],n[2],1e-7},output[4096];
        if(!om9_circle_history_replay(config.data(),p.data(),1,replayInput,output))throw std::runtime_error("InvalidBasis: Circle legacy History replay failed");
        const auto result=replayResult(*this,shape.value,config,points,replayInput,{},output,evidence,true,state);
        {Guard g(changing);setReplayResult(*this,shape.value,result,evidence,state);}
        replay->verify();auto* execution=Part::Feature::execute();
        if(!replay->commit())throw std::runtime_error("StaleGeometry: Circle legacy native savepoint commit rejected");
        reportReplayDisposal(*replay,false);
        return execution;
    }catch(const Base::Exception& e){return rejected(e.what());}
     catch(const std::exception& e){return rejected(e.what());}
     catch(...){return rejected("KernelFailure: recompute failed");}
}
void CircleHistory::initialize(PyObject* shape,const SolidCurveReference& ref,double fraction,double radius){
    initialResultPending=true;
    Guard g(changing);auto* source=getDocument()->getObject(ref.input.name.c_str());if(!source)throw std::runtime_error("Circle History source was deleted");
    SourceCurve.setValue(source,std::vector<std::string>{ref.sub});PathFraction.setValue(fraction);Radius.setValue(radius);Shape.setPyObject(shape);Recorded.setValue(true);refreshPlacements();
}
void CircleHistory::initializeRecipe(PyObject* shape,const std::vector<double>& config,const std::vector<SolidPoint>& points,const std::vector<CircleHistorySource>& inputs,const std::optional<SolidCurveReference>& path,double fraction){
    initialResultPending=true;
    if(config.size()!=24||points.empty()||points.size()>1024)throw std::runtime_error("Cannot record invalid Circle History recipe");
    Guard g(changing);Recipe.setValues(config);std::vector<Base::Vector3d> stored;for(const auto& p:points)stored.emplace_back(p[0],p[1],p[2]);Points.setValues(stored);
    std::vector<App::DocumentObject*> objects;std::vector<std::string> subs;std::vector<long> roles;std::vector<double> parameters;
    for(const auto& input:inputs){auto* object=getDocument()->getObject(input.name.c_str());if(!object)throw std::runtime_error("Circle History source was deleted");objects.push_back(object);subs.push_back(input.sub);roles.insert(roles.end(),{input.role,long(input.index),long(input.count)});
        if(input.role==2){const auto identity=input.name+"."+input.sub;auto ref=solidCurveReference(*getDocument(),identity,"Circle History",false,[&](unsigned representation){historyAdmission(config.data(),config.size(),points.size(),representation,identity);});parameters.push_back(solidCurveFraction(ref,points.at(input.index)));}else parameters.push_back(0.);
    }
    Sources.setValues(objects,subs);SourceRoles.setValues(roles);SourceParameters.setValues(parameters);
    if(path){auto* source=getDocument()->getObject(path->input.name.c_str());if(!source)throw std::runtime_error("Circle History path was deleted");SourceCurve.setValue(source,std::vector<std::string>{path->sub});PathFraction.setValue(fraction);}
    double plan[7];if(!om9_curve_circle_plan(plan))throw std::runtime_error("Circle History has no committed plan");Radius.setValue(plan[6]);FitDeviation.setValue(om9_circle_deviation(false));ApproxDeviation.setValue(om9_circle_deviation(true));Shape.setPyObject(shape);Recorded.setValue(true);refreshPlacements();
}
void CircleHistory::detach(){Guard g(changing);Recorded.setValue(false);SourceCurve.setValue(nullptr);Sources.setValues(std::vector<App::DocumentObject*>{},std::vector<std::string>{});PlacementSources.setValues({});}
void CircleHistory::onBeforeChange(const App::Property* p){
    const auto& sources=Sources.getValues();const bool deletingSource=(p==&SourceCurve&&SourceCurve.getValue()&&SourceCurve.getValue()->isRemoving())||(p==&Sources&&std::any_of(sources.begin(),sources.end(),[](auto* source){return source&&source->isRemoving();}));
    const bool recipeProperty=p==&Recipe||p==&Points||p==&Sources||p==&SourceRoles||p==&SourceParameters;
    if(!changing&&!guarded(this)&&!isRecomputing()&&!deletingSource&&(p==&Shape||p==&Placement||p==&SourceCurve||p==&Radius||p==&PathFraction||recipeProperty)&&historyEditLocked(this))throw Base::RuntimeError("History Lock prevents child Circle edits");
    // Recorded links are immutable recipes. Detach before replacing a source;
    // read-only UI properties alone cannot guard assignments through Python.
    if(!changing&&!guarded(this)&&Recorded.getValue()&&p==&SourceCurve&&SourceCurve.getValue()&&!SourceCurve.getValue()->isRemoving())throw Base::RuntimeError("Detach Circle History before changing its source");
    if(!changing&&!guarded(this)&&Recorded.getValue()&&recipeProperty&&!deletingSource)throw Base::RuntimeError("Detach Circle History before changing its recorded recipe");
    Part::Feature::onBeforeChange(p);
}
void CircleHistory::onChanged(const App::Property* p){
    if(!changing&&!guarded(this)&&!isRecomputing()){
        if(p==&SourceCurve&&Recorded.getValue()&&!SourceCurve.getValue()){clearShape();setError();touch();}
        if(p==&Sources&&Recorded.getValue()&&Sources.getValues().size()*3!=SourceRoles.getValues().size()){clearShape();setError();touch();}
        if((p==&Shape||p==&Placement)&&Recorded.getValue()){warnBrokenHistory(this);detach();}
        else if(p==&Recorded&&!Recorded.getValue())detach();
    }
    Guard g(changing);Part::Feature::onChanged(p);
}
void CircleHistory::onLostLinkToObject(App::DocumentObject* o){
    const auto& sources=Sources.getValues();const bool lost=SourceCurve.getValue()==o||std::find(sources.begin(),sources.end(),o)!=sources.end();Part::Feature::onLostLinkToObject(o);if(lost&&Recorded.getValue()){clearShape();setError();touch();}
}
void CircleHistory::onDocumentRestored(){Guard g(changing);Part::Feature::onDocumentRestored();refreshPlacements();}
CircleHistory* createCircleHistory(App::Document& doc,PyObject* shape,const SolidCurveReference& ref,double fraction,double radius){
    auto* object=doc.addObject("OpenMatrix9Gui::CircleHistory","Circle");if(!object)throw std::runtime_error("Cannot create Circle History");const std::string name=object->getNameInDocument();
    try{auto* result=dynamic_cast<CircleHistory*>(object);if(!result)throw std::runtime_error("Circle History type mismatch");result->initialize(shape,ref,fraction,radius);return result;}catch(...){doc.removeObject(name.c_str());throw;}
}
}
