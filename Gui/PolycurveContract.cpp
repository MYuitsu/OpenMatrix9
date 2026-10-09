// SPDX-License-Identifier: LGPL-2.1-or-later
#include "PolycurveContract.h"
#include "NurbsBasisAdapter.h"
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Tool.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <QJsonArray>
#include <Geom_BSplineCurve.hxx>
#include "opennurbs_polyedgecurve.h"
namespace OpenMatrix9Gui::ThreeDm::Polycurve {
namespace {
void accepted(std::uint32_t status){if(status)throw ExchangeError("Polycurve contract rejected: "+std::to_string(status));}
void accepted(Summary summary){accepted(summary.status);if(summary.schema!=1)throw ExchangeError("Invalid polycurve contract schema");}
Input input(std::size_t count,Options options){
 Input out{count,0,0,1,0,options.lengthTolerance,options.parameterTolerance,{}};
 accepted(om9_polycurve_preflight(out));return out;
}
template<std::size_t N> QJsonArray jsonArray(const std::array<double,N>& data){QJsonArray out;for(double v:data)out.append(v);return out;}
std::array<double,3> point(const ON_3dPoint& p){return {p.x,p.y,p.z};}
std::array<double,3> point(const gp_Pnt& p){return {p.X(),p.Y(),p.Z()};}
// Count/capacity/pointer envelope precedes parent Domain and child reads.
// A live immutable owner is still required; this does not prove foreign pointers.
std::size_t container(const ON_PolyCurve& p,Options options){
 const auto& curves=p.SegmentCurves();const auto& parameters=p.SegmentParameters();
 const int count=curves.Count();
 if(count<0 || curves.Capacity()<count || parameters.Count()<0 || parameters.Capacity()<parameters.Count())throw ExchangeError("Invalid native polycurve container envelope");
 input(static_cast<std::size_t>(count),options);
 if(parameters.Count()!=count+1 || !curves.Array() || !parameters.Array())throw ExchangeError("Incomplete native polycurve containers");
 return static_cast<std::size_t>(count);
}
std::uint32_t leaf(const ON_Curve& c){
 const auto id=c.ClassId();
 if(id==&ON_CLASS_RTTI(ON_LineCurve))return 1;
 if(id==&ON_CLASS_RTTI(ON_ArcCurve))return 2;
 if(id==&ON_CLASS_RTTI(ON_NurbsCurve)){BasisContract::checkedBasis(static_cast<const ON_NurbsCurve&>(c));return 3;}
 if(id==&ON_CLASS_RTTI(ON_PolylineCurve)){
  const auto& p=static_cast<const ON_PolylineCurve&>(c);
  if(p.m_pline.Count()<2 || p.m_pline.Count()>static_cast<int>(om9_polycurve_limit()) || p.m_pline.Capacity()<p.m_pline.Count() || !p.m_pline.Array() || p.m_t.Count()!=p.m_pline.Count() || p.m_t.Capacity()<p.m_t.Count() || !p.m_t.Array())throw ExchangeError("Invalid native polyline storage envelope");
  return 4;
 }
 throw GeometryRepresentationUnavailable("Nested, reference or unverified polycurve child remains retained; flat owned Line/Arc/NURBS/Polyline children only",0);
}
std::vector<Source> sourceRows(const ON_PolyCurve& p,Input& i,Options options){
 const auto count=container(p,options);
 // Gate every raw child before parent metadata/evaluation or CRC traversal.
 for(std::size_t n=0;n<count;++n){auto c=p.SegmentCurves()[static_cast<int>(n)];if(!c)throw GeometryRepresentationUnavailable("Missing polycurve child remains retained",0);leaf(*c);}
 i=input(count,options);i.parent_known=1;
 const auto& parameters=p.SegmentParameters();i.parent_domain={parameters[0],parameters[static_cast<int>(count)]};
 accepted(om9_polycurve_preflight(i));
 std::vector<Source> rows;rows.reserve(count);
 for(std::size_t n=0;n<count;++n){const auto& c=*p.SegmentCurves()[static_cast<int>(n)];
  const auto d=c.Domain();
  Source s{n,leaf(c),1,{d.Min(),d.Max()},{parameters[static_cast<int>(n)],parameters[static_cast<int>(n+1)]},{},{}};
  // Domain validation must precede native evaluation at those values.
  auto previousDomain=n?rows.back():Source{};previousDomain.start={};previousDomain.end={};
  accepted(om9_polycurve_source(i,s,previousDomain,n?1:0));
  if(!c.IsValid())throw ExchangeError("Invalid admitted polycurve child");
  s.start=point(c.PointAt(d.Min()));s.end=point(c.PointAt(d.Max()));
  accepted(om9_polycurve_source(i,s,n?rows.back():Source{},n?1:0));rows.push_back(s);
 }
 i.closed=p.IsClosed()?1:0;accepted(om9_polycurve_preflight(i));return rows;
}
struct WireRows {Input input{};std::vector<TopoDS_Edge> edges;std::vector<Source> source;std::vector<Native> native;bool closed=false;};
WireRows wireRows(const TopoDS_Wire& w,Options options){
 std::size_t count=0;
 for(TopExp_Explorer e(w,TopAbs_EDGE);e.More();e.Next()){
  if(++count>om9_polycurve_limit())throw ExchangeError("Wire edge occurrence limit exceeded");
 }
 WireRows out;out.input=input(count,options);out.edges.reserve(count);out.source.reserve(count);out.native.reserve(count);
 TopoDS_Vertex firstVertex,previousLast;
 for(BRepTools_WireExplorer e(w);e.More();e.Next()){
  if(out.edges.size()>=count)throw ExchangeError("Wire traversal exceeds all occurrences");
  const auto edge=e.Current();const auto orientation=edge.Orientation();
  if(orientation!=TopAbs_FORWARD && orientation!=TopAbs_REVERSED)throw ExchangeError("Unsupported wire edge direction");
  double a,b;TopLoc_Location location;auto c=BRep_Tool::Curve(edge,location,a,b);
  if(c.IsNull())throw ExchangeError("Wire edge has no supporting 3D curve");
  if(auto n=Handle(Geom_BSplineCurve)::DownCast(c);!n.IsNull())BasisContract::checkedBasis(n);
  Source s{out.edges.size(),5,0,{a,b},{},{},{}};
  // Independently validate this domain before native evaluation (no joint yet).
  auto single=out.input;single.count=1;s.ordinal=0;accepted(om9_polycurve_source(single,s,{},0));s.ordinal=out.edges.size();
  auto start=c->Value(a).Transformed(location.Transformation()),end=c->Value(b).Transformed(location.Transformation());
  if(orientation==TopAbs_REVERSED)std::swap(start,end);
  s.start=point(start);s.end=point(end);
  TopoDS_Vertex v0,v1;TopExp::Vertices(edge,v0,v1,true);
  if(v0.IsNull() || v1.IsNull())throw ExchangeError("Wire occurrence has missing endpoint vertices");
  Native n{s.ordinal,orientation==TopAbs_FORWARD?1u:2u,s.ordinal?(previousLast.IsSame(v0)?2u:1u):0u,s.start,s.end};
  accepted(om9_polycurve_native(out.input,s,n,s.ordinal?out.source.back():Source{},s.ordinal?out.native.back():Native{},s.ordinal?1:0));
  if(!s.ordinal)firstVertex=v0;previousLast=v1;
  out.edges.push_back(edge);out.source.push_back(s);out.native.push_back(n);
 }
 if(out.edges.size()!=count)throw ExchangeError("Wire traversal did not cover every edge occurrence");
 out.closed=firstVertex.IsSame(previousLast);out.input.closed=out.closed?1:0;
 accepted(om9_polycurve_finish(out.input,out.source.front(),out.source.back(),out.native.front(),out.native.back(),count,out.closed?1:0,out.closed?2:0));
 return out;
}
Report compare(Input i,const std::vector<Source>& source,const WireRows& actual){
 if(actual.native.size()!=i.count)throw ExchangeError("Polycurve native occurrence count changed");
 Report out;out.input=i;out.source=source;out.native=actual.native;out.measurements.reserve(i.count);
 for(std::size_t n=0;n<i.count;++n){
  auto m=om9_polycurve_native(i,source[n],actual.native[n],n?source[n-1]:Source{},n?actual.native[n-1]:Native{},n?1:0);accepted(m);out.measurements.push_back(m);
 }
 out.closure=om9_polycurve_finish(i,source.front(),source.back(),actual.native.front(),actual.native.back(),actual.native.size(),actual.closed?1:0,actual.closed?2:0);accepted(out.closure);return out;
}
}
QJsonObject Report::json() const {
 QJsonArray rows;
 for(std::size_t n=0;n<source.size();++n){const auto& s=source[n];const auto& v=native[n];
  rows.append(QJsonObject{{"ordinal",static_cast<qint64>(s.ordinal)},{"source_family",static_cast<int>(s.family)},{"child_domain",jsonArray(s.child_domain)},
   {"parent_domain",s.parent_known?QJsonValue(jsonArray(s.parent_domain)):QJsonValue(QJsonValue::Null)},
   {"source_start",jsonArray(s.start)},{"source_end",jsonArray(s.end)},{"native_start",jsonArray(v.start)},{"native_end",jsonArray(v.end)},
   {"native_direction",static_cast<int>(v.direction)},{"shared_previous",v.shared_previous?QJsonValue(v.shared_previous==2):QJsonValue(QJsonValue::Null)},
   {"endpoint_and_joint_residuals",jsonArray(measurements[n].metrics)}});
 }
 return QJsonObject{{"schema_version",1},{"method","flat-owned-segments-to-native-wire"},{"frame","native-conversion-local"},{"length_unit",QJsonValue::Null},
  {"source_identity",QJsonValue::Null},{"segment_count",static_cast<qint64>(input.count)},{"length_tolerance",input.length_tolerance},{"parameter_tolerance",input.parameter_tolerance},
  {"parent_domain",input.parent_known?QJsonValue(jsonArray(input.parent_domain)):QJsonValue(QJsonValue::Null)},{"closed",input.closed==1},
  {"joint_policy","preserve-segment-curves-no-heal"},{"segments",rows},{"closing_residuals",input.closed?QJsonValue(jsonArray(closure.metrics)):QJsonValue(QJsonValue::Null)},
  {"parameter_map",QJsonValue::Null},{"parameterization_preserved",QJsonValue::Null},{"locus_preserved",QJsonValue::Null},{"topology_preserved",QJsonValue::Null},
  {"attributes_preserved",QJsonValue::Null},{"global_deviation",QJsonValue::Null}};
}
ObservedWire importPolyCurveObserved(const ON_PolyCurve& p,Options options){
 try {
  if(p.ClassId()!=&ON_CLASS_RTTI(ON_PolyCurve))throw GeometryRepresentationUnavailable("Only ordinary owned polycurves have an editable partition adapter",0);
  Input i{};auto source=sourceRows(p,i,options);const auto crc=p.DataCRC(0);
  BRepBuilderAPI_MakeWire make;
  for(std::size_t n=0;n<i.count;++n){auto shape=importCurve(*p.SegmentCurves()[static_cast<int>(n)],options.lengthTolerance);make.Add(TopoDS::Edge(shape));if(!make.IsDone())throw ExchangeError("Native polycurve wire construction failed");}
  auto wire=make.Wire();auto actual=wireRows(wire,options);auto report=compare(i,source,actual);
  if(!BRepCheck_Analyzer(wire).IsValid())throw ExchangeError("Converted polycurve wire is invalid");
  if(p.DataCRC(0)!=crc)throw ExchangeError("Polycurve conversion changed source");
  return {std::move(wire),std::move(report)};
 }catch(const Standard_Failure& e){throw ExchangeError(std::string("Native polycurve conversion failed: ")+e.GetMessageString());}
}
std::unique_ptr<ON_PolyCurve> exportWireChecked(const TopoDS_Wire& wire,Options options){
 try {
  auto original=wireRows(wire,options);auto poly=std::make_unique<ON_PolyCurve>();
  std::vector<Source> emitted;emitted.reserve(original.input.count);
  for(std::size_t n=0;n<original.input.count;++n){
   auto c=exportCurve(original.edges[n],options.lengthTolerance);leaf(*c);const auto d=c->Domain();
   auto s=original.source[n];s.family=3;s.child_domain={d.Min(),d.Max()};
   // The native converter supplies a new domain; validate before evaluation.
   auto single=original.input;single.count=1;single.closed=0;auto domainOnly=s;domainOnly.ordinal=0;domainOnly.start={};domainOnly.end={};accepted(om9_polycurve_source(single,domainOnly,{},0));
   s.start=point(c->PointAt(d.Min()));s.end=point(c->PointAt(d.Max()));
   accepted(om9_polycurve_native(original.input,s,original.native[n],n?emitted.back():Source{},n?original.native[n-1]:Native{},n?1:0));emitted.push_back(s);
   if(!poly->Append(c.get()))throw ExchangeError("Cannot append exported wire child");c.release();
  }
  accepted(om9_polycurve_finish(original.input,emitted.front(),emitted.back(),original.native.front(),original.native.back(),original.input.count,original.closed?1:0,original.closed?2:0));
  Input nativeInput{};sourceRows(*poly,nativeInput,options);
  if(!poly->IsValid())throw ExchangeError("Invalid exported polycurve");return poly;
 }catch(const Standard_Failure& e){throw ExchangeError(std::string("Native wire export failed: ")+e.GetMessageString());}
}
void verifyWirePartition(const TopoDS_Wire& expected,const TopoDS_Wire& actual,Options options){
 try {auto before=wireRows(expected,options);auto after=wireRows(actual,options);compare(before.input,before.source,after);}
 catch(const Standard_Failure& e){throw ExchangeError(std::string("Native wire verification failed: ")+e.GetMessageString());}
}
}
