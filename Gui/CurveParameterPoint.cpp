// SPDX-License-Identifier: LGPL-2.1-or-later
// Native SDK extraction, document properties and owned geometry; Rust owns
// numeric correspondence and permission to use an original parameter directly.
#include "CurveParameterPoint.h"
#include "CurveConversionContract.h"
#include "CoreObjects.h"
#include "CoreUnits.h"
#include "HistoryFeature.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/GeoFeature.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Base/Interpreter.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Line.hxx>
#include <gp_Circ.hxx>
#include <TopoDS.hxx>
#include <QJsonDocument>
#include <QJsonArray>
#include <set>
#include <algorithm>
#include <cmath>
#include <numbers>
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::CurveParameterPoint,Part::Feature)
namespace {
struct Write {bool& value;bool old;explicit Write(bool& v):value(v),old(v){v=true;}~Write(){value=old;}};
bool guarded(const App::DocumentObject* o){return !o->getDocument()||o->isRestoring()||o->isRemoving()||o->getDocument()->testStatus(App::Document::Restoring)||o->getDocument()->isPerformingTransaction();}
QJsonArray xyz(const std::array<double,3>& p){return {p[0],p[1],p[2]};}
ON_3dPoint onPoint(const gp_Pnt& p){return ON_3dPoint(p.X(),p.Y(),p.Z());}
ON_3dVector onVector(const gp_Dir& p){return ON_3dVector(p.X(),p.Y(),p.Z());}
struct Evaluation {ObjectSnapshot guard;Part::TopoShape converted,point;QJsonObject report;};
Evaluation evaluate(App::Document& doc,App::DocumentObject* source,const std::string& sub,double parameter,double lengthTolerance,double parameterTolerance,bool raw){
    auto* feature=dynamic_cast<Part::Feature*>(source);
    if(!feature||source->getDocument()!=&doc||!doc.containsObject(source)||!source->isValid()||feature->Shape.getShape().isNull())throw std::runtime_error("Parameter source must be a live valid native Part curve");
    auto guard=captureObject(doc,source->getNameInDocument(),sub);
    const auto units=unitContextSnapshot(doc);
    if(units.empty())throw std::runtime_error("Parameter dependency requires explicit document unit context");
    unitInputScale(doc); // Existing Rust unit-context parser rejects corrupt context.
    auto selected=feature->Shape.getShape().getSubShape(sub.c_str());
    if(selected.ShapeType()!=TopAbs_EDGE)throw std::runtime_error("Parameter source requires an Edge subelement");
    auto edge=TopoDS::Edge(selected);TopLoc_Location location;double first,last;
    auto original=BRep_Tool::Curve(edge,location,first,last);
    if(original.IsNull())throw std::runtime_error("Parameter source edge has no native support curve");
    auto worldCurve=Handle(Geom_Curve)::DownCast(original->Transformed(location.Transformation()));
    const auto parent=App::GeoFeature::getGlobalPlacement(source)*feature->Placement.getValue().inverse();
    const auto m=parent.toMatrix();gp_Trsf transform;
    transform.SetValues(m[0][0],m[0][1],m[0][2],m[0][3],m[1][0],m[1][1],m[1][2],m[1][3],m[2][0],m[2][1],m[2][2],m[2][3]);
    worldCurve=Handle(Geom_Curve)::DownCast(worldCurve->Transformed(transform));
    using namespace ThreeDm::CurveConversion;
    const Input provisional{1,1,1,0,{first,last},{first,last},0.,lengthTolerance,parameterTolerance};
    auto status=om9_curve_conversion_preflight(provisional,1);if(status)throw QueryError(status);
    auto action=om9_curve_conversion_request(provisional,parameter);if(action!=1&&action!=2)throw QueryError(action);
    std::unique_ptr<ON_Curve> native;
    if(auto circle=Handle(Geom_Circle)::DownCast(worldCurve);!circle.IsNull()){
        // This first host extraction slice admits the full analytic circle only.
        if(std::abs(first)>1e-12||std::abs(last-2*std::numbers::pi)>1e-12)throw std::runtime_error("Trimmed Circle parameter dependency is not verified");
        const auto c=circle->Circ();const auto axes=c.Position();
        native=std::make_unique<ON_ArcCurve>(ON_Circle(ON_Plane(onPoint(c.Location()),onVector(axes.XDirection()),onVector(axes.YDirection())),c.Radius()),first,last);
    }else if(auto line=Handle(Geom_Line)::DownCast(worldCurve);!line.IsNull()){
        native=std::make_unique<ON_LineCurve>(ON_Line(onPoint(line->Value(first)),onPoint(line->Value(last))),first,last);
    }else throw std::runtime_error("Host parameter dependency currently verifies analytic Circle and Line only");
    const std::array<double,1> request{parameter};
    auto observed=importCurveObserved(*native,{0.,lengthTolerance,parameterTolerance},request);
    const auto actual=worldCurve->Value(parameter);
    const DependencyInput dependency{observed.report.input,observed.report.queries.at(0).observation,{actual.X(),actual.Y(),actual.Z()},std::uint32_t(raw),0};
    const auto dependent=om9_curve_parameter_dependency(dependency);if(dependent.status)throw QueryError(dependent.status);
    verifyObject(doc,guard);
    auto report=observed.report.json();
    report["length_unit"]="mm";report["host_frame"]="FreeCAD_world_mm";report["unit_context_snapshot"]=QString::fromStdString(units);
    report["source_identity"]=QJsonObject{{"document_uid",QString::fromStdString(guard.documentUid)},{"object_name",QString::fromStdString(guard.objectName)},{"subelement",QString::fromStdString(guard.subelement)},
        {"native_object_id",QString::number(guard.stamp.object)},{"document_session",QString::number(guard.stamp.document)},{"generation",QString::number(guard.stamp.generation)},{"geometry_revision",QString::number(guard.stamp.revision)},{"document_epoch",QString::number(guard.stamp.epoch)}};
    QJsonArray world;for(double v:guard.world)world.append(v);report["host_world_placement"]=world;
    report["source_edge_orientation"]=int(edge.Orientation());
    report["dependent_query"]=QJsonObject{{"source_parameter",parameter},{"target_parameter",dependent.target_parameter},{"used_raw",bool(dependent.used_raw)},
        {"host_source_point",QJsonArray{actual.X(),actual.Y(),actual.Z()}},{"host_source_residual",dependent.host_source_residual},{"host_point_residual",dependent.host_point_residual},{"point",xyz(dependent.point)}};
    report["converted_shape_frame"]="FreeCAD_world_mm";
    auto point=BRepBuilderAPI_MakeVertex(gp_Pnt(dependent.point[0],dependent.point[1],dependent.point[2])).Vertex();
    return {std::move(guard),Part::TopoShape(observed.shape),Part::TopoShape(point),std::move(report)};
}
void editable(App::Document& doc){auto* gui=Gui::Application::Instance->activeDocument();if(App::GetApplication().getActiveDocument()!=&doc||!gui||gui->getInEdit()||gui->isAboutToClose()||!om9AlterDocument(&doc))throw std::runtime_error("The active parameter dependency document is not editable");}
PyObject* createAPI(PyObject*,PyObject* args){
    const char *docName,*sourceName,*sub;double parameter,lengthTolerance,parameterTolerance;int raw=0;
    if(!PyArg_ParseTuple(args,"sssddd|p",&docName,&sourceName,&sub,&parameter,&lengthTolerance,&parameterTolerance,&raw))return nullptr;
    try{
        auto* doc=App::GetApplication().getDocument(docName);if(!doc)throw std::runtime_error("Parameter source document is missing");editable(*doc);
        auto* source=doc->getObject(sourceName);auto result=evaluate(*doc,source,sub,parameter,lengthTolerance,parameterTolerance,raw);
        verifyObject(*doc,result.guard);om9OpenTransaction(*doc,"Record Curve Parameter Point");
        try{
            auto* point=dynamic_cast<CurveParameterPoint*>(doc->addObject("OpenMatrix9Gui::CurveParameterPoint","CurveParameterPoint"));
            if(!point)throw std::runtime_error("Cannot create native parameter point");
            point->initialize(source,sub,parameter,lengthTolerance,parameterTolerance,raw);doc->recompute();
            if(point->isError())throw std::runtime_error("Parameter point initial recompute failed");
            auto* py=point->getPyObject();om9CommitTransaction(*doc);return py;
        }catch(...){om9AbortTransaction(*doc);throw;}
    }catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}
    catch(const std::exception& e){PyErr_SetString(PyExc_ValueError,e.what());}
    catch(...){PyErr_SetString(PyExc_RuntimeError,"Native parameter dependency failed");}return nullptr;
}
}
CurveParameterPoint::CurveParameterPoint(){
    ADD_PROPERTY(Source,(nullptr));ADD_PROPERTY(UnitSource,(nullptr));ADD_PROPERTY(PlacementSources,(nullptr));
    ADD_PROPERTY(SourceParameter,(0.));ADD_PROPERTY(LengthTolerance,(1e-7));ADD_PROPERTY(ParameterTolerance,(1e-10));ADD_PROPERTY(UseRawParameter,(false));ADD_PROPERTY(Recorded,(false));ADD_PROPERTY(HistoryDirty,(false));
    ADD_PROPERTY(ConvertedShape,(Part::TopoShape()));ADD_PROPERTY(QueryReport,(""));
    for(auto* p:{static_cast<App::Property*>(&Source),static_cast<App::Property*>(&UnitSource),static_cast<App::Property*>(&ConvertedShape),static_cast<App::Property*>(&QueryReport),static_cast<App::Property*>(&Recorded),static_cast<App::Property*>(&HistoryDirty)})p->setReadOnly(true);
    PlacementSources.setStatus(App::Property::Hidden,true);ConvertedShape.setStatus(App::Property::Hidden,true);
}
CurveParameterPoint::~CurveParameterPoint(){changed.disconnect();recomputed.disconnect();}
void CurveParameterPoint::initialize(App::DocumentObject* source,const std::string& sub,double parameter,double lengthTolerance,double parameterTolerance,bool raw){
    Write internal(writing);Source.setValue(source,std::vector<std::string>{sub});SourceParameter.setValue(parameter);LengthTolerance.setValue(lengthTolerance);ParameterTolerance.setValue(parameterTolerance);UseRawParameter.setValue(raw);
    for(auto* o:getDocument()->getObjects())if(dynamic_cast<UnitSettings*>(o)){UnitSource.setValue(o);break;}
    Recorded.setValue(true);refreshFrames();
    auto result=evaluate(*getDocument(),source,sub,parameter,lengthTolerance,parameterTolerance,raw);
    ConvertedShape.setValue(result.converted);setWorldPoint(result.point);QueryReport.setValue(QJsonDocument(result.report).toJson(QJsonDocument::Compact).toStdString());HistoryDirty.setValue(false);
    if(!historyRecordingEnabled(*getDocument()))detach();
}
void CurveParameterPoint::refreshFrames(){
    std::vector<App::DocumentObject*> frames;std::set<App::DocumentObject*> seen;
    for(auto* root:{Source.getValue(),static_cast<App::DocumentObject*>(this)}){
        std::set<App::DocumentObject*> path;
        for(auto* p=root?App::GeoFeatureGroupExtension::getGroupOfObject(root):nullptr;p;p=App::GeoFeatureGroupExtension::getGroupOfObject(p)){
            if(!path.insert(p).second||path.size()>256)throw std::runtime_error("Invalid parameter dependency placement hierarchy");
            if(seen.insert(p).second)frames.push_back(p);
        }
    }
    if(frames!=PlacementSources.getValues()){Write internal(writing);PlacementSources.setValues(frames);}
}
void CurveParameterPoint::clearOutputs(){Write internal(writing);Shape.setValue(Part::TopoShape());ConvertedShape.setValue(Part::TopoShape());QueryReport.setValue("");HistoryDirty.setValue(true);}
void CurveParameterPoint::setWorldPoint(const Part::TopoShape& world){
    Write internal(writing);Placement.setValue(Base::Placement());
    auto local=world;local.transformShape(App::GeoFeature::getGlobalPlacement(this).inverse().toMatrix(),true);
    Shape.setValue(local);
}
short CurveParameterPoint::mustExecute()const{return Recorded.getValue()&&(isTouched()||HistoryDirty.getValue())?1:Part::Feature::mustExecute();}
App::DocumentObjectExecReturn* CurveParameterPoint::execute(){
    if(!Recorded.getValue()||!historyUpdatesEnabled(*getDocument()))return Part::Feature::execute();
    try{
        const auto& subs=Source.getSubValues();if(subs.size()!=1)throw std::runtime_error("Original parameter source is missing or unresolved");
        if(!dynamic_cast<UnitSettings*>(UnitSource.getValue())||UnitSource.getValue()->getDocument()!=getDocument())throw std::runtime_error("Original parameter unit context is missing");
        refreshFrames();auto result=evaluate(*getDocument(),Source.getValue(),subs[0],SourceParameter.getValue(),LengthTolerance.getValue(),ParameterTolerance.getValue(),UseRawParameter.getValue());
        Write internal(writing);ConvertedShape.setValue(result.converted);setWorldPoint(result.point);QueryReport.setValue(QJsonDocument(result.report).toJson(QJsonDocument::Compact).toStdString());HistoryDirty.setValue(false);return Part::Feature::execute();
    }catch(const std::exception& e){clearOutputs();return new App::DocumentObjectExecReturn(e.what(),this);}
    catch(const Base::Exception& e){clearOutputs();return new App::DocumentObjectExecReturn(e.what(),this);}
    catch(...){clearOutputs();return new App::DocumentObjectExecReturn("Native parameter dependency recompute failed",this);}
}
void CurveParameterPoint::onSettingDocument(){
    Part::Feature::onSettingDocument();changed.disconnect();recomputed.disconnect();
    if(auto* doc=getDocument())changed=doc->signalChangedObject.connect([this](const App::DocumentObject& o,const App::Property& p){
        if(writing||!Recorded.getValue()||&o==this||!getDocument()||!getDocument()->containsObject(this)||isRestoring()||isRemoving()||getDocument()->testStatus(App::Document::Restoring))return;
        const auto& frames=PlacementSources.getValues();
        const bool group=o.getPropertyByName("Group")==&p;
        if(&o==Source.getValue()||&o==UnitSource.getValue()||group||(o.getPropertyByName("Placement")==&p&&std::find(frames.begin(),frames.end(),&o)!=frames.end())){
            if(getDocument()->isPerformingTransaction()){setStatus(App::ObjectStatus::PendingTransactionUpdate,true);return;}
            if(group)refreshFrames();{Write internal(writing);HistoryDirty.setValue(true);}touch();
            if(auto* source=dynamic_cast<const Part::Feature*>(&o);source&&&p==&source->Shape&&source->Shape.getShape().isNull()&&historyUpdatesEnabled(*getDocument())){clearOutputs();setError();}
        }
    });
    if(auto* doc=getDocument())recomputed=doc->signalRecomputed.connect([this](const App::Document&,const std::vector<App::DocumentObject*>&){
        if(writing||!getDocument()||!getDocument()->containsObject(this)||!Recorded.getValue()||!historyUpdatesEnabled(*getDocument()))return;
        auto* source=dynamic_cast<Part::Feature*>(Source.getValue());
        if(!source||!source->isValid()||source->Shape.getShape().isNull()||!UnitSource.getValue()){clearOutputs();setError();touch();}
    });
}
void CurveParameterPoint::unsetupObject(){changed.disconnect();recomputed.disconnect();Part::Feature::unsetupObject();}
void CurveParameterPoint::onDocumentRestored(){Write internal(writing);Part::Feature::onDocumentRestored();refreshFrames();if(Recorded.getValue()){HistoryDirty.setValue(true);touch();}}
void CurveParameterPoint::onUndoRedoFinished(){Part::Feature::onUndoRedoFinished();refreshFrames();if(Recorded.getValue()){Write internal(writing);HistoryDirty.setValue(true);touch();if(!Source.getValue()||!UnitSource.getValue()){clearOutputs();setError();}}}
void CurveParameterPoint::onLostLinkToObject(App::DocumentObject* o){const bool lost=o==Source.getValue()||o==UnitSource.getValue();Part::Feature::onLostLinkToObject(o);if(lost&&Recorded.getValue()){clearOutputs();setError();touch();}}
void CurveParameterPoint::onBeforeChange(const App::Property* p){
    const bool breakingSource=p==&Source&&Source.getValue()&&Source.getValue()->isRemoving();
    if(!breakingSource&&!writing&&!guarded(this)&&!isRecomputing()&&(p==&Shape||p==&Placement||p==&Source||p==&SourceParameter||p==&UseRawParameter||p==&LengthTolerance||p==&ParameterTolerance)&&historyEditLocked(this))throw Base::RuntimeError("History Lock prevents parameter dependency edits");
    Part::Feature::onBeforeChange(p);
}
void CurveParameterPoint::onChanged(const App::Property* p){
    if(getDocument()&&getDocument()->isPerformingTransaction())setStatus(App::ObjectStatus::PendingTransactionUpdate,true);
    const bool lost=Recorded.getValue()&&((p==&Source&&!Source.getValue())||(p==&UnitSource&&!UnitSource.getValue()));
    if(!writing&&!guarded(this)&&!isRecomputing()&&Recorded.getValue()&&!lost&&(p==&Shape||p==&Placement||p==&Source)){warnBrokenHistory(this);detach();}
    const bool dirty=!writing&&!guarded(this)&&(p==&SourceParameter||p==&UseRawParameter||p==&LengthTolerance||p==&ParameterTolerance);
    Write internal(writing);Part::Feature::onChanged(p);if(!guarded(this)&&lost){clearOutputs();setError();touch();}else if(dirty&&Recorded.getValue()){HistoryDirty.setValue(true);touch();}
}
void CurveParameterPoint::detach(){Write internal(writing);Recorded.setValue(false);Source.setValue(nullptr);UnitSource.setValue(nullptr);PlacementSources.setValues({});HistoryDirty.setValue(false);}
bool curveParameterRecorded(App::DocumentObject* o){auto* p=dynamic_cast<CurveParameterPoint*>(o);return p&&p->Recorded.getValue();}
std::vector<App::DocumentObject*> curveParameterParents(App::DocumentObject* o){auto* p=dynamic_cast<CurveParameterPoint*>(o);std::vector<App::DocumentObject*> parents;if(p){if(p->Source.getValue())parents.push_back(p->Source.getValue());if(p->UnitSource.getValue())parents.push_back(p->UnitSource.getValue());}return parents;}
bool detachCurveParameter(App::DocumentObject* o){if(auto* p=dynamic_cast<CurveParameterPoint*>(o)){p->detach();return true;}return false;}
void initializeCurveParameterTypes(){CurveParameterPoint::init();}
void addCurveParameterMethods(PyObject* module){static PyMethodDef methods[]={{"createCurveParameterPoint",createAPI,METH_VARARGS,"Create a durable native dependent point at an original analytic parameter through a guarded observed NURBS map."},{nullptr,nullptr,0,nullptr}};if(PyModule_AddFunctions(module,methods)<0)throw Base::RuntimeError("Cannot register native parameter dependency API");}
}
