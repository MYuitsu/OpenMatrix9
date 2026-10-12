// SPDX-License-Identifier: LGPL-2.1-or-later
#include "SurfaceSeams.h"
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_CompCurve.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <TopoDS.hxx>
#include <gp_Quaternion.hxx>
#include <cmath>
#include <algorithm>
#include <sstream>
#include <stdexcept>
#include <vector>

namespace {
struct Ref {PyObject* p;explicit Ref(PyObject* v):p(v){if(!p){PyErr_Clear();throw std::runtime_error("Native seam operation failed");}}~Ref(){Py_XDECREF(p);}Ref(const Ref&)=delete;};
TopoDS_Wire wire(PyObject* value) {
    Ref text(PyObject_CallMethod(value,"exportBrepToString",nullptr));const char* data=PyUnicode_AsUTF8(text.p);
    if(!data)throw std::runtime_error("Invalid seam input");
    std::istringstream stream(data);TopoDS_Shape shape;BRep_Builder builder;BRepTools::Read(shape,stream,builder);
    if(shape.IsNull()||shape.ShapeType()!=TopAbs_WIRE)throw std::runtime_error("Seam requires a wire");
    return TopoDS::Wire(shape);
}
PyObject* partShape(const TopoDS_Shape& shape) {
    std::ostringstream data;BRepTools::Write(shape,data);
    Ref part(PyImport_ImportModule("Part")),type(PyObject_GetAttrString(part.p,"Shape")),output(PyObject_CallNoArgs(type.p));
    Ref loaded(PyObject_CallMethod(output.p,"importBrepFromString","s",data.str().c_str()));Py_INCREF(output.p);return output.p;
}
struct Edge {TopoDS_Edge shape;double length;};
std::vector<Edge> edges(const TopoDS_Wire& shape) {
    std::vector<Edge> result;
    for(BRepTools_WireExplorer explorer(shape);explorer.More();explorer.Next()){
        auto edge=explorer.Current();BRepAdaptor_Curve c(edge);const double length=GCPnts_AbscissaPoint::Length(c);
        if(!std::isfinite(length)||length<=1e-9)throw std::runtime_error("Seam input contains a degenerate edge");
        result.push_back({edge,length});
    }
    if(result.empty())throw std::runtime_error("Empty seam wire");return result;
}
double fraction(const TopoDS_Wire& shape,const gp_Pnt& point) {
    const auto segments=edges(shape);double total=0;for(const auto& e:segments)total+=e.length;
    double offset=0,best=1e100,position=0;
    for(const auto& e:segments){
        Standard_Real a,b;TopLoc_Location location;auto c=BRep_Tool::Curve(e.shape,location,a,b);
        c=occ::handle<Geom_Curve>::DownCast(c->Transformed(location.Transformation()));
        GeomAPI_ProjectPointOnCurve project(point,c,a,b);
        if(project.NbPoints()&&project.LowerDistance()<best){
            best=project.LowerDistance();BRepAdaptor_Curve adapt(e.shape);
            double local=GCPnts_AbscissaPoint::Length(adapt,a,project.LowerDistanceParameter());
            if(e.shape.Orientation()==TopAbs_REVERSED)local=e.length-local;
            position=offset+local;
        }
        offset+=e.length;
    }
    if(best==1e100)throw std::runtime_error("Cannot project the seam onto this wire");
    const double value=position/total;return value>1-1e-9?0:std::clamp(value,0.,1-1e-9);
}
std::vector<gp_Pnt> points(const TopoDS_Wire& shape,bool closed) {
    BRepAdaptor_CompCurve c(shape);std::vector<gp_Pnt> p;const int n=closed?128:2;
    for(int i=0;i<n;++i)p.push_back(c.Value(c.FirstParameter()+(c.LastParameter()-c.FirstParameter())*i/(closed?n:n-1)));
    return p;
}
gp_Pnt center(const std::vector<gp_Pnt>& p) {gp_XYZ total(0,0,0);for(const auto& v:p)total+=v.XYZ();return gp_Pnt(total/double(p.size()));}
gp_Vec normal(const std::vector<gp_Pnt>& p,const gp_Pnt& c) {
    gp_Vec total;for(std::size_t i=0;i<p.size();++i)total+=gp_Vec(c,p[i]).Crossed(gp_Vec(c,p[(i+1)%p.size()]));
    if(total.Magnitude()<1e-9)throw std::runtime_error("Automatic seams need nondegenerate planar profiles");total.Normalize();return total;
}
}
namespace OpenMatrix9Gui {
PyObject* rotateSurfaceWire(PyObject* input,double seam) {
    try {
        auto shape=wire(input);const auto list=edges(shape);double total=0;for(const auto& e:list)total+=e.length;
        double position=seam*total;std::size_t index=0;while(index+1<list.size()&&position>=list[index].length){position-=list[index++].length;}
        BRepBuilderAPI_MakeWire result;const auto& item=list[index];TopoDS_Edge head,tail;
        if(position<1e-8)tail=item.shape;
        else {
            BRepAdaptor_Curve adapt(item.shape);Standard_Real a,b;TopLoc_Location location;
            auto curve=BRep_Tool::Curve(item.shape,location,a,b);curve=occ::handle<Geom_Curve>::DownCast(curve->Transformed(location.Transformation()));
            const bool reversed=item.shape.Orientation()==TopAbs_REVERSED;
            GCPnts_AbscissaPoint cut(adapt,reversed?-position:position,reversed?b:a);
            if(!cut.IsDone())throw std::runtime_error("Cannot locate wire seam");
            const double t=cut.Parameter();head=BRepBuilderAPI_MakeEdge(curve,a,t);tail=BRepBuilderAPI_MakeEdge(curve,t,b);
            if(reversed){std::swap(head,tail);head.Reverse();tail.Reverse();}
        }
        result.Add(tail);for(std::size_t i=index+1;i<list.size();++i)result.Add(list[i].shape);
        for(std::size_t i=0;i<index;++i)result.Add(list[i].shape);if(!head.IsNull())result.Add(head);
        if(!result.IsDone())throw std::runtime_error("Cannot join rotated seam edges");
        return partShape(result.Wire());
    }catch(const Standard_Failure& e){throw std::runtime_error(e.GetMessageString());}
}
void automaticSurfaceAlignment(PyObject* reference,PyObject* profile,double& seam,bool& reverse) {
    try {
        const auto a=wire(reference),b=wire(profile);const bool closed=a.Closed();
        if(closed!=b.Closed())throw std::runtime_error("Automatic alignment needs matching profile closure");
        const auto pa=points(a,closed),pb=points(b,closed);const auto ca=center(pa),cb=center(pb);
        gp_Vec direction(ca,pa.front());if(direction.Magnitude()<1e-9)throw std::runtime_error("Cannot align a degenerate profile");direction.Normalize();
        reverse=false;seam=0;
        if(!closed){gp_Vec start(cb,pb.front());start.Normalize();reverse=direction.Dot(start)<0;return;}
        auto na=normal(pa,ca),nb=normal(pb,cb);reverse=na.Dot(nb)<0;if(reverse)nb.Reverse();
        gp_Quaternion rotation;rotation.SetRotation(na,nb);direction=gp_Vec(rotation*direction.XYZ());
        double best=1e100;gp_Pnt chosen=pb.front();
        for(const auto& p:pb){gp_Vec v(cb,p);if(v.Magnitude()<1e-9)continue;v.Normalize();const double error=(v-direction).SquareMagnitude();if(error<best){best=error;chosen=p;}}
        // Single edges retain the original normalized parameter definition.
        BRepTools_WireExplorer explorer(b);auto edge=explorer.Current();explorer.Next();
        if(!explorer.More()){
            Standard_Real first,last;TopLoc_Location location;auto c=BRep_Tool::Curve(edge,location,first,last);
            c=occ::handle<Geom_Curve>::DownCast(c->Transformed(location.Transformation()));GeomAPI_ProjectPointOnCurve project(chosen,c,first,last);
            if(!project.NbPoints())throw std::runtime_error("Cannot locate automatic seam");
            seam=(project.LowerDistanceParameter()-first)/(last-first);if(seam>1-1e-9)seam=0;
        }else seam=fraction(b,chosen);
    }catch(const Standard_Failure& e){throw std::runtime_error(e.GetMessageString());}
}
}
