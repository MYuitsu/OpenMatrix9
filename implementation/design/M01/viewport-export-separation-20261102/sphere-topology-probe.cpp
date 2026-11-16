// Diagnostic only: inspect full and partial OCCT sphere boundary representations.
#include <BRepAdaptor_Surface.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Check.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <BRep_Builder.hxx>
#include <BRep_TEdge.hxx>
#include <BRep_TVertex.hxx>
#include <BRep_Curve3D.hxx>
#include <BRep_GCurve.hxx>
#include <BRep_PointRepresentation.hxx>
#include <BRep_CurveOnSurface.hxx>
#include <BRep_CurveOnClosedSurface.hxx>
#include <gp_Vec2d.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <BinTools.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <cstdio>
#include <sstream>
#include <fstream>
#include <vector>
#include <array>
#include <cstring>
#include <limits>
#include "sphere-certificate-experiment.hxx"
#include "../../../../crates/om9-kernel/src/axis_box_certificate.hxx"

template <size_t Capacity> class FixedStreamBuffer : public std::streambuf {
 std::array<char,Capacity> bytes_{};
public:
 FixedStreamBuffer() {setp(bytes_.data(),bytes_.data()+bytes_.size());}
 size_t size() const {return static_cast<size_t>(pptr()-pbase());}
 const char* data() const {return bytes_.data();}
protected:
 int_type overflow(int_type) override {return traits_type::eof();}
};

template <size_t Capacity> static bool bounded_binary(const TopoDS_Shape& shape, FixedStreamBuffer<Capacity>& buffer) {
 std::ostream out(&buffer);
 BinTools::Write(shape,out,false,false,BinTools_FormatVersion_CURRENT);
 return out.good();
}

template <size_t Capacity> static bool bounded_text(const TopoDS_Shape& shape, FixedStreamBuffer<Capacity>& buffer) {
 std::ostream out(&buffer); BRepTools::Write(shape,out); return out.good();
}

static std::string binary_brep_without_mesh(const TopoDS_Shape& shape, BinTools_FormatVersion version=BinTools_FormatVersion_CURRENT) {
 std::ostringstream stream(std::ios::binary);
 BinTools::Write(shape,stream,false,false,version);
 return stream.str();
}

// A strict diagnostic preflight. Unsupported representations are rejected;
// a successful result is not, by itself, proof of shape equivalence.
static bool bounded_location(TopLoc_Location location) {
 int links=0;
 while (!location.IsIdentity()) {
  if (++links>8) return false;
  const gp_Trsf trsf=location.FirstDatum()->Trsf();
  for (int i=1;i<=3;++i) for (int j=1;j<=4;++j) if (!std::isfinite(trsf.Value(i,j))) return false;
  location=location.NextLocation();
 }
 return true;
}

static bool sphere_preflight(const TopoDS_Shape& shape) {
 if (shape.IsNull() || shape.ShapeType()!=TopAbs_SOLID || shape.Orientation()!=TopAbs_FORWARD || !bounded_location(shape.Location())) return false;
 const TopAbs_ShapeEnum types[]={TopAbs_SHELL,TopAbs_FACE,TopAbs_WIRE,TopAbs_EDGE,TopAbs_VERTEX};
 const int expected[]={1,1,1,4,8};
 for (int i=0;i<5;++i) {int count=0; for (TopExp_Explorer e(shape,types[i]);e.More();e.Next()) if (++count>expected[i] || !bounded_location(e.Current().Location())) return false; if (count!=expected[i]) return false;}
 TopTools_IndexedMapOfShape faces,edges,vertices;
 TopExp::MapShapes(shape,TopAbs_FACE,faces); TopExp::MapShapes(shape,TopAbs_EDGE,edges); TopExp::MapShapes(shape,TopAbs_VERTEX,vertices);
 if (faces.Extent()!=1 || edges.Extent()!=3 || vertices.Extent()!=2) return false;
 TopLoc_Location location; auto surface=BRep_Tool::Surface(TopoDS::Face(faces(1)),location);
 if (surface.IsNull() || surface->DynamicType()!=STANDARD_TYPE(Geom_SphericalSurface) || !bounded_location(location)) return false;
 const auto face=TopoDS::Face(faces(1)); const auto sphere=BRepAdaptor_Surface(face).Sphere();
 const auto center=sphere.Location();
 if (!std::isfinite(center.X()) || !std::isfinite(center.Y()) || !std::isfinite(center.Z()) || !std::isfinite(sphere.Radius()) || sphere.Radius()<=0. || !std::isfinite(BRep_Tool::Tolerance(face))) return false;
 for (int i=1;i<=vertices.Extent();++i) {
  auto data=occ::handle<BRep_TVertex>::DownCast(vertices(i).TShape());
  if (data.IsNull() || !data->Points().IsEmpty()) return false;
  const auto v=TopoDS::Vertex(vertices(i)); const auto p=BRep_Tool::Pnt(v);
  if (!std::isfinite(p.X()) || !std::isfinite(p.Y()) || !std::isfinite(p.Z()) || !std::isfinite(BRep_Tool::Tolerance(v))) return false;
 }
 for (int i=1;i<=edges.Extent();++i) {
  auto edge=TopoDS::Edge(edges(i)); auto data=occ::handle<BRep_TEdge>::DownCast(edge.TShape());
  if (data.IsNull() || data->Curves().Size()!=2 || !std::isfinite(BRep_Tool::Tolerance(edge))) return false;
  int curves=0,surfaces=0;
  for (const auto& r:data->Curves()) {
   if (r.IsNull() || !bounded_location(r->Location())) return false;
   auto gc=occ::handle<BRep_GCurve>::DownCast(r); if (gc.IsNull()) return false;
   double first,last; gc->Range(first,last); if (!std::isfinite(first) || !std::isfinite(last)) return false;
   if (r->DynamicType()==STANDARD_TYPE(BRep_Curve3D)) {
    ++curves; auto c=r->Curve3D();
    if (BRep_Tool::Degenerated(edge) ? !c.IsNull() : (c.IsNull() || c->DynamicType()!=STANDARD_TYPE(Geom_Circle))) return false;
    if (c.IsNull() && !r->Location().IsIdentity()) return false;
   } else if (r->DynamicType()==STANDARD_TYPE(BRep_CurveOnSurface) || r->DynamicType()==STANDARD_TYPE(BRep_CurveOnClosedSurface)) {
    ++surfaces;
    if (r->Surface().IsNull() || r->Surface()->DynamicType()!=STANDARD_TYPE(Geom_SphericalSurface) || r->PCurve().IsNull() || r->PCurve()->DynamicType()!=STANDARD_TYPE(Geom2d_Line)) return false;
    if (r->IsCurveOnClosedSurface() && (r->PCurve2().IsNull() || r->PCurve2()->DynamicType()!=STANDARD_TYPE(Geom2d_Line))) return false;
    auto c=occ::handle<BRep_CurveOnSurface>::DownCast(r); gp_Pnt2d a,b; c->UVPoints(a,b);
    if (!std::isfinite(a.X()) || !std::isfinite(a.Y()) || !std::isfinite(b.X()) || !std::isfinite(b.Y())) return false;
    if (r->IsCurveOnClosedSurface()) {auto cs=occ::handle<BRep_CurveOnClosedSurface>::DownCast(r); cs->UVPoints2(a,b); if (!std::isfinite(a.X()) || !std::isfinite(a.Y()) || !std::isfinite(b.X()) || !std::isfinite(b.Y())) return false;}
   } else return false;
  }
  if (curves!=1 || surfaces!=1) return false;
 }
 return true;
}

