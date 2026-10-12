// SPDX-License-Identifier: LGPL-2.1-or-later
#include "SurfaceRefit.h"
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <GeomConvert.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <TopoDS.hxx>
#include <algorithm>
#include <cmath>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <vector>
namespace {
struct Ref {PyObject* p;explicit Ref(PyObject* p):p(p){if(!p){PyErr_Clear();throw std::runtime_error("Cannot inspect native curve");}}~Ref(){Py_XDECREF(p);}};
TopoDS_Wire wire(PyObject* input){
    Ref data(PyObject_CallMethod(input,"exportBrepToString",nullptr));const char* text=PyUnicode_AsUTF8(data.p);
    if(!text)throw std::runtime_error("Invalid curve exchange");
    std::istringstream stream(text);TopoDS_Shape shape;BRep_Builder b;BRepTools::Read(shape,stream,b);
    if(shape.IsNull()||shape.ShapeType()!=TopAbs_WIRE)throw std::runtime_error("Expected a native wire");return TopoDS::Wire(shape);
}
struct Segment {TopoDS_Edge edge;occ::handle<Geom_Curve> curve;double a,b,length,offset;bool reverse;};
std::vector<Segment> segments(const TopoDS_Wire& wire){
    std::vector<Segment> out;double offset=0;
    for(BRepTools_WireExplorer e(wire);e.More();e.Next()){
        const auto edge=e.Current();double a,b;TopLoc_Location location;auto curve=BRep_Tool::Curve(edge,location,a,b);
        if(curve.IsNull())throw std::runtime_error("Curve has no native geometry");
        curve=occ::handle<Geom_Curve>::DownCast(curve->Transformed(location.Transformation()));
        BRepAdaptor_Curve adapt(edge);const double length=GCPnts_AbscissaPoint::Length(adapt,1e-10);
        if(!std::isfinite(length)||length<=1e-9)throw std::runtime_error("Degenerate curve segment");
        out.push_back({edge,curve,a,b,length,offset,edge.Orientation()==TopAbs_REVERSED});offset+=length;
    }
    if(out.empty()||out.size()>256)throw std::runtime_error("Use 1 to 256 curve edges");return out;
}
gp_Pnt value(const Segment& s,double distance){
    const double start=s.reverse?s.b:s.a;
    if(distance<=1e-12)return s.curve->Value(start);
    if(distance>=s.length-1e-12)return s.curve->Value(s.reverse?s.a:s.b);
    BRepAdaptor_Curve adapt(s.edge);GCPnts_AbscissaPoint p(1e-10,adapt,s.reverse?-distance:distance,start);
    if(!p.IsDone())throw std::runtime_error("Cannot locate rail arc-length position");return s.curve->Value(p.Parameter());
}
double distanceToChord(const gp_Pnt& p,const gp_Pnt& a,const gp_Pnt& b){
    const gp_Vec v(a,b);const double n=v.SquareMagnitude();
    const double t=n<1e-25?0:std::clamp(gp_Vec(a,p).Dot(v)/n,0.,1.);
    return p.Distance(a.Translated(v*t));
}
double coordinateScale(const gp_Pnt& p){return std::max({1.,std::abs(p.X()),std::abs(p.Y()),std::abs(p.Z())});}
double hull(occ::handle<Geom_Curve> curve,double a,double b){
    if(a==b)return 0;
    if(!std::isfinite(a)||!std::isfinite(b)||b<a)throw std::runtime_error("Invalid Refit certificate interval");
    occ::handle<Geom_TrimmedCurve> trimmed=new Geom_TrimmedCurve(curve,a,b);
    auto spline=GeomConvert::CurveToBSplineCurve(trimmed);const auto p=curve->Value(a),q=curve->Value(b);double bound=0;
    double scale=std::max(coordinateScale(p),coordinateScale(q));
    for(int i=1;i<=spline->NbPoles();++i){
        const double w=spline->Weight(i);
        if(!std::isfinite(w)||w<=0)throw std::runtime_error("Refit certificate requires positive finite curve weights");
        bound=std::max(bound,distanceToChord(spline->Pole(i),p,q));
        scale=std::max(scale,coordinateScale(spline->Pole(i)));
    }
    // Numerical allowance grows with world coordinates, including poles.
    // Tolerances below this allowance must fail rather than claim precision
    // that native floating-point curve conversion cannot establish.
    return bound+64*std::numeric_limits<double>::epsilon()*scale+1e-9;
}
}
namespace OpenMatrix9Gui {
std::array<double,3> surfaceRailPoint(PyObject* input,double fraction){
    if(!std::isfinite(fraction)||fraction<0||fraction>1)throw std::runtime_error("Rail fraction must be between zero and one");
    try{const auto list=segments(wire(input));const double length=list.back().offset+list.back().length;double distance=fraction*length;
        for(const auto& s:list)if(distance<=s.offset+s.length+1e-10){const auto p=value(s,std::clamp(distance-s.offset,0.,s.length));return {p.X(),p.Y(),p.Z()};}
        throw std::runtime_error("Cannot evaluate rail");
    }catch(const Standard_Failure& e){throw std::runtime_error(e.GetMessageString());}
}
double surfaceRailFraction(PyObject* input,const std::array<double,3>& point){
    try{const auto list=segments(wire(input));const gp_Pnt p(point[0],point[1],point[2]);double best=1e100,position=0;
        for(const auto& s:list){GeomAPI_ProjectPointOnCurve project(p,s.curve,s.a,s.b);
            if(project.NbPoints()&&project.LowerDistance()<best){best=project.LowerDistance();BRepAdaptor_Curve adapt(s.edge);
                double length=GCPnts_AbscissaPoint::Length(adapt,s.a,project.LowerDistanceParameter(),1e-10);
                position=s.offset+(s.reverse?s.length-length:length);}}
        if(best==1e100)throw std::runtime_error("Cannot project picked rail point");
        return std::clamp(position/(list.back().offset+list.back().length),0.,1.);
    }catch(const Standard_Failure& e){throw std::runtime_error(e.GetMessageString());}
}
double certifySurfaceRefit(PyObject* original,PyObject* candidate,double tolerance){
    if(!std::isfinite(tolerance)||tolerance<=0)throw std::runtime_error("Invalid Refit tolerance");
    try {
        const auto source=segments(wire(original)),fit=segments(wire(candidate));
        if(fit.size()!=1)throw std::runtime_error("Refit certificate needs one fitted spline edge");
        const auto& target=fit.front();const double total=source.back().offset+source.back().length;
        double accepted=0;unsigned nodes=0;
        // An interval's positive NURBS hull lies within flatness of its chord.
        // Conversely the curve's continuous projection covers that chord.
        // Add the endpoint chord distance and both flatness bounds: symmetric
        // Hausdorff bound for the two continuous curve intervals, not samples.
        auto certify=[&](auto&& self,const Segment& s,double a,double b,unsigned depth)->void {
            if(++nodes>16384||depth>24)throw std::runtime_error("Refit could not certify tolerance within the bounded subdivision budget");
            BRepAdaptor_Curve adapt(s.edge);
            auto fraction=[&](double u){const double l=GCPnts_AbscissaPoint::Length(adapt,s.a,u,1e-10);
                return std::clamp((s.offset+(s.reverse?s.length-l:l))/total,0.,1.);};
            const double fa=fraction(a),fb=fraction(b);
            const double ta=target.a+fa*(target.b-target.a),tb=target.a+fb*(target.b-target.a);
            const auto pa=s.curve->Value(a),pb=s.curve->Value(b),qa=target.curve->Value(ta),qb=target.curve->Value(tb);
            const double endpoints=std::max(pa.Distance(qa),pb.Distance(qb));
            if(!std::isfinite(endpoints)||endpoints>tolerance)throw std::runtime_error("Refit candidate exceeds the requested geometric tolerance");
            const double scale=std::max({coordinateScale(pa),coordinateScale(pb),coordinateScale(qa),coordinateScale(qb)});
            const double padding=64*std::numeric_limits<double>::epsilon()*scale+1e-9;
            if(padding>tolerance)throw std::runtime_error("Refit tolerance is below native numerical resolution at these world coordinates");
            const double bound=endpoints+hull(s.curve,a,b)+hull(target.curve,std::min(ta,tb),std::max(ta,tb))+padding;
            if(std::isfinite(bound)&&bound<=tolerance){accepted=std::max(accepted,bound);return;}
            const double mid=a+(b-a)/2;
            if(mid==a||mid==b)throw std::runtime_error("Refit certificate interval is below native parameter resolution");
            self(self,s,a,mid,depth+1);self(self,s,mid,b,depth+1);
        };
        for(const auto& s:source)certify(certify,s,s.a,s.b,0);
        return accepted;
    }catch(const Standard_Failure& e){throw std::runtime_error(e.GetMessageString());}
}
}
