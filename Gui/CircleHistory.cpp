// SPDX-License-Identifier: LGPL-2.1-or-later
// OM9-CURVE-005 analytic AroundCurve dependency adapter; Rust validates the plan.
#include "CircleHistory.h"
#include "HistoryFeature.h"
#include "RustBridge.h"
#include "CircleTangent.h"
#include <App/Document.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Base/Exception.h>
#include <Base/Interpreter.h>
#include <algorithm>
#include <set>
#include <stdexcept>
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::CircleHistory,Part::Feature)
namespace {
struct Guard {bool& flag;bool old;explicit Guard(bool& b):flag(b),old(b){flag=true;}~Guard(){flag=old;}};
bool guarded(const CircleHistory* o){return o->isRestoring()||o->isRemoving()||!o->getDocument()||o->getDocument()->testStatus(App::Document::Restoring)||o->getDocument()->isPerformingTransaction();}
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
void CircleHistory::clearShape(){Guard g(changing);Shape.setValue(Part::TopoShape());}
App::DocumentObjectExecReturn* CircleHistory::execute(){
    if(!Recorded.getValue()||!historyUpdatesEnabled(*getDocument()))return Part::Feature::execute();
    try{
        if(!Recipe.getValues().empty()){
            refreshPlacements();Base::PyGILStateLocker lock;auto config=Recipe.getValues();if(config.size()!=24)throw std::runtime_error("Circle History recipe is corrupt");
            std::vector<SolidPoint> points;for(const auto& p:Points.getValues())points.push_back({p.x,p.y,p.z});
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
                    if(config[1]!=6.||count!=1)throw std::runtime_error("Invalid Circle History tangent source");auto ref=solidCurveReference(*getDocument(),std::string(source->getNameInDocument())+"."+subs[i]);
                    points[std::size_t(offset)]=solidOnCurve(ref,points[std::size_t(offset)],parameters[i]).first;constraints[std::size_t(offset)*4+3]=0.;tangentRefs.emplace(std::size_t(offset),std::move(ref));
                }else{
                    auto current=circleHistorySourcePoints(*getDocument(),source->getNameInDocument(),subs[i]);
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
            double native[7];const double* nativePlan=nullptr;
            if(config[1]==4.){
                auto* source=SourceCurve.getValue();const auto& pathSubs=SourceCurve.getSubValues();if(!source||source==this||source->getDocument()!=getDocument()||!source->isValid()||pathSubs.size()!=1)throw std::runtime_error("Circle History path is missing or invalid");
                auto ref=solidCurveReference(*getDocument(),std::string(source->getNameInDocument())+"."+pathSubs[0]);const auto [p,n]=solidOnCurve(ref,{0,0,0},PathFraction.getValue());for(unsigned i=0;i<3;++i){native[i]=p[i];native[i+3]=n[i];}native[6]=1e-7;nativePlan=native;
            }else if(config[1]==6.){
                if(points.size()>3)throw std::runtime_error("Circle History tangent count exceeds three");for(std::size_t i=0;i<points.size();++i)for(unsigned j=0;j<3;++j)constraints[i*4+j]=points[i][j];
                double frame[13];for(unsigned i=0;i<3;++i)frame[i]=points.at(0)[i];for(unsigned i=0;i<9;++i)frame[i+3]=config[i+9];frame[12]=config[4];
                const auto solved=circleSolveNative(tangentRefs,constraints.data(),points.size(),frame,int(config[22]),config[21]!=0.,config[23]!=0.);std::copy(solved.begin(),solved.end(),native);nativePlan=native;
            }
            std::vector<double> flat;for(const auto& p:points)flat.insert(flat.end(),p.begin(),p.end());double output[4096];
            if(!om9_circle_history_replay(config.data(),flat.data(),points.size(),nativePlan,output))throw std::runtime_error("Circle History replay rejected invalid source geometry or parameters");
            auto shape=circleHistoryShape(output);{Guard g(changing);Placement.setValue(Base::Placement());Shape.setPyObject(shape.value);Radius.setValue(output[6]);FitDeviation.setValue(output[7]);ApproxDeviation.setValue(output[8]);}return Part::Feature::execute();
        }
        auto* source=SourceCurve.getValue();const auto& subs=SourceCurve.getSubValues();
        if(!source||source==this||source->getDocument()!=getDocument()||!source->isValid()||subs.size()!=1)throw std::runtime_error("Circle History source is missing or invalid");
        refreshPlacements();Base::PyGILStateLocker lock;
        auto ref=solidCurveReference(*getDocument(),std::string(source->getNameInDocument())+"."+subs[0]);
        const auto [p,n]=solidOnCurve(ref,{0,0,0},PathFraction.getValue());
        double input[7]={p[0],p[1],p[2],n[0],n[1],n[2],Radius.getValue()},plan[7];
        if(!om9_circle_validate_plan(input,plan))throw std::runtime_error("Circle History exceeds validated analytic bounds");
        CurvePyRef app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part"));
        CurvePyRef center(PyObject_CallMethod(app.value,"Vector","ddd",plan[0],plan[1],plan[2])),normal(PyObject_CallMethod(app.value,"Vector","ddd",plan[3],plan[4],plan[5]));
        CurvePyRef edge(PyObject_CallMethod(part.value,"makeCircle","dOO",plan[6],center.value,normal.value)),list(PyList_New(1));PyList_SET_ITEM(list.value,0,Py_NewRef(edge.value));
        CurvePyRef shape(PyObject_CallMethod(part.value,"Wire","O",list.value)),valid(PyObject_CallMethod(shape.value,"isValid",nullptr));if(PyObject_IsTrue(valid.value)!=1)throw std::runtime_error("Circle History created invalid geometry");
        {Guard g(changing);Placement.setValue(Base::Placement());Shape.setPyObject(shape.value);}
        return Part::Feature::execute();
    }catch(const Base::Exception& e){clearShape();return new App::DocumentObjectExecReturn(e.what(),this);}
     catch(const std::exception& e){clearShape();return new App::DocumentObjectExecReturn(e.what(),this);}
     catch(...){clearShape();return new App::DocumentObjectExecReturn("Circle History recompute failed",this);}
}
void CircleHistory::initialize(PyObject* shape,const SolidCurveReference& ref,double fraction,double radius){
    Guard g(changing);auto* source=getDocument()->getObject(ref.input.name.c_str());if(!source)throw std::runtime_error("Circle History source was deleted");
    SourceCurve.setValue(source,std::vector<std::string>{ref.sub});PathFraction.setValue(fraction);Radius.setValue(radius);Shape.setPyObject(shape);Recorded.setValue(true);refreshPlacements();
}
void CircleHistory::initializeRecipe(PyObject* shape,const std::vector<double>& config,const std::vector<SolidPoint>& points,const std::vector<CircleHistorySource>& inputs,const std::optional<SolidCurveReference>& path,double fraction){
    if(config.size()!=24||points.empty()||points.size()>1024)throw std::runtime_error("Cannot record invalid Circle History recipe");
    Guard g(changing);Recipe.setValues(config);std::vector<Base::Vector3d> stored;for(const auto& p:points)stored.emplace_back(p[0],p[1],p[2]);Points.setValues(stored);
    std::vector<App::DocumentObject*> objects;std::vector<std::string> subs;std::vector<long> roles;std::vector<double> parameters;
    for(const auto& input:inputs){auto* object=getDocument()->getObject(input.name.c_str());if(!object)throw std::runtime_error("Circle History source was deleted");objects.push_back(object);subs.push_back(input.sub);roles.insert(roles.end(),{input.role,long(input.index),long(input.count)});
        if(input.role==2){auto ref=solidCurveReference(*getDocument(),input.name+"."+input.sub);parameters.push_back(solidCurveFraction(ref,points.at(input.index)));}else parameters.push_back(0.);
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