static std::vector<double> uv_cache_values(const TopoDS_Shape& shape) {
 std::vector<double> values; TopTools_IndexedMapOfShape edges; TopExp::MapShapes(shape,TopAbs_EDGE,edges);
 for (int i=1;i<=edges.Extent();++i) {
  auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(edges(i)).TShape());
  for (const auto& repr:data->Curves()) {
   if (!repr->IsCurveOnSurface()) continue;
   const auto curve=occ::handle<BRep_CurveOnSurface>::DownCast(repr);
   gp_Pnt2d a,b; curve->UVPoints(a,b);
   values.insert(values.end(),{a.X(),a.Y(),b.X(),b.Y()});
   if (repr->IsCurveOnClosedSurface()) {
    auto closed=occ::handle<BRep_CurveOnClosedSurface>::DownCast(repr);
    closed->UVPoints2(a,b); values.insert(values.end(),{a.X(),a.Y(),b.X(),b.Y()});
   }
  }
 }
 return values;
}

// Preflight has already restricted the exact representation count. BinTools
// does not write the range/location of a null 3D curve, or the UV caches.
struct OmittedValues {
 std::array<double,32> values{};
 size_t count=0;
 bool add(double value) {if (!std::isfinite(value) || count==values.size()) return false; values[count++]=value; return true;}
};

static bool omitted_values(const TopoDS_Shape& shape,OmittedValues& result) {
 TopTools_IndexedMapOfShape edges; TopExp::MapShapes(shape,TopAbs_EDGE,edges);
 for (int i=1;i<=edges.Extent();++i) {
  auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(edges(i)).TShape());
  for (const auto& repr:data->Curves()) {
   if (repr->IsCurve3D() && repr->Curve3D().IsNull()) {
    double a,b; occ::handle<BRep_GCurve>::DownCast(repr)->Range(a,b);
    if (!result.add(a) || !result.add(b)) return false;
   }
   if (!repr->IsCurveOnSurface()) continue;
   auto curve=occ::handle<BRep_CurveOnSurface>::DownCast(repr); gp_Pnt2d a,b; curve->UVPoints(a,b);
   if (!result.add(a.X()) || !result.add(a.Y()) || !result.add(b.X()) || !result.add(b.Y())) return false;
   if (repr->IsCurveOnClosedSurface()) {
    occ::handle<BRep_CurveOnClosedSurface>::DownCast(repr)->UVPoints2(a,b);
    if (!result.add(a.X()) || !result.add(a.Y()) || !result.add(b.X()) || !result.add(b.Y())) return false;
   }
  }
 }
 return true;
}

static bool text_roundtrip(const TopoDS_Shape& reference,TopoDS_Shape& normalized) {
 FixedStreamBuffer<8192> text_buffer; if (!bounded_text(reference,text_buffer)) return false;
 std::istringstream in(std::string(text_buffer.data(),text_buffer.size())); BRep_Builder builder;
 BRepTools::Read(normalized,in,builder);
 return in.good() && !normalized.IsNull();
}

static bool candidate_equals_source(const TopoDS_Shape& source,const TopoDS_Shape& reference) {
 TopoDS_Shape normalized; if (!text_roundtrip(reference,normalized) || !sphere_preflight(normalized) || !BRepCheck_Analyzer(normalized).IsValid()) return false;
 FixedStreamBuffer<8192> a,b; if (!bounded_binary(source,a) || !bounded_binary(normalized,b) || a.size()!=b.size() || std::memcmp(a.data(),b.data(),a.size())!=0) return false;
 OmittedValues x,y;
 return omitted_values(source,x) && omitted_values(normalized,y) && x.count==y.count && x.values==y.values;
}

static bool experimental_canonical_identity(const TopoDS_Shape& source) {
 if (!sphere_preflight(source) || !BRepCheck_Analyzer(source).IsValid()) return false;
 TopTools_IndexedMapOfShape faces; TopExp::MapShapes(source,TopAbs_FACE,faces);
 const auto sphere=BRepAdaptor_Surface(TopoDS::Face(faces(1))).Sphere();
 BRepPrimAPI_MakeSphere maker(sphere.Position().Ax2(),sphere.Radius()); auto reference=maker.Shape();
 if (!maker.IsDone() || reference.IsNull()) return false;
 if (candidate_equals_source(source,reference)) return true;
 auto origin=sphere.Position().Ax2(); origin.SetLocation(gp_Pnt(0,0,0));
 BRepPrimAPI_MakeSphere origin_maker(origin,sphere.Radius()); TopoDS_Shape origin_saved;
 const auto origin_shape=origin_maker.Shape();
 if (!origin_maker.IsDone() || !text_roundtrip(origin_shape,origin_saved)) return false;
 gp_Trsf translation; translation.SetTranslation(gp_Vec(sphere.Location().X(),sphere.Location().Y(),sphere.Location().Z()));
 BRepBuilderAPI_Transform transform(origin_saved,translation,true);
 return transform.IsDone() && candidate_equals_source(source,transform.Shape());
}

