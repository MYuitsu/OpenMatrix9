// SPDX-License-Identifier: LGPL-2.1-or-later
#include "EditGeometry.h"
#include <algorithm>
#include <cmath>
#include <limits>
namespace {
using namespace OpenMatrix9Gui;
using Ref=CurvePyRef;
using Point=std::array<double,3>;
double number(PyObject* o,const char* key){Ref v(PyObject_GetAttrString(o,key));double d=PyFloat_AsDouble(v.value);if(PyErr_Occurred()||!std::isfinite(d))throw std::runtime_error("Invalid curve coordinate");return d;}
Point point(PyObject* p){return {number(p,"x"),number(p,"y"),number(p,"z")};}
Ref vector(const Point& p){Ref app(PyImport_ImportModule("FreeCAD"));return Ref(PyObject_CallMethod(app.value,"Vector","ddd",p[0],p[1],p[2]));}
double distance(const Point& a,const Point& b){double s=0;for(unsigned i=0;i<3;++i)s+=(a[i]-b[i])*(a[i]-b[i]);return std::sqrt(s);}
EditShape owned(PyObject* p){return std::make_shared<Ref>(p);}
EditShapes elements(PyObject* p,const char* key){Ref list(PyObject_GetAttrString(p,key));EditShapes r;for(Py_ssize_t i=0;i<PySequence_Size(list.value);++i)r.push_back(owned(PySequence_GetItem(list.value,i)));return r;}
bool isLine(PyObject* curve){Ref part(PyImport_ImportModule("Part")),type(PyObject_GetAttrString(part.value,"Line"));return PyObject_IsInstance(curve,type.value)==1;}
Ref value(PyObject* curve,double parameter){return Ref(PyObject_CallMethod(curve,"value","d",parameter));}
Ref spline(PyObject* edge){Ref curve(PyObject_GetAttrString(edge,"Curve"));return Ref(PyObject_CallMethod(curve.value,"toBSpline","dd",number(edge,"FirstParameter"),number(edge,"LastParameter")));}
void append(PyObject* list,PyObject* item){if(PyList_Append(list,item)<0)throw std::runtime_error("Cannot collect curve options");}
struct Projected {
    EditShape original,bSpline,curve,edge;
    std::size_t source;
    std::vector<double> cuts;
};
Projected projected(const EditShape& edge,std::size_t source,const EditProjection& frame){
    auto converted=spline(edge->value);auto bs=owned(Py_NewRef(converted.value));
    Ref poles(PyObject_CallMethod(bs->value,"getPoles",nullptr)),weights(PyObject_CallMethod(bs->value,"getWeights",nullptr)),newPoles(PyList_New(0)),newWeights(PyList_New(0));
    Point previous{};double diameter=0;
    for(Py_ssize_t i=0;i<PySequence_Size(poles.value);++i){Ref pole(PySequence_GetItem(poles.value,i)),weight(PySequence_GetItem(weights.value,i));const auto world=point(pole.value);Point relative;for(unsigned k=0;k<3;++k)relative[k]=world[k]-frame.eye[k];double x=0,y=0,z=0;for(unsigned k=0;k<3;++k){x+=relative[k]*frame.right[k];y+=relative[k]*frame.up[k];z+=relative[k]*frame.forward[k];}
        double w=PyFloat_AsDouble(weight.value);if(frame.perspective){if(z<=1e-9)throw std::runtime_error("Curve crosses or lies behind the frozen camera plane");x/=z;y/=z;w*=z;}
        Point plane{x,y,0};if(i)diameter=std::max(diameter,distance(previous,plane));previous=plane;auto p=vector(plane);Ref wn(PyFloat_FromDouble(w));append(newPoles.value,p.value);append(newWeights.value,wn.value);
    }
    if(diameter<1e-12)throw std::runtime_error("Curve projection collapses to a point; choose another view");
    Ref mults(PyObject_CallMethod(bs->value,"getMultiplicities",nullptr)),knots(PyObject_CallMethod(bs->value,"getKnots",nullptr)),periodic(PyObject_CallMethod(bs->value,"isPeriodic",nullptr)),degree(PyObject_GetAttrString(bs->value,"Degree")),part(PyImport_ImportModule("Part"));auto curve=owned(PyObject_CallMethod(part.value,"BSplineCurve",nullptr));
    Ref built(PyObject_CallMethod(curve->value,"buildFromPolesMultsKnots","OOOOiO",newPoles.value,mults.value,knots.value,periodic.value,int(PyLong_AsLong(degree.value)),newWeights.value));
    auto flat=owned(PyObject_CallMethod(curve->value,"toShape",nullptr));
    return {edge,bs,curve,flat,source,{number(edge->value,"FirstParameter"),number(edge->value,"LastParameter")}};
}
}
namespace OpenMatrix9Gui {
EditShape joinEditCurves(PyObject* list,double tolerance){
    Ref part(PyImport_ImportModule("Part"));EditShapes edges;std::vector<Point> ends;
    for(Py_ssize_t i=0;i<PySequence_Size(list);++i){auto e=owned(PySequence_GetItem(list,i));Ref c(PyObject_GetAttrString(e->value,"Curve"));auto a=value(c.value,number(e->value,"FirstParameter")),b=value(c.value,number(e->value,"LastParameter"));edges.push_back(e);ends.push_back(point(a.value));ends.push_back(point(b.value));}
    auto moved=ends;
    std::vector<bool> reached(edges.size(),false);if(!reached.empty())reached[0]=true;
    for(std::size_t pass=0;pass<edges.size();++pass)for(std::size_t i=0;i<ends.size();++i)if(reached[i/2])for(std::size_t j=0;j<ends.size();++j)if(i/2!=j/2&&distance(ends[i],ends[j])<=tolerance)reached[j/2]=true;
    if(std::find(reached.begin(),reached.end(),false)!=reached.end())throw std::runtime_error("Curves are disconnected at the explicit Join tolerance");
    for(std::size_t i=0;i<ends.size();++i){std::vector<std::size_t> near;for(std::size_t j=0;j<ends.size();++j)if(i/2!=j/2&&distance(ends[i],ends[j])<=tolerance)near.push_back(j);
        if(near.size()>1)throw std::runtime_error("Join endpoints form an ambiguous branch within tolerance");
        if(near.size()==1)for(unsigned k=0;k<3;++k)moved[i][k]=ends[i][k]+(ends[near[0]][k]-ends[i][k])/2;
    }
    Ref joined(PyList_New(0));
    for(std::size_t i=0;i<edges.size();++i){auto e=edges[i];if(moved[2*i]!=ends[2*i]||moved[2*i+1]!=ends[2*i+1]){
        auto a=vector(moved[2*i]),b=vector(moved[2*i+1]);Ref curve(PyObject_GetAttrString(e->value,"Curve"));
        if(isLine(curve.value))e=owned(PyObject_CallMethod(part.value,"makeLine","OO",a.value,b.value));
        else{auto bs=spline(e->value);Ref count(PyObject_GetAttrString(bs.value,"NbPoles"));Ref sa(PyObject_CallMethod(bs.value,"setPole","iO",1,a.value)),sb(PyObject_CallMethod(bs.value,"setPole","iO",int(PyLong_AsLong(count.value)),b.value));e=owned(PyObject_CallMethod(bs.value,"toShape",nullptr));}
    }append(joined.value,e->value);}
    // Endpoint modification is explicit and bounded; kernel ordering gets no extra fuzzy tolerance.
    Ref sorted(PyObject_CallMethod(part.value,"sortEdges","Od",joined.value,std::min(tolerance,1e-7)));if(PySequence_Size(sorted.value)!=1)throw std::runtime_error("Curves must meet within the explicit Join tolerance in one chain");Ref chain(PySequence_GetItem(sorted.value,0));return owned(PyObject_CallMethod(part.value,"Wire","O",chain.value));
}
EditShapes extendedEditLines(const std::vector<EditInput>& inputs){
    Point lo{std::numeric_limits<double>::infinity(),std::numeric_limits<double>::infinity(),std::numeric_limits<double>::infinity()},hi{-lo[0],-lo[1],-lo[2]};
    for(const auto& i:inputs){Ref box(PyObject_GetAttrString(i.shape->value,"BoundBox"));const char* mins[]={"XMin","YMin","ZMin"};const char* maxs[]={"XMax","YMax","ZMax"};for(unsigned k=0;k<3;++k){lo[k]=std::min(lo[k],number(box.value,mins[k]));hi[k]=std::max(hi[k],number(box.value,maxs[k]));}}
    const double margin=std::max(1.0,2*distance(lo,hi));Ref part(PyImport_ImportModule("Part"));EditShapes result;
    for(const auto& i:inputs)for(auto& e:elements(i.shape->value,"Edges")){Ref curve(PyObject_GetAttrString(e->value,"Curve"));if(!isLine(curve.value))continue;auto av=value(curve.value,number(e->value,"FirstParameter")),bv=value(curve.value,number(e->value,"LastParameter"));auto a=point(av.value),b=point(bv.value);double length=distance(a,b);if(length<=1e-12)throw std::runtime_error("Degenerate line cutter");Point aa,bb;for(unsigned k=0;k<3;++k){double d=(b[k]-a[k])/length;aa[k]=a[k]-margin*d;bb[k]=b[k]+margin*d;}auto va=vector(aa),vb=vector(bb);result.push_back(owned(PyObject_CallMethod(part.value,"makeLine","OO",va.value,vb.value)));}
    return result;
}
std::vector<EditShapes> splitProjectedEditCurves(const std::vector<EditInput>& inputs,const EditProjection& frame,bool extendLines){
    std::vector<Projected> curves;for(std::size_t i=0;i<inputs.size();++i){if(!elements(inputs[i].shape->value,"Faces").empty())throw std::runtime_error("ApparentIntersections supports curves only, not surfaces");for(auto& e:elements(inputs[i].shape->value,"Edges"))curves.push_back(projected(e,i,frame));}
    EditShapes cutters;if(extendLines)for(auto& e:extendedEditLines(inputs)){
        if(frame.perspective){Ref curve(PyObject_GetAttrString(e->value,"Curve"));auto av=value(curve.value,number(e->value,"FirstParameter")),bv=value(curve.value,number(e->value,"LastParameter"));Point a=point(av.value),b=point(bv.value);auto depth=[&](const Point& p){double d=0;for(unsigned k=0;k<3;++k)d+=(p[k]-frame.eye[k])*frame.forward[k];return d;};const double da=depth(a),db=depth(b),near=std::max(1e-7,std::max(da,db)*1e-6);
            // Virtual endpoints may cross the eye plane although the originals
            // are visible. Clip only the imaginary cutter, never an input curve.
            if(da<=near&&db<=near)continue;if(da<=near||db<=near){const double t=(near-da)/(db-da);Point clipped;for(unsigned k=0;k<3;++k)clipped[k]=a[k]+t*(b[k]-a[k]);if(da<=near)a=clipped;else b=clipped;Ref part(PyImport_ImportModule("Part"));auto aa=vector(a),bb=vector(b);e=owned(PyObject_CallMethod(part.value,"makeLine","OO",aa.value,bb.value));}
        }
        cutters.push_back(projected(e,inputs.size(),frame).edge);
    }
    for(std::size_t i=0;i<curves.size();++i){auto& c=curves[i];auto cutWith=[&](PyObject* other){Ref section(PyObject_CallMethod(c.edge->value,"section","O",other));if(!elements(section.value,"Edges").empty())throw std::runtime_error("Overlapping projected curves are ambiguous; choose another view");
        for(auto& v:elements(section.value,"Vertexes")){Ref p(PyObject_GetAttrString(v->value,"Point")),u(PyObject_CallMethod(c.curve->value,"parameter","O",p.value));double parameter=PyFloat_AsDouble(u.value);auto world=value(c.bSpline->value,parameter);Ref original(PyObject_GetAttrString(c.original->value,"Curve")),t(PyObject_CallMethod(original.value,"parameter","O",world.value));double native=PyFloat_AsDouble(t.value);const auto a=c.cuts.front(),b=c.cuts.back();if(native>a+1e-10&&native<b-1e-10)c.cuts.insert(c.cuts.end()-1,native);}
    };
        for(std::size_t j=0;j<curves.size();++j)if(i!=j&&curves[j].source!=c.source)cutWith(curves[j].edge->value);
        for(auto& cutter:cutters){Ref same(PyObject_CallMethod(c.edge->value,"common","O",cutter->value));if(!elements(same.value,"Edges").empty())continue;cutWith(cutter->value);}
    }
    std::vector<EditShapes> result(inputs.size());bool changed=false;
    for(auto& c:curves){std::sort(c.cuts.begin(),c.cuts.end());c.cuts.erase(std::unique(c.cuts.begin(),c.cuts.end(),[](double a,double b){return std::abs(a-b)<=1e-10;}),c.cuts.end());changed|=c.cuts.size()>2;Ref curve(PyObject_GetAttrString(c.original->value,"Curve"));for(std::size_t j=1;j<c.cuts.size();++j)result[c.source].push_back(owned(PyObject_CallMethod(curve.value,"toShape","dd",c.cuts[j-1],c.cuts[j])));}
    if(!changed)throw std::runtime_error("No intersections in the frozen view projection");return result;
}
}
