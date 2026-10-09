// SPDX-License-Identifier: LGPL-2.1-or-later
// FreeCAD topology/selection/shape adapter; Rust owns Circle replay and bounds.
#include "CircleHistoryAdapter.h"
#include "CircleHistory.h"
#include "HistoryFeature.h"
#include "RustBridge.h"
#include <App/Document.h>
#include <Base/Interpreter.h>
#include <Gui/Selection/Selection.h>
#include <algorithm>
#include <cmath>
#include <sstream>
#include <iomanip>
namespace OpenMatrix9Gui {
namespace {
using Ref=CurvePyRef;
// Only value snapshots/identities are retained; no Python reference survives a
// command or runs a destructor after Python interpreter shutdown.
std::vector<CircleHistorySource> sources;
SolidPoint point(PyObject* object){
    SolidPoint p;const char* axes[]={"x","y","z"};
    for(unsigned i=0;i<3;++i){Ref n(PyObject_GetAttrString(object,axes[i]));p[i]=PyFloat_AsDouble(n.value);if(PyErr_Occurred()||!std::isfinite(p[i]))throw std::runtime_error("Invalid Circle source point");}return p;
}
std::string signature(App::Document& doc,const std::string& name,const std::vector<SolidPoint>& points){
    auto* native=doc.getObject(name.c_str());if(!native)throw std::runtime_error("Circle source was deleted");
    Ref object(native->getPyObject());
    if(PyObject_HasAttrString(object.value,"Shape"))return nativeShapeInput(doc,name).signature;
    std::ostringstream out;out<<std::setprecision(17);for(const auto& p:points)for(double v:p)out<<v<<',';return out.str();
}
CircleHistorySource capture(App::Document& doc,std::string name,std::string sub,long role,std::size_t index){
    auto points=circleHistorySourcePoints(doc,name,sub);
    CircleHistorySource source;source.name=std::move(name);source.sub=std::move(sub);source.role=role;source.index=index;source.count=role==-1?0:points.size();source.signature=signature(doc,source.name,points);source.snapshot=std::move(points);source.identity=doc.getObject(source.name.c_str());return source;
}
std::pair<std::string,std::string> vertexPath(const QString& text){
    const auto value=text.toStdString();const auto dot=value.find('.');
    if(dot==std::string::npos||dot==0||value.substr(dot+1).find("Vertex")!=0)throw std::runtime_error("Use Point=Object.VertexN or Direction=Object.VertexN");
    const auto sub=value.substr(dot+1);if(sub.size()<=6||sub.find_first_not_of("0123456789",6)!=std::string::npos||std::stoul(sub.substr(6))==0)throw std::runtime_error("Use a native VertexN subelement");return {value.substr(0,dot),sub};
}
Ref list(const double* values,std::size_t count,bool integer=false){Ref result(PyList_New(Py_ssize_t(count)));for(std::size_t i=0;i<count;++i)PyList_SET_ITEM(result.value,Py_ssize_t(i),integer?PyLong_FromLong(long(values[i])):PyFloat_FromDouble(values[i]));return result;}
}
std::vector<SolidPoint> circleHistorySourcePoints(App::Document& doc,const std::string& name,const std::string& sub){
    auto* native=doc.getObject(name.c_str());if(!native||!native->isValid())throw std::runtime_error("Circle History source is missing or invalid");
    if(native->isDerivedFrom(Base::Type::fromName("App::Link")))throw std::runtime_error("Circle History does not accept linked references");
    Ref object(native->getPyObject());Ref global(PyObject_CallMethod(object.value,"getGlobalPlacement",nullptr)),local(PyObject_GetAttrString(object.value,"Placement")),inverse(PyObject_CallMethod(local.value,"inverse",nullptr)),parent(PyNumber_Multiply(global.value,inverse.value));
    std::vector<SolidPoint> result;
    auto append=[&](PyObject* value){if(result.size()>=1024)throw std::runtime_error("Circle source supports at most 1024 points");Ref transformed(PyObject_CallMethod(parent.value,"multVec","O",value));result.push_back(point(transformed.value));};
    auto surface=[&](PyObject* face){Ref geometry(PyObject_GetAttrString(face,"Surface"));if(!PyObject_HasAttrString(geometry.value,"getPoles"))return false;Ref rows(PyObject_CallMethod(geometry.value,"getPoles",nullptr));for(Py_ssize_t i=0;i<PySequence_Size(rows.value);++i){Ref row(PySequence_GetItem(rows.value,i));for(Py_ssize_t j=0;j<PySequence_Size(row.value);++j){Ref p(PySequence_GetItem(row.value,j));append(p.value);}}return true;};
    if(PyObject_HasAttrString(object.value,"Shape")){
        Ref shape(PyObject_GetAttrString(object.value,"Shape"));
        if(!sub.empty()){
            Ref element(PyObject_CallMethod(shape.value,"getElement","s",sub.c_str()));
            if(sub.starts_with("Vertex")){Ref p(PyObject_GetAttrString(element.value,"Point"));append(p.value);}
            else if(sub.starts_with("Face")){if(!surface(element.value))throw std::runtime_error("Selected Circle face has no surface control points");}
            else throw std::runtime_error("Circle point sources require vertices or supported faces");
        }else {
            Ref faces(PyObject_GetAttrString(shape.value,"Faces")),edges(PyObject_GetAttrString(shape.value,"Edges"));bool poles=false;
            if(PySequence_Size(faces.value)==1){Ref face(PySequence_GetItem(faces.value,0));poles=surface(face.value);}
            if(!poles&&PySequence_Size(edges.value)==1){Ref edge(PySequence_GetItem(edges.value,0)),curve(PyObject_GetAttrString(edge.value,"Curve"));if(PyObject_HasAttrString(curve.value,"getPoles")){Ref values(PyObject_CallMethod(curve.value,"getPoles",nullptr));for(Py_ssize_t i=0;i<PySequence_Size(values.value);++i){Ref p(PySequence_GetItem(values.value,i));append(p.value);}poles=true;}}
            if(!poles){Ref vertices(PyObject_GetAttrString(shape.value,"Vertexes"));for(Py_ssize_t i=0;i<PySequence_Size(vertices.value);++i){Ref vertex(PySequence_GetItem(vertices.value,i)),p(PyObject_GetAttrString(vertex.value,"Point"));append(p.value);}}
        }
    }else if(PyObject_HasAttrString(object.value,"Mesh")){
        if(!sub.empty())throw std::runtime_error("Select the whole Circle mesh source");Ref mesh(PyObject_GetAttrString(object.value,"Mesh")),topology(PyObject_GetAttrString(mesh.value,"Topology")),points(PySequence_GetItem(topology.value,0));for(Py_ssize_t i=0;i<PySequence_Size(points.value);++i){Ref p(PySequence_GetItem(points.value,i));append(p.value);}
    }else if(PyObject_HasAttrString(object.value,"Points")){
        if(!sub.empty())throw std::runtime_error("Select the whole Circle point cloud source");Ref cloud(PyObject_GetAttrString(object.value,"Points")),points(PyObject_GetAttrString(cloud.value,"Points"));for(Py_ssize_t i=0;i<PySequence_Size(points.value);++i){Ref p(PySequence_GetItem(points.value,i));append(p.value);}
    }else throw std::runtime_error("Circle source has no supported native points");
    if(result.empty())throw std::runtime_error("Circle source has no points");return result;
}
std::optional<unsigned> circleHistoryNativeInput(App::Document& doc,const QString& text){
    Base::PyGILStateLocker lock;const auto t=text.trimmed();const bool direction=t.startsWith("Direction=",Qt::CaseInsensitive),p=t.startsWith("Point=",Qt::CaseInsensitive);
    if(p||direction){
        verifyCircleHistoryReferences(doc);const auto [name,sub]=vertexPath(t.mid(direction?10:6));
        const auto index=om9_curve_preview_count();auto source=capture(doc,name,sub,direction?-1:0,direction?0:index);
        const auto point=source.snapshot.front();
        unsigned effect=0;
        if(direction){
            // Native kind 4 requests the existing Circle orientation transition;
            // the session validates that it is waiting for a direction endpoint.
            effect=om9_circle_reference(point[0],point[1],point[2],0,0,0,4);
        }else if(om9_circle_reference_mode()==3){
            effect=om9_circle_reference(point[0],point[1],point[2],0,0,0,5);
        }else effect=om9_curve_point(point[0],point[1],point[2]);
        if(effect==1||effect==2){
            if(p){double config[24],picked[3072];const auto count=om9_circle_history_snapshot(config,picked,1024);
                if(count==index){
                    if(config[1]==0.&&config[5]==1.){source.role=-1;source.index=0;source.count=0;}
                    else if((config[1]==2.||config[1]==6.)&&config[4]>0.&&index>0){source.role=4;source.index=index-1;source.count=0;}
                    else throw std::runtime_error("Circle point source did not produce a recordable input");
                }
            }
            if(source.role==-1||source.role==4)sources.erase(std::remove_if(sources.begin(),sources.end(),[&](const auto& old){return old.role==source.role;}),sources.end());
            sources.push_back(std::move(source));
        }return effect;
    }
    if(om9_circle_reference_mode()==4&&(t.compare("Selection",Qt::CaseInsensitive)==0||(t.isEmpty()&&om9_curve_preview_count()==0))){
        verifyCircleHistoryReferences(doc);std::vector<CircleHistorySource> picked;std::vector<double> flat;auto offset=om9_curve_preview_count();
        for(const auto& sel:Gui::Selection().getSelection(doc.getName())){
            auto source=capture(doc,sel.FeatName,sel.SubName?sel.SubName:"",1,offset);offset+=source.count;for(const auto& point:source.snapshot)flat.insert(flat.end(),point.begin(),point.end());picked.push_back(std::move(source));
        }
        if(flat.empty())throw std::runtime_error("Select native Circle fit points");const auto effect=om9_circle_fit_batch(flat.data(),flat.size()/3);
        if(effect!=1)return effect;sources.insert(sources.end(),picked.begin(),picked.end());
        if(t.isEmpty())return om9_curve_input("");return effect;
    }
    const auto key=t.section('=',0,0).trimmed();
    const bool orientation=key.compare("Orientation",Qt::CaseInsensitive)==0||key.compare("o",Qt::CaseInsensitive)==0;
    const bool radiusOption=key.compare("Radius",Qt::CaseInsensitive)==0||key.compare("r",Qt::CaseInsensitive)==0;
    if(orientation||radiusOption){
        const auto utf8=t.toUtf8();const auto effect=om9_curve_input(utf8.constData());
        if(effect==1||effect==2)sources.erase(std::remove_if(sources.begin(),sources.end(),[&](const auto& source){return source.role==(orientation?-1:4);}),sources.end());
        return effect;
    }
    return {};
}
void clearCircleHistoryReferences(){sources.clear();}
void verifyCircleHistoryReferences(App::Document& doc){
    Base::PyGILStateLocker lock;
    for(const auto& source:sources){if(doc.getObject(source.name.c_str())!=source.identity)throw std::runtime_error("Circle point source changed or was deleted; Undo and reselect");const auto points=circleHistorySourcePoints(doc,source.name,source.sub);if(points!=source.snapshot||signature(doc,source.name,points)!=source.signature)throw std::runtime_error("Circle point source changed; Undo and reselect");}
}
void pruneCircleHistoryReferences(unsigned effect){
    if(effect==3){clearCircleHistoryReferences();return;}
    if(effect==2)return; // Creation needs the committed recipe; main clears after commit.
    const auto count=om9_curve_preview_count();double config[24],points[3072];const auto recorded=om9_circle_history_snapshot(config,points,1024);
    const bool oriented=recorded<=1024&&config[5]==1.,radius=recorded<=1024&&config[4]>0.;
    sources.erase(std::remove_if(sources.begin(),sources.end(),[&](const auto& source){return source.role==-1?!oriented:source.role==4?(!radius||source.index>=count):source.index>=count;}),sources.end());
    for(auto& source:sources)if((source.role==1||source.role==3)&&source.index+source.count>count){source.count=count-source.index;source.role=3;}
}
App::DocumentObject* createCircleHistoryForCommand(App::Document& doc,PyObject* shape,const std::map<std::size_t,SolidCurveReference>& tangentRefs,const std::optional<SolidCurveReference>& path){
    if(!om9_circle_history()||!historyRecordingEnabled(doc))return nullptr;
    Base::PyGILStateLocker lock;verifyCircleHistoryReferences(doc);
    double config[24],flat[3072];const auto count=om9_circle_history_snapshot(config,flat,1024);if(count==0||count>1024)throw std::runtime_error("No valid Circle History recipe");
    std::vector<SolidPoint> points;for(std::size_t i=0;i<count;++i)points.push_back({flat[i*3],flat[i*3+1],flat[i*3+2]});
    auto inputs=sources;
    if(int(config[1])==6)for(const auto& [index,ref]:tangentRefs){CircleHistorySource source;source.name=ref.input.name;source.sub=ref.sub;source.role=2;source.index=index;source.count=1;source.identity=doc.getObject(source.name.c_str());inputs.push_back(std::move(source));}
    double fraction=0.;if(path){double plan[7];if(!om9_curve_circle_plan(plan))throw std::runtime_error("No Circle path plan");fraction=solidCurveFraction(*path,{plan[0],plan[1],plan[2]});}
    auto* object=doc.addObject("OpenMatrix9Gui::CircleHistory","Circle");if(!object)throw std::runtime_error("Cannot create Circle History");const std::string name=object->getNameInDocument();
    try{auto* result=dynamic_cast<CircleHistory*>(object);if(!result)throw std::runtime_error("Circle History type mismatch");result->initializeRecipe(shape,{config,config+24},points,inputs,path,fraction);return result;}catch(...){doc.removeObject(name.c_str());throw;}
}
CurvePyRef circleHistoryShape(const double* replay){
    Ref app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part"));Ref edge(Py_NewRef(Py_None));
    if(replay[10]>0){
        const auto np=std::size_t(replay[10]),nk=std::size_t(replay[11]);if(np>256||nk>300||13+np*3+nk*2>4096)throw std::runtime_error("Circle History spline buffer exceeds bounds");
        Ref poles(PyList_New(Py_ssize_t(np)));for(std::size_t i=0;i<np;++i){const auto* p=replay+13+i*3;PyList_SET_ITEM(poles.value,Py_ssize_t(i),PyObject_CallMethod(app.value,"Vector","ddd",p[0],p[1],p[2]));}
        auto knots=list(replay+13+np*3,nk),mults=list(replay+13+np*3+nk,nk,true);Ref curve(PyObject_CallMethod(part.value,"BSplineCurve",nullptr));
        Ref built(PyObject_CallMethod(curve.value,"buildFromPolesMultsKnots","OOOOi",poles.value,mults.value,knots.value,replay[12]!=0.?Py_True:Py_False,int(replay[9])));edge=Ref(PyObject_CallMethod(curve.value,"toShape",nullptr));
    }else{Ref center(PyObject_CallMethod(app.value,"Vector","ddd",replay[0],replay[1],replay[2])),normal(PyObject_CallMethod(app.value,"Vector","ddd",replay[3],replay[4],replay[5]));edge=Ref(PyObject_CallMethod(part.value,"makeCircle","dOO",replay[6],center.value,normal.value));}
    Ref edges(PyList_New(1));PyList_SET_ITEM(edges.value,0,Py_NewRef(edge.value));Ref shape(PyObject_CallMethod(part.value,"Wire","O",edges.value)),valid(PyObject_CallMethod(shape.value,"isValid",nullptr)),closed(PyObject_CallMethod(shape.value,"isClosed",nullptr));if(PyObject_IsTrue(valid.value)!=1||PyObject_IsTrue(closed.value)!=1)throw std::runtime_error("Circle History generated invalid or open geometry");return shape;
}
}