static bool product_mutation_checks(const TopoDS_Shape& source) {
 FixedStreamBuffer<8192> saved; if (!bounded_text(source,saved)) return false;
 auto load=[&]() {
  std::istringstream in(std::string(saved.data(),saved.size())); TopoDS_Shape s; BRep_Builder b;
  BRepTools::Read(s,in,b); return s;
 };
 auto baseline=load(); if (!experimental_canonical_identity(baseline)) return false;
 auto pole=load(); TopTools_IndexedMapOfShape vertices; TopExp::MapShapes(pole,TopAbs_VERTEX,vertices);
 auto vertex=TopoDS::Vertex(vertices(1)); auto p=BRep_Tool::Pnt(vertex); p.SetZ(std::nextafter(p.Z(),INFINITY));
 BRep_Builder b; b.UpdateVertex(vertex,p,BRep_Tool::Tolerance(vertex));
 auto uv=load(); TopTools_IndexedMapOfShape edges; TopExp::MapShapes(uv,TopAbs_EDGE,edges); bool changed_uv=false;
 for (int i=1;i<=edges.Extent() && !changed_uv;++i) {
  auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(edges(i)).TShape());
  for (const auto& r:data->Curves()) if (r->IsCurveOnSurface()) {
   auto c=occ::handle<BRep_CurveOnSurface>::DownCast(r); gp_Pnt2d a,z; c->UVPoints(a,z);
   a.SetX(std::nextafter(a.X(),INFINITY)); c->SetUVPoints(a,z); changed_uv=true; break;
  }
 }
 auto extra=load(); TopTools_IndexedMapOfShape extra_edges; TopExp::MapShapes(extra,TopAbs_EDGE,extra_edges);
 auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(extra_edges(1)).TShape());
 data->ChangeCurves().Append(new BRep_Curve3D(occ::handle<Geom_Curve>(),TopLoc_Location()));
 const bool pole_rejected=!experimental_canonical_identity(pole),uv_rejected=!experimental_canonical_identity(uv),extra_rejected=!experimental_canonical_identity(extra);
 std::printf("product mutation gate pole=%d uv=%d extra=%d\n",pole_rejected,uv_rejected,extra_rejected);
 return changed_uv && pole_rejected && uv_rejected && extra_rejected;
}

static bool vertices_survive_roundtrip_exactly(const TopoDS_Shape& source) {
 std::stringstream stream; BRepTools::Write(source,stream);
 TopoDS_Shape copy; BRep_Builder reader; BRepTools::Read(copy,stream,reader);
 TopTools_IndexedMapOfShape a,b; TopExp::MapShapes(source,TopAbs_VERTEX,a); TopExp::MapShapes(copy,TopAbs_VERTEX,b);
 if (a.Extent()!=b.Extent()) return false;
 for (int i=1;i<=a.Extent();++i) {
  auto p=BRep_Tool::Pnt(TopoDS::Vertex(a(i))),q=BRep_Tool::Pnt(TopoDS::Vertex(b(i)));
  if (p.X()!=q.X() || p.Y()!=q.Y() || p.Z()!=q.Z()) return false;
 }
 return true;
}

