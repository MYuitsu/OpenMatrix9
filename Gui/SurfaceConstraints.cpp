// SPDX-License-Identifier: LGPL-2.1-or-later
#include "SurfaceConstraints.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepOffsetAPI_MakeFilling.hxx>
#include <GeomAPI_ProjectPointOnSurf.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Surface.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <gp_Pnt2d.hxx>
#include <algorithm>
#include <cmath>
#include <sstream>
#include <stdexcept>
#include <utility>

namespace {
constexpr double distanceTolerance=1e-4, angleTolerance=2e-3;
struct Ref {
    PyObject* p;
    explicit Ref(PyObject* value):p(value) {
        if(!p){PyErr_Clear();throw std::runtime_error("Cannot read native surface constraint geometry");}
    }
    ~Ref(){Py_XDECREF(p);}
    Ref(const Ref&)=delete;
};
TopoDS_Shape nativeShape(PyObject* object) {
    Ref text(PyObject_CallMethod(object,"exportBrepToString",nullptr));
    const char* data=PyUnicode_AsUTF8(text.p);
    if(!data)throw std::runtime_error("Invalid native constraint BRep");
    std::istringstream stream(data);TopoDS_Shape shape;BRep_Builder builder;
    BRepTools::Read(shape,stream,builder);
    if(shape.IsNull()||!BRepCheck_Analyzer(shape).IsValid())
        throw std::runtime_error("Surface constraints require valid native shapes");
    return shape;
}
TopoDS_Edge singleEdge(App::Document& doc,const OpenMatrix9Gui::SurfaceInput& input) {
    Ref wire(OpenMatrix9Gui::surfaceWire(doc,input));const auto shape=nativeShape(wire.p);
    TopExp_Explorer edges(shape,TopAbs_EDGE);
    if(!edges.More())throw std::runtime_error("Empty surface constraint curve");
    auto edge=TopoDS::Edge(edges.Current());edges.Next();
    if(edges.More()||BRep_Tool::IsClosed(edge))
        throw std::runtime_error("Surface constraints currently need open single-edge profiles and rails");
    return edge;
}
struct Support {TopoDS_Edge edge;TopoDS_Face face;};
Support support(App::Document& doc,const OpenMatrix9Gui::SurfaceInput& input) {
    if(!input.chain.empty()||input.sub.size()<5||input.sub.compare(0,4,"Edge")!=0
       ||input.sub.find_first_not_of("0123456789",4)!=std::string::npos)
        throw std::runtime_error("Tangency/curvature needs one selected surface EdgeN, with exactly one adjacent face");
    auto* object=doc.getObject(input.object.c_str());
    if(!object)throw std::runtime_error("Surface constraint support was deleted");
    Ref pyObject(object->getPyObject()),original(PyObject_GetAttrString(pyObject.p,"Shape"));
    Ref shape(PyObject_CallMethod(original.p,"copy","OO",Py_True,Py_False));
    if(PyObject_HasAttrString(pyObject.p,"Placement")&&PyObject_HasAttrString(pyObject.p,"getGlobalPlacement")){
        Ref global(PyObject_CallMethod(pyObject.p,"getGlobalPlacement",nullptr)),local(PyObject_GetAttrString(pyObject.p,"Placement"));
        Ref inverse(PyObject_CallMethod(local.p,"inverse",nullptr)),parent(PyNumber_Multiply(global.p,inverse.p));
        Ref matrix(PyObject_CallMethod(parent.p,"toMatrix",nullptr));
        Ref transformed(PyObject_CallMethod(shape.p,"transformShape","OO",matrix.p,Py_False));
    }
    const auto native=nativeShape(shape.p);TopTools_IndexedMapOfShape edges;
    TopExp::MapShapes(native,TopAbs_EDGE,edges);
    const auto index=std::stoul(input.sub.substr(4));
    if(index==0||index>static_cast<unsigned long>(edges.Extent()))
        throw std::runtime_error("Selected surface edge no longer exists");
    Support result;result.edge=TopoDS::Edge(edges(int(index)));int count=0;
    for(TopExp_Explorer faces(native,TopAbs_FACE);faces.More();faces.Next()){
        const auto face=TopoDS::Face(faces.Current());bool contains=false;
        for(TopExp_Explorer boundary(face,TopAbs_EDGE);boundary.More();boundary.Next())
            if(boundary.Current().IsSame(result.edge)){contains=true;break;}
        if(contains){result.face=face;++count;}
    }
    if(count!=1)throw std::runtime_error("Surface edge tangency is ambiguous: select an edge belonging to exactly one face");
    Standard_Real first,last;
    if(BRep_Tool::CurveOnSurface(result.edge,result.face,first,last).IsNull())
        throw std::runtime_error("Selected surface edge has no support-face parameter curve");
    return result;
}
bool nonrational(const TopoDS_Edge& edge) {
    BRepAdaptor_Curve curve(edge);
    switch(curve.GetType()){
    case GeomAbs_Line:return true;
    case GeomAbs_BSplineCurve:return !curve.BSpline()->IsRational();
    case GeomAbs_BezierCurve:return !curve.Bezier()->IsRational();
    default:return false;
    }
}
bool sameStructure(const TopoDS_Edge& a,const TopoDS_Edge& b) {
    BRepAdaptor_Curve ca(a),cb(b);if(ca.GetType()!=cb.GetType())return false;
    if(ca.GetType()==GeomAbs_Line)return true;
    if(ca.GetType()==GeomAbs_BezierCurve)return ca.Bezier()->Degree()==cb.Bezier()->Degree();
    if(ca.GetType()!=GeomAbs_BSplineCurve)return false;
    const auto aa=ca.BSpline(),bb=cb.BSpline();
    if(aa->Degree()!=bb->Degree()||aa->NbPoles()!=bb->NbPoles()||aa->NbKnots()!=bb->NbKnots())return false;
    const double ar=aa->LastParameter()-aa->FirstParameter(),br=bb->LastParameter()-bb->FirstParameter();
    for(int i=1;i<=aa->NbKnots();++i)
        if(aa->Multiplicity(i)!=bb->Multiplicity(i)||std::abs((aa->Knot(i)-aa->FirstParameter())/ar-(bb->Knot(i)-bb->FirstParameter())/br)>1e-9)return false;
    return true;
}
gp_Pnt endpoint(const TopoDS_Edge& edge,bool last) {
    BRepAdaptor_Curve curve(edge);
    const bool reverse=edge.Orientation()==TopAbs_REVERSED;
    return curve.Value(last!=reverse?curve.LastParameter():curve.FirstParameter());
}
TopoDS_Edge oriented(TopoDS_Edge edge,const gp_Pnt& first,const gp_Pnt& last) {
    if(endpoint(edge,false).Distance(first)>distanceTolerance)edge.Reverse();
    if(endpoint(edge,false).Distance(first)>distanceTolerance||endpoint(edge,true).Distance(last)>distanceTolerance)
        throw std::runtime_error("Constrained Sweep2 boundary profiles must meet both rail endpoints in the selected order");
    return edge;
}
struct Derivatives {
    gp_Vec u,v,uu,vv,uv,normal;
    Derivatives(const occ::handle<Geom_Surface>& surface,double a,double b) {
        gp_Pnt point;surface->D2(a,b,point,u,v,uu,vv,uv);normal=u.Crossed(v);
        if(normal.SquareMagnitude()<1e-20)throw std::runtime_error("Singular support/result surface prevents continuity verification");
        normal.Normalize();
    }
    double curvature(const gp_Vec& direction,const gp_Vec& commonNormal)const {
        const double e=u.Dot(u),f=u.Dot(v),g=v.Dot(v),det=e*g-f*f;
        if(det<1e-20)throw std::runtime_error("Singular surface derivative metric");
        const double du=(g*u.Dot(direction)-f*v.Dot(direction))/det;
        const double dv=(e*v.Dot(direction)-f*u.Dot(direction))/det;
        const double denominator=e*du*du+2*f*du*dv+g*dv*dv;
        return (uu.Dot(commonNormal)*du*du+2*uv.Dot(commonNormal)*du*dv+vv.Dot(commonNormal)*dv*dv)/denominator;
    }
};
std::pair<double,double> project(const gp_Pnt& point,const TopoDS_Face& face) {
    double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
    GeomAPI_ProjectPointOnSurf projection(point,BRep_Tool::Surface(face),u0,u1,v0,v1,1e-9);
    if(!projection.IsDone()||projection.NbPoints()==0||projection.LowerDistance()>distanceTolerance)
        throw std::runtime_error("Constrained surface failed independent rail/profile distance verification");
    double u,v;projection.LowerDistanceParameters(u,v);
    const BRepClass_FaceClassifier classifier(face,gp_Pnt2d(u,v),1e-7);
    if(classifier.State()!=TopAbs_IN&&classifier.State()!=TopAbs_ON)
        throw std::runtime_error("Surface constraint curve leaves the bounded output/support face");
    return {u,v};
}
void compatibleContacts(const std::vector<TopoDS_Edge>& curves,unsigned first,
                        const TopoDS_Edge& rail,const Support* ref,unsigned order) {
    Standard_Real a,b;TopLoc_Location location;
    auto railCurve=BRep_Tool::Curve(rail,location,a,b);
    if(railCurve.IsNull())throw std::runtime_error("Rail has no native contact curve");
    railCurve=occ::handle<Geom_Curve>::DownCast(railCurve->Transformed(location.Transformation()));
    for(std::size_t i=first;i<curves.size();++i){BRepAdaptor_Curve profile(curves[i]);
        const auto start=profile.Value(profile.FirstParameter()),end=profile.Value(profile.LastParameter());
        GeomAPI_ProjectPointOnCurve p0(start,railCurve,a,b),p1(end,railCurve,a,b);
        if(p0.NbPoints()==0||p1.NbPoints()==0)throw std::runtime_error("Cannot project profile endpoints onto the rail");
        const bool useStart=p0.LowerDistance()<=p1.LowerDistance();
        if(std::min(p0.LowerDistance(),p1.LowerDistance())>distanceTolerance)
            throw std::runtime_error("Each original Sweep2 profile must meet both rails at its endpoints");
        if(!order)continue;
        const auto point=useStart?start:end;const auto uv=project(point,ref->face);
        const Derivatives source(BRep_Tool::Surface(ref->face),uv.first,uv.second);
        gp_Pnt position;gp_Vec d1,d2;
        profile.D2(useStart?profile.FirstParameter():profile.LastParameter(),position,d1,d2);
        if(d1.SquareMagnitude()<1e-20||std::abs(d1.Dot(source.normal))/d1.Magnitude()>std::sin(angleTolerance))
            throw std::runtime_error("Original Sweep2 profile tangent conflicts with the rail support tangent plane");
        if(order==2){const double curveCurvature=d2.Dot(source.normal)/d1.SquareMagnitude();
            const double faceCurvature=source.curvature(d1,source.normal);
            if(!std::isfinite(curveCurvature)||!std::isfinite(faceCurvature)
               ||std::abs(curveCurvature-faceCurvature)>1e-3*std::max({1.,std::abs(curveCurvature),std::abs(faceCurvature)}))
                throw std::runtime_error("Original Sweep2 profile curvature conflicts with the rail support curvature");
        }
    }
}
TopoDS_Edge connector(const gp_Pnt& a,const gp_Pnt& b,const Support* start,const Support* end) {
    const gp_Vec chord(a,b);
    auto tangent=[&](const gp_Pnt& point,const Support* ref){
        if(!ref)return chord;
        const auto uv=project(point,ref->face);
        const Derivatives derivatives(BRep_Tool::Surface(ref->face),uv.first,uv.second);
        const auto n=derivatives.normal;gp_Vec result=chord;
        // Preserve the chord's two independent coordinate increments and solve
        // the remaining one from the support tangent plane. This makes planar
        // and polynomial extrusions exact while imposing no source mutation.
        int axis=1;for(int i=2;i<=3;++i)if(std::abs(n.Coord(i))>std::abs(n.Coord(axis)))axis=i;
        result.SetCoord(axis,result.Coord(axis)-n.Dot(result)/n.Coord(axis));
        if(result.SquareMagnitude()<1e-14||result.Dot(chord)<=0)
            throw std::runtime_error("Loft endpoint supports cannot define a forward connecting tangent");
        return result;
    };
    TColgp_Array1OfPnt poles(1,4);poles.SetValue(1,a);poles.SetValue(4,b);
    poles.SetValue(2,a.Translated(tangent(a,start)/3.));
    poles.SetValue(3,b.Translated(-tangent(b,end)/3.));
    occ::handle<Geom_BezierCurve> curve=new Geom_BezierCurve(poles);
    return BRepBuilderAPI_MakeEdge(curve).Edge();
}
void verify(const TopoDS_Face& face,const std::vector<TopoDS_Edge>& curves,const std::vector<std::pair<Support,unsigned>>& constraints) {
    const auto resultSurface=BRep_Tool::Surface(face);
    for(const auto& edge:curves){BRepAdaptor_Curve curve(edge);
        for(int i=0;i<=64;++i)project(curve.Value(curve.FirstParameter()+(curve.LastParameter()-curve.FirstParameter())*i/64.),face);
    }
    for(const auto& item:constraints){const auto& ref=item.first;const unsigned order=item.second;
        BRepAdaptor_Curve curve(ref.edge);const auto supportSurface=BRep_Tool::Surface(ref.face);
        // Interior samples avoid corner singularities where the two independent
        // boundary constraints meet. G0 checks above include all endpoints.
        for(int i=1;i<64;++i){const auto point=curve.Value(curve.FirstParameter()+(curve.LastParameter()-curve.FirstParameter())*i/64.);
            const auto a=project(point,ref.face),b=project(point,face);
            const Derivatives source(supportSurface,a.first,a.second),target(resultSurface,b.first,b.second);
            const double cosine=std::clamp(std::abs(source.normal.Dot(target.normal)),0.,1.);
            if(std::acos(cosine)>angleTolerance)throw std::runtime_error("Constrained surface failed independent G1 normal verification");
            if(order==2){const auto t1=source.u.Normalized(),t2=source.normal.Crossed(t1);
                for(const auto direction:{t1,t2,(t1+t2).Normalized()}){
                    const double k1=source.curvature(direction,source.normal),k2=target.curvature(direction,source.normal);
                    if(!std::isfinite(k1)||!std::isfinite(k2)||std::abs(k1-k2)>1e-3*std::max({1.,std::abs(k1),std::abs(k2)}))
                        throw std::runtime_error("Constrained surface failed independent G2 normal-curvature verification");
                }
            }
        }
    }
}
}
namespace OpenMatrix9Gui {
bool surfaceConstraintHasSupport(App::Document& doc,const SurfaceInput& input)noexcept {
    try{support(doc,input);return true;}catch(...){PyErr_Clear();return false;}
}
bool surfaceConstraintProfilesEligible(App::Document& doc,const std::vector<SurfaceInput>& inputs,unsigned first)noexcept {
    try{if(inputs.size()<first+2||inputs.size()>first+32)return false;
        const auto reference=singleEdge(doc,inputs[first]);
        for(std::size_t i=first;i<inputs.size();++i){const auto edge=singleEdge(doc,inputs[i]);
            if(!nonrational(edge)||(first==2&&!sameStructure(reference,edge)))return false;
        }
        for(unsigned i=0;i<first;++i)singleEdge(doc,inputs[i]);
        return true;
    }catch(...){PyErr_Clear();return false;}
}
PyObject* constrainedSurface(App::Document& doc,const std::vector<SurfaceInput>& inputs,const SurfaceOptions& options) {
    try{
        if(options.closed||options.sectionMode!=0||options.maintainHeight||!options.slashes.empty())
            throw std::runtime_error("Surface continuity currently requires open original sections; clear Closed, section fitting, Maintain Height and Slashes");
        const bool sweep=options.kind==2,loft=options.kind==3;const unsigned first=sweep?2:0;
        if((!sweep&&!loft)||inputs.size()<first+2||inputs.size()>first+32)
            throw std::runtime_error("Surface continuity requires two to 32 open single-edge profiles");
        if((sweep&&(options.matchStart||options.matchEnd))||(loft&&(options.continuityA||options.continuityB)))
            throw std::runtime_error("Surface continuity options do not match this command");
        if(loft&&options.style!=0&&options.style!=3)
            throw std::runtime_error("Match Tangents currently supports Normal and Tight Loft");
        if(options.continuityA>2||options.continuityB>2)
            throw std::runtime_error("Unknown surface continuity order");
        std::vector<TopoDS_Edge> curves;
        for(const auto& input:inputs)curves.push_back(singleEdge(doc,input));
        for(std::size_t i=first;i<curves.size();++i){
            if(!nonrational(curves[i]))throw std::runtime_error("Surface continuity requires nonrational line, Bezier or BSpline profiles");
            if(sweep&&i>first&&!sameStructure(curves[first],curves[i]))
                throw std::runtime_error("Sweep2 continuity requires profiles with matching degree, poles and knot structure");
        }
        std::vector<std::pair<Support,unsigned>> constraints;
        if(sweep){for(unsigned rail=0;rail<2;++rail){const unsigned order=rail==0?options.continuityA:options.continuityB;
            Support ref;if(order)ref=support(doc,inputs[rail]);
            compatibleContacts(curves,first,curves[rail],order?&ref:nullptr,order);
        }}
        BRepOffsetAPI_MakeFilling filling(4,24,3,false,1e-7,1e-6,1e-4,1e-4,8,16);
        auto add=[&](std::size_t input,TopoDS_Edge edge,unsigned order){
            if(order){auto ref=support(doc,inputs[input]);
                // Use the edge belonging to the copied face, retaining pcurves.
                ref.edge=oriented(ref.edge,endpoint(edge,false),endpoint(edge,true));
                // OCCT's BRepFill_Filling::AddConstraints forwards the enum's
                // raw integer to BRepFill_CurveConstraint(int Tang). That
                // constructor accepts plate G orders 0/1/2, although this API's
                // header names GeomAbs_G2 (whose enum value is 3). Pass the
                // actual plate ordinal: 2 imposes normal AND curvature, and
                // the independent D2 validation below checks both explicitly.
                // Upstream: OCCT/src/ModelingAlgorithms/TKBool/BRepFill/
                // BRepFill_Filling.cxx and BRepFill_CurveConstraint.cxx.
                filling.Add(ref.edge,ref.face,static_cast<GeomAbs_Shape>(order),true);constraints.emplace_back(ref,order);
            }else filling.Add(edge,GeomAbs_C0,true);
        };
        if(sweep){
            const auto a0=endpoint(curves[0],false),a1=endpoint(curves[0],true),b0=endpoint(curves[1],false),b1=endpoint(curves[1],true);
            const auto end=oriented(curves.back(),a1,b1),begin=oriented(curves[first],b0,a0);
            add(0,curves[0],options.continuityA);add(inputs.size()-1,end,0);
            auto reversed=curves[1];reversed.Reverse();add(1,reversed,options.continuityB);add(first,begin,0);
        }else{
            const auto a0=endpoint(curves.front(),false),b0=endpoint(curves.front(),true),a1=endpoint(curves.back(),false),b1=endpoint(curves.back(),true);
            Support start,end;
            if(options.matchStart)start=support(doc,inputs.front());
            if(options.matchEnd)end=support(doc,inputs.back());
            add(0,curves.front(),options.matchStart?1:0);
            filling.Add(connector(b0,b1,options.matchStart?&start:nullptr,options.matchEnd?&end:nullptr),GeomAbs_C0,true);
            auto reversed=curves.back();reversed.Reverse();add(inputs.size()-1,reversed,options.matchEnd?1:0);
            auto side=connector(a0,a1,options.matchStart?&start:nullptr,options.matchEnd?&end:nullptr);side.Reverse();
            filling.Add(side,GeomAbs_C0,true);
        }
        for(std::size_t i=first+1;i+1<curves.size();++i)filling.Add(curves[i],GeomAbs_C0,false);
        filling.Build();
        if(!filling.IsDone())throw std::runtime_error("Native surface continuity solve failed");
        const auto shape=filling.Shape();
        if(shape.IsNull()||!BRepCheck_Analyzer(shape).IsValid())throw std::runtime_error("Native surface continuity solve produced invalid geometry");
        TopExp_Explorer faces(shape,TopAbs_FACE);
        if(!faces.More())throw std::runtime_error("Native surface continuity solve produced no face");
        const auto face=TopoDS::Face(faces.Current());faces.Next();
        if(faces.More())throw std::runtime_error("Surface continuity solver needs a single bounded patch");
        verify(face,curves,constraints);
        std::ostringstream data;BRepTools::Write(shape,data);
        Ref part(PyImport_ImportModule("Part")),constructor(PyObject_GetAttrString(part.p,"Shape")),output(PyObject_CallNoArgs(constructor.p));
        Ref loaded(PyObject_CallMethod(output.p,"importBrepFromString","s",data.str().c_str()));
        Py_INCREF(output.p);return output.p;
    }catch(const Standard_Failure& e){throw std::runtime_error(e.GetMessageString()?e.GetMessageString():"Native continuity kernel failure");}
}
}
