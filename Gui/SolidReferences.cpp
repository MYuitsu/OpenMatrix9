// SPDX-License-Identifier: LGPL-2.1-or-later
// OM9-SOLID-014: exact native curve references and planar tangent construction.
#include "SolidReferences.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <Gui/Selection/Selection.h>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_CartesianPoint.hxx>
#include <Geom2dAPI_ProjectPointOnCurve.hxx>
#include <Geom2dGcc_QualifiedCurve.hxx>
#include <Geom2dGcc_Circ2d3Tan.hxx>
#include <Geom2dGcc_Circ2d2TanRad.hxx>
#include <Standard_Failure.hxx>
#include <algorithm>
#include <cmath>
#include <memory>
#include <sstream>
namespace {
using namespace OpenMatrix9Gui;
using Ref=CurvePyRef;
double number(PyObject* o,const char* attr){Ref value(PyObject_GetAttrString(o,attr));double n=PyFloat_AsDouble(value.value);if(PyErr_Occurred()||!std::isfinite(n))throw std::runtime_error("Invalid native curve data");return n;}
SolidPoint point(PyObject* o){return {number(o,"x"),number(o,"y"),number(o,"z")};}
SolidPoint add(SolidPoint a,SolidPoint b){for(unsigned i=0;i<3;++i)a[i]+=b[i];return a;}
SolidPoint sub(SolidPoint a,SolidPoint b){for(unsigned i=0;i<3;++i)a[i]-=b[i];return a;}
SolidPoint scale(SolidPoint a,double k){for(auto& x:a)x*=k;return a;}
double dot(SolidPoint a,SolidPoint b){double n=0;for(unsigned i=0;i<3;++i)n+=a[i]*b[i];return n;}
Ref vector(SolidPoint p){Ref app(PyImport_ImportModule("FreeCAD"));return Ref(PyObject_CallMethod(app.value,"Vector","ddd",p[0],p[1],p[2]));}
SolidPoint value(PyObject* edge,double u){Ref v(PyObject_CallMethod(edge,"valueAt","d",u));return point(v.value);}
SolidPoint tangent(PyObject* edge,double u){Ref v(PyObject_CallMethod(edge,"tangentAt","d",u));auto t=point(v.value);const double norm=std::sqrt(dot(t,t));if(norm<1e-7)throw std::runtime_error("Curve tangent is degenerate");return scale(t,1./norm);}
struct Plane {
    SolidPoint o,x,y,z;
    explicit Plane(const double* b):o{b[0],b[1],b[2]},x{b[3],b[4],b[5]},y{b[6],b[7],b[8]},z{b[9],b[10],b[11]}{}
    gp_Pnt2d project(SolidPoint p)const{const auto d=sub(p,o);if(std::abs(dot(d,z))>1e-6)throw std::runtime_error("Tangent constraints must lie in one plane parallel to the active CPlane");return {dot(d,x),dot(d,y)};}
    SolidPoint world(gp_Pnt2d p)const{return add(add(o,scale(x,p.X())),scale(y,p.Y()));}
};
struct Constraint {
    bool isPoint=false;
    bool fixed=false;
    occ::handle<Geom2d_Curve> curve;
    gp_Pnt2d pick;
    double first=0,last=0,seed=0;
    bool general=false;
    Geom2dGcc_QualifiedCurve qualified()const{return {Geom2dAdaptor_Curve(curve,first,last),GccEnt_unqualified};}
};
Constraint curveConstraint(const SolidCurveReference& ref,const Plane& plane,SolidPoint pick){
    Constraint out;out.pick=plane.project(pick);Ref native(PyObject_GetAttrString(ref.edge->value,"Curve"));
    const std::string type=Py_TYPE(native.value)->tp_name;
    if(type.find("Line")!=std::string::npos){
        const auto a=plane.project(value(ref.edge->value,number(ref.edge->value,"FirstParameter"))),b=plane.project(value(ref.edge->value,number(ref.edge->value,"LastParameter")));
        if(a.Distance(b)<1e-7)throw std::runtime_error("Tangent line is degenerate");out.curve=new Geom2d_Line(a,gp_Dir2d(gp_Vec2d(a,b)));out.first=0;out.last=a.Distance(b);
    }else if(type.find("Circle")!=std::string::npos){
        Ref center(PyObject_GetAttrString(native.value,"Center")),axis(PyObject_GetAttrString(native.value,"Axis"));
        const auto n=point(axis.value);if(std::abs(std::abs(dot(n,plane.z))-1.)>1e-8)throw std::runtime_error("Circle plane differs from tangent CPlane");
        const auto c=plane.project(point(center.value));const double first=number(ref.edge->value,"FirstParameter"),last=number(ref.edge->value,"LastParameter");const auto a=plane.project(value(ref.edge->value,first));
        out.curve=new Geom2d_Circle(gp_Ax2d(c,gp_Dir2d(gp_Vec2d(c,a))),number(native.value,"Radius"),dot(n,plane.z)>0);out.first=0;out.last=last-first;
    }else{
        Ref nurbs(PyObject_CallMethod(ref.edge->value,"toNurbs",nullptr)),edges(PyObject_GetAttrString(nurbs.value,"Edges")),edge(PySequence_GetItem(edges.value,0)),curve(PyObject_GetAttrString(edge.value,"Curve"));
        Ref poles(PyObject_CallMethod(curve.value,"getPoles",nullptr)),weights(PyObject_CallMethod(curve.value,"getWeights",nullptr)),knots(PyObject_CallMethod(curve.value,"getKnots",nullptr)),mults(PyObject_CallMethod(curve.value,"getMultiplicities",nullptr)),periodic(PyObject_CallMethod(curve.value,"isPeriodic",nullptr));
        const auto np=PySequence_Size(poles.value),nk=PySequence_Size(knots.value);
        if(np<2||np>10000||nk<2||PySequence_Size(weights.value)!=np||PySequence_Size(mults.value)!=nk)throw std::runtime_error("Unsupported native NURBS data");
        NCollection_Array1<gp_Pnt2d> p(1,int(np));NCollection_Array1<double>w(1,int(np)),k(1,int(nk));NCollection_Array1<int> m(1,int(nk));
        for(int i=1;i<=np;++i){Ref pole(PySequence_GetItem(poles.value,i-1)),weight(PySequence_GetItem(weights.value,i-1));p.SetValue(i,plane.project(point(pole.value)));w.SetValue(i,PyFloat_AsDouble(weight.value));}
        for(int i=1;i<=nk;++i){Ref knot(PySequence_GetItem(knots.value,i-1)),mult(PySequence_GetItem(mults.value,i-1));k.SetValue(i,PyFloat_AsDouble(knot.value));m.SetValue(i,int(PyLong_AsLong(mult.value)));}
        if(PyErr_Occurred())throw std::runtime_error("Invalid NURBS scalar data");
        out.curve=new Geom2d_BSplineCurve(p,w,k,m,int(number(curve.value,"Degree")),PyObject_IsTrue(periodic.value)==1);
        out.first=number(edge.value,"FirstParameter");out.last=number(edge.value,"LastParameter");out.general=true;
    }
    Geom2dAPI_ProjectPointOnCurve projection(out.pick,out.curve,out.first,out.last);if(projection.NbPoints()==0)throw std::runtime_error("Cannot locate tangent pick on curve");out.seed=projection.LowerDistanceParameter();return out;
}
struct Candidate {gp_Pnt2d center;double radius=0,score=0;std::array<gp_Pnt2d,3> contacts;};
template<class Solver> void collect(const Solver& solver,const std::vector<Constraint>& refs,std::vector<Candidate>& result){
    if(!solver.IsDone())return;
    for(int index=1;index<=solver.NbSolutions();++index){
        const auto circle=solver.ThisSolution(index);Candidate c{circle.Location(),circle.Radius(),0};if(!std::isfinite(c.radius)||c.radius<1e-7)continue;bool valid=true;
        for(std::size_t i=0;i<refs.size();++i){const auto& ref=refs[i];if(ref.isPoint){if(std::abs(ref.pick.Distance(c.center)-c.radius)>1e-6*std::max(1.,c.radius))valid=false;continue;}
            double parSol=0,parArg=0;gp_Pnt2d contact;bool same=false;
            if(i==0){same=solver.IsTheSame1(index);if(!same)solver.Tangency1(index,parSol,parArg,contact);}
            if(i==1){same=solver.IsTheSame2(index);if(!same)solver.Tangency2(index,parSol,parArg,contact);}
            if constexpr(std::is_same_v<Solver,Geom2dGcc_Circ2d3Tan>)if(i==2){same=solver.IsTheSame3(index);if(!same)solver.Tangency3(index,parSol,parArg,contact);}
            if(same){valid=false;break;}
            if(ref.curve->IsPeriodic()){const double period=ref.curve->Period();parArg+=std::ceil((ref.first-parArg)/period)*period;}
            if(!std::isfinite(parArg)||parArg<ref.first-1e-8||parArg>ref.last+1e-8){valid=false;break;}
            gp_Pnt2d nativePoint;gp_Vec2d derivative;ref.curve->D1(std::clamp(parArg,ref.first,ref.last),nativePoint,derivative);gp_Vec2d radial(c.center,contact);
            if(derivative.Magnitude()<1e-9||nativePoint.Distance(contact)>1e-6*std::max(1.,c.radius)||std::abs(radial.Magnitude()-c.radius)>1e-6*std::max(1.,c.radius)||std::abs(radial.Dot(derivative))/(c.radius*derivative.Magnitude())>1e-6){valid=false;break;}
            if(ref.fixed&&contact.Distance(ref.pick)>1e-6){valid=false;break;}
            c.contacts[i]=contact;
            c.score+=contact.SquareDistance(ref.pick);
        }
        if(valid && std::none_of(result.begin(),result.end(),[&](const auto& old){return old.center.Distance(c.center)<1e-6&&std::abs(old.radius-c.radius)<1e-6;}))result.push_back(c);
    }
}
}
namespace OpenMatrix9Gui {
SolidCurveReference solidCurveReference(App::Document& doc,const std::string& path){
    const auto pos=path.find('.');const auto name=path.substr(0,pos);const auto sub=pos==std::string::npos?std::string():path.substr(pos+1);auto input=nativeShapeInput(doc,name);
    Ref edges(PyObject_GetAttrString(input.shape->value,"Edges"));const auto count=PySequence_Size(edges.value);std::size_t index=0;
    if(sub.empty()){if(count!=1)throw std::runtime_error("Specify one bounded native edge: Object.EdgeN");}
    else {if(!sub.starts_with("Edge")||sub.size()<=4||sub.find_first_not_of("0123456789",4)!=std::string::npos)throw std::runtime_error("Select a native EdgeN subelement");index=std::stoul(sub.substr(4));if(index==0||index>std::size_t(count))throw std::runtime_error("Curve edge reference is stale");--index;}
    auto edge=std::make_shared<Ref>(PySequence_GetItem(edges.value,Py_ssize_t(index)));return {std::move(input),std::move(edge),"Edge"+std::to_string(index+1)};
}
std::pair<SolidPoint,SolidPoint> solidOnCurve(const SolidCurveReference& ref,const SolidPoint& seed,std::optional<double> fraction){
    const double first=number(ref.edge->value,"FirstParameter"),last=number(ref.edge->value,"LastParameter");double u=first;
    if(fraction){if(!std::isfinite(*fraction)||*fraction<0||*fraction>1)throw std::runtime_error("OnCurve fraction must be within 0..1");u=first+*fraction*(last-first);}
    else {Ref part(PyImport_ImportModule("Part")),v(vector(seed)),vertex(PyObject_CallMethod(part.value,"Vertex","O",v.value)),distance(PyObject_CallMethod(ref.edge->value,"distToShape","O",vertex.value)),pairs(PySequence_GetItem(distance.value,1));
        if(PySequence_Size(pairs.value)!=1)throw std::runtime_error("Curve center pick has ambiguous nearest branches; use OnCurve=fraction");
        Ref pair(PySequence_GetItem(pairs.value,0)),p(PySequence_GetItem(pair.value,0)),curve(PyObject_GetAttrString(ref.edge->value,"Curve")),parameter(PyObject_CallMethod(curve.value,"parameter","O",p.value));u=PyFloat_AsDouble(parameter.value);
        if(PyErr_Occurred()||!std::isfinite(u))throw std::runtime_error("Cannot project center onto native curve");
        Ref periodic(PyObject_CallMethod(curve.value,"isPeriodic",nullptr));
        if(PyObject_IsTrue(periodic.value)==1){Ref periodValue(PyObject_CallMethod(curve.value,"period",nullptr));const double period=PyFloat_AsDouble(periodValue.value);if(period>0)u+=std::ceil((first-u)/period)*period;}
        u=std::clamp(u,first,last);
    }
    if(last-first<=1e-12)throw std::runtime_error("Curve parameter range is degenerate");
    const double eps=(last-first)*1e-6;const auto t=tangent(ref.edge->value,u);
    if(u>first+eps&&u<last-eps){const auto a=tangent(ref.edge->value,u-eps),b=tangent(ref.edge->value,u+eps);if(dot(a,b)<0.99)throw std::runtime_error("Pick a smooth curve location, not a cusp/corner");}
    return {value(ref.edge->value,u),t};
}
std::vector<SolidPoint> solidSelectedFitPoints(App::Document& doc,bool surfacePoles){
    std::vector<SolidPoint> result;Ref app(PyImport_ImportModule("FreeCAD"));
    for(const auto& sel:Gui::Selection().getSelection(doc.getName())){
        auto* native=doc.getObject(sel.FeatName);if(!native)throw std::runtime_error("FitPoints selection is stale");Ref object(native->getPyObject());
        if(native->isDerivedFrom(Base::Type::fromName("App::Link")))throw std::runtime_error("FitPoints does not accept linked references");
        Ref global(PyObject_CallMethod(object.value,"getGlobalPlacement",nullptr)),local(PyObject_GetAttrString(object.value,"Placement")),inverse(PyObject_CallMethod(local.value,"inverse",nullptr)),parent(PyNumber_Multiply(global.value,inverse.value));
        auto append=[&](PyObject* p,PyObject* transform){Ref moved(PyObject_CallMethod(transform,"multVec","O",p));const auto v=point(moved.value);if(result.size()>=1024)throw std::runtime_error("FitPoints supports up to 1024 selected points");result.push_back(v);};
        auto surfacePoints=[&](PyObject* face){Ref surface(PyObject_GetAttrString(face,"Surface"));if(!PyObject_HasAttrString(surface.value,"getPoles"))return false;Ref rows(PyObject_CallMethod(surface.value,"getPoles",nullptr));for(Py_ssize_t i=0;i<PySequence_Size(rows.value);++i){Ref row(PySequence_GetItem(rows.value,i));for(Py_ssize_t j=0;j<PySequence_Size(row.value);++j){Ref p(PySequence_GetItem(row.value,j));append(p.value,parent.value);}}return true;};
        if(PyObject_HasAttrString(object.value,"Shape")){
            Ref shape(PyObject_GetAttrString(object.value,"Shape"));
            if(sel.SubName&&*sel.SubName){if(surfacePoles&&std::string(sel.SubName).starts_with("Face")){Ref face(PyObject_CallMethod(shape.value,"getElement","s",sel.SubName));if(!surfacePoints(face.value))throw std::runtime_error("Selected face has no surface control points");}else{if(!std::string(sel.SubName).starts_with("Vertex"))throw std::runtime_error("FitPoints subelements must be vertices or supported surface faces");Ref vertex(PyObject_CallMethod(shape.value,"getElement","s",sel.SubName)),p(PyObject_GetAttrString(vertex.value,"Point"));append(p.value,parent.value);}}
            else {Ref edges(PyObject_GetAttrString(shape.value,"Edges"));bool poles=false;
                if(surfacePoles){Ref faces(PyObject_GetAttrString(shape.value,"Faces"));if(PySequence_Size(faces.value)==1){Ref face(PySequence_GetItem(faces.value,0));poles=surfacePoints(face.value);}}
                if(!poles&&PySequence_Size(edges.value)==1){Ref edge(PySequence_GetItem(edges.value,0)),curve(PyObject_GetAttrString(edge.value,"Curve"));if(PyObject_HasAttrString(curve.value,"getPoles")){Ref values(PyObject_CallMethod(curve.value,"getPoles",nullptr));for(Py_ssize_t i=0;i<PySequence_Size(values.value);++i){Ref p(PySequence_GetItem(values.value,i));append(p.value,parent.value);}poles=true;}}
                if(!poles){Ref vertices(PyObject_GetAttrString(shape.value,"Vertexes"));for(Py_ssize_t i=0;i<PySequence_Size(vertices.value);++i){Ref vertex(PySequence_GetItem(vertices.value,i)),p(PyObject_GetAttrString(vertex.value,"Point"));append(p.value,parent.value);}}
            }
        }else if(PyObject_HasAttrString(object.value,"Mesh")){Ref mesh(PyObject_GetAttrString(object.value,"Mesh")),topology(PyObject_GetAttrString(mesh.value,"Topology")),points(PySequence_GetItem(topology.value,0));
            if(sel.SubName&&*sel.SubName)throw std::runtime_error("Select the whole mesh to fit its vertices");for(Py_ssize_t i=0;i<PySequence_Size(points.value);++i){Ref p(PySequence_GetItem(points.value,i));append(p.value,parent.value);}
        }else if(PyObject_HasAttrString(object.value,"Points")){Ref cloud(PyObject_GetAttrString(object.value,"Points")),points(PyObject_GetAttrString(cloud.value,"Points"));for(Py_ssize_t i=0;i<PySequence_Size(points.value);++i){Ref p(PySequence_GetItem(points.value,i));append(p.value,parent.value);}}
        else throw std::runtime_error("FitPoints accepts native vertices, curve poles, mesh vertices or point clouds");
    }
    return result;
}
std::vector<SolidPoint> solidCurvePlanePoints(const SolidCurveReference& ref){
    Ref shape(PyObject_CallMethod(ref.edge->value,"toNurbs",nullptr)),edges(PyObject_GetAttrString(shape.value,"Edges")),edge(PySequence_GetItem(edges.value,0)),curve(PyObject_GetAttrString(edge.value,"Curve")),poles(PyObject_CallMethod(curve.value,"getPoles",nullptr));
    const auto count=PySequence_Size(poles.value);if(count<2||count>10000)throw std::runtime_error("Invalid tangent curve plane evidence");
    std::vector<SolidPoint> result;result.reserve(count);for(Py_ssize_t i=0;i<count;++i){Ref p(PySequence_GetItem(poles.value,i));result.push_back(point(p.value));}return result;
}
double solidCurveFraction(const SolidCurveReference& ref,const SolidPoint& p){
    const double first=number(ref.edge->value,"FirstParameter"),last=number(ref.edge->value,"LastParameter");
    Ref curve(PyObject_GetAttrString(ref.edge->value,"Curve")),v(vector(p)),parameter(PyObject_CallMethod(curve.value,"parameter","O",v.value));double u=PyFloat_AsDouble(parameter.value);
    if(PyErr_Occurred()||!std::isfinite(u)||last-first<=1e-12)throw std::runtime_error("Cannot record Circle path parameter");
    Ref periodic(PyObject_CallMethod(curve.value,"isPeriodic",nullptr));if(PyObject_IsTrue(periodic.value)==1){Ref value(PyObject_CallMethod(curve.value,"period",nullptr));const double period=PyFloat_AsDouble(value.value);if(period>0)u+=std::ceil((first-u)/period)*period;}
    return std::clamp((u-first)/(last-first),0.,1.);
}
void verifySolidReferences(App::Document& doc,const std::map<std::size_t,SolidCurveReference>& refs,const std::optional<SolidCurveReference>& path){
    std::vector<EditInput> inputs;for(const auto& [_,ref]:refs)inputs.push_back(ref.input);if(path)inputs.push_back(path->input);verifyEditInputs(doc,inputs);
}
std::pair<SolidPoint,double> solidTangentSphere(const std::map<std::size_t,SolidCurveReference>& refs,const double* b,std::size_t count,const double* frame,double radius,int solution){
    return nativeTangentCircle(refs,b,count,frame,radius,solution,false);
}
std::pair<SolidPoint,double> nativeTangentCircle(const std::map<std::size_t,SolidCurveReference>& refs,const double* b,std::size_t count,const double* frame,double radius,int solution,bool fromFirst,double* contacts){
    if(count<2||count>3||((radius>0)!=(count==2)))throw std::runtime_error("Select three tangent/Point constraints, or two and a Radius");
    try{Plane plane(frame);std::vector<Constraint> curves,points;
        for(std::size_t i=0;i<count;++i){const SolidPoint pick{b[i*4],b[i*4+1],b[i*4+2]};if(b[i*4+3]!=0){if(i==0&&fromFirst)throw std::runtime_error("FromFirstPoint requires a curve first");Constraint p;p.isPoint=true;p.pick=plane.project(pick);points.push_back(p);}else{const auto it=refs.find(i);if(it==refs.end())throw std::runtime_error("A tangent curve reference is missing; Undo and reselect");auto c=curveConstraint(it->second,plane,pick);c.fixed=i==0&&fromFirst;curves.push_back(c);}}
        std::vector<Constraint> ordered=curves;ordered.insert(ordered.end(),points.begin(),points.end());std::vector<Candidate> candidates;
        auto pointHandle=[&](std::size_t i)->occ::handle<Geom2d_Point>{return new Geom2d_CartesianPoint(points.at(i).pick);};
        if(radius>0){
            if(curves.size()==2){Geom2dGcc_Circ2d2TanRad solver(curves[0].qualified(),curves[1].qualified(),radius,1e-7);collect(solver,ordered,candidates);}
            else if(curves.size()==1){Geom2dGcc_Circ2d2TanRad solver(curves[0].qualified(),pointHandle(0),radius,1e-7);collect(solver,ordered,candidates);}
            else{Geom2dGcc_Circ2d2TanRad solver(pointHandle(0),pointHandle(1),radius,1e-7);collect(solver,ordered,candidates);}
        }else{
            // Analytic curves enumerate solutions; generic NURBS get bounded multistart.
            const bool general=std::any_of(curves.begin(),curves.end(),[](const auto& c){return c.general;});const int trials=general?9:1;
            for(int trial=0;trial<trials;++trial){std::vector<double> seeds;for(std::size_t i=0;i<curves.size();++i){const auto& c=curves[i];const double shift=trial==0?0.:((trial+int(i)*3)%8-3.5)/8.*(c.last-c.first);seeds.push_back(std::clamp(c.seed+shift,c.first,c.last));}
                if(curves.size()==3){Geom2dGcc_Circ2d3Tan solver(curves[0].qualified(),curves[1].qualified(),curves[2].qualified(),1e-7,seeds[0],seeds[1],seeds[2]);collect(solver,ordered,candidates);}
                else if(curves.size()==2){Geom2dGcc_Circ2d3Tan solver(curves[0].qualified(),curves[1].qualified(),pointHandle(0),1e-7,seeds[0],seeds[1]);collect(solver,ordered,candidates);}
                else if(curves.size()==1){Geom2dGcc_Circ2d3Tan solver(curves[0].qualified(),pointHandle(0),pointHandle(1),1e-7,seeds[0]);collect(solver,ordered,candidates);}
                else{Geom2dGcc_Circ2d3Tan solver(pointHandle(0),pointHandle(1),pointHandle(2),1e-7);collect(solver,ordered,candidates);}
            }
        }
        if(candidates.empty())throw std::runtime_error("No valid tangent circle on the selected bounded curves");
        std::sort(candidates.begin(),candidates.end(),[](const auto& a,const auto& b){if(a.score!=b.score)return a.score<b.score;if(a.center.X()!=b.center.X())return a.center.X()<b.center.X();if(a.center.Y()!=b.center.Y())return a.center.Y()<b.center.Y();return a.radius<b.radius;});
        if(solution<0 && candidates.size()>1 && std::abs(candidates[0].score-candidates[1].score)<1e-8*std::max(1.,candidates[0].score))throw std::runtime_error("Ambiguous tangent branches; choose Solution=1.."+std::to_string(candidates.size()));
        const std::size_t index=solution<0?0:std::size_t(solution);if(index>=candidates.size())throw std::runtime_error("Solution index exceeds validated branches");
        if(contacts){std::size_t curve=0;for(std::size_t i=0;i<count;++i)if(!b[i*4+3]){const auto p=plane.world(candidates[index].contacts[curve++]);std::copy(p.begin(),p.end(),contacts+i*3);}}
        return {plane.world(candidates[index].center),candidates[index].radius};
    }catch(const Standard_Failure& e){throw std::runtime_error(std::string("Native tangent solver: ")+(e.GetMessageString()?e.GetMessageString():"failure"));}
}
}
