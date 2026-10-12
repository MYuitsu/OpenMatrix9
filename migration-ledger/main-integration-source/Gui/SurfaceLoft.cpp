// SPDX-License-Identifier: LGPL-2.1-or-later
#include "SurfaceLoft.h"
#include "RustBridge.h"
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepOffsetAPI_ThruSections.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <GeomFill_Profiler.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <GeomLProp_SLProps.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopExp_Explorer.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <cmath>
#include <sstream>
#include <stdexcept>
#include <vector>

namespace {
struct Ref {
    PyObject* p;
    explicit Ref(PyObject* value):p(value) {if(!p){PyErr_Clear();throw std::runtime_error("Cannot exchange native Part geometry");}}
    ~Ref(){Py_XDECREF(p);}
    Ref(const Ref&)=delete;
};
TopoDS_Wire nativeWire(PyObject* object) {
    Ref text(PyObject_CallMethod(object,"exportBrepToString",nullptr));
    const char* data=PyUnicode_AsUTF8(text.p);
    if(!data)throw std::runtime_error("Invalid native wire data");
    std::istringstream stream(data);TopoDS_Shape shape;BRep_Builder builder;
    BRepTools::Read(shape,stream,builder);
    if(shape.IsNull()||shape.ShapeType()!=TopAbs_WIRE)throw std::runtime_error("Loft needs native wires");
    return TopoDS::Wire(shape);
}
TopoDS_Shape controlNet(const std::vector<TopoDS_Wire>& wires,bool closed,bool uniform) {
    GeomFill_Profiler profiler;
    for(const auto& wire:wires){
        TopExp_Explorer edges(wire,TopAbs_EDGE);
        if(!edges.More())throw std::runtime_error("Empty Loft profile");
        const auto edge=TopoDS::Edge(edges.Current());edges.Next();
        if(edges.More())throw std::runtime_error("Loose/Uniform currently need single-edge profiles; use section Rebuild for joined curves");
        Standard_Real first,last;TopLoc_Location location;
        auto curve=BRep_Tool::Curve(edge,location,first,last);
        if(curve.IsNull())throw std::runtime_error("Profile has no native curve");
        curve=occ::handle<Geom_Curve>::DownCast(curve->Transformed(location.Transformation()));
        occ::handle<Geom_TrimmedCurve> trimmed=new Geom_TrimmedCurve(curve,first,last);
        if(edge.Orientation()==TopAbs_REVERSED)trimmed->Reverse();
        profiler.AddCurve(trimmed);
    }
    profiler.Perform(1e-9);
    const int nu=profiler.NbPoles(),nv=int(wires.size());
    int degree=std::min(3,nv-1);if(uniform&&closed&&degree%2==0)--degree;
    if(nu>4096||nv>256)throw std::runtime_error("Loose control net exceeds host limits");
    if(uniform&&std::uint64_t(nu)*nv*nv*nv>50000000)
        throw std::runtime_error("Uniform control net exceeds the bounded solve cost; reduce sections or Rebuild point count");
    TColgp_Array2OfPnt poles(1,nu,1,nv);TColStd_Array2OfReal weights(1,nu,1,nv);
    for(int j=1;j<=nv;++j){
        TColgp_Array1OfPnt row(1,nu);TColStd_Array1OfReal w(1,nu);
        profiler.Poles(j,row);profiler.Weights(j,w);
        for(int i=1;i<=nu;++i){poles.SetValue(i,j,row(i));weights.SetValue(i,j,w(i));}
    }
    TColStd_Array1OfReal u(1,profiler.NbKnots());TColStd_Array1OfInteger um(1,profiler.NbKnots());
    profiler.KnotsAndMults(u,um);
    if(uniform){
        const double step=u(2)-u(1);
        for(int i=2;i<u.Upper();++i)if(std::abs(u(i+1)-u(i)-step)>1e-8*std::max(1.,std::abs(step)))
            throw std::runtime_error("Uniform requires uniformly knotted profiles; enable section Rebuild");
    }
    const int nk=closed?nv+1:nv-degree+1;
    TColStd_Array1OfReal v(1,nk);TColStd_Array1OfInteger vm(1,nk);
    for(int j=1;j<=nk;++j){v.SetValue(j,double(j-1)/double(nk-1));vm.SetValue(j,closed?1:(j==1||j==nk?degree+1:1));}
    if(uniform){
        for(int i=1;i<=nu;++i){
            std::vector<double> xyz,weightRows;
            for(int j=1;j<=nv;++j){const auto& p=poles(i,j);const double w=weights(i,j);
                xyz.insert(xyz.end(),{p.X()*w,p.Y()*w,p.Z()*w});weightRows.insert(weightRows.end(),{w,0.,0.});}
            auto fit=[&](const std::vector<double>& rows){
                if(!om9_surface_uniform_spline(rows.data(),nv,closed)){
                    char message[2048]={};om9_spline_message(message,sizeof(message));throw std::runtime_error(message);}
                if(om9_spline_pole_count()!=std::size_t(nv)||om9_spline_knot_count()!=std::size_t(nk))
                    throw std::runtime_error("Uniform returned inconsistent control net");
            };
            fit(xyz);std::vector<gp_Pnt> homogeneous;
            for(int j=0;j<nv;++j)homogeneous.emplace_back(om9_spline_pole(j,0),om9_spline_pole(j,1),om9_spline_pole(j,2));
            fit(weightRows);
            for(int j=1;j<=nv;++j){const double w=om9_spline_pole(j-1,0);
                if(!std::isfinite(w)||w<=1e-10)throw std::runtime_error("Uniform interpolation would create a nonpositive rational weight; use section Rebuild");
                const auto& h=homogeneous[j-1];poles.SetValue(i,j,gp_Pnt(h.X()/w,h.Y()/w,h.Z()/w));weights.SetValue(i,j,w);}
        }
    }
    occ::handle<Geom_BSplineSurface> surface=new Geom_BSplineSurface(
        poles,weights,u,v,um,vm,profiler.Degree(),degree,profiler.IsPeriodic(),closed);
    BRepBuilderAPI_MakeFace face(surface,1e-7);
    if(!face.IsDone())throw std::runtime_error("Loose control-net face construction failed");
    return face.Shape();
}
TopoDS_Shape interpolate(const std::vector<TopoDS_Wire>& wires,unsigned style,bool closed) {
    BRepOffsetAPI_ThruSections loft(false,false,1e-7);
    loft.SetMutableInput(false);
    // Preserve the explicitly chosen directions/seams. Unsupported edge-count
    // correspondence is rejected by the kernel rather than reordered silently.
    loft.CheckCompatibility(false);
    loft.SetParType(Approx_Centripetal);
    loft.SetMaxDegree(3);
    for(const auto& wire:wires)loft.AddWire(wire);
    if(closed)loft.AddWire(wires.front());
    loft.Build();
    if(!loft.IsDone())throw std::runtime_error("Loft style cannot interpolate this profile correspondence");
    return loft.Shape();
}
bool parallelLines(const TopoDS_Wire& a,const TopoDS_Wire& b) {
    TopExp_Explorer ea(a,TopAbs_EDGE),eb(b,TopAbs_EDGE);
    if(!ea.More()||!eb.More())return true;
    auto edgeA=TopoDS::Edge(ea.Current()),edgeB=TopoDS::Edge(eb.Current());ea.Next();eb.Next();
    if(ea.More()||eb.More())return true;
    BRepAdaptor_Curve ca(edgeA),cb(edgeB);
    if(ca.GetType()!=GeomAbs_Line||cb.GetType()!=GeomAbs_Line)return true;
    return ca.Line().Direction().IsParallel(cb.Line().Direction(),1e-7);
}
void checkDevelopable(const TopoDS_Shape& shape) {
    for(TopExp_Explorer faces(shape,TopAbs_FACE);faces.More();faces.Next()){
        auto face=TopoDS::Face(faces.Current());Standard_Real u0,u1,v0,v1;
        BRepTools::UVBounds(face,u0,u1,v0,v1);
        const auto surface=BRep_Tool::Surface(face);
        GProp_GProps props;BRepGProp::SurfaceProperties(face,props);
        if(!std::isfinite(props.Mass())||props.Mass()<1e-10)throw std::runtime_error("Degenerate developable pair");
        const double scale2=std::max(1.,props.Mass());
        // Conservative bounded curvature validation, not an unroll certificate.
        // Refuse arbitrary ruled surfaces with nonzero Gaussian curvature.
        for(int i=1;i<=9;++i)for(int j=1;j<=9;++j){
            GeomLProp_SLProps p(surface,u0+(u1-u0)*i/10.,v0+(v1-v0)*j/10.,2,1e-9);
            if(!p.IsCurvatureDefined()||!std::isfinite(p.GaussianCurvature())||std::abs(p.GaussianCurvature())*scale2>1e-7)
                throw std::runtime_error("Curve pair is not developable within the host curvature check; no partial output created");
        }
    }
}
TopoDS_Shape developable(const std::vector<TopoDS_Wire>& wires) {
    TopoDS_Compound result;BRep_Builder builder;builder.MakeCompound(result);
    for(std::size_t i=1;i<wires.size();++i){
        if(!parallelLines(wires[i-1],wires[i]))throw std::runtime_error("Developable does not support two nonparallel straight profiles");
        BRepOffsetAPI_ThruSections pair(false,true,1e-7);pair.SetMutableInput(false);pair.CheckCompatibility(false);
        pair.AddWire(wires[i-1]);pair.AddWire(wires[i]);pair.Build();
        if(!pair.IsDone())throw std::runtime_error("Cannot create this developable pair; no partial output created");
        checkDevelopable(pair.Shape());builder.Add(result,pair.Shape());
    }
    return result;
}
}
namespace OpenMatrix9Gui {
PyObject* advancedLoft(PyObject* wires,unsigned style,bool closed) {
    try {
        std::vector<TopoDS_Wire> profiles;
        for(Py_ssize_t i=0;i<PyList_Size(wires);++i)profiles.push_back(nativeWire(PyList_GetItem(wires,i)));
        TopoDS_Shape shape=(style==2||style==4)?controlNet(profiles,closed,style==4):style==5?developable(profiles):interpolate(profiles,style,closed);
        std::ostringstream data;BRepTools::Write(shape,data);
        Ref part(PyImport_ImportModule("Part")),constructor(PyObject_GetAttrString(part.p,"Shape")),output(PyObject_CallNoArgs(constructor.p));
        Ref loaded(PyObject_CallMethod(output.p,"importBrepFromString","s",data.str().c_str()));
        Py_INCREF(output.p);return output.p;
    }catch(const Standard_Failure& e){throw std::runtime_error(e.GetMessageString());}
}
}