static void inspect(const char* label, const TopoDS_Shape& shape) {
 int occurrences=0; for (TopExp_Explorer e(shape,TopAbs_VERTEX);e.More();e.Next()) ++occurrences;
 TopTools_IndexedMapOfShape shells,faces,wires,edges,vertices;
 TopExp::MapShapes(shape,TopAbs_SHELL,shells); TopExp::MapShapes(shape,TopAbs_FACE,faces);
 TopExp::MapShapes(shape,TopAbs_WIRE,wires); TopExp::MapShapes(shape,TopAbs_EDGE,edges);
 TopExp::MapShapes(shape,TopAbs_VERTEX,vertices);
 std::printf("%s valid=%d shells=%d faces=%d wires=%d edges=%d vertices=%d occurrences=%d orientation=%d\n",label,BRepCheck_Analyzer(shape).IsValid(),shells.Extent(),faces.Extent(),wires.Extent(),edges.Extent(),vertices.Extent(),occurrences,int(shape.Orientation()));
 for (int i=1;i<=edges.Extent();++i) {
  auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(edges(i)).TShape()); int n=0;
  for (const auto& r:data->Curves()) {++n; std::printf(" edge %d repr %s",i,r->DynamicType()->Name());
   if (r->IsCurve3D() && !r->Curve3D().IsNull()) std::printf(" curve=%s",r->Curve3D()->DynamicType()->Name());
   if (r->IsCurveOnSurface()) std::printf(" surface=%s pcurve=%s",r->Surface()->DynamicType()->Name(),r->PCurve()->DynamicType()->Name());
   std::printf("\n"); }
  std::printf(" edge %d repr_count=%d\n",i,n);
 }
 for (int i=1;i<=vertices.Extent();++i) {
  auto data=occ::handle<BRep_TVertex>::DownCast(TopoDS::Vertex(vertices(i)).TShape()); int n=0;
  for (const auto& r:data->Points()) {++n; std::printf(" vertex %d repr %s\n",i,r->DynamicType()->Name());}
  std::printf(" vertex %d repr_count=%d\n",i,n);
 }
 for (int i=1;i<=faces.Extent();++i) {
  auto face=TopoDS::Face(faces(i)); BRepAdaptor_Surface surface(face);
  double u0,u1,v0,v1; BRepTools::UVBounds(face,u0,u1,v0,v1);
  std::printf(" face=%d surface=%d orientation=%d uv=(%.17g,%.17g,%.17g,%.17g) tol=%.17g\n",i,int(surface.GetType()),int(face.Orientation()),u0,u1,v0,v1,BRep_Tool::Tolerance(face));
  for (TopExp_Explorer e(face,TopAbs_EDGE);e.More();e.Next()) {
   auto edge=TopoDS::Edge(e.Current()); double first,last;
   auto curve=BRep_Tool::CurveOnSurface(edge,face,first,last);
   std::printf("  edge=%d ori=%d deg=%d closed=%d tol=%.17g",edges.FindIndex(edge),int(edge.Orientation()),BRep_Tool::Degenerated(edge),BRep_Tool::IsClosed(edge,face),BRep_Tool::Tolerance(edge));
   if (!curve.IsNull()) {Geom2dAdaptor_Curve c(curve,first,last); auto a=c.Value(first),b=c.Value(last); std::printf(" type=%d uv=(%.17g,%.17g)-(%.17g,%.17g)",int(c.GetType()),a.X(),a.Y(),b.X(),b.Y());}
   std::printf("\n");
  }
 }
}
int main(int argc,char** argv) {
 const double pi=std::acos(-1.);
 auto sphere=BRepPrimAPI_MakeSphere(3.).Shape();
 TopLoc_Location deep;
 for (int i=0;i<8;++i) {gp_Trsf t; t.SetTranslation(gp_Vec(i+1.,0,0)); deep=TopLoc_Location(t)*deep;}
 const bool depth8=bounded_location(deep); gp_Trsf ninth; ninth.SetTranslation(gp_Vec(9.,0,0)); deep=TopLoc_Location(ninth)*deep;
 const bool depth9=bounded_location(deep);
 std::printf("location preflight depth8=%d depth9=%d\n",depth8,depth9);
 if (!depth8 || depth9) return 17;
 auto nan_uv=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape nan_edges;
 TopExp::MapShapes(nan_uv,TopAbs_EDGE,nan_edges); bool mutated_nan=false;
 for (int i=1;i<=nan_edges.Extent() && !mutated_nan;++i) {
  auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(nan_edges(i)).TShape());
  for (const auto& r:data->Curves()) {
   if (!r->IsCurveOnSurface()) continue;
   auto c=occ::handle<BRep_CurveOnSurface>::DownCast(r); gp_Pnt2d a,b; c->UVPoints(a,b);
   a.SetX(std::numeric_limits<double>::quiet_NaN()); c->SetUVPoints(a,b); mutated_nan=true; break;
  }
 }
 const bool nan_accepted=sphere_preflight(nan_uv);
 std::printf("nonfinite UV preflight accepted=%d\n",nan_accepted);
 if (!mutated_nan || nan_accepted) return 18;
 auto extra_null=BRepPrimAPI_MakeSphere(3.).Shape();
 const auto before_extra=binary_brep_without_mesh(extra_null); TopTools_IndexedMapOfShape extra_edges;
 TopExp::MapShapes(extra_null,TopAbs_EDGE,extra_edges);
 auto extra_data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(extra_edges(1)).TShape());
 extra_data->ChangeCurves().Append(new BRep_Curve3D(occ::handle<Geom_Curve>(),TopLoc_Location()));
 std::printf("extra null representation binary_equal=%d preflight_accepted=%d\n",binary_brep_without_mesh(extra_null)==before_extra,sphere_preflight(extra_null));
 if (!sphere_preflight(sphere) || sphere_preflight(extra_null)) return 15;
 FixedStreamBuffer<8192> bounded_sphere; FixedStreamBuffer<64> tiny_buffer;
 FixedStreamBuffer<8192> bounded_ascii; FixedStreamBuffer<64> tiny_ascii;
 const bool bounded_ok=bounded_binary(sphere,bounded_sphere);
 const bool tiny_ok=bounded_binary(sphere,tiny_buffer);
 const bool ascii_ok=bounded_text(sphere,bounded_ascii),tiny_ascii_ok=bounded_text(sphere,tiny_ascii);
 const auto unbounded_sphere=binary_brep_without_mesh(sphere);
 const bool identical=bounded_sphere.size()==unbounded_sphere.size() &&
  std::memcmp(bounded_sphere.data(),unbounded_sphere.data(),bounded_sphere.size())==0;
 std::printf("bounded binary passed=%d bytes=%zu matches_stream=%d tiny_accepted=%d; text_passed=%d text_bytes=%zu tiny_text_accepted=%d\n",bounded_ok,bounded_sphere.size(),identical,tiny_ok,ascii_ok,bounded_ascii.size(),tiny_ascii_ok);
 if (!bounded_ok || !identical || tiny_ok || !ascii_ok || tiny_ascii_ok) return 14;
 auto longitude=BRepPrimAPI_MakeSphere(3.,pi).Shape();
 auto latitude=BRepPrimAPI_MakeSphere(3.,-pi/4,pi/4).Shape();
 inspect("full",sphere); inspect("partial longitude",longitude); inspect("partial latitude",latitude);
 auto reversed=sphere.Reversed();
 gp_GTrsf stretch; stretch.SetValue(1,1,2.);
 auto ellipsoid=BRepBuilderAPI_GTransform(sphere,stretch,true).Shape();
 if (sphere_preflight(longitude) || sphere_preflight(latitude) || sphere_preflight(reversed) || sphere_preflight(ellipsoid)) return 16;
 auto tool=BRepPrimAPI_MakeBox(gp_Pnt(0,-4,-4),gp_Pnt(4,4,4)).Shape();
 BRepAlgoAPI_Cut cut(sphere,tool); cut.Build();
 if (experimental_canonical_identity(longitude) || experimental_canonical_identity(latitude) ||
     experimental_canonical_identity(reversed) || experimental_canonical_identity(ellipsoid) ||
     experimental_canonical_identity(cut.Shape())) return 24;
 auto altered=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape altered_faces,altered_edges;
 TopExp::MapShapes(altered,TopAbs_FACE,altered_faces); auto altered_face=TopoDS::Face(altered_faces(1));
 TopExp::MapShapes(altered_face,TopAbs_EDGE,altered_edges); double first,last; bool stored=false;
 auto trim=BRep_Tool::CurveOnSurface(TopoDS::Edge(altered_edges(1)),altered_face,first,last,&stored);
 if (trim.IsNull() || !stored) return 2; trim->Translate(gp_Vec2d(0.,1e-9));
 auto tilted=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape tilted_edges;
 TopExp::MapShapes(tilted,TopAbs_EDGE,tilted_edges);
 for (int i=1;i<=tilted_edges.Extent();++i) {
  auto edge=TopoDS::Edge(tilted_edges(i)); if (BRep_Tool::Degenerated(edge)) continue;
  BRepAdaptor_Curve old(edge); gp_Circ circle=old.Circle(); circle.Rotate(gp_Ax1(gp_Pnt(0,0,0),gp_Dir(0,0,1)),1e-9);
  BRep_Builder builder; builder.UpdateEdge(edge,new Geom_Circle(circle),BRep_Tool::Tolerance(edge));
 }
 auto phase=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape phase_edges;
 TopExp::MapShapes(phase,TopAbs_EDGE,phase_edges);
 for (int i=1;i<=phase_edges.Extent();++i) {
  auto edge=TopoDS::Edge(phase_edges(i)); if (BRep_Tool::Degenerated(edge)) continue;
  BRepAdaptor_Curve old(edge); gp_Circ circle=old.Circle(); const auto original=circle.Position();
  std::printf("original circle x=(%.17g %.17g %.17g) y=(%.17g %.17g %.17g)\n",original.XDirection().X(),original.XDirection().Y(),original.XDirection().Z(),original.YDirection().X(),original.YDirection().Y(),original.YDirection().Z());
  circle.Rotate(circle.Axis(),1e-9);
  const auto p=circle.Position();
  std::printf("shifted circle x=(%.17g %.17g %.17g) y=(%.17g %.17g %.17g)\n",p.XDirection().X(),p.XDirection().Y(),p.XDirection().Z(),p.YDirection().X(),p.YDirection().Y(),p.YDirection().Z());
  BRep_Builder builder; builder.UpdateEdge(edge,new Geom_Circle(circle),BRep_Tool::Tolerance(edge));
 }
 auto moved_pole=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape pole_vertices;
 TopExp::MapShapes(moved_pole,TopAbs_VERTEX,pole_vertices); auto pole=TopoDS::Vertex(pole_vertices(1));
 auto moved=BRep_Tool::Pnt(pole); moved.SetZ(moved.Z()+1e-9);
 auto before=BRep_Tool::Pnt(pole); std::printf("original pole=(%.17g %.17g %.17g)\n",before.X(),before.Y(),before.Z());
 BRep_Builder change_pole; change_pole.UpdateVertex(pole,moved,BRep_Tool::Tolerance(pole));
 auto one_ulp=BRepPrimAPI_MakeSphere(3.).Shape(); std::stringstream before_ulp; BRepTools::Write(one_ulp,before_ulp);
 TopTools_IndexedMapOfShape ulp_vertices; TopExp::MapShapes(one_ulp,TopAbs_VERTEX,ulp_vertices);
 auto ulp_vertex=TopoDS::Vertex(ulp_vertices(1)); auto ulp_point=BRep_Tool::Pnt(ulp_vertex);
 ulp_point.SetZ(std::nextafter(ulp_point.Z(),INFINITY)); BRep_Builder ulp_builder;
 ulp_builder.UpdateVertex(ulp_vertex,ulp_point,BRep_Tool::Tolerance(ulp_vertex));
 std::stringstream after_ulp; BRepTools::Write(one_ulp,after_ulp);
 const auto reference_ulp=BRepPrimAPI_MakeSphere(3.).Shape();
 std::printf("one_ulp_changed_pole same_serialization=%d binary_equal=%d valid=%d vertices_exact_after_roundtrip=%d\n",before_ulp.str()==after_ulp.str(),binary_brep_without_mesh(one_ulp)==binary_brep_without_mesh(reference_ulp),BRepCheck_Analyzer(one_ulp).IsValid(),vertices_survive_roundtrip_exactly(one_ulp));
 if (before_ulp.str()!=after_ulp.str() || vertices_survive_roundtrip_exactly(one_ulp)) return 4;
 auto shifted_range=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape range_edges;
 TopExp::MapShapes(shifted_range,TopAbs_EDGE,range_edges);
 for (int i=1;i<=range_edges.Extent();++i) {
  auto edge=TopoDS::Edge(range_edges(i)); if (BRep_Tool::Degenerated(edge)) continue;
  BRepAdaptor_Curve old(edge); BRep_Builder builder;
  builder.Range(edge,old.FirstParameter()+1e-9,old.LastParameter()+1e-9,true);
 }
 int yes=experimental_full_sphere(sphere), a=experimental_full_sphere(longitude), b=experimental_full_sphere(latitude);
 int c=experimental_full_sphere(reversed), d=experimental_full_sphere(ellipsoid), e=experimental_full_sphere(cut.Shape());
 int f=experimental_full_sphere(altered);
 int g=experimental_full_sphere(tilted);
 int h=experimental_full_sphere(phase);
 int j=experimental_full_sphere(moved_pole);
 int k=experimental_full_sphere(shifted_range);
 auto changed_flag=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape flag_edges;
 TopExp::MapShapes(changed_flag,TopAbs_EDGE,flag_edges); BRep_Builder flag_builder;
 flag_builder.SameRange(TopoDS::Edge(flag_edges(1)),false);
 const auto expected_binary=binary_brep_without_mesh(BRepPrimAPI_MakeSphere(3.).Shape());
 if (binary_brep_without_mesh(BRepPrimAPI_MakeSphere(3.).Shape())!=expected_binary) return 12;
 auto changed_uv_cache=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape cached_edges;
 const auto before_uv_cache=binary_brep_without_mesh(changed_uv_cache);
 const auto before_uv_cache_v3=binary_brep_without_mesh(changed_uv_cache,BinTools_FormatVersion_VERSION_3);
 const auto baseline_uv_cache=uv_cache_values(changed_uv_cache);
 TopExp::MapShapes(changed_uv_cache,TopAbs_EDGE,cached_edges); bool mutated_uv_cache=false;
 for (int i=1;i<=cached_edges.Extent() && !mutated_uv_cache;++i) {
  const auto edge=TopoDS::Edge(cached_edges(i));
  auto data=occ::handle<BRep_TEdge>::DownCast(edge.TShape());
  for (const auto& repr : data->Curves()) {
   if (!repr->IsCurveOnSurface() || repr->IsCurveOnClosedSurface()) continue;
   auto curve=occ::handle<BRep_CurveOnSurface>::DownCast(repr); gp_Pnt2d a,b;
   curve->UVPoints(a,b); a.SetX(a.X()+1e-9); curve->SetUVPoints(a,b); mutated_uv_cache=true; break;
  }
 }
 if (!mutated_uv_cache) return 8;
 const bool same_v4=binary_brep_without_mesh(changed_uv_cache)==before_uv_cache;
 const bool same_v3=binary_brep_without_mesh(changed_uv_cache,BinTools_FormatVersion_VERSION_3)==before_uv_cache_v3;
 std::printf("changed_uv_cache binary_v4_equal=%d binary_v3_equal=%d valid=%d\n",same_v4,same_v3,BRepCheck_Analyzer(changed_uv_cache).IsValid());
 if (!same_v4 || same_v3 || uv_cache_values(changed_uv_cache)==baseline_uv_cache) return 9;
 auto changed_closed_uv=BRepPrimAPI_MakeSphere(3.).Shape(); TopTools_IndexedMapOfShape closed_edges;
 const auto before_closed_v3=binary_brep_without_mesh(changed_closed_uv,BinTools_FormatVersion_VERSION_3);
 const auto baseline_closed_uv=uv_cache_values(changed_closed_uv);
 TopExp::MapShapes(changed_closed_uv,TopAbs_EDGE,closed_edges); bool mutated_closed=false;
 for (int i=1;i<=closed_edges.Extent() && !mutated_closed;++i) {
  auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(closed_edges(i)).TShape());
  for (const auto& repr:data->Curves()) {
   if (!repr->IsCurveOnClosedSurface()) continue;
   auto curve=occ::handle<BRep_CurveOnSurface>::DownCast(repr); gp_Pnt2d a,b;
   curve->UVPoints(a,b); a.SetX(a.X()+1e-9); curve->SetUVPoints(a,b); mutated_closed=true; break;
  }
 }
 if (!mutated_closed) return 10;
 const bool closed_same_v3=binary_brep_without_mesh(changed_closed_uv,BinTools_FormatVersion_VERSION_3)==before_closed_v3;
 std::printf("changed_closed_seam_first_uv_cache binary_v3_equal=%d valid=%d\n",closed_same_v3,BRepCheck_Analyzer(changed_closed_uv).IsValid());
 if (!closed_same_v3 || uv_cache_values(changed_closed_uv)==baseline_closed_uv) return 11;
 int binary_false_accepts=0;
 for (const auto& bad : {longitude,latitude,reversed,ellipsoid,cut.Shape(),altered,tilted,phase,moved_pole,shifted_range,changed_flag,one_ulp})
  if (binary_brep_without_mesh(bad)==expected_binary) ++binary_false_accepts;
 std::printf("experimental certificate full=%d longitude=%d latitude=%d reversed=%d ellipsoid=%d cut=%d shifted_trim=%d shifted_trim_valid=%d\n",yes,a,b,c,d,e,f,BRepCheck_Analyzer(altered).IsValid());
 std::printf("tilted_seam_1e-9 accepted=%d valid=%d\n",g,BRepCheck_Analyzer(tilted).IsValid());
 std::printf("shifted_seam_phase_1e-9 accepted=%d valid=%d\n",h,BRepCheck_Analyzer(phase).IsValid());
 std::printf("shifted_pole_1e-9 accepted=%d valid=%d\n",j,BRepCheck_Analyzer(moved_pole).IsValid());
 std::printf("shifted_range_1e-9 accepted=%d valid=%d\n",k,BRepCheck_Analyzer(shifted_range).IsValid());
 std::printf("binary comparison false_accepts=%d in 12 mutated fixtures\n",binary_false_accepts);
 if (!yes || a || b || c || d || e || f || g || h || !BRepCheck_Analyzer(phase).IsValid() || j || !BRepCheck_Analyzer(moved_pole).IsValid() || binary_false_accepts) return 1;
 int successes=0,failed=0,reopened_certified=0,canonical_equal=0,vertices_fixed=0,binary_equal=0,uv_equal=0,preflight_count=0,identity_count=0;
 for (double x : {-0.001,-0.000001,0.,0.000001,0.001}) {
  auto source=BRepPrimAPI_MakeSphere(gp_Pnt(x,0,0),3.).Shape();
  std::stringstream serialized; BRepTools::Write(source,serialized); TopoDS_Shape reopened; BRep_Builder reader;
  BRepTools::Read(reopened,serialized,reader);
  if (sphere_preflight(reopened)) ++preflight_count;
  if (experimental_canonical_identity(reopened)) ++identity_count;
  if (!reopened.IsNull() && experimental_full_sphere(reopened)) ++reopened_certified;
  if (vertices_survive_roundtrip_exactly(reopened)) ++vertices_fixed;
  TopTools_IndexedMapOfShape rf; TopExp::MapShapes(reopened,TopAbs_FACE,rf);
  auto carrier=BRepAdaptor_Surface(TopoDS::Face(rf(1))).Sphere();
  auto canonical=BRepPrimAPI_MakeSphere(carrier.Position().Ax2(),carrier.Radius()).Shape();
  std::stringstream out_a,out_b; BRepTools::Write(reopened,out_a); BRepTools::Write(canonical,out_b);
  if (out_a.str()==out_b.str()) ++canonical_equal;
  std::stringstream normalized_text(out_b.str()); TopoDS_Shape normalized; BRep_Builder normalized_reader;
  BRepTools::Read(normalized,normalized_text,normalized_reader);
  if (binary_brep_without_mesh(reopened)==binary_brep_without_mesh(normalized)) ++binary_equal;
  if (uv_cache_values(reopened)==uv_cache_values(normalized)) ++uv_equal;
  if (x==0.) {
   inspect("reopened",reopened);
   TopTools_IndexedMapOfShape reopened_faces; TopExp::MapShapes(reopened,TopAbs_FACE,reopened_faces);
   auto ref_sphere=BRepAdaptor_Surface(TopoDS::Face(reopened_faces(1))).Sphere();
   auto regenerated=BRepPrimAPI_MakeSphere(ref_sphere.Position().Ax2(),ref_sphere.Radius()).Shape();
   std::stringstream a,b; BRepTools::Write(reopened,a); BRepTools::Write(regenerated,b);
   auto aa=a.str(),bb=b.str(); size_t mismatch=0; while (mismatch<aa.size() && mismatch<bb.size() && aa[mismatch]==bb[mismatch]) ++mismatch;
   std::printf("canonical serialization equal=%d mismatch=%zu source_length=%zu reference_length=%zu\n",aa==bb,mismatch,aa.size(),bb.size());
   if (mismatch<aa.size() && mismatch<bb.size()) std::printf("source: %.100s\nreference: %.100s\n",aa.c_str()+mismatch,bb.c_str()+mismatch);
   std::stringstream input(aa); TopoDS_Shape tiny_curve; BRep_Builder tiny_reader; BRepTools::Read(tiny_curve,input,tiny_reader);
   std::stringstream uv_input(aa); TopoDS_Shape altered_uv; BRepTools::Read(altered_uv,uv_input,tiny_reader);
   TopTools_IndexedMapOfShape uv_edges; TopExp::MapShapes(altered_uv,TopAbs_EDGE,uv_edges);
   bool changed_one=false;
   for (int i=1;i<=uv_edges.Extent() && !changed_one;++i) {
    auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(uv_edges(i)).TShape());
    for (const auto& r:data->Curves()) if (r->IsCurveOnSurface()) {
     auto c=occ::handle<BRep_CurveOnSurface>::DownCast(r); gp_Pnt2d a,b; c->UVPoints(a,b); a.SetX(a.X()+1e-9); c->SetUVPoints(a,b); changed_one=true; break;
    }
   }
   std::stringstream null_input(aa); TopoDS_Shape altered_null; BRepTools::Read(altered_null,null_input,tiny_reader);
   TopTools_IndexedMapOfShape null_edges; TopExp::MapShapes(altered_null,TopAbs_EDGE,null_edges); bool changed_range=false;
   for (int i=1;i<=null_edges.Extent() && !changed_range;++i) {
    auto data=occ::handle<BRep_TEdge>::DownCast(TopoDS::Edge(null_edges(i)).TShape());
    for (const auto& r:data->Curves()) if (r->IsCurve3D() && r->Curve3D().IsNull()) {
     auto gc=occ::handle<BRep_GCurve>::DownCast(r); double a,b; gc->Range(a,b); gc->SetRange(std::nextafter(a,INFINITY),b); changed_range=true; break;
    }
   }
   const bool uv_same_binary=binary_brep_without_mesh(altered_uv)==binary_brep_without_mesh(reopened);
   const bool null_same_binary=binary_brep_without_mesh(altered_null)==binary_brep_without_mesh(reopened);
   std::printf("canonical gate uv_mutation_accepted=%d null_range_mutation_accepted=%d uv_same_binary=%d null_range_same_binary=%d\n",experimental_canonical_identity(altered_uv),experimental_canonical_identity(altered_null),uv_same_binary,null_same_binary);
   if (!changed_one || !changed_range || !uv_same_binary || !null_same_binary || experimental_canonical_identity(altered_uv) || experimental_canonical_identity(altered_null)) return 19;
   TopTools_IndexedMapOfShape tiny_edges; TopExp::MapShapes(tiny_curve,TopAbs_EDGE,tiny_edges);
   for (int i=1;i<=tiny_edges.Extent();++i) {
    auto edge=TopoDS::Edge(tiny_edges(i)); if (BRep_Tool::Degenerated(edge)) continue;
    gp_Circ circle=BRepAdaptor_Curve(edge).Circle(); circle.SetRadius(std::nextafter(circle.Radius(),INFINITY));
    BRep_Builder change; change.UpdateEdge(edge,new Geom_Circle(circle),BRep_Tool::Tolerance(edge));
   }
   std::stringstream tiny_out; BRepTools::Write(tiny_curve,tiny_out);
   std::printf("one_ulp_changed_seam_radius same_serialization=%d binary_equal=%d valid=%d vertices_exact_after_roundtrip=%d\n",aa==tiny_out.str(),binary_brep_without_mesh(tiny_curve)==binary_brep_without_mesh(reopened),BRepCheck_Analyzer(tiny_curve).IsValid(),vertices_survive_roundtrip_exactly(tiny_curve));
   if (!BRepCheck_Analyzer(tiny_curve).IsValid() || !vertices_survive_roundtrip_exactly(tiny_curve)) return 5;
  }
  if (!experimental_full_sphere(source)) {std::printf("source not certified x=%.9g\n",x); ++failed; continue;}
  gp_Ax2 axes(gp_Pnt(x,0,0),gp_Dir(1,0,0),gp_Dir(0,0,-1));
  auto replacement=BRepPrimAPI_MakeSphere(axes,3.).Shape();
  if (!experimental_full_sphere(replacement)) {std::printf("replacement not certified x=%.9g\n",x); ++failed;continue;}
  const double expected=pi*(3.+x)*(3.+x)*(6.-x)/3.;
  for (int order=0;order<2;++order) {
   BRepAlgoAPI_Common common(order ? tool : replacement,order ? replacement : tool);
   common.SetUseOBB(true); common.Build();
   if (!common.IsDone() || common.HasErrors() || common.Shape().IsNull()) {++failed; continue;}
   BRepAlgoAPI_Check check(common.Shape(),true,true); GProp_GProps props;
   BRepGProp::VolumePropertiesGK(common.Shape(),props,1e-10,false,true);
   const bool ok=check.IsValid() && std::abs(props.Mass()-expected)<1e-5;
   std::printf("rotated sphere x=%.9g order=%d valid=%d volume=%.12f expected=%.12f\n",x,order,check.IsValid(),props.Mass(),expected);
   if (ok) ++successes; else ++failed;
  }
 }
 std::printf("rotated sphere successes=%d failures=%d reopened_certified=%d canonical_serialization_equal=%d vertices_fixed=%d binary_equal=%d uv_equal=%d preflight=%d identity=%d\n",successes,failed,reopened_certified,canonical_equal,vertices_fixed,binary_equal,uv_equal,preflight_count,identity_count);
 if (argc>1) {
  std::ifstream in(argv[1],std::ios::binary); TopoDS_Shape actual; BRep_Builder builder; BRepTools::Read(actual,in,builder);
  TopTools_IndexedMapOfShape actual_faces; TopExp::MapShapes(actual,TopAbs_FACE,actual_faces);
  if (actual.IsNull() || actual_faces.Extent()!=1) return 6;
  inspect("product sphere",actual);
  BRepAdaptor_Surface surface(TopoDS::Face(actual_faces(1)));
  if (surface.GetType()!=GeomAbs_Sphere) return 7;
  const auto carrier=surface.Sphere(); auto reference=BRepPrimAPI_MakeSphere(carrier.Position().Ax2(),carrier.Radius()).Shape();
  std::stringstream serialized_reference; BRepTools::Write(reference,serialized_reference); TopoDS_Shape normalized;
  BRepTools::Read(normalized,serialized_reference,builder);
  const bool binary_matches=binary_brep_without_mesh(actual)==binary_brep_without_mesh(normalized);
  const bool uv_matches=uv_cache_values(actual)==uv_cache_values(normalized);
  std::printf("product sphere source valid=%d direct_candidate=%d canonical_normalized_binary_equal=%d uv_equal=%d identity=%d\n",BRepCheck_Analyzer(actual).IsValid(),experimental_full_sphere(actual),binary_matches,uv_matches,experimental_canonical_identity(actual));
  if (!binary_matches) {
   const auto a=binary_brep_without_mesh(actual),b=binary_brep_without_mesh(normalized); size_t pos=0;
   while (pos<a.size() && pos<b.size() && a[pos]==b[pos]) ++pos;
   std::printf("product binary mismatch offset=%zu sizes=%zu,%zu source_root_location_identity=%d reference_root_location_identity=%d center=(%.17g,%.17g,%.17g)\n",pos,a.size(),b.size(),actual.Location().IsIdentity(),normalized.Location().IsIdentity(),carrier.Location().X(),carrier.Location().Y(),carrier.Location().Z());
   std::stringstream ta,tb; BRepTools::Write(actual,ta); BRepTools::Write(normalized,tb); auto sa=ta.str(),sb=tb.str(); size_t q=0;
   while (q<sa.size() && q<sb.size() && sa[q]==sb[q]) ++q;
   const size_t begin=q>50?q-50:0;
   std::printf("product text mismatch offset=%zu\nsource: %.150s\nreference: %.150s\n",q,sa.c_str()+begin,sb.c_str()+begin);
  }
  FixedStreamBuffer<8192> bounded_actual, bounded_reference, bounded_actual_text;
  if (!uv_matches || !sphere_preflight(actual) || !experimental_canonical_identity(actual) || !product_mutation_checks(actual) || !bounded_binary(actual,bounded_actual) ||
      !bounded_binary(normalized,bounded_reference) || !bounded_text(actual,bounded_actual_text)) return 13;
  if (binary_matches && (bounded_actual.size()!=bounded_reference.size() || std::memcmp(bounded_actual.data(),bounded_reference.data(),bounded_actual.size())!=0)) return 13;
  std::printf("product sphere bounded bytes binary=%zu text=%zu\n",bounded_actual.size(),bounded_actual_text.size());
  if (argc>2) {
   std::ifstream box_file(argv[2],std::ios::binary); TopoDS_Shape product_box; BRepTools::Read(product_box,box_file,builder);
   auto box=om9_axis_box::make_operand(product_box); if (box.IsNull()) return 20;
   const auto before_sphere=binary_brep_without_mesh(actual),before_box=binary_brep_without_mesh(product_box);
   const auto before_uv=uv_cache_values(actual); const auto center=carrier.Location();
   gp_Ax2 rotated(center,gp_Dir(1,0,0),gp_Dir(0,0,-1));
   auto temporary=BRepPrimAPI_MakeSphere(rotated,carrier.Radius()).Shape();
   const double r=carrier.Radius(), x=center.X();
   const double expected=pi*(r+x)*(r+x)*(2.*r-x)/3.;
   for (int order=0;order<2;++order) {
    BRepAlgoAPI_Common common(order ? box : temporary,order ? temporary : box);
    common.SetUseOBB(true); common.Build();
    if (!common.IsDone() || common.HasErrors() || common.Shape().IsNull()) return 21;
    BRepAlgoAPI_Check check(common.Shape(),true,true); GProp_GProps props;
    BRepGProp::VolumePropertiesGK(common.Shape(),props,1e-10,false,true);
    std::printf("product kernel source oriented sphere common order=%d valid=%d volume=%.12f\n",order,check.IsValid(),props.Mass());
    if (!check.IsValid() || std::abs(props.Mass()-expected)>1e-5) return 22;
   }
   if (before_sphere!=binary_brep_without_mesh(actual) || before_box!=binary_brep_without_mesh(product_box) || before_uv!=uv_cache_values(actual)) return 23;
   std::printf("product kernel source representations unchanged after both common operations\n");
  }
 }
 return successes==10 && failed==0 && reopened_certified==0 && canonical_equal==5 && vertices_fixed==5 && binary_equal==5 && uv_equal==5 && preflight_count==5 && identity_count==5 ? 0 : 3;
}
