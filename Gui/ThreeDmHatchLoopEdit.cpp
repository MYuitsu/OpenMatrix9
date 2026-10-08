#include "ThreeDmHatch.h"
#include <QJsonDocument>
#include <QJsonArray>
#include <cmath>
#include <memory>
#include <vector>
#include <limits>
#include <algorithm>
#include <Geom2dAPI_InterCurveCurve.hxx>
#include <Geom2dInt_GInter.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <BndLib_Add2dCurve.hxx>
#include <Bnd_Box2d.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepCheck_Face.hxx>
#include <BRepCheck_Wire.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pln.hxx>
namespace OpenMatrix9Gui::ThreeDm {
namespace {
void keys(const QJsonObject& o,QStringList allowed){if(o.size()!=allowed.size())throw ExchangeError("Incomplete native Hatch loop fields");for(auto k:o.keys())if(!allowed.contains(k))throw ExchangeError("Unknown native Hatch loop field: "+k.toStdString());}
double number(QJsonValue v){if(!v.isDouble()||!std::isfinite(v.toDouble()))throw ExchangeError("Nonfinite/non-numeric native Hatch loop field");return v.toDouble();}
int integer(QJsonValue v,int min,int max){auto d=number(v);if(d<min||d>max||std::floor(d)!=d)throw ExchangeError("Invalid integer native Hatch loop field");return static_cast<int>(d);}
QJsonArray array(QJsonValue v,int min,int max){if(!v.isArray())throw ExchangeError("Native Hatch loop field must be an array");auto a=v.toArray();if(a.size()<min||a.size()>max)throw ExchangeError("Native Hatch loop array cardinality exceeds limits");return a;}
ON_Interval domain(QJsonValue v){auto a=array(v,2,2);ON_Interval d(number(a[0]),number(a[1]));if(!d.IsIncreasing()||!d.IsValid())throw ExchangeError("Invalid native Hatch loop domain");return d;}
ON_3dPoint point(QJsonValue v){auto a=array(v,3,3);ON_3dPoint p(number(a[0]),number(a[1]),number(a[2]));if(!p.IsValid()||p.z!=0)throw ExchangeError("Native Hatch loop point must be finite2D with zero z");return p;}
ON_Plane plane(QJsonValue v){auto a=array(v,16,16);ON_Plane p;for(int i=0;i<3;++i){p.origin[i]=number(a[i]);p.xaxis[i]=number(a[i+3]);p.yaxis[i]=number(a[i+6]);p.zaxis[i]=number(a[i+9]);}p.plane_equation.x=number(a[12]);p.plane_equation.y=number(a[13]);p.plane_equation.z=number(a[14]);p.plane_equation.d=number(a[15]);if(!p.IsValid())throw ExchangeError("Invalid native Hatch loop arc plane");return p;}
QJsonObject numeric(QJsonObject o,int index){for(auto k:{"class_uuid","user_strings","userdata"})o.remove(k);o["source_index"]=index;if(o.contains("segments")){auto a=o["segments"].toArray();for(int i=0;i<a.size();++i)a[i]=numeric(a[i].toObject(),i);o["segments"]=a;}return o;}
QJsonValue withoutSources(QJsonValue v){if(v.isArray()){QJsonArray a;for(auto x:v.toArray())a.append(withoutSources(x));return a;}if(v.isObject()){auto o=v.toObject();o.remove("source_index");for(auto k:o.keys())o[k]=withoutSources(o[k]);return o;}return v;}
struct Budget{unsigned nodes=0;size_t numbers=0;void reserve(size_t n){if(n>2000000-numbers)throw ExchangeError("Native Hatch loop edit numeric limit exceeded");numbers+=n;}};
void checkIntersections(const Geom2dAPI_InterCurveCurve& test,const ON_Interval& d){
    if(!test.Intersector().IsDone())throw ExchangeError("Cannot verify native Hatch loop intersections");
    const double parameterTolerance=1e-12*std::max(1.0,std::abs(d[1]-d[0]));
    for(int j=1;j<=test.NbSegments();++j){
        const auto& segment=test.Intersector().Segment(j);
        // OCCT can report the trivial parameter diagonal on joined linear spans.
        bool diagonal=!segment.IsOpposite()&&segment.HasFirstPoint()&&segment.HasLastPoint()
            &&std::abs(segment.FirstPoint().ParamOnFirst()-segment.FirstPoint().ParamOnSecond())<=parameterTolerance
            &&std::abs(segment.LastPoint().ParamOnFirst()-segment.LastPoint().ParamOnSecond())<=parameterTolerance;
        if(!diagonal)throw ExchangeError("Overlapping native Hatch loop boundary");
    }
    for(int j=1;j<=test.NbPoints();++j){auto p=test.Intersector().Point(j);auto a=p.ParamOnFirst(),b=p.ParamOnSecond();
        bool same=std::abs(a-b)<=parameterTolerance;
        bool seam=(std::abs(a-d[0])<=parameterTolerance&&std::abs(b-d[1])<=parameterTolerance)||(std::abs(b-d[0])<=parameterTolerance&&std::abs(a-d[1])<=parameterTolerance);
        if(!same&&!seam)throw ExchangeError("Self-intersecting native Hatch loop boundary");
    }
}
void checkSelf(const Handle(Geom2d_Curve)& uv,double tolerance){
    ON_Interval d(uv->FirstParameter(),uv->LastParameter());
    checkIntersections(Geom2dAPI_InterCurveCurve(uv,tolerance),d);
    auto spline=Handle(Geom2d_BSplineCurve)::DownCast(uv);
    if(spline.IsNull())throw ExchangeError("Missing derived Hatch validation spline");
    if(spline->NbKnots()>1025)throw ExchangeError("Hatch topology validation exceeds1024 knot spans");
    std::vector<Handle(Geom2d_Curve)> spans;std::vector<Bnd_Box2d> bounds;
    for(int i=1;i<spline->NbKnots();++i){auto a=spline->Knot(i),b=spline->Knot(i+1);
        Handle(Geom2d_Curve) span=new Geom2d_TrimmedCurve(uv,a,b);
        Bnd_Box2d box;BndLib_Add2dCurve::Add(span,tolerance,box);spans.push_back(span);bounds.push_back(box);
    }
    // Whole-spline self-intersection may miss retraced linear spans. Check all
    // overlapping span pairs independently, allowing only their shared parameter
    // endpoints and the closed seam. Geometry remains native on export.
    for(size_t i=0;i<spans.size();++i)for(size_t j=i+1;j<spans.size();++j)
        if(!bounds[i].IsOut(bounds[j]))checkIntersections(Geom2dAPI_InterCurveCurve(spans[i],spans[j],tolerance),d);
}
void topology(const ON_Hatch& h){
    // OCC geometry here is a derived validation oracle. Export remains the
    // staged native curve tree, including original type, knots and metadata.
    constexpr double tolerance=1e-7;
    if(h.LoopCount()>1024)throw ExchangeError("Native Hatch loop topology exceeds1024 boundary limit");
    std::vector<TopoDS_Wire> wires;std::vector<TopoDS_Face> faces;std::vector<double> areas;
    try{
        for(int i=0;i<h.LoopCount();++i){auto c=h.Loop(i)->Curve();auto uv=curve2d(*c);checkSelf(uv,tolerance);
            auto edge=TopoDS::Edge(importCurve(*c,tolerance));BRepBuilderAPI_MakeWire wire(edge);if(!wire.IsDone())throw ExchangeError("Cannot validate closed native Hatch loop wire");auto w=wire.Wire();if(ON_ClosedCurveOrientation(*c,ON_xy_plane)<0)w.Reverse();BRepBuilderAPI_MakeFace face(gp_Pln(gp::XOY()),w,true);if(!face.IsDone()||!BRepCheck_Analyzer(face.Face()).IsValid())throw ExchangeError("Invalid native Hatch loop planar boundary");
            TopoDS_Edge e1,e2;BRepCheck_Wire check(w);if(check.SelfIntersect(face.Face(),e1,e2)!=BRepCheck_NoError)throw ExchangeError("Self-intersecting native Hatch loop wire");GProp_GProps area;BRepGProp::SurfaceProperties(face.Face(),area);if(!std::isfinite(area.Mass())||area.Mass()<=0)throw ExchangeError("Degenerate native Hatch loop area");wires.push_back(w);faces.push_back(face.Face());areas.push_back(area.Mass());
        }
        std::vector<int> parent(h.LoopCount(),-1);
        for(int i=0;i<h.LoopCount();++i)for(int j=i+1;j<h.LoopCount();++j){BRepExtrema_DistShapeShape distance(wires[i],wires[j]);if(!distance.IsDone()||distance.Value()<=tolerance)throw ExchangeError("Intersecting/touching native Hatch loop boundaries");}
        for(int i=0;i<h.LoopCount();++i){auto p=h.Loop(i)->Curve()->PointAtStart();double nearest=std::numeric_limits<double>::infinity();for(int j=0;j<h.LoopCount();++j)if(i!=j){BRepClass_FaceClassifier classifier(faces[j],gp_Pnt(p.x,p.y,0),tolerance);if(classifier.State()!=TopAbs_IN&&classifier.State()!=TopAbs_OUT)throw ExchangeError("Cannot verify native Hatch loop containment");if(classifier.State()==TopAbs_IN&&areas[j]<nearest){parent[i]=j;nearest=areas[j];}}if(parent[i]<0){if(h.Loop(i)->Type()!=ON_HatchLoop::ltOuter)throw ExchangeError("Native Hatch inner loop has no containing outer boundary");}else if(h.Loop(i)->Type()==h.Loop(parent[i])->Type())throw ExchangeError("Native Hatch containment requires alternating outer/inner roles");}
        for(int i=0;i<h.LoopCount();++i)if(h.Loop(i)->Type()==ON_HatchLoop::ltOuter){BRepBuilderAPI_MakeFace region(gp_Pln(gp::XOY()),wires[i],true);for(int j=0;j<h.LoopCount();++j)if(parent[j]==i){auto hole=wires[j];hole.Reverse();region.Add(hole);}if(!region.IsDone()||!BRepCheck_Analyzer(region.Face()).IsValid())throw ExchangeError("Invalid native Hatch outer/inner region topology");BRepCheck_Face check(region.Face());if(check.IntersectWires()!=BRepCheck_NoError||check.OrientationOfWires()!=BRepCheck_NoError)throw ExchangeError("Invalid native Hatch region wire relations");}
    }catch(const Standard_Failure& e){throw ExchangeError(std::string("Native Hatch loop topology validation failed: ")+e.GetMessageString());}
}
const ON_Curve* childSource(const ON_Curve* source,QJsonValue index){if(index.isNull())return nullptr;auto poly=source&&source->ClassId()==&ON_CLASS_RTTI(ON_PolyCurve)?static_cast<const ON_PolyCurve*>(source):nullptr;if(!poly)throw ExchangeError("Hatch loop child source requires original native PolyCurve");return poly->SegmentCurve(integer(index,0,poly->Count()-1));}
std::unique_ptr<ON_Curve> decode(const QJsonValue& value,const ON_Curve* source,unsigned depth,Budget& budget){
    if(depth>=64||++budget.nodes>16384)throw ExchangeError("Native Hatch loop edit depth/node limit exceeded");if(!value.isObject())throw ExchangeError("Native Hatch loop curve must be an object");auto o=value.toObject();
    if(!o.contains("source_index")||!o["class_name"].isString()||integer(o["dimension"],2,2)!=2)throw ExchangeError("Missing native Hatch loop curve identity/dimension");auto name=o["class_name"].toString();if(source&&name!=source->ClassId()->ClassName())throw ExchangeError("Native Hatch loop edit cannot implicitly change source curve class");
    auto d=domain(o["domain"]);QStringList base{"class_name","dimension","domain","source_index"};std::unique_ptr<ON_Curve> result;
    if(name=="ON_NurbsCurve"){
        auto allowed=base;allowed.append({"order","rational","cvs","knots"});keys(o,allowed);if(!o["rational"].isBool())throw ExchangeError("Native Hatch rational flag must be boolean");int order=integer(o["order"],2,1024);auto cvs=array(o["cvs"],order,500000),knots=array(o["knots"],order,1000000);bool rational=o["rational"].toBool();int size=rational?3:2;budget.reserve(static_cast<size_t>(cvs.size())*size+knots.size());auto n=std::make_unique<ON_NurbsCurve>(2,rational,order,cvs.size());if(knots.size()!=n->KnotCount())throw ExchangeError("Native Hatch NURBS knot cardinality mismatch");for(int i=0;i<cvs.size();++i){auto cv=array(cvs[i],size,size);for(int j=0;j<size;++j)n->CV(i)[j]=number(cv[j]);}for(int i=0;i<knots.size();++i)if(!n->SetKnot(i,number(knots[i])))throw ExchangeError("Cannot set native Hatch NURBS knot");if(n->Domain()!=d)throw ExchangeError("Hatch NURBS domain must agree with knots");result=std::move(n);
    }else if(name=="ON_ArcCurve"){
        auto allowed=base;allowed.append({"plane","radius","angle_domain"});keys(o,allowed);auto radius=number(o["radius"]);if(radius<=0)throw ExchangeError("Native Hatch arc requires positive radius");auto angle=domain(o["angle_domain"]);auto n=std::make_unique<ON_ArcCurve>(ON_Arc(ON_Circle(plane(o["plane"]),radius),angle),d[0],d[1]);n->m_dim=2;result=std::move(n);
    }else if(name=="ON_LineCurve"){
        auto allowed=base;allowed.append("points");keys(o,allowed);auto pts=array(o["points"],2,2);auto n=std::make_unique<ON_LineCurve>(point(pts[0]),point(pts[1]));n->ChangeDimension(2);if(!n->SetDomain(d[0],d[1]))throw ExchangeError("Cannot set native Hatch line domain");result=std::move(n);
    }else if(name=="ON_PolylineCurve"){
        auto allowed=base;allowed.append({"points","parameters"});keys(o,allowed);auto pts=array(o["points"],2,500000),params=array(o["parameters"],2,500000);if(pts.size()!=params.size())throw ExchangeError("Hatch polyline point/parameter cardinality mismatch");budget.reserve(static_cast<size_t>(pts.size())*4);auto n=std::make_unique<ON_PolylineCurve>();n->m_dim=2;for(int i=0;i<pts.size();++i){n->m_pline.Append(point(pts[i]));n->m_t.Append(number(params[i]));}if(n->Domain()!=d)throw ExchangeError("Hatch polyline domain must agree with parameters");result=std::move(n);
    }else if(name=="ON_PolyCurve"){
        auto allowed=base;allowed.append({"segments","parameters"});keys(o,allowed);auto segments=array(o["segments"],1,16384),params=array(o["parameters"],2,16385);if(params.size()!=segments.size()+1)throw ExchangeError("Hatch PolyCurve segment/parameter cardinality mismatch");auto n=std::make_unique<ON_PolyCurve>();for(auto v:segments){if(!v.isObject()||!v.toObject().contains("source_index"))throw ExchangeError("Missing native Hatch segment source index");auto child=decode(v,childSource(source,v.toObject()["source_index"]),depth+1,budget);if(!n->Append(child.get()))throw ExchangeError("Cannot append native Hatch segment");child.release();}std::vector<double> t;for(auto v:params)t.push_back(number(v));if(!n->SetParameterization(t.data())||n->Domain()!=d)throw ExchangeError("Invalid native Hatch PolyCurve parameters/domain");result=std::move(n);
    }else throw ExchangeError("Native Hatch loop class requires its own edit/reference adapter: "+name.toStdString());
    if(!result||!result->IsValid()||result->Dimension()!=2||result->ClassId()->ClassName()!=name)throw ExchangeError("Invalid reconstructed native Hatch loop curve");if(source)result->CopyUserData(*source);return result;
}
}
QJsonObject hatchLoopFields(const ON_Hatch& hatch){validateHatchRhino5Data(hatch);auto loops=hatchLoopInventory(hatch);for(int i=0;i<loops.size();++i){auto o=loops[i].toObject();o["source_index"]=i;o["curve"]=numeric(o["curve"].toObject(),i);loops[i]=o;}return {{"schema_version",1},{"loops",loops}};}
void applyHatchLoopFields(ON_Hatch& hatch,const QJsonValue& value){
    validateHatchRhino5Data(hatch);if(!value.isObject()||QJsonDocument(value.toObject()).toJson(QJsonDocument::Compact).size()>32ULL*1024*1024)throw ExchangeError("Invalid/oversized native Hatch loop payload");auto fields=value.toObject();keys(fields,{"schema_version","loops"});integer(fields["schema_version"],1,1);auto values=array(fields["loops"],1,16384);Budget budget;std::vector<std::unique_ptr<ON_HatchLoop>> loops;
    for(auto v:values){if(!v.isObject())throw ExchangeError("Native Hatch loop row must be an object");auto o=v.toObject();keys(o,{"type","source_index","curve"});auto index=o["source_index"];const ON_Curve* source=index.isNull()?nullptr:hatch.Loop(integer(index,0,hatch.LoopCount()-1))->Curve();if(!o["curve"].isObject()||o["curve"].toObject()["source_index"]!=index)throw ExchangeError("Native Hatch loop and curve source index differ");auto curve=decode(o["curve"],source,0,budget);if(!curve->IsClosed())throw ExchangeError("Native Hatch loop edit must be closed");auto type=static_cast<ON_HatchLoop::eLoopType>(integer(o["type"],0,1));loops.push_back(std::make_unique<ON_HatchLoop>(curve.release(),type));}
    ON_Hatch staged(hatch);while(staged.LoopCount())if(!staged.RemoveLoop(staged.LoopCount()-1))throw ExchangeError("Cannot stage native Hatch loop replacement");for(auto& loop:loops)staged.AddLoop(loop.release());validateHatchRhino5Data(staged);topology(staged);
    if(withoutSources(hatchLoopFields(staged))!=withoutSources(fields))throw ExchangeError("Native Hatch loop construction changed requested fields");hatch=staged;
}
}
