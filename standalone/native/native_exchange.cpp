// SPDX-License-Identifier: LGPL-2.1-or-later
// Owned geometry and selected retained current-state SDK adapters.
// Run only under the om9-native-owner process/job policy. Rust owns action/unit/
// identity policy and publication. No plugins, macros or C++ safety claim.
#include <opennurbs.h>
#include <json.hpp>
#include <algorithm>
#include <cmath>
#include <filesystem>
#include <fstream>
#include <functional>
#include <iostream>
#include <map>
#include <memory>
#include <set>
#include <stdexcept>
#include <string>
#include <vector>
#include <fcntl.h>
#include <io.h>
#include <share.h>
#include <sys/stat.h>
#include <bcrypt.h>

namespace {
using J = nlohmann::json;
constexpr size_t max_depth = 32, max_nodes = 262144, max_array = 65536, max_records = 4096;
constexpr uintmax_t max_packet_bytes = 16 * 1024 * 1024, max_archive_bytes = 64 * 1024 * 1024;
constexpr const char* default_layer = "ffffffff-ffff-4fff-8fff-ffffffffffff";
std::string operation_record = "arguments";
void require(bool yes, const char* message) { if (!yes) throw std::runtime_error(message); }
struct Library { Library() { ON::Begin(); } ~Library() { ON::End(); } };
struct File {
  FILE* value=nullptr;
  explicit File(const std::filesystem::path& p,bool deny_writes=false) {
    if(deny_writes) { int fd=-1; require(_wsopen_s(&fd,p.c_str(),_O_RDONLY|_O_BINARY,_SH_DENYWR,_S_IREAD)==0,"open held native archive"); value=_fdopen(fd,"rb"); if(!value)_close(fd); }
    else value=ON::OpenFile(p.c_str(),L"rb");
    require(value,"open native archive");
  }
  ~File() { if(value)ON::CloseFile(value); }
  File(const File&)=delete;
  File& operator=(const File&)=delete;
};
struct PrivateFile {
  std::filesystem::path path;
  FILE* value=nullptr;
  bool complete=false;
  explicit PrivateFile(const std::filesystem::path& p) : path(p) {
    int fd=-1;
    require(_wsopen_s(&fd,p.c_str(),_O_WRONLY|_O_CREAT|_O_EXCL|_O_BINARY,_SH_DENYRW,_S_IREAD|_S_IWRITE)==0,"exclusive private output creation");
    value=_fdopen(fd,"wb");
    if(!value) { _close(fd); std::error_code ignored; std::filesystem::remove(path,ignored); throw std::runtime_error("private output stream"); }
  }
  ~PrivateFile() { if(value)std::fclose(value); if(!complete) { std::error_code ignored; std::filesystem::remove(path,ignored); } }
  PrivateFile(const PrivateFile&)=delete;
  PrivateFile& operator=(const PrivateFile&)=delete;
};

// A first SAX pass bounds the JSON tree before a DOM or native geometry exists.
// The enclosing file-byte limit bounds the parser's token buffer too.
struct Preflight : nlohmann::json_sax<J> {
  struct Frame { bool object; size_t count = 0; std::set<std::string> keys; };
  std::vector<Frame> frames;
  size_t nodes = 0;
  bool node() {
    require(++nodes <= max_nodes, "packet node budget");
    if (!frames.empty()) require(++frames.back().count <= max_array, "packet container budget");
    return true;
  }
  bool null() override { return node(); }
  bool boolean(bool) override { return node(); }
  bool number_integer(number_integer_t) override { return node(); }
  bool number_unsigned(number_unsigned_t) override { return node(); }
  bool number_float(number_float_t value, const string_t&) override { require(std::isfinite(value), "nonfinite JSON number"); return node(); }
  bool string(string_t& value) override { require(value.size() <= 4096 && value.find('\0') == std::string::npos, "packet string budget/NUL"); return node(); }
  bool binary(binary_t&) override { throw std::runtime_error("binary JSON unsupported"); }
  bool start_object(size_t) override { node(); require(frames.size() < max_depth, "packet depth budget"); frames.push_back({true,0,{}}); return true; }
  bool key(string_t& value) override {
    require(value.size() <= 4096 && value.find('\0') == std::string::npos, "packet key budget/NUL");
    require(!frames.empty() && frames.back().object && frames.back().keys.insert(value).second, "duplicate JSON key"); return true;
  }
  bool end_object() override { frames.pop_back(); return true; }
  bool start_array(size_t) override { node(); require(frames.size() < max_depth, "packet depth budget"); frames.push_back({false,0,{}}); return true; }
  bool end_array() override { frames.pop_back(); return true; }
  bool parse_error(size_t, const std::string&, const nlohmann::detail::exception&) override { throw std::runtime_error("invalid/trailing JSON bytes"); }
};
J packet(const std::filesystem::path& path) {
  const auto size = std::filesystem::file_size(path);
  require(size > 0 && size <= max_packet_bytes, "packet byte budget");
  std::ifstream in(path, std::ios::binary); require(in.good(), "open packet");
  std::string bytes(static_cast<size_t>(size), '\0');
  in.read(bytes.data(), static_cast<std::streamsize>(bytes.size())); require(in.good() && in.peek() == EOF, "packet changed while reading");
  Preflight guard; require(J::sax_parse(bytes, &guard), "packet preflight");
  return J::parse(bytes);
}
void fields(const J& j, std::initializer_list<const char*> names) {
  require(j.is_object() && j.size() == names.size(), "missing/unknown record fields");
  for (auto name : names) require(j.contains(name), "missing/unknown record fields");
}
void array(const J& j, size_t n) { require(j.is_array() && j.size() == n, "array shape"); }
double real(const J& j) { require(j.is_number(), "expected finite number"); const double v=j.get<double>(); require(std::isfinite(v), "nonfinite number"); return v; }
bool flag(const J& j) { require(j.is_boolean(), "expected boolean"); return j.get<bool>(); }
size_t count(const J& j, size_t cap=max_array) {
  require(j.is_number_unsigned() || (j.is_number_integer() && j.get<int64_t>() >= 0), "expected unsigned integer");
  const auto n=j.get<uint64_t>(); require(n<=cap, "integer count budget"); return static_cast<size_t>(n);
}
void revision(const J& j) { require((j.is_number_unsigned() && j.get<uint64_t>()>0) || (j.type()==J::value_t::number_integer && j.get<int64_t>()>0), "invalid revision"); }
std::string text(const J& j) {
  require(j.is_string(), "expected string"); auto s=j.get<std::string>(); require(s.size()<=4096 && s.find('\0')==std::string::npos, "name budget/NUL"); return s;
}
ON_UUID uuid(const J& j) {
  const auto s=text(j); require(s.size()==36, "UUID format");
  for (size_t i=0;i<s.size();++i) {
    if (i==8||i==13||i==18||i==23) require(s[i]=='-', "UUID separators");
    else require((s[i]>='0'&&s[i]<='9')||(s[i]>='a'&&s[i]<='f'), "UUID canonical lowercase hex");
  }
  const auto id=ON_UuidFromString(s.c_str()); require(!ON_UuidIsNil(id), "nil UUID"); return id;
}
std::string id_string(ON_UUID id) { char buffer[37]{}; ON_UuidToString(id, buffer); return buffer; }
J nullable_id(ON_UUID id) { return ON_UuidIsNil(id) ? J(nullptr) : J(id_string(id)); }
std::string native_text(const ON_wString& s) { ON_String utf8(s); return std::string(utf8.Array()?utf8.Array():""); }
ON_3dPoint point(const J& j) { array(j,3); ON_3dPoint p(real(j[0]),real(j[1]),real(j[2])); require(p.IsValid(), "invalid point"); return p; }
J point_json(const ON_3dPoint& p) { require(p.IsValid(), "nonfinite native point"); return J::array({p.x,p.y,p.z}); }
J vector_json(const ON_3dVector& p) { require(p.IsValid(), "nonfinite native vector"); return J::array({p.x,p.y,p.z}); }
ON_Interval domain(const J& j) { array(j,2); ON_Interval d(real(j[0]),real(j[1])); require(d.IsIncreasing() && std::isfinite(d.Length()), "parameter domain"); return d; }
J interval_json(const ON_Interval& d) { require(d.IsIncreasing(), "invalid native domain"); return J::array({d[0],d[1]}); }
ON_Xform matrix(const J& j) {
  array(j,4); ON_Xform x;
  for (int i=0;i<4;++i) { array(j[i],4); for (int k=0;k<4;++k) x[i][k]=real(j[i][k]); }
  require(x[3][0]==0 && x[3][1]==0 && x[3][2]==0 && x[3][3]==1, "affine last row");
  const double det=x.Determinant(); require(std::isfinite(det) && det!=0, "singular affine matrix"); return x;
}
J matrix_json(const ON_Xform& x) { J rows=J::array(); for(int i=0;i<4;++i) rows.push_back(J::array({x[i][0],x[i][1],x[i][2],x[i][3]})); (void)matrix(rows); return rows; }
ON_Color color(const J& j) { array(j,4); return ON_Color(static_cast<int>(count(j[0],255)),static_cast<int>(count(j[1],255)),static_cast<int>(count(j[2],255)),255-static_cast<int>(count(j[3],255))); }
J color_json(ON_Color c) { return J::array({c.Red(),c.Green(),c.Blue(),255-c.Alpha()}); }
ON::LengthUnitSystem unit(const J& j) {
  const auto s=text(j);
  if(s=="Millimeter") return ON::LengthUnitSystem::Millimeters;
  if(s=="Centimeter") return ON::LengthUnitSystem::Centimeters;
  if(s=="Meter") return ON::LengthUnitSystem::Meters;
  if(s=="Inch") return ON::LengthUnitSystem::Inches;
  if(s=="Foot") return ON::LengthUnitSystem::Feet;
  if(s=="Micrometer") return ON::LengthUnitSystem::Microns;
  throw std::runtime_error("unsupported unit");
}
std::string unit_string(ON::LengthUnitSystem u) {
  switch(u) {
    case ON::LengthUnitSystem::Millimeters:return "Millimeter";
    case ON::LengthUnitSystem::Centimeters:return "Centimeter";
    case ON::LengthUnitSystem::Meters:return "Meter";
    case ON::LengthUnitSystem::Inches:return "Inch";
    case ON::LengthUnitSystem::Feet:return "Foot";
    case ON::LengthUnitSystem::Microns:return "Micrometer";
    default:return "UnsupportedNativeUnit:"+std::to_string(static_cast<unsigned>(u));
  }
}

void axis(size_t degree,size_t n,const J& knots,const J& dom) {
  require(degree>=1 && degree<=16 && n>degree && n<=max_array, "NURBS degree/pole count");
  array(knots,n+degree+1); const auto d=domain(dom); size_t run=1;
  require(real(knots[0])==real(knots[1]) && real(knots[knots.size()-1])==real(knots[knots.size()-2]), "unsupported noncanonical expanded outer knots");
  for(size_t i=0;i<knots.size();++i) {
    const double v=real(knots[i]);
    if(i) { const double prev=real(knots[i-1]); require(prev<=v,"NURBS sorted knots"); run=prev==v?run+1:1; require(run<=degree+1,"NURBS knot multiplicity"); }
  }
  // Trimming a strict subdomain changes native CV/knot structure. Until that
  // conversion is independently accepted, refuse it instead of reparameterizing.
  require(d[0]==real(knots[degree]) && d[1]==real(knots[n]), "unsupported trimmed NURBS domain");
}
void poles(const J& p,const J& w) {
  require(p.is_array() && !p.empty() && p.size()<=max_array,"pole array budget"); array(w,p.size());
  for(size_t i=0;i<p.size();++i) { (void)point(p[i]); require(real(w[i])>0,"positive rational weight required"); }
}
void nurbs_curve_validate(const J& j) {
  fields(j,{"degree","poles","weights","knots","domain","periodic","reversed","seam"});
  poles(j["poles"],j["weights"]); axis(count(j["degree"],16),j["poles"].size(),j["knots"],j["domain"]);
  (void)flag(j["periodic"]); require(!flag(j["reversed"]),"unsupported reversed curve metadata"); require(j["seam"].is_null(),"unsupported curve seam metadata");
}
std::unique_ptr<ON_Curve> curve(const J& j,size_t depth=0) {
  require(depth<=max_depth && j.is_object() && j.size()==1,"curve variant/depth");
  const auto& name=j.begin().key(); const auto& v=j.begin().value();
  if(name=="Line") {
    fields(v,{"start","end","domain"}); const auto a=point(v["start"]),b=point(v["end"]); require(a!=b,"degenerate line"); const auto d=domain(v["domain"]);
    auto c=std::make_unique<ON_LineCurve>(a,b); require(c->SetDomain(d[0],d[1]),"line domain"); return c;
  }
  if(name=="Circle") {
    if(v.contains("angle_domain"))fields(v,{"center","x","y","radius","domain","angle_domain"});
    else fields(v,{"center","x","y","radius","domain"});
    const auto origin=point(v["center"]); const auto xp=point(v["x"]),yp=point(v["y"]);
    const ON_3dVector x(xp.x,xp.y,xp.z),y(yp.x,yp.y,yp.z); require(std::abs(x*x-1)<=1e-12 && std::abs(y*y-1)<=1e-12 && std::abs(x*y)<=1e-12,"circle orthonormal frame");
    const double r=real(v["radius"]); require(r>0,"positive circle radius"); const auto d=domain(v["domain"]);
    // Missing/null means legacy native-radians evaluation, not a canonical
    // circle reparameterized into an unrelated parameter interval.
    const auto angles=v.contains("angle_domain")&&!v["angle_domain"].is_null()?domain(v["angle_domain"]):d;
    require(angles.Length()==2*ON_PI,"unsupported Circle angular span: exact full turn required");
    const double rate=angles.Length()/d.Length(); require(std::isfinite(rate)&&rate>0,"circle angular parameter scale");
    const ON_Plane plane(origin,x,y); require(plane.IsValid(),"circle plane");
    const ON_Arc arc(ON_Circle(plane,r),angles); require(arc.IsValid()&&arc.DomainRadians()==angles,"circle angular interval preservation");
    auto c=std::make_unique<ON_ArcCurve>(arc,d[0],d[1]); require(c->IsValid()&&c->Domain()==d,"circle parameter domain preservation"); return c;
  }
  if(name=="Nurbs") {
    nurbs_curve_validate(v); const int degree=static_cast<int>(count(v["degree"],16)),n=static_cast<int>(v["poles"].size());
    const bool rational=std::any_of(v["weights"].begin(),v["weights"].end(),[](const J& w){return real(w)!=1;});
    auto c=std::make_unique<ON_NurbsCurve>(3,rational,degree+1,n);
    for(int i=0;i<n;++i) { const auto p=point(v["poles"][i]); const double w=real(v["weights"][i]); require(c->SetCV(i,ON_4dPoint(p.x*w,p.y*w,p.z*w,w)),"curve homogeneous pole"); }
    for(int i=0;i<c->KnotCount();++i) require(c->SetKnot(i,real(v["knots"][i+1])),"curve compact knot");
    require(c->IsValid(),"native NURBS curve validity"); require(c->IsPeriodic()==flag(v["periodic"]),"unsupported unmatched curve periodic metadata"); return c;
  }
  if(name=="Polycurve") {
    fields(v,{"segments","reversed"}); require(!flag(v["reversed"]),"unsupported reversed polycurve metadata");
    const auto& seg=v["segments"]; require(seg.is_array() && !seg.empty() && seg.size()<=max_records,"polycurve segment budget");
    auto c=std::make_unique<ON_PolyCurve>();
    for(const auto& item:seg) {
      auto child=curve(item,depth+1);
      // Append may change noncontiguous parameter domains. Refuse that mapping.
      if(c->Count()) { require(c->Domain()[1]==child->Domain()[0],"unsupported noncontiguous polycurve domains"); require(c->PointAtEnd().DistanceTo(child->PointAtStart())<=1e-12,"disconnected polycurve segments"); }
      require(c->Append(child.get()),"polycurve append"); child.release();
    }
    require(c->IsValid(),"native polycurve validity"); return c;
  }
  throw std::runtime_error("unsupported curve variant");
}
void circle_transform_admission(const ON_Curve& c,const ON_Xform& x,size_t depth=0) {
  require(depth<=max_depth,"curve transform depth");
  if(const auto* a=ON_ArcCurve::Cast(&c)) {
    const ON_3dVector u=x*a->m_arc.plane.xaxis,v=x*a->m_arc.plane.yaxis;
    const double uu=u*u,vv=v*v;
    require(std::isfinite(uu)&&std::isfinite(vv)&&uu>0&&std::abs(uu-vv)<=1e-12*std::max(uu,vv)&&std::abs(u*v)<=1e-12*std::max(uu,vv),"unsupported nonuniform analytic circle parameter conversion");
  }
  if(const auto* p=ON_PolyCurve::Cast(&c)) for(int i=0;i<p->Count();++i) circle_transform_admission(*p->SegmentCurve(i),x,depth+1);
}
std::unique_ptr<ON_Geometry> geometry(const J& j,const ON_Xform& placement,const std::map<std::string,ON_BoundingBox>& boxes) {
  require(j.is_object() && j.size()==1,"geometry variant"); const auto name=j.begin().key(); const auto& v=j.begin().value();
  std::unique_ptr<ON_Geometry> g;
  if(name=="Point") g=std::make_unique<ON_Point>(point(v));
  else if(name=="Curve") { auto c=curve(v); circle_transform_admission(*c,placement); g=std::move(c); }
  else if(name=="Surface") {
    fields(v,{"degree","size","poles","weights","knots","domain","periodic","reversed"}); array(v["degree"],2); array(v["size"],2); array(v["knots"],2); array(v["domain"],2); array(v["periodic"],2);
    const size_t nu=count(v["size"][0]),nv=count(v["size"][1]); require(nu>0&&nv>0&&nu<=max_array/nv,"surface grid budget"); poles(v["poles"],v["weights"]); require(v["poles"].size()==nu*nv,"surface U-major pole grid");
    const int du=static_cast<int>(count(v["degree"][0],16)),dv=static_cast<int>(count(v["degree"][1],16)); axis(du,nu,v["knots"][0],v["domain"][0]); axis(dv,nv,v["knots"][1],v["domain"][1]); require(!flag(v["reversed"]),"unsupported reversed surface metadata");
    const bool rational=std::any_of(v["weights"].begin(),v["weights"].end(),[](const J& w){return real(w)!=1;});
    auto s=std::make_unique<ON_NurbsSurface>(3,rational,du+1,dv+1,static_cast<int>(nu),static_cast<int>(nv));
    for(size_t u=0;u<nu;++u) for(size_t vindex=0;vindex<nv;++vindex) { const auto i=u*nv+vindex; const auto p=point(v["poles"][i]); const double w=real(v["weights"][i]); require(s->SetCV(static_cast<int>(u),static_cast<int>(vindex),ON_4dPoint(p.x*w,p.y*w,p.z*w,w)),"surface homogeneous pole"); }
    for(int dir=0;dir<2;++dir) { for(int i=0;i<s->KnotCount(dir);++i) require(s->SetKnot(dir,i,real(v["knots"][dir][i+1])),"surface compact knot"); require(s->IsPeriodic(dir)==flag(v["periodic"][dir]),"unsupported unmatched surface periodic metadata"); }
    g=std::move(s);
  }
  else if(name=="Mesh"||name=="Cloud") {
    const bool mesh=name=="Mesh";
    if(mesh) fields(v,{"vertices","faces","normals","colors"}); else fields(v,{"points","normals","colors","hidden"});
    const auto& points=v[mesh?"vertices":"points"]; const auto& normals=v["normals"]; const auto& colors=v["colors"];
    require(points.is_array() && !points.empty() && points.size()<=max_array,"point channel budget");
    require(normals.is_array() && (normals.empty()||normals.size()==points.size()),"normal channel count"); require(colors.is_array() && (colors.empty()||colors.size()==points.size()),"color channel count");
    if(mesh) {
      auto m=std::make_unique<ON_Mesh>();
      for(size_t i=0;i<points.size();++i) require(m->SetVertex(static_cast<int>(i),point(points[i])),"mesh double vertex");
      for(size_t i=0;i<normals.size();++i) { const auto p=point(normals[i]); const ON_3dVector n(p.x,p.y,p.z); require(n.Length()>0,"zero mesh normal"); require(m->SetVertexNormal(static_cast<int>(i),n),"mesh normal"); }
      for(const auto& c:colors) m->m_C.Append(color(c));
      const auto& faces=v["faces"]; require(faces.is_array() && !faces.empty() && faces.size()<=max_array,"mesh face budget");
      for(size_t i=0;i<faces.size();++i) { const auto& f=faces[i]; require(f.is_array()&&(f.size()==3||f.size()==4),"mesh polygon size"); std::set<size_t> unique; int index[4]{}; for(size_t k=0;k<f.size();++k) { const size_t n=count(f[k]); require(n<points.size() && unique.insert(n).second,"mesh face indices"); index[k]=static_cast<int>(n); }
        require(f.size()==3?m->SetTriangle(static_cast<int>(i),index[0],index[1],index[2]):m->SetQuad(static_cast<int>(i),index[0],index[1],index[2],index[3]),"mesh native face"); }
      g=std::move(m);
    } else {
      const auto& hidden=v["hidden"]; require(hidden.is_array() && (hidden.empty()||hidden.size()==points.size()),"cloud hidden count"); for(const auto& h:hidden) require(!flag(h),"unsupported runtime-only cloud hidden=true");
      auto c=std::make_unique<ON_PointCloud>(); for(const auto& p:points) c->AppendPoint(point(p)); for(const auto& n:normals) { const auto p=point(n); c->m_N.Append(ON_3dVector(p.x,p.y,p.z)); } for(const auto& col:colors) c->m_C.Append(color(col)); g=std::move(c);
    }
  }
  else if(name=="Instance") {
    const auto id=uuid(v); const auto found=boxes.find(id_string(id)); require(found!=boxes.end(),"missing/unordered definition dependency");
    auto i=std::make_unique<ON_InstanceRef>(); i->m_instance_definition_uuid=id; i->m_xform=placement; i->m_bbox=found->second; require(i->m_bbox.Transform(placement),"instance bounding transform"); require(i->IsValid(),"instance native validity"); return i;
  } else throw std::runtime_error("unsupported geometry variant");
  require(g && g->IsValid(),"native geometry validity");
  // Upstream ON_PointCloud::Transform transforms point positions but leaves
  // its normal channel unchanged. Transform that channel exactly once here.
  if(auto* cloud=ON_PointCloud::Cast(g.get()); cloud && cloud->m_N.Count()) {
    ON_Xform normal; require(placement.GetSurfaceNormalXform(normal)!=0,"cloud normal inverse transpose");
    for(int i=0;i<cloud->m_N.Count();++i) { auto n=normal*cloud->m_N[i]; require(n.IsValid() && n.Unitize(),"unsupported zero/nonfinite cloud normal"); cloud->m_N[i]=n; }
  }
  require(g->Transform(placement) && g->IsValid(),"native world placement");
  // Keep the authored polygon indices and covector-normal convention. Native
  // mesh Transform adds an orientation sign for mirrored matrices; the owned
  // representation defines normals by inverse transpose alone.
  if(auto* mesh=ON_Mesh::Cast(g.get()); mesh && placement.Determinant()<0 && mesh->m_N.Count())mesh->FlipVertexNormals();
  return g;
}

struct Writer {
  ONX_Model model;
  std::map<std::string,int> layer_indices;
  std::map<std::string,ON_BoundingBox> boxes;
  std::set<std::string> object_ids;
  size_t objects=0;
  bool default_needed=false;
  ON_3dmObjectAttributes attributes(const J& e,bool member) {
    fields(e,{"id","label","generation","geometry_revision","attributes_revision","geometry","placement","attributes"});
    const auto id=uuid(e["id"]); const auto key=id_string(id); require(object_ids.insert(key).second && ++objects<=max_records,"duplicate entity ID/object budget");
    operation_record="entity "+key; revision(e["generation"]); revision(e["geometry_revision"]); revision(e["attributes_revision"]);
    ON_3dmObjectAttributes a; a.m_uuid=id; const auto label=text(e["label"]);
    require(a.SetName(ON_wString(label.c_str()),false) && native_text(a.m_name)==label,"unsupported native entity name normalization");
    const auto& v=e["attributes"]; fields(v,{"layer","visible","selectable","locked","color"}); require(flag(v["selectable"]),"unsupported independent selectable=false"); const bool locked=flag(v["locked"]),visible=flag(v["visible"]);
    require(!member||!locked,"unsupported locked instance-definition member"); require(!locked||visible,"unsupported independently hidden and locked entity");
    a.SetMode(member?ON::idef_object:locked?ON::locked_object:visible?ON::normal_object:ON::hidden_object); if(member)a.SetVisible(visible);
    const auto layer=v["layer"].is_null()?std::string(default_layer):id_string(uuid(v["layer"])); const auto found=layer_indices.find(layer); require(found!=layer_indices.end(),"missing entity layer"); a.m_layer_index=found->second;
    if(v["color"].is_null()) a.SetColorSource(ON::color_from_layer); else { a.m_color=color(v["color"]); a.SetColorSource(ON::color_from_object); }
    return a;
  }
  ON_BoundingBox add_entity(const J& e,bool member) {
    const auto a=attributes(e,member); auto g=geometry(e["geometry"],matrix(e["placement"]),boxes); const auto box=g->BoundingBox(); require(box.IsValid(),"native entity bounding box");
    require(!model.AddModelGeometryComponent(g.get(),&a,false).IsEmpty(),"native owned geometry component"); return box;
  }
  void add_definition(const std::string& id,const J& definitions,std::set<std::string>& active,size_t depth) {
    require(depth<=max_depth,"definition depth budget"); if(boxes.contains(id)) return; require(active.insert(id).second,"definition cycle"); require(definitions.contains(id),"missing definition");
    const auto& d=definitions[id]; operation_record="definition "+id; fields(d,{"id","label","revision","members"}); require(id_string(uuid(d["id"]))==id,"definition key mismatch"); revision(d["revision"]);
    const auto& members=d["members"]; require(members.is_array() && !members.empty() && members.size()<=max_records,"empty/oversized definition members unsupported");
    for(const auto& m:members) if(m.contains("geometry") && m["geometry"].is_object() && m["geometry"].contains("Instance")) add_definition(id_string(uuid(m["geometry"]["Instance"])),definitions,active,depth+1);
    ON_BoundingBox box; ON_SimpleArray<ON_UUID> ids;
    for(const auto& m:members) { box.Union(add_entity(m,true)); ids.Append(uuid(m["id"])); }
    operation_record="definition "+id; const auto label=text(d["label"]); ON_InstanceDefinition def; require(def.SetId(uuid(d["id"])) && def.SetName(ON_wString(label.c_str())) && native_text(def.Name())==label,"unsupported native definition ID/name normalization"); def.SetUnitSystem(ON::LengthUnitSystem::Millimeters); def.SetInstanceGeometryIdList(ids); def.SetBoundingBox(box);
    require(def.IsValid() && !model.AddModelComponent(def,false).IsEmpty(),"native static instance definition"); boxes.emplace(id,box); active.erase(id);
  }
  J write(const J& p,const std::filesystem::path& output) {
    fields(p,{"schema_version","mode","document"}); require(count(p["schema_version"],1)==1,"unsupported schema version"); require(text(p["mode"])=="geometry_only_current","unsupported export mode");
    const auto& d=p["document"]; fields(d,{"id","name","page_unit","current_layer","layer_order","layers","entities","definitions"}); (void)uuid(d["id"]); (void)text(d["name"]);
    model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON::LengthUnitSystem::Millimeters; model.m_settings.m_PageUnitsAndTolerances.m_unit_system=unit(d["page_unit"]); model.m_properties.m_Application.m_application_name=L"OpenMatrix9 standalone current geometry export";
    const auto& layers=d["layers"]; const auto& order=d["layer_order"]; const auto& entities=d["entities"]; const auto& definitions=d["definitions"];
    require(layers.is_object() && layers.size()<=max_records && order.is_array() && order.size()==layers.size(),"layer table/order budget"); require(entities.is_object() && entities.size()<=max_records && definitions.is_object() && definitions.size()<=max_records,"object/definition table budget");
    // Check longest occurrence depth before native masters are allocated.
    // Caching a completed master must not let a long chain of shared masters
    // bypass the depth budget when the input table is in child-first order.
    std::map<std::string,size_t> definition_heights; std::set<std::string> graph_active;
    std::function<size_t(const std::string&,size_t)> graph_height=[&](const std::string& id,size_t depth)->size_t {
      require(depth<=max_depth,"definition depth budget");
      if(definition_heights.contains(id)) { require(definition_heights.at(id)<=max_depth-depth,"definition cached depth budget"); return definition_heights.at(id); }
      require(definitions.contains(id)&&graph_active.insert(id).second,"missing definition/definition cycle"); operation_record="definition graph "+id;
      const auto& def=definitions[id]; fields(def,{"id","label","revision","members"}); require(id_string(uuid(def["id"]))==id,"definition key mismatch"); revision(def["revision"]); (void)text(def["label"]); const auto& members=def["members"]; require(members.is_array()&&!members.empty()&&members.size()<=max_records,"definition member budget");
      size_t height=0;
      for(const auto& member:members) {
        require(member.is_object()&&member.contains("geometry")&&member["geometry"].is_object()&&member["geometry"].size()==1,"definition member geometry variant");
        if(member["geometry"].contains("Instance"))height=std::max(height,graph_height(id_string(uuid(member["geometry"]["Instance"])),depth+1)+1);
      }
      require(height<=max_depth-depth,"definition longest depth budget"); graph_active.erase(id); definition_heights.emplace(id,height); return height;
    };
    for(const auto& [id,def]:definitions.items()) { (void)def; (void)uuid(J(id)); (void)graph_height(id,0); }
    std::set<std::string> layer_ids;
    for(const auto& id:order) { const auto key=id_string(uuid(id)); require(layer_ids.insert(key).second && layers.contains(key),"layer order/key"); }
    for(const auto& [key,l]:layers.items()) {
      require(key!=default_layer,"reserved native default layer UUID");
      operation_record="layer "+key; fields(l,{"id","name","parent","visible","selectable","locked","color","revision"}); require(id_string(uuid(l["id"]))==key,"layer key mismatch"); revision(l["revision"]); require(flag(l["selectable"]),"unsupported independent layer selectable=false");
      std::set<std::string> visited; const J* item=&l;
      while(!(*item)["parent"].is_null()) { const auto parent=id_string(uuid((*item)["parent"])); require(layers.contains(parent)&&visited.insert(parent).second&&visited.size()<=max_depth,"layer missing parent/cycle/depth"); item=&layers[parent]; }
    }
    auto scan_default=[&](const J& e){ require(e.is_object() && e.contains("attributes") && e["attributes"].is_object() && e["attributes"].contains("layer"),"entity attributes shape"); if(e["attributes"]["layer"].is_null()) default_needed=true; };
    for(const auto& e:entities) scan_default(e);
    for(const auto& def:definitions) { require(def.is_object()&&def.contains("members")&&def["members"].is_array(),"definition members shape"); for(const auto& e:def["members"]) scan_default(e); }
    default_needed|=layers.empty();
    for(const auto& id:order) {
      const auto key=text(id); const auto& l=layers[key]; operation_record="layer "+key; const auto name=text(l["name"]); ON_Layer native; require(native.SetId(uuid(l["id"])) && native.SetName(ON_wString(name.c_str())) && native_text(native.Name())==name,"unsupported native layer ID/name normalization");
      if(!l["parent"].is_null()) native.SetParentLayerId(uuid(l["parent"])); native.SetColor(color(l["color"])); native.SetVisible(flag(l["visible"])); native.SetLocked(flag(l["locked"]));
      const auto ref=model.AddModelComponent(native,false); require(!ref.IsEmpty(),"native layer component/conflicting name"); const auto* added=ON_Layer::Cast(ref.ModelComponent()); require(added,"native layer component type"); layer_indices.emplace(key,added->Index());
    }
    if(default_needed) { require(!layers.contains(default_layer),"reserved native default layer UUID"); ON_Layer l; require(l.SetId(ON_UuidFromString(default_layer))&&l.SetName(L"Default"),"default layer ID/name"); l.SetColor(ON_Color(0,0,0)); const auto ref=model.AddModelComponent(l,false); require(!ref.IsEmpty(),"native default layer name conflict"); layer_indices.emplace(default_layer,ref.ModelComponent()->Index()); }
    if(!d["current_layer"].is_null()) { const auto key=id_string(uuid(d["current_layer"])); require(layer_indices.contains(key),"missing current layer"); model.m_settings.SetCurrentLayerId(uuid(d["current_layer"])); model.m_settings.SetV5CurrentLayerIndex(layer_indices.at(key)); }
    else if(default_needed) { model.m_settings.SetCurrentLayerId(ON_UuidFromString(default_layer)); model.m_settings.SetV5CurrentLayerIndex(layer_indices.at(default_layer)); }
    else { const auto key=text(order[0]); model.m_settings.SetCurrentLayerId(uuid(order[0])); model.m_settings.SetV5CurrentLayerIndex(layer_indices.at(key)); }
    std::set<std::string> active;
    for(const auto& [key,def]:definitions.items()) { (void)def; (void)uuid(J(key)); add_definition(key,definitions,active,0); }
    for(const auto& [key,e]:entities.items()) { require(e.contains("id")&&id_string(uuid(e["id"]))==key,"entity key mismatch"); (void)add_entity(e,false); }
    operation_record="write private archive"; require(!std::filesystem::exists(output),"output already exists; private unique destination required");
    // No output path is touched until the complete conversion succeeds. Failed
    // writes remove only the new private file; public atomic publish is Rust-owned.
    ON_wString log_text; ON_TextLog log(log_text);
    {
      PrivateFile file(output); ON_BinaryFile archive(ON::archive_mode::write3dm,file.value);
      require(model.Write(archive,5,&log),"native Rhino 5 writer"); require(std::fflush(file.value)==0,"flush private native archive"); file.complete=true;
    }
    require(std::filesystem::file_size(output)>32,"native writer output size");
    J diagnostics=J::array(); if(default_needed) diagnostics.push_back("unlayered entities mapped to native default layer");
    if(d["current_layer"].is_null())diagnostics.push_back("current_layer=None mapped to native first/default layer");
    return J{{"schema_version",1},{"status","written"},{"objects",objects},{"definitions",definitions.size()},{"layers",layer_indices.size()},{"diagnostics",diagnostics}};
  }
};

J compact_knots(const ON_NurbsCurve& c) {
  const int n=c.KnotCount(); require(n>0&&static_cast<size_t>(n)<=max_array,"native knot budget"); J values=J::array(); values.push_back(c.Knot(0)); for(int i=0;i<n;++i) values.push_back(c.Knot(i)); values.push_back(c.Knot(n-1)); return values;
}
J curve_inventory(const ON_Curve& c,size_t depth=0) {
  require(depth<=max_depth && c.IsValid(),"native curve validity/depth");
  if(const auto* line=ON_LineCurve::Cast(&c)) return J{{"Line",{{"start",point_json(line->PointAtStart())},{"end",point_json(line->PointAtEnd())},{"domain",interval_json(line->Domain())}}}};
  if(const auto* arc=ON_ArcCurve::Cast(&c)) {
    const auto angles=arc->m_arc.DomainRadians(),parameters=arc->Domain();
    // SDK IsCircle() admits near-full arcs by tolerance. Those endpoints and
    // derivatives must not be silently projected as an exact owned Circle.
    require(angles.Length()==2*ON_PI,"unsupported native Circle angular span: exact full turn required");
    const double rate=angles.Length()/parameters.Length();
    require(std::isfinite(parameters.Length())&&std::isfinite(rate)&&rate>0,"unsupported native circle angular parameter scale");
    J circle{{"center",point_json(arc->m_arc.plane.origin)},{"x",vector_json(arc->m_arc.plane.xaxis)},{"y",vector_json(arc->m_arc.plane.yaxis)},{"radius",arc->m_arc.radius},{"domain",interval_json(parameters)}};
    if(angles!=parameters)circle["angle_domain"]=interval_json(angles);
    return J{{"Circle",std::move(circle)}};
  }
  if(const auto* n=ON_NurbsCurve::Cast(&c)) {
    require(n->Dimension()==3 && n->CVCount()>0 && static_cast<size_t>(n->CVCount())<=max_array,"native curve pole budget/dimension"); J p=J::array(),w=J::array();
    for(int i=0;i<n->CVCount();++i) { ON_3dPoint pt; require(n->GetCV(i,pt),"native rational Euclidean pole"); p.push_back(point_json(pt)); w.push_back(n->Weight(i)); }
    return J{{"Nurbs",{{"degree",n->Degree()},{"poles",p},{"weights",w},{"knots",compact_knots(*n)},{"domain",interval_json(n->Domain())},{"periodic",n->IsPeriodic()},{"reversed",false},{"seam",nullptr}}}};
  }
  if(const auto* poly=ON_PolyCurve::Cast(&c)) { require(poly->Count()>0&&static_cast<size_t>(poly->Count())<=max_records,"native polycurve budget"); J segments=J::array(); for(int i=0;i<poly->Count();++i) { require(poly->SegmentCurve(i),"null native polycurve segment"); segments.push_back(curve_inventory(*poly->SegmentCurve(i),depth+1)); } return J{{"Polycurve",{{"segments",segments},{"reversed",false}}}}; }
  throw std::runtime_error("unsupported native curve type");
}
std::string kind(const ON_Geometry& g) {
  if(ON_Point::Cast(&g))return "ON_Point";
  if(ON_LineCurve::Cast(&g))return "ON_LineCurve";
  if(ON_ArcCurve::Cast(&g))return "ON_ArcCurve";
  if(ON_NurbsCurve::Cast(&g))return "ON_NurbsCurve";
  if(ON_PolyCurve::Cast(&g))return "ON_PolyCurve";
  if(ON_NurbsSurface::Cast(&g))return "ON_NurbsSurface";
  if(ON_Mesh::Cast(&g))return "ON_Mesh";
  if(ON_PointCloud::Cast(&g))return "ON_PointCloud";
  if(ON_InstanceRef::Cast(&g))return "ON_InstanceRef";
  const auto* info=g.ClassId(); return info?std::string(info->ClassName()):"UnsupportedNativeGeometry:"+std::to_string(static_cast<unsigned>(g.ObjectType()));
}
J geometry_inventory(const ON_Geometry& g) {
  require(g.IsValid(),"invalid native geometry");
  if(const auto* p=ON_Point::Cast(&g))return J{{"Point",point_json(p->point)}};
  if(const auto* c=ON_Curve::Cast(&g))return J{{"Curve",curve_inventory(*c)}};
  if(const auto* s=ON_NurbsSurface::Cast(&g)) {
    const int nu=s->CVCount(0),nv=s->CVCount(1); require(nu>0&&nv>0&&static_cast<size_t>(nu)<=max_array/static_cast<size_t>(nv)&&s->Dimension()==3,"native surface grid/dimension budget"); J p=J::array(),w=J::array(),knots=J::array();
    for(int u=0;u<nu;++u)for(int v=0;v<nv;++v) { ON_3dPoint pt; require(s->GetCV(u,v,pt),"native surface Euclidean pole"); p.push_back(point_json(pt)); w.push_back(s->Weight(u,v)); }
    for(int dir=0;dir<2;++dir) { require(s->KnotCount(dir)>0&&static_cast<size_t>(s->KnotCount(dir))<=max_array,"native surface knot budget"); J k=J::array(); k.push_back(s->Knot(dir,0)); for(int i=0;i<s->KnotCount(dir);++i)k.push_back(s->Knot(dir,i)); k.push_back(s->Knot(dir,s->KnotCount(dir)-1)); knots.push_back(k); }
    return J{{"Surface",{{"degree",J::array({s->Degree(0),s->Degree(1)})},{"size",J::array({nu,nv})},{"poles",p},{"weights",w},{"knots",knots},{"domain",J::array({interval_json(s->Domain(0)),interval_json(s->Domain(1))})},{"periodic",J::array({s->IsPeriodic(0),s->IsPeriodic(1)})},{"reversed",false}}}};
  }
  if(const auto* m=ON_Mesh::Cast(&g)) {
    require(m->VertexCount()>0&&static_cast<size_t>(m->VertexCount())<=max_array&&static_cast<size_t>(m->FaceCount())<=max_array,"native mesh budget"); J v=J::array(),f=J::array(),n=J::array(),c=J::array();
    for(int i=0;i<m->VertexCount();++i)v.push_back(point_json(m->Vertex(i)));
    for(int i=0;i<m->m_F.Count();++i) { const auto& face=m->m_F[i]; J a=J::array({face.vi[0],face.vi[1],face.vi[2]}); if(face.IsQuad())a.push_back(face.vi[3]); f.push_back(a); }
    require(m->m_N.Count()==0||m->m_N.Count()==m->VertexCount(),"native mesh normal count"); require(m->m_C.Count()==0||m->m_C.Count()==m->VertexCount(),"native mesh color count");
    for(int i=0;i<m->m_N.Count();++i)n.push_back(vector_json(ON_3dVector(m->m_N[i]))); for(int i=0;i<m->m_C.Count();++i)c.push_back(color_json(m->m_C[i]));
    return J{{"Mesh",{{"vertices",v},{"faces",f},{"normals",n},{"colors",c}}}};
  }
  if(const auto* cloud=ON_PointCloud::Cast(&g)) {
    require(cloud->m_P.Count()>0&&static_cast<size_t>(cloud->m_P.Count())<=max_array,"native cloud budget"); require(cloud->m_N.Count()==0||cloud->m_N.Count()==cloud->m_P.Count(),"native cloud normal count"); require(cloud->m_C.Count()==0||cloud->m_C.Count()==cloud->m_P.Count(),"native cloud color count");
    J p=J::array(),n=J::array(),c=J::array(); for(int i=0;i<cloud->m_P.Count();++i)p.push_back(point_json(cloud->m_P[i])); for(int i=0;i<cloud->m_N.Count();++i)n.push_back(vector_json(cloud->m_N[i])); for(int i=0;i<cloud->m_C.Count();++i)c.push_back(color_json(cloud->m_C[i]));
    return J{{"Cloud",{{"points",p},{"normals",n},{"colors",c},{"hidden",J::array()}}}};
  }
  if(const auto* i=ON_InstanceRef::Cast(&g))return J{{"Instance",id_string(i->m_instance_definition_uuid)}};
  throw std::runtime_error("native family not represented by owned geometry schema");
}
J inspect(const std::filesystem::path& path) {
  require(std::filesystem::file_size(path)<=max_archive_bytes,"archive file-byte budget"); ONX_Model model; ON_wString log_text; ON_TextLog log(log_text);
  { File file(path); ON_BinaryFile archive(ON::archive_mode::read3dm,file.value); require(model.Read(archive,&log),"native archive read"); require(archive.BadCRCCount()==0 && archive.CriticalErrorCount()==0,"native archive CRC/critical errors"); }
  J result{{"schema_version",1},{"status","inspected"},{"archive_version",model.m_3dm_file_version},{"model_unit",unit_string(model.m_settings.m_ModelUnitsAndTolerances.m_unit_system.UnitSystem())},{"page_unit",unit_string(model.m_settings.m_PageUnitsAndTolerances.m_unit_system.UnitSystem())},{"objects",J::array()},{"layers",J::array()},{"definitions",J::array()},{"diagnostics",J::array()}};
  std::map<int,std::string> layers; std::map<std::string,std::string> layer_parents; size_t count_records=0;
  ONX_ModelComponentIterator layer_it(model,ON_ModelComponent::Type::Layer);
  for(const auto* c=layer_it.FirstComponent();c;c=layer_it.NextComponent()) {
    require(++count_records<=max_records,"native layer budget"); const auto* l=ON_Layer::Cast(c); require(l&&l->IsValid()&&!ON_UuidIsNil(l->Id()),"native layer validity/identity"); operation_record="inspect layer "+id_string(l->Id()); require(layers.emplace(l->Index(),id_string(l->Id())).second,"native duplicate layer index"); require(layer_parents.emplace(id_string(l->Id()),ON_UuidIsNil(l->ParentLayerId())?"":id_string(l->ParentLayerId())).second,"native duplicate layer UUID");
    result["layers"].push_back(J{{"id",id_string(l->Id())},{"name",native_text(l->Name())},{"parent",nullable_id(l->ParentLayerId())},{"visible",l->IsVisible()},{"locked",l->IsLocked()},{"color",color_json(l->Color())},{"native_index",l->Index()}});
  }
  for(const auto& [id,parent]:layer_parents) { (void)parent; std::string cursor=id; std::set<std::string> visited; while(!cursor.empty()) { require(layer_parents.contains(cursor)&&visited.insert(cursor).second&&visited.size()<=max_depth,"native layer missing parent/cycle/depth"); cursor=layer_parents.at(cursor); } }
  const auto current=model.m_settings.CurrentLayerId(); const auto current_index=model.m_settings.CurrentLayerIndex(); result["current_layer"]=ON_UuidIsNil(current)?(layers.contains(current_index)?J(layers.at(current_index)):J(nullptr)):J(id_string(current));
  ONX_ModelComponentIterator defs(model,ON_ModelComponent::Type::InstanceDefinition); count_records=0;
  std::set<std::string> definition_members,definition_ids,observed_members;
  std::map<std::string,std::vector<std::string>> definition_children;
  std::map<std::string,std::string> member_definitions;
  for(const auto* c=defs.FirstComponent();c;c=defs.NextComponent()) {
    require(++count_records<=max_records,"native definition budget"); const auto* d=ON_InstanceDefinition::Cast(c); require(d&&d->IsValid()&&!ON_UuidIsNil(d->Id()),"native definition validity/identity"); operation_record="inspect definition "+id_string(d->Id()); require(definition_ids.insert(id_string(d->Id())).second,"native duplicate definition UUID"); definition_children.emplace(id_string(d->Id()),std::vector<std::string>{}); const auto& ids=d->InstanceGeometryIdList(); require(static_cast<size_t>(ids.Count())<=max_records,"native definition member budget"); J members=J::array();
    for(int i=0;i<ids.Count();++i) { require(!ON_UuidIsNil(ids[i]),"native nil member UUID"); const auto id=id_string(ids[i]); require(definition_members.insert(id).second,"native member shared by definitions"); member_definitions.emplace(id,id_string(d->Id())); members.push_back(id); }
    result["definitions"].push_back(J{{"id",id_string(d->Id())},{"label",native_text(d->Name())},{"members",members},{"model_unit",unit_string(d->UnitSystem().UnitSystem())},{"linked",d->IsLinkedType()}});
    if(d->IsLinkedType())result["diagnostics"].push_back("linked definition retained as inventory only; external file not loaded");
  }
  ONX_ModelComponentIterator objects(model,ON_ModelComponent::Type::ModelGeometry); count_records=0;
  for(const auto* c=objects.FirstComponent();c;c=objects.NextComponent()) {
    require(++count_records<=max_records,"native object budget"); const auto* item=ON_ModelGeometryComponent::Cast(c); require(item,"native geometry component type"); const auto* g=item->Geometry(nullptr); const auto* a=item->Attributes(nullptr); require(g&&a,"native geometry/attributes missing"); operation_record="inspect entity "+id_string(a->m_uuid);
    require(g->IsValid() && a->IsValid() && !ON_UuidIsNil(a->m_uuid),"native geometry/attributes/identity invalid");
    require(layers.contains(a->m_layer_index),"native object missing layer"); const bool member=a->IsInstanceDefinitionObject(); require(member==definition_members.contains(id_string(a->m_uuid)),"native definition-member mode/reference mismatch"); if(member)require(observed_members.insert(id_string(a->m_uuid)).second,"native duplicate member UUID");
    J object{{"id",id_string(a->m_uuid)},{"label",native_text(a->m_name)},{"is_definition_member",member},{"native_kind",kind(*g)},{"placement",matrix_json(ON_Xform::IdentityTransformation)},{"attributes",{{"layer",layers.at(a->m_layer_index)},{"visible",a->IsVisible()},{"locked",a->Mode()==ON::locked_object},{"color",a->ColorSource()==ON::color_from_object?color_json(a->m_color):J(nullptr)},{"color_source",a->ColorSource()==ON::color_from_layer?"layer":a->ColorSource()==ON::color_from_object?"object":"unsupported_native_source"}}},{"diagnostics",J::array()}};
    if(const auto* instance=ON_InstanceRef::Cast(g)) { require(definition_ids.contains(id_string(instance->m_instance_definition_uuid)),"native instance missing definition"); object["placement"]=matrix_json(instance->m_xform); if(member)definition_children.at(member_definitions.at(id_string(a->m_uuid))).push_back(id_string(instance->m_instance_definition_uuid)); }
    try { object["geometry"]=geometry_inventory(*g); }
    catch(const std::runtime_error& error) { object["geometry"]=nullptr; object["diagnostics"].push_back(error.what()); }
    if(ON_PointCloud::Cast(g))object["diagnostics"].push_back("cloud hidden flags are runtime-only and absent from native 3DM");
    if(a->ColorSource()!=ON::color_from_layer&&a->ColorSource()!=ON::color_from_object)object["diagnostics"].push_back("native color source is not represented by owned attributes");
    result["objects"].push_back(std::move(object));
  }
  require(definition_members==observed_members,"native missing definition members");
  std::map<std::string,size_t> heights; std::set<std::string> active;
  std::function<size_t(const std::string&,size_t)> visit=[&](const std::string& id,size_t depth)->size_t {
    require(depth<=max_depth,"native definition depth budget");
    if(heights.contains(id)) { require(heights.at(id)<=max_depth-depth,"native cached definition depth budget"); return heights.at(id); }
    require(active.insert(id).second,"native definition cycle"); size_t height=0;
    for(const auto& child:definition_children.at(id))height=std::max(height,visit(child,depth+1)+1);
    require(height<=max_depth-depth,"native longest definition depth budget"); heights.emplace(id,height); active.erase(id); return height;
  };
  for(const auto& id:definition_ids)(void)visit(id,0);
  return result;
}

// Source inventory is deliberately separate from schema-1 current-state
// conversion. It describes decoded native records and the original archive;
// it does not create a merged archive or grant an application capability.
using Type = ON_ModelComponent::Type;
using Table = ON_3dmArchiveTableType;
size_t source_record_count=0;
struct SourceHash {
  BCRYPT_ALG_HANDLE algorithm=nullptr; BCRYPT_HASH_HANDLE hash=nullptr;
  SourceHash() {
    require(BCryptOpenAlgorithmProvider(&algorithm,BCRYPT_SHA256_ALGORITHM,nullptr,0)>=0,"source SHA256 provider");
    if(BCryptCreateHash(algorithm,&hash,nullptr,0,nullptr,0,0)<0) { BCryptCloseAlgorithmProvider(algorithm,0); algorithm=nullptr; throw std::runtime_error("source SHA256 hash"); }
  }
  ~SourceHash() { if(hash)BCryptDestroyHash(hash); if(algorithm)BCryptCloseAlgorithmProvider(algorithm,0); }
  SourceHash(const SourceHash&)=delete; SourceHash& operator=(const SourceHash&)=delete;
};
std::string source_sha256(FILE* file,uintmax_t expected_bytes) {
  require(_filelengthi64(_fileno(file))>=0&&static_cast<uintmax_t>(_filelengthi64(_fileno(file)))==expected_bytes,"source held-byte size changed");
  require(_fseeki64(file,0,SEEK_SET)==0,"rewind held source for SHA256"); SourceHash hash; unsigned char buffer[8192]{}; uintmax_t bytes=0;
  for(;;) { const auto n=std::fread(buffer,1,sizeof(buffer),file); require(!std::ferror(file),"read held source SHA256");
    require(n<=max_archive_bytes-bytes,"source SHA256 byte budget"); bytes+=n;
    if(n)require(BCryptHashData(hash.hash,buffer,static_cast<ULONG>(n),0)>=0,"source SHA256 update");
    if(n<sizeof(buffer))break;
  }
  require(bytes==expected_bytes&&static_cast<uintmax_t>(_filelengthi64(_fileno(file)))==expected_bytes,"source held-byte size changed");
  unsigned char digest[32]{}; require(BCryptFinishHash(hash.hash,digest,sizeof(digest),0)>=0,"source SHA256 finish");
  std::string result; result.reserve(64); constexpr char hex[]="0123456789abcdef";
  for(const auto b:digest) { result.push_back(hex[b>>4]); result.push_back(hex[b&15]); }
  require(_fseeki64(file,0,SEEK_SET)==0,"rewind held source after SHA256"); return result;
}
J source_resolution(const std::filesystem::path& path) {
  auto input=packet(path); fields(input,{"schema_version","model_mm_per_unit","page_mm_per_unit","definition_mm_per_unit"});
  require(count(input["schema_version"],1)==1,"source resolution schema version");
  for(const auto* field:{"model_mm_per_unit","page_mm_per_unit"})if(!input[field].is_null())require(real(input[field])>0,"positive source resolution factor");
  const auto& definitions=input["definition_mm_per_unit"]; require(definitions.is_object()&&definitions.size()<=max_records,"source definition resolution budget");
  for(auto entry=definitions.begin();entry!=definitions.end();++entry) {
    require(id_string(uuid(J(entry.key())))==entry.key(),"canonical definition resolution UUID"); require(real(entry.value())>0,"positive definition resolution factor");
  }
  return input;
}
J resolved_unit(const J& original,const J& explicit_factor,bool mandatory) {
  const bool unresolved=original["requires_resolution"].get<bool>();
  if(unresolved&&explicit_factor.is_null()) { require(!mandatory,"explicit model/page unit resolution required"); return nullptr; }
  const double scale=unresolved?real(explicit_factor):real(original["millimeters_per_unit"]);
  require(std::isfinite(scale)&&scale>0,"positive finite resolved unit factor");
  if(!unresolved&&!explicit_factor.is_null())require(real(explicit_factor)==scale,"conflicting declared unit override");
  return J{{"millimeters_per_unit",scale},{"resolved_by",unresolved?"explicit":"declared"},{"original_native_enum",original["native_enum"]}};
}
std::string component_type(Type t) {
  switch(t) {
    case Type::Image:return "Image"; case Type::TextureMapping:return "TextureMapping";
    case Type::RenderMaterial:return "RenderMaterial"; case Type::LinePattern:return "LinePattern";
    case Type::Layer:return "Layer"; case Type::Group:return "Group"; case Type::TextStyle:return "TextStyle";
    case Type::DimStyle:return "DimStyle"; case Type::RenderLight:return "RenderLight";
    case Type::HatchPattern:return "HatchPattern"; case Type::InstanceDefinition:return "InstanceDefinition";
    case Type::ModelGeometry:return "ModelGeometry"; case Type::HistoryRecord:return "HistoryRecord";
    default:throw std::runtime_error("unsupported native component type");
  }
}
std::string bounded_native_text(const ON_wString& value) {
  require(value.Length()<=4096,"native string budget"); const ON_String encoded(value);
  require(encoded.Length()>=0&&encoded.Length()<=4096,"native UTF-8 string budget");
  const std::string s(encoded.Array()?encoded.Array():"",static_cast<size_t>(encoded.Length()));
  require(s.size()<=4096 && s.find('\0')==std::string::npos,"native UTF-8 string budget/NUL"); return s;
}
std::string native_kind(const ON_Object& value) {
  const auto* c=value.ClassId(); require(c,"native class identity"); return c->ClassName();
}
J native_class(const ON_Object& value) {
  const auto* c=value.ClassId(); require(c,"native class identity"); return id_string(c->Uuid());
}
std::string hash_bytes(const void* data,size_t bytes) {
  require(bytes<=max_archive_bytes && (bytes==0 || data),"native payload size/pointer budget");
  return native_text(ON_SHA1_Hash::BufferContentHash(data,bytes).ToString(false));
}
J native_userdata(const ON_Object& value) {
  J result=J::array(); size_t count_=0;
  for(const auto* data=value.FirstUserData();data;data=data->Next()) {
    require(++count_<=max_records&&++source_record_count<=max_records,"native userdata aggregate record budget");
    J record{{"class_id",id_string(data->UserDataClassUuid())},{"application_id",nullable_id(data->m_application_uuid)},
      {"item_id",nullable_id(data->m_userdata_uuid)},{"unknown",data->IsUnknownUserData()},
      {"copy_count",data->m_userdata_copycount},{"native_sizeof_bytes",data->SizeOf()},{"opaque_references",true}};
    if(const auto* unknown=ON_UnknownUserData::Cast(data)) {
      require(unknown->m_sizeof_buffer>=0,"negative native userdata buffer");
      const auto n=static_cast<size_t>(unknown->m_sizeof_buffer);
      record["byte_length"]=n; record["payload_sha1"]=hash_bytes(unknown->m_buffer,n);
      record["archive_version"]=unknown->m_3dm_version;
      record["opennurbs_version"]=unknown->m_3dm_opennurbs_version_number;
    }
    result.push_back(std::move(record));
  }
  return result;
}
J native_units(const ON_3dmUnitsAndTolerances& value) {
  const auto& u=value.m_unit_system; const auto tag=u.UnitSystem();
  const double meters=u.MetersPerUnit(ON_DBL_QNAN),millimeters=u.MillimetersPerUnit(ON_DBL_QNAN);
  const bool unresolved=tag==ON::LengthUnitSystem::None || tag==ON::LengthUnitSystem::Unset
    || !u.IsValid() || !std::isfinite(meters) || meters<=0 || !std::isfinite(millimeters) || millimeters<=0;
  require(std::isfinite(value.m_absolute_tolerance)&&value.m_absolute_tolerance>0
    &&std::isfinite(value.m_angle_tolerance)&&value.m_angle_tolerance>0&&value.m_angle_tolerance<=ON_PI
    &&std::isfinite(value.m_relative_tolerance)&&value.m_relative_tolerance>0&&value.m_relative_tolerance<1,
    "invalid native source tolerances");
  return J{{"native_enum",static_cast<unsigned>(tag)},{"name",bounded_native_text(u.UnitSystemName())},
    {"meters_per_unit",unresolved?J(nullptr):J(meters)},{"millimeters_per_unit",unresolved?J(nullptr):J(millimeters)},
    {"requires_resolution",unresolved},{"absolute_tolerance",value.m_absolute_tolerance},
    {"angle_tolerance_radians",value.m_angle_tolerance},{"relative_tolerance",value.m_relative_tolerance},
    {"distance_display_mode",static_cast<unsigned>(value.m_distance_display_mode)},
    {"distance_display_precision",value.m_distance_display_precision}};
}
J definition_units(const ON_UnitSystem& u) {
  ON_3dmUnitsAndTolerances value; value.m_unit_system=u; auto result=native_units(value);
  // An instance definition stores a unit system, not document tolerances.
  for(const auto* field:{"absolute_tolerance","angle_tolerance_radians","relative_tolerance","distance_display_mode","distance_display_precision"})result.erase(field);
  return result;
}
J reference(const char* role,Type type,ON_UUID id,int index=ON_UNSET_INT_INDEX) {
  return J{{"role",role},{"component_type",component_type(type)},{"id",nullable_id(id)},
    {"native_index",index==ON_UNSET_INT_INDEX?J(nullptr):J(index)}};
}
void material_references(J& refs,const ON_RenderingAttributes& a) {
  require(static_cast<size_t>(a.m_materials.Count())<=max_records,"native render material reference budget");
  for(int i=0;i<a.m_materials.Count();++i) {
    const auto& m=a.m_materials[i];
    for(const auto& item:std::initializer_list<std::pair<const char*,ON_UUID>>{{"render_material",m.m_material_id},{"backface_material",m.m_material_backface_id}}) {
      if(ON_UuidIsNil(item.second))continue;
      auto r=reference(item.first,Type::RenderMaterial,item.second); r["application_id"]=nullable_id(m.m_plugin_id); refs.push_back(std::move(r));
    }
  }
}
J attribute_inventory(const ON_3dmObjectAttributes& a,J& refs) {
  require(a.IsValid() && !ON_UuidIsNil(a.m_uuid),"invalid native attributes/identity");
  refs.push_back(reference("layer",Type::Layer,ON_nil_uuid,a.m_layer_index));
  refs.push_back(reference("material",Type::RenderMaterial,ON_nil_uuid,a.m_material_index));
  refs.push_back(reference("line_pattern",Type::LinePattern,ON_nil_uuid,a.m_linetype_index));
  require(a.GroupCount()>=0&&static_cast<size_t>(a.GroupCount())<=max_records,"native group reference budget");
  J groups=J::array(); for(int i=0;i<a.GroupCount();++i) { const int index=a.GroupList()[i]; groups.push_back(index); refs.push_back(reference("group",Type::Group,ON_nil_uuid,index)); }
  material_references(refs,a.m_rendering_attributes);
  require(static_cast<size_t>(a.m_rendering_attributes.m_mappings.Count())<=max_records,"native mapping reference budget");
  for(int i=0;i<a.m_rendering_attributes.m_mappings.Count();++i) {
    const auto& m=a.m_rendering_attributes.m_mappings[i]; require(static_cast<size_t>(m.m_mapping_channels.Count())<=max_records,"native mapping channel budget");
    for(int k=0;k<m.m_mapping_channels.Count();++k) {
      const auto& channel=m.m_mapping_channels[k]; if(ON_UuidIsNil(channel.m_mapping_id))continue;
      auto r=reference("texture_mapping",Type::TextureMapping,channel.m_mapping_id);
      r["application_id"]=nullable_id(m.m_plugin_id); r["channel_id"]=channel.m_mapping_channel_id; r["transform"]=matrix_json(channel.m_object_xform); refs.push_back(std::move(r));
    }
  }
  return J{{"layer",nullptr},{"visible",a.IsVisible()},{"locked",a.Mode()==ON::locked_object},
    {"color",a.ColorSource()==ON::color_from_object?color_json(a.m_color):J(nullptr)},
    {"color_source",static_cast<unsigned>(a.ColorSource())},{"material_source",static_cast<unsigned>(a.MaterialSource())},
    {"line_pattern_source",static_cast<unsigned>(a.LinetypeSource())},{"mode",static_cast<unsigned>(a.Mode())},
    {"native_layer_index",a.m_layer_index},{"native_material_index",a.m_material_index},{"native_line_pattern_index",a.m_linetype_index},
    {"groups",groups},{"viewport_id",nullable_id(a.m_viewport_id)},{"space",static_cast<unsigned>(a.m_space)},
    {"display_order",a.m_display_order},{"plot_weight_mm",a.m_plot_weight_mm},{"userdata",native_userdata(a)}};
}
J record_fingerprint(const ON_Object& value,int version,unsigned ov) {
  require(value.SizeOf()<=2*1024*1024,"source classification serialization budget");
  const int errors=ON_GetErrorCount(),warnings=ON_GetWarningCount();
  ON_Write3dmBufferArchive archive(0,2*1024*1024,version,ov); require(value.Write(archive)&&archive.SizeOfArchive()<=2*1024*1024,"source classification native serialization");
  if(ON_GetErrorCount()!=errors||ON_GetWarningCount()!=warnings)throw std::runtime_error("source classification serialization diagnostics: "+native_kind(value));
  return J{{"byte_length",archive.SizeOfArchive()},{"sha1",hash_bytes(archive.Buffer(),archive.SizeOfArchive())}};
}
J preservation(const J& actual,const J& defaults) {
  J result{{"admission_required",true},{"default_fields",J::object()},{"nondefault_fields",J::object()}};
  for(auto field=actual.begin();field!=actual.end();++field)result[defaults.contains(field.key())&&defaults[field.key()]==field.value()?"default_fields":"nondefault_fields"][field.key()]=field.value();
  return result;
}
J attr_values(const ON_3dmObjectAttributes& a,const J& userdata) {
  J groups=J::array(),display=J::array(),render=J::array(),mapping=J::array();
  for(int i=0;i<a.GroupCount();++i)groups.push_back(a.GroupList()[i]);
  require(static_cast<size_t>(a.m_dmref.Count())<=max_records,"source display override budget");
  for(int i=0;i<a.m_dmref.Count();++i)display.push_back(J{{"viewport_id",nullable_id(a.m_dmref[i].m_viewport_id)},{"display_material_id",nullable_id(a.m_dmref[i].m_display_material_id)}});
  material_references(render,a.m_rendering_attributes);
  for(int i=0;i<a.m_rendering_attributes.m_mappings.Count();++i) { const auto& m=a.m_rendering_attributes.m_mappings[i];
    for(int k=0;k<m.m_mapping_channels.Count();++k) { const auto& c=m.m_mapping_channels[k]; mapping.push_back(J{{"application_id",nullable_id(m.m_plugin_id)},{"channel_id",c.m_mapping_channel_id},{"mapping_id",nullable_id(c.m_mapping_id)},{"transform",matrix_json(c.m_object_xform)}}); }
  }
  const auto frame=a.ObjectFrame(ON_COMPONENT_INDEX());
  return J{{"color_source",static_cast<unsigned>(a.ColorSource())},{"native_color",color_json(a.m_color)},{"url",bounded_native_text(a.m_url)},
    {"material_source",static_cast<unsigned>(a.MaterialSource())},{"native_material_index",a.m_material_index},
    {"line_pattern_source",static_cast<unsigned>(a.LinetypeSource())},{"native_line_pattern_index",a.m_linetype_index},
    {"groups",groups},{"viewport_id",nullable_id(a.m_viewport_id)},{"space",static_cast<unsigned>(a.m_space)},
    {"display_order",a.m_display_order},{"plot_color",color_json(a.m_plot_color)},{"plot_color_source",static_cast<unsigned>(a.PlotColorSource())},
    {"plot_weight_mm",a.m_plot_weight_mm},{"plot_weight_source",static_cast<unsigned>(a.PlotWeightSource())},
    {"wire_density",a.m_wire_density},{"object_decoration",static_cast<unsigned>(a.m_object_decoration)},
    {"display_mode_overrides",display},{"render_material_references",render},{"texture_mapping_channels",mapping},
    {"casts_shadows",a.m_rendering_attributes.m_bCastsShadows},{"receives_shadows",a.m_rendering_attributes.m_bReceivesShadows},
    {"object_frame",frame.IsValid()?J{{"origin",point_json(frame.origin)},{"x",vector_json(frame.xaxis)},{"y",vector_json(frame.yaxis)}}:J(nullptr)},
    {"section_attributes_source",static_cast<unsigned>(a.SectionAttributesSource())},{"linetype_scale",a.LinetypePatternScale()},
    {"custom_linetype_present",a.CustomLinetype()!=nullptr},{"custom_section_style_present",a.CustomSectionStyle()!=nullptr},
    {"hatch_background_fill",color_json(a.HatchBackgroundFillColor())},{"hatch_boundary_visible",a.HatchBoundaryVisible()},
    {"clipping_plane_label_style",static_cast<unsigned>(a.ClippingPlaneLabelStyle())},{"userdata",userdata}};
}
J attribute_preservation(const ON_3dmObjectAttributes& a,const J& userdata,int,unsigned) {
  const ON_3dmObjectAttributes defaults; auto actual=attr_values(a,userdata),baseline=attr_values(defaults,J::array());
  if(userdata.empty() && a.SizeOf()<=2*1024*1024) {
    auto normalized=a; normalized.m_uuid=defaults.m_uuid; normalized.m_name=defaults.m_name; normalized.m_layer_index=defaults.m_layer_index;
    normalized.SetMode(defaults.Mode()); normalized.SetVisible(defaults.IsVisible());
    // Color-from-object is represented directly by host RGBA. Normalize only
    // the supported color selectors; from-parent remains an uncovered field.
    if(a.ColorSource()==ON::color_from_layer||a.ColorSource()==ON::color_from_object) { normalized.SetColorSource(defaults.ColorSource()); normalized.m_color=defaults.m_color; }
    actual["opaque_native_extensions"]=J{{"sdk_default_comparison_equal",normalized==defaults}};
    baseline["opaque_native_extensions"]=J{{"sdk_default_comparison_equal",true}};
  } else actual["opaque_native_extensions"]=J{{"serialization_omitted",true},{"native_sizeof_bytes",a.SizeOf()}};
  return preservation(actual,baseline);
}
J layer_values(const ON_Layer& layer,const J& userdata) {
  J render=J::array(); material_references(render,layer.m_rendering_attributes);
  return J{{"native_material_index",layer.RenderMaterialIndex()},{"native_line_pattern_index",layer.LinetypeIndex()},
    {"plot_color",color_json(layer.m_plot_color)},{"plot_weight_mm",layer.m_plot_weight_mm},
    {"display_material_id",nullable_id(layer.m_display_material_id)},{"iges_level",layer.IgesLevel()},{"expanded",layer.m_bExpanded},
    {"persistent_visibility",layer.PersistentVisibility()},{"persistent_locking",layer.PersistentLocking()},
    {"per_viewport_settings",{{"present",layer.HasPerViewportSettings(ON_nil_uuid)},{"crc",layer.PerViewportSettingsCRC()}}},
    {"render_material_references",render},{"userdata",userdata}};
}
J layer_preservation(const ON_Layer& layer,const J& userdata,int version,unsigned ov) {
  const ON_Layer defaults; auto actual=layer_values(layer,userdata),baseline=layer_values(defaults,J::array());
  if(userdata.empty()&&layer.SizeOf()<=2*1024*1024) {
    auto normalized=layer; normalized.SetId(defaults.Id()); normalized.SetIndex(defaults.Index()); normalized.SetName(defaults.Name());
    normalized.SetParentLayerId(defaults.ParentLayerId()); normalized.SetVisible(defaults.IsVisible()); normalized.SetLocked(defaults.IsLocked()); normalized.SetColor(defaults.Color());
    actual["opaque_native_extensions"]=record_fingerprint(normalized,version,ov); baseline["opaque_native_extensions"]=record_fingerprint(defaults,version,ov);
  } else actual["opaque_native_extensions"]=J{{"serialization_omitted",true},{"native_sizeof_bytes",layer.SizeOf()}};
  return preservation(actual,baseline);
}
J definition_values(const ON_InstanceDefinition& d,const J& userdata) {
  return J{{"description",bounded_native_text(d.Description())},{"url",bounded_native_text(d.URL())},{"url_tag",bounded_native_text(d.URL_Tag())},
    {"native_definition_type",static_cast<unsigned>(d.InstanceDefinitionType())},{"linked_file_path",bounded_native_text(d.LinkedFilePath())},
    {"has_linked_component_settings",d.HasLinkedIdefReferenceComponentSettings()},{"skip_nested_linked_definitions",d.SkipNestedLinkedDefinitions()},
    {"linked_component_appearance",static_cast<unsigned>(d.LinkedComponentAppearance())},{"userdata",userdata}};
}
J definition_preservation(const ON_InstanceDefinition& d,const J& userdata,int version,unsigned ov) {
  const ON_InstanceDefinition defaults; auto actual=definition_values(d,userdata),baseline=definition_values(defaults,J::array());
  if(userdata.empty()&&d.SizeOf()<=2*1024*1024) {
    auto normalized=d; normalized.SetId(defaults.Id()); normalized.SetIndex(defaults.Index()); normalized.SetName(defaults.Name());
    normalized.ClearInstanceGeometryIdList(); normalized.ClearBoundingBox(); normalized.SetUnitSystem(defaults.UnitSystem());
    actual["opaque_native_extensions"]=record_fingerprint(normalized,version,ov); baseline["opaque_native_extensions"]=record_fingerprint(defaults,version,ov);
  } else actual["opaque_native_extensions"]=J{{"serialization_omitted",true},{"native_sizeof_bytes",d.SizeOf()}};
  return preservation(actual,baseline);
}
struct UnitProjectionError:std::runtime_error { using std::runtime_error::runtime_error; };
double scale_coordinate(double value,double factor) {
  const double scaled=value*factor; if(!std::isfinite(scaled)||(value!=0&&scaled==0))throw UnitProjectionError("source unit conversion overflow/underflow"); return scaled;
}
void scale_point(J& p,double scale) { for(auto& value:p)value=scale_coordinate(real(value),scale); }
void curve_to_mm(J& curve,double scale) {
  const auto tag=curve.begin().key(); auto& c=curve.begin().value();
  if(tag=="Line") { scale_point(c["start"],scale); scale_point(c["end"],scale); }
  else if(tag=="Circle") { scale_point(c["center"],scale); const double radius=scale_coordinate(real(c["radius"]),scale); require(radius>0,"circle source scale"); c["radius"]=radius; }
  else if(tag=="Nurbs")for(auto& p:c["poles"])scale_point(p,scale);
  else if(tag=="Polycurve")for(auto& segment:c["segments"])curve_to_mm(segment,scale);
}
void geometry_to_mm(J& geometry,double scale) {
  const auto tag=geometry.begin().key(); auto& value=geometry.begin().value();
  if(tag=="Point")scale_point(value,scale);
  else if(tag=="Curve")curve_to_mm(value,scale);
  else if(tag=="Surface")for(auto& p:value["poles"])scale_point(p,scale);
  else if(tag=="Mesh")for(auto& p:value["vertices"])scale_point(p,scale);
  else if(tag=="Cloud")for(auto& p:value["points"])scale_point(p,scale);
}
bool exact_curve_parameters(const ON_Curve& curve,size_t depth=0) {
  require(depth<=max_depth,"native source curve depth budget");
  if(const auto* poly=ON_PolyCurve::Cast(&curve)) {
    require(poly->Count()>0&&static_cast<size_t>(poly->Count())<=max_records,"native source polycurve segment budget");
    for(int i=0;i<poly->Count();++i) {
      const auto* segment=poly->SegmentCurve(i); if(!segment || poly->SegmentDomain(i)!=segment->Domain() || !exact_curve_parameters(*segment,depth+1))return false;
    }
  }
  return true;
}
size_t source_curve_points(const ON_Curve& curve,size_t depth=0) {
  require(depth<=max_depth,"source curve point depth budget");
  if(const auto* n=ON_NurbsCurve::Cast(&curve)) { require(n->CVCount()>=0&&static_cast<size_t>(n->CVCount())<=max_array,"source curve point budget"); return static_cast<size_t>(n->CVCount()); }
  if(const auto* poly=ON_PolyCurve::Cast(&curve)) {
    require(poly->Count()>=0&&static_cast<size_t>(poly->Count())<=max_records,"source curve segment budget"); size_t total=0;
    for(int i=0;i<poly->Count();++i) { const auto* segment=poly->SegmentCurve(i); require(segment,"source missing curve segment"); const auto n=source_curve_points(*segment,depth+1); require(n<=max_array-total,"source polycurve point budget"); total+=n; } return total;
  }
  return ON_LineCurve::Cast(&curve)?2:0;
}
void source_output_tree(const J& value,size_t& nodes,size_t depth=0) {
  require(++nodes<=max_nodes&&depth<=max_depth,"source inventory node/depth budget");
  if(value.is_number_float())require(std::isfinite(value.get<double>()),"nonfinite source inventory field");
  else if(value.is_string()) { const auto& s=value.get_ref<const std::string&>(); require(s.size()<=4096&&s.find('\0')==std::string::npos,"source inventory string budget/NUL"); }
  else if(value.is_structured()) { require(value.size()<=max_array,"source inventory container budget"); for(const auto& item:value)source_output_tree(item,nodes,depth+1); }
}
struct SourceInventory {
  J result{{"schema_version",2},{"status","source_inventoried"},{"complete_read",false},
    {"table_receipts",J::array()},{"components",J::array()},{"objects",J::array()},
    {"layers",J::array()},{"definitions",J::array()},{"user_tables",J::array()},{"diagnostics",J::array()}};
  size_t records=0,points=0; uintmax_t native_bytes=0;
  std::set<std::string> ids;
  std::map<std::pair<std::string,int>,std::string> indexes;
  std::map<std::string,std::string> types;
  std::map<std::string,std::string> member_owners;
  const J* resolution=nullptr; int archive_version=50; unsigned opennurbs_version=ON::Version();
  bool units_resolved=false; double scale=1;
  explicit SourceInventory(const J* requested=nullptr):resolution(requested) {}
  bool projecting() const { return resolution!=nullptr; }
  void configure_units() {
    if(projecting()) {
      result["schema_version"]=3; result["status"]="source_projected"; result["decoder_policy"]="om9-source-projection-v1";
      result["unit_resolution"]=J{{"model",resolved_unit(result["units"]["model"],(*resolution)["model_mm_per_unit"],true)},
        {"page",resolved_unit(result["units"]["page"],(*resolution)["page_mm_per_unit"],true)},{"definitions",J::object()}};
      units_resolved=true; scale=real(result["unit_resolution"]["model"]["millimeters_per_unit"]);
    } else {
      units_resolved=!result["units"]["model"]["requires_resolution"].get<bool>();
      if(units_resolved)scale=real(result["units"]["model"]["millimeters_per_unit"]);
    }
  }
  void admit(const ON_Object& o) {
    require(++records<=max_records&&++source_record_count<=max_records,"source aggregate record budget");
    const auto bytes=static_cast<uintmax_t>(o.SizeOf()); require(bytes<=max_archive_bytes-native_bytes,"source native decoded-size budget"); native_bytes+=bytes;
  }
  void identity(const std::string& type,ON_UUID id,int index) {
    require(!ON_UuidIsNil(id),"nil source component UUID"); const auto s=id_string(id);
    require(ids.insert(s).second,"duplicate source component UUID"); types.emplace(s,type);
    if(index>=0)require(indexes.emplace(std::make_pair(type,index),s).second,"duplicate source component index");
  }
  J component(const ON_ModelComponent& c) {
    admit(c); const auto type=component_type(c.ComponentType()); identity(type,c.Id(),c.Index());
    J item{{"component_type",type},{"id",id_string(c.Id())},{"native_index",c.Index()==ON_UNSET_INT_INDEX?J(nullptr):J(c.Index())},
      {"name",bounded_native_text(c.Name())},{"native_kind",native_kind(c)},{"native_class_uuid",native_class(c)},
      {"relations",J::array()},{"userdata",native_userdata(c)},{"capability","retained_only"}};
    if(projecting()) { item["native_component_type"]=static_cast<unsigned>(c.ComponentType()); item["admission_required"]=true; }
    if(const auto* layer=ON_Layer::Cast(&c)) {
      if(!ON_UuidIsNil(layer->ParentLayerId()))item["relations"].push_back(reference("parent_layer",Type::Layer,layer->ParentLayerId()));
      item["relations"].push_back(reference("material",Type::RenderMaterial,ON_nil_uuid,layer->RenderMaterialIndex()));
      item["relations"].push_back(reference("line_pattern",Type::LinePattern,ON_nil_uuid,layer->LinetypeIndex()));
      material_references(item["relations"],layer->m_rendering_attributes);
      J layer_record{{"id",id_string(c.Id())},{"name",bounded_native_text(c.Name())},{"native_index",c.Index()},
        {"parent",nullable_id(layer->ParentLayerId())},{"visible",layer->IsVisible()},{"locked",layer->IsLocked()},
        {"color",color_json(layer->Color())},{"relations",item["relations"]},{"userdata",item["userdata"]}};
      if(projecting()) { item["preservation"]=layer_preservation(*layer,item["userdata"],archive_version,opennurbs_version);
        layer_record["native_component_type"]=static_cast<unsigned>(c.ComponentType()); layer_record["admission_required"]=true; layer_record["preservation"]=item["preservation"]; }
      result["layers"].push_back(std::move(layer_record));
    } else if(const auto* d=ON_InstanceDefinition::Cast(&c)) {
      const auto& members=d->InstanceGeometryIdList(); require(static_cast<size_t>(members.Count())<=max_records,"source definition member budget");
      J member_ids=J::array(); for(int i=0;i<members.Count();++i) { require(!ON_UuidIsNil(members[i]),"nil source definition member"); const auto member=id_string(members[i]); member_ids.push_back(member); item["relations"].push_back(reference("definition_member",Type::ModelGeometry,members[i]));
        require(member_owners.emplace(member,id_string(c.Id())).second,"source member shared between definitions"); }
      J definition{{"id",id_string(c.Id())},{"label",bounded_native_text(c.Name())},{"members",member_ids},
        {"linked",d->IsLinkedType()},{"units",definition_units(d->UnitSystem())},{"relations",item["relations"]},{"userdata",item["userdata"]}};
      if(projecting()) {
        const auto id=id_string(c.Id()); const auto& explicit_definitions=(*resolution)["definition_mm_per_unit"];
        result["unit_resolution"]["definitions"][id]=resolved_unit(definition["units"],explicit_definitions.contains(id)?explicit_definitions[id]:J(nullptr),false);
        item["preservation"]=definition_preservation(*d,item["userdata"],archive_version,opennurbs_version);
        definition["native_component_type"]=static_cast<unsigned>(c.ComponentType()); definition["preservation"]=item["preservation"];
        const auto bbox=d->BoundingBox(); definition["native_bbox"]=bbox.IsValid()?J::array({point_json(bbox.m_min),point_json(bbox.m_max)}):J(nullptr);
      }
      result["definitions"].push_back(std::move(definition));
    } else if(const auto* style=ON_DimStyle::Cast(&c)) {
      if(!ON_UuidIsNil(style->ParentId()))item["relations"].push_back(reference("parent_dimension_style",Type::DimStyle,style->ParentId()));
      item["settings"]=J{{"text_height",style->TextHeight()},{"length_resolution",style->LengthResolution()}};
    } else if(const auto* history=ON_HistoryRecord::Cast(&c)) {
      ON_SimpleArray<ON_UUID> a,b; history->m_antecedents.GetUuids(a); history->m_descendants.GetUuids(b);
      require(static_cast<size_t>(a.Count()+b.Count())<=max_records,"source history relation budget");
      for(int i=0;i<a.Count();++i)item["relations"].push_back(reference("history_antecedent",Type::ModelGeometry,a[i]));
      for(int i=0;i<b.Count();++i)item["relations"].push_back(reference("history_descendant",Type::ModelGeometry,b[i]));
      item["opaque_history_values"]=true;
    } else if(const auto* material=ON_Material::Cast(&c)) {
      item["render_material_instance_id"]=nullable_id(material->RdkMaterialInstanceId());
      require(static_cast<size_t>(material->m_textures.Count())<=max_records,"source material texture budget");
      item["texture_count"]=material->m_textures.Count();
    }
    if(projecting()&&!item.contains("preservation")) {
      // Some table writers require full archive/table context. Never synthesize
      // a body by standalone Write() and mistake writer repairs for read truth.
      const J body{{"serialization_omitted",true},{"native_sizeof_bytes",c.SizeOf()},{"native_kind",native_kind(c)}};
      item["preservation"]=preservation(J{{"userdata",item["userdata"]},{"native_component_body",body}},J{{"userdata",J::array()}});
    }
    result["components"].push_back(item); return item;
  }
  void object(const ON_Object& object,const ON_3dmObjectAttributes& a,Type type) {
    admit(object); require(object.IsValid(),"invalid source native object"); const auto type_name=component_type(type); identity(type_name,a.m_uuid,ON_UNSET_INT_INDEX);
    const auto* g=ON_Geometry::Cast(&object); J refs=J::array(); auto attrs=attribute_inventory(a,refs);
    J item{{"id",id_string(a.m_uuid)},{"label",bounded_native_text(a.m_name)},{"native_kind",native_kind(object)},
      {"native_class_uuid",native_class(object)},{"native_index",nullptr},{"is_definition_member",a.IsInstanceDefinitionObject()},
      {"capability","retained_only"},{"admission_required",true},{"geometry",nullptr},{"placement",nullptr},{"native_placement",matrix_json(ON_Xform::IdentityTransformation)},
      {"attributes",attrs},{"relations",refs},{"userdata",native_userdata(object)},{"channels",J::object()},
      {"unsupported_fields",J::array()},{"unrepresented_attributes",J::array({"material_source","native_material_index","groups","line_pattern_source","native_line_pattern_index",
        "viewport_id","space","display_order","plot_weight_mm","plot_color","plot_color_source","plot_weight_source","display_mode_overrides","render_material_references","texture_mapping_channels","userdata","independent_selectable"})},
      {"diagnostics",J::array()}};
    if(projecting()) { item["native_component_type"]=static_cast<unsigned>(type); item["preservation"]=attribute_preservation(a,attrs["userdata"],archive_version,opennurbs_version); }
    size_t n=0;
    if(const auto* mesh=ON_Mesh::Cast(g)) {
      n=static_cast<size_t>(mesh->VertexCount());
      item["channels"]=J{{"vertices",mesh->VertexCount()},{"faces",mesh->FaceCount()},{"normals",mesh->m_N.Count()},
        {"colors",mesh->m_C.Count()},{"face_normals",mesh->m_FN.Count()},{"texture_coordinates",mesh->m_T.Count()},
        {"surface_parameters",mesh->m_S.Count()},{"curvatures",mesh->m_K.Count()},{"ngons",mesh->NgonCount()}};
      for(const auto& channel:std::initializer_list<const char*>{"face_normals","texture_coordinates","surface_parameters","curvatures","ngons"})
        if(item["channels"][channel].get<int>()>0)item["unsupported_fields"].push_back(std::string("Mesh.")+channel);
    } else if(const auto* cloud=ON_PointCloud::Cast(g)) {
      n=static_cast<size_t>(cloud->m_P.Count()); item["channels"]=J{{"points",cloud->m_P.Count()},{"normals",cloud->m_N.Count()},
        {"colors",cloud->m_C.Count()},{"scalar_values",cloud->m_V.Count()},{"runtime_hidden",cloud->m_H.Count()},{"flags",cloud->m_flags}};
      if(cloud->m_V.Count()!=0)item["unsupported_fields"].push_back("Cloud.scalar_values");
      if(cloud->m_flags!=0)item["unsupported_fields"].push_back("Cloud.flags");
      item["diagnostics"].push_back("cloud hidden flags are runtime-only and absent from native 3DM");
    } else if(const auto* s=ON_NurbsSurface::Cast(g)) {
      require(s->CVCount(0)>0&&s->CVCount(1)>0&&static_cast<size_t>(s->CVCount(0))<=max_array/static_cast<size_t>(s->CVCount(1)),"source surface grid budget"); n=static_cast<size_t>(s->CVCount(0))*static_cast<size_t>(s->CVCount(1));
    } else if(const auto* c=ON_Curve::Cast(g))n=source_curve_points(*c);
    else if(ON_Point::Cast(g))n=1;
    require(n<=max_array-points,"source aggregate point budget"); points+=n;
    if(const auto* instance=ON_InstanceRef::Cast(g)) {
      item["native_placement"]=matrix_json(instance->m_xform);
      item["relations"].push_back(reference("instance_definition",Type::InstanceDefinition,instance->m_instance_definition_uuid));
    }
    if(const auto* annotation=ON_Annotation::Cast(g))item["relations"].push_back(reference("dimension_style",Type::DimStyle,annotation->DimensionStyleId()));
    if(const auto* hatch=ON_Hatch::Cast(g))item["relations"].push_back(reference("hatch_pattern",Type::HatchPattern,ON_nil_uuid,hatch->PatternIndex()));
    if(!item["userdata"].empty())item["unsupported_fields"].push_back("Geometry.userdata");
    if(a.m_space!=ON::model_space)item["unsupported_fields"].push_back("Attributes.page_space");
    double object_scale=scale; bool local_units_resolved=units_resolved; std::string definition;
    if(projecting()&&member_owners.contains(id_string(a.m_uuid))) {
      definition=member_owners.at(id_string(a.m_uuid)); const auto& local=result["unit_resolution"]["definitions"][definition];
      local_units_resolved=!local.is_null(); if(local_units_resolved)object_scale=real(local["millimeters_per_unit"]);
    }
    if(!local_units_resolved)item["diagnostics"].push_back(projecting()?"definition units require explicit resolution before canonical geometry projection":"model units require explicit resolution before canonical geometry admission");
    else if(g && item["unsupported_fields"].empty() && type==Type::ModelGeometry) {
      try {
        if(const auto* curve=ON_Curve::Cast(g))require(exact_curve_parameters(*curve),"polycurve segment parameter correspondence is not identity");
        if(projecting()&&ON_InstanceRef::Cast(g))(void)matrix(item["native_placement"]);
        auto geometry=geometry_inventory(*g); geometry_to_mm(geometry,object_scale);
        auto placement=item["native_placement"];
        if(projecting())if(const auto* instance=ON_InstanceRef::Cast(g)) {
          const auto target=id_string(instance->m_instance_definition_uuid);
          require(result["unit_resolution"]["definitions"].contains(target),"source instance missing definition resolution");
          const auto& resolved=result["unit_resolution"]["definitions"][target]; require(!resolved.is_null(),"instance target definition units require explicit resolution");
          const double ratio=object_scale/real(resolved["millimeters_per_unit"]);
          if(!std::isfinite(ratio)||ratio<=0)throw UnitProjectionError("source definition unit ratio overflow/underflow");
          for(int i=0;i<3;++i)for(int k=0;k<3;++k)placement[i][k]=scale_coordinate(real(placement[i][k]),ratio);
        }
        for(int i=0;i<3;++i)placement[i][3]=scale_coordinate(real(placement[i][3]),object_scale);
        if(projecting()&&ON_InstanceRef::Cast(g)) {
          try { (void)matrix(placement); }
          catch(const std::runtime_error&) { throw UnitProjectionError("source canonical instance transform overflow/underflow"); }
        }
        item["geometry"]=std::move(geometry); item["placement"]=std::move(placement); item["capability"]="geometry_projection_exact";
        item["projection_units_basis"]=projecting()?(definition.empty()?"resolved_model_units_to_canonical_mm":"resolved_definition_units_to_canonical_mm"):"archive_model_units_to_canonical_mm";
      } catch(const UnitProjectionError& error) { if(projecting())throw; item["diagnostics"].push_back(error.what()); }
      catch(const std::runtime_error& error) { item["diagnostics"].push_back(error.what()); }
    }
    if(type==Type::RenderLight) { item["component_type"]=type_name; result["components"].push_back(std::move(item)); }
    else result["objects"].push_back(std::move(item));
  }
  void resolve_refs(J& refs) const {
    for(auto& r:refs) {
      const auto type=r["component_type"].get<std::string>();
      if(!r["id"].is_null()) {
        const auto id=r["id"].get<std::string>(); require(types.contains(id)&&types.at(id)==type,"source missing or wrong-type UUID reference"); r["scope"]="source";
      } else if(!r["native_index"].is_null() && r["native_index"].get<int>()>=0) {
        const auto key=std::make_pair(type,r["native_index"].get<int>()); require(indexes.contains(key),"source missing index dependency"); r["id"]=indexes.at(key); r["scope"]="source";
      } else r["scope"]="system_or_unset";
    }
  }
  void finish_graph() {
    if(projecting())for(auto entry=(*resolution)["definition_mm_per_unit"].begin();entry!=(*resolution)["definition_mm_per_unit"].end();++entry)
      require(result["unit_resolution"]["definitions"].contains(entry.key()),"resolution references missing definition UUID");
    for(auto& table:std::initializer_list<const char*>{"components","objects","layers","definitions"})for(auto& item:result[table]) {
      resolve_refs(item["relations"]);
      if(item.contains("attributes"))for(const auto& r:item["relations"])if(r["role"]=="layer")item["attributes"]["layer"]=r["id"];
    }
    std::map<std::string,std::string> parents,members; std::map<std::string,std::vector<std::string>> children;
    for(const auto& l:result["layers"])parents.emplace(l["id"].get<std::string>(),l["parent"].is_null()?"":l["parent"].get<std::string>());
    for(const auto& [id,parent]:parents) { (void)parent; std::set<std::string> active; auto p=id; while(!p.empty()) { require(parents.contains(p)&&active.insert(p).second&&active.size()<=max_depth,"source layer dependency/cycle/depth"); p=parents.at(p); } }
    std::map<std::string,bool> definition_unit_mismatch;
    for(const auto& d:result["definitions"]) {
      const auto id=d["id"].get<std::string>(); children.emplace(id,std::vector<std::string>{});
      for(const auto& m:d["members"])require(members.emplace(m.get<std::string>(),id).second,"source member shared between definitions");
      definition_unit_mismatch[id]=projecting()
        ? d["linked"].get<bool>() || result["unit_resolution"]["definitions"][id].is_null()
        : d["linked"].get<bool>() || d["units"]["requires_resolution"].get<bool>() || !units_resolved
          || d["units"]["millimeters_per_unit"]!=result["units"]["model"]["millimeters_per_unit"];
    }
    std::set<std::string> observed;
    for(auto& o:result["objects"]) {
      const auto id=o["id"].get<std::string>(); require(o["is_definition_member"].get<bool>()==members.contains(id),"source definition membership/mode mismatch"); if(members.contains(id))observed.insert(id);
      for(const auto& r:o["relations"])if(r["role"]=="instance_definition") {
        if(members.contains(id))children.at(members.at(id)).push_back(r["id"].get<std::string>());
      }
    }
    require(observed.size()==members.size(),"source missing definition members");
    std::set<std::string> active; std::map<std::string,size_t> heights;
    std::function<size_t(const std::string&,size_t)> visit=[&](const std::string& id,size_t depth)->size_t {
      require(depth<=max_depth,"source definition depth budget"); if(heights.contains(id)) { require(heights.at(id)<=max_depth-depth,"source shared definition depth budget"); return heights.at(id); }
      require(active.insert(id).second,"source definition cycle"); size_t height=0; for(const auto& child:children.at(id))height=std::max(height,visit(child,depth+1)+1);
      require(height<=max_depth-depth,"source nested definition depth budget"); active.erase(id); heights.emplace(id,height); return height;
    };
    for(const auto& [id,child]:children) { (void)child; (void)visit(id,0); }
    std::map<std::string,const J*> objects;
    for(const auto& o:result["objects"])objects.emplace(o["id"].get<std::string>(),&o);
    std::map<std::string,J> readiness;
    std::function<J(const std::string&)> dependency=[&](const std::string& id)->J {
      if(readiness.contains(id))return readiness.at(id);
      std::set<std::string> retained,attributes,unit_requirements;
      if(definition_unit_mismatch.at(id))unit_requirements.insert(id);
      for(const auto& [member,owner]:members)if(owner==id) {
        const auto& object=*objects.at(member);
        if(object["capability"]!="geometry_projection_exact")retained.insert(member);
        if(!object["unrepresented_attributes"].empty() || !object["attributes"]["userdata"].empty())attributes.insert(member);
      }
      for(const auto& child:children.at(id)) {
        const auto nested=dependency(child);
        for(const auto& value:nested["retained_only_members"])retained.insert(value.get<std::string>());
        for(const auto& value:nested["attribute_admission_members"])attributes.insert(value.get<std::string>());
        for(const auto& value:nested["unit_policy_definitions"])unit_requirements.insert(value.get<std::string>());
      }
      J value{{"status","admission_required"},{"host_ready",false},{"retained_only_members",retained},
        {"attribute_admission_members",attributes},{"unit_policy_definitions",unit_requirements}};
      readiness.emplace(id,value); return value;
    };
    for(auto& d:result["definitions"]) { const auto id=d["id"].get<std::string>(); d["dependency_readiness"]=dependency(id); d["admission_required"]=true; }
    for(auto& o:result["objects"])for(const auto& r:o["relations"])if(r["role"]=="instance_definition") {
      const auto readiness_=dependency(r["id"].get<std::string>()); o["dependency_readiness"]=readiness_;
      if(projecting()&&!readiness_["unit_policy_definitions"].empty()) {
        o["geometry"]=nullptr; o["placement"]=nullptr; o["capability"]="retained_only";
        o["diagnostics"].push_back("instance definition closure has unresolved units or linked dependencies");
        o.erase("projection_units_basis");
      }
    }
  }
};
void table_receipt(SourceInventory& out,ON_BinaryArchive& archive,Table table,const char* name_,size_t decoded,bool derived=false) {
  const auto status=archive.Archive3dmTableStatus(table);
  require(status.m_table_type==table && status.m_state==ON_3dmArchiveTableStatus::TableState::Finished,"source incomplete archive table");
  require(status.m_crc_error_count==0&&status.m_recoverable_error_count==0&&status.m_critical_error_count==0,"source table CRC/recoverable/critical error");
  require(derived || status.m_item_count==decoded,"source skipped/converted table records");
  out.result["table_receipts"].push_back(J{{"table",name_},{"read_records",status.m_item_count},{"retained_records",derived?0:decoded},
    {"derived_records",derived?decoded:0},{"crc_errors",status.m_crc_error_count},{"recoverable_errors",status.m_recoverable_error_count},{"critical_errors",status.m_critical_error_count}});
}
template<class C,class Reader>void source_table(SourceInventory& out,ON_BinaryArchive& archive,Table table,const char* name_,
  bool (ON_BinaryArchive::*begin)(),Reader read,bool (ON_BinaryArchive::*end)()) {
  if(!archive.ArchiveContains3dmTable(table)) {
    out.result["table_receipts"].push_back(J{{"table",name_},{"present",false},{"read_records",0},{"retained_records",0},{"derived_records",0},{"crc_errors",0},{"recoverable_errors",0},{"critical_errors",0}}); return;
  }
  operation_record=std::string("source table ")+name_; require((archive.*begin)(),"source begin table"); size_t count_=0;
  std::vector<std::unique_ptr<C>> deferred_styles;
  for(;;) {
    C* pointer=nullptr; const int rc=read(archive,pointer); std::unique_ptr<C> item(pointer);
    if(rc==0) { require(!item,"source end table with record"); break; }
    require(rc==1&&item,"source failed/skipped native table record"); require(++count_<=max_records,"source table record budget");
    if(table==Table::dimension_style_table)deferred_styles.push_back(std::move(item));
    else out.component(*item);
  }
  require((archive.*end)(),"source end table");
  const auto final_status=archive.Archive3dmTableStatus(table);
  const bool derived=table==Table::dimension_style_table && final_status.m_item_count==0;
  if(derived)require(count_<=1,"source synthesized dimension style conversion requires explicit policy");
  else if(table==Table::dimension_style_table)for(const auto& style:deferred_styles)out.component(*style);
  table_receipt(out,archive,table,name_,count_,derived);
}
J source_settings(const ON_3dmSettings& s,int version,unsigned opennurbs_version) {
  ON_Write3dmBufferArchive serialized(0,2*1024*1024,version,opennurbs_version); require(s.Write(serialized),"bounded native settings serialization");
  require(static_cast<size_t>(s.m_views.Count()+s.m_named_views.Count()+s.m_named_cplanes.Count()+s.m_plugin_list.Count())<=max_records,"source settings record budget");
  J views=J::array(),named=J::array(),planes=J::array(),plugins=J::array();
  auto add_views=[](J& target,const ON_ClassArray<ON_3dmView>& source) { for(int i=0;i<source.Count();++i) {
    const auto& v=source[i]; target.push_back(J{{"id",nullable_id(v.m_vp.ViewportId())},{"name",bounded_native_text(v.m_name)},
      {"camera_location",point_json(v.m_vp.CameraLocation())},{"camera_direction",vector_json(v.m_vp.CameraDirection())},
      {"camera_up",vector_json(v.m_vp.CameraUp())},{"projection",static_cast<unsigned>(v.m_vp.Projection())}}); } };
  add_views(views,s.m_views); add_views(named,s.m_named_views);
  for(int i=0;i<s.m_named_cplanes.Count();++i) { const auto& p=s.m_named_cplanes[i]; planes.push_back(J{{"name",bounded_native_text(p.m_name)},
    {"origin",point_json(p.m_plane.origin)},{"x",vector_json(p.m_plane.xaxis)},{"y",vector_json(p.m_plane.yaxis)},{"grid_spacing",p.m_grid_spacing},{"snap_spacing",p.m_snap_spacing}}); }
  for(int i=0;i<s.m_plugin_list.Count();++i)plugins.push_back(J{{"id",nullable_id(s.m_plugin_list[i].m_plugin_id)}});
  return J{{"basepoint",point_json(s.m_model_basepoint)},{"model_url",bounded_native_text(s.m_model_URL)},
    {"active_view_id",nullable_id(s.m_active_view_id)},{"views",views},{"named_views",named},{"named_cplanes",planes},{"plugins",plugins},
    {"current_color",color_json(s.m_current_color)},{"current_color_source",static_cast<unsigned>(s.m_current_color_source)},
    {"wire_density",s.m_current_wire_density},{"linetype_display_scale",s.m_linetype_display_scale},
    {"relations",J::array({reference("current_layer",Type::Layer,s.CurrentLayerId(),s.CurrentLayerIndex()),
      reference("current_material",Type::RenderMaterial,s.CurrentMaterialId(),s.CurrentMaterialIndex()),
      reference("current_line_pattern",Type::LinePattern,s.CurrentLinePatternId(),s.CurrentLinePatternIndex()),
      reference("current_dimension_style",Type::DimStyle,s.CurrentDimensionStyleId(),s.CurrentDimensionStyleIndex()),
      reference("current_hatch_pattern",Type::HatchPattern,s.CurrentHatchPatternId())})},
    {"native_settings_serialization",{{"byte_length",serialized.SizeOfArchive()},{"sha1",hash_bytes(serialized.Buffer(),serialized.SizeOfArchive())}}},
    {"unrepresented_sections",J::array({"render_mesh","analysis_mesh","annotation","render_settings","grid_defaults","earth_anchor","io_settings","view_print_settings"})}};
}
J inventory_source(const std::filesystem::path& path,const J* resolution=nullptr,ON_ManifestMap* retained_reference_map=nullptr,FILE* retained_held_file=nullptr) {
  const auto file_bytes=retained_held_file?static_cast<uintmax_t>(_filelengthi64(_fileno(retained_held_file))):std::filesystem::file_size(path); require(file_bytes>0&&file_bytes<=max_archive_bytes,"source archive file-byte budget");
  source_record_count=0; const int initial_errors=ON_GetErrorCount(),initial_warnings=ON_GetWarningCount();
  SourceInventory out(resolution);auto owned_file=retained_held_file?std::unique_ptr<File>{}:std::make_unique<File>(path,resolution!=nullptr);
  auto* file=retained_held_file?retained_held_file:owned_file->value;
  require(_fseeki64(file,0,SEEK_SET)==0,"source archive held-file rewind");
  const auto source_hash=resolution?source_sha256(file,file_bytes):std::string();
  ON_BinaryFile archive(ON::archive_mode::read3dm,file);
  archive.SetShouldSerializeUserDataDefault(true); int version=0; ON_String comment;
  operation_record="source start section"; require(archive.Read3dmStartSection(&version,comment),"source native start section");
  require(version>=2,"source archive version requires explicit legacy policy");
  table_receipt(out,archive,Table::start_section,"start_section",0);
  ON_3dmProperties properties; operation_record="source properties"; require(archive.Read3dmProperties(properties),"source native properties"); table_receipt(out,archive,Table::properties_table,"properties",0);
  ON_3dmSettings settings; operation_record="source settings"; require(archive.Read3dmSettings(settings),"source native settings"); table_receipt(out,archive,Table::settings_table,"settings",0);
  out.result["units"]=J{{"model",native_units(settings.m_ModelUnitsAndTolerances)},{"page",native_units(settings.m_PageUnitsAndTolerances)}};
  out.configure_units();
  const unsigned ov=archive.ArchiveOpenNURBSVersion(); out.result["settings"]=source_settings(settings,version,ov);
  out.archive_version=version; out.opennurbs_version=ov;
#define OM9_SOURCE_TABLE(C,T,N,B,R,E) source_table<C>(out,archive,Table::T,N,&ON_BinaryArchive::B,[](ON_BinaryArchive& a,C*& p){return a.R(&p);},&ON_BinaryArchive::E)
  OM9_SOURCE_TABLE(ON_Bitmap,bitmap_table,"images",BeginRead3dmBitmapTable,Read3dmBitmap,EndRead3dmBitmapTable);
  OM9_SOURCE_TABLE(ON_TextureMapping,texture_mapping_table,"texture_mappings",BeginRead3dmTextureMappingTable,Read3dmTextureMapping,EndRead3dmTextureMappingTable);
  OM9_SOURCE_TABLE(ON_Material,material_table,"materials",BeginRead3dmMaterialTable,Read3dmMaterial,EndRead3dmMaterialTable);
  OM9_SOURCE_TABLE(ON_Linetype,linetype_table,"line_patterns",BeginRead3dmLinetypeTable,Read3dmLinetype,EndRead3dmLinetypeTable);
  OM9_SOURCE_TABLE(ON_Layer,layer_table,"layers",BeginRead3dmLayerTable,Read3dmLayer,EndRead3dmLayerTable);
  OM9_SOURCE_TABLE(ON_Group,group_table,"groups",BeginRead3dmGroupTable,Read3dmGroup,EndRead3dmGroupTable);
  OM9_SOURCE_TABLE(ON_DimStyle,dimension_style_table,"dimension_styles",BeginRead3dmDimStyleTable,Read3dmDimStyle,EndRead3dmDimStyleTable);
  const auto text_status=archive.Archive3dmTableStatus(Table::text_style_table); require(text_status.m_item_count<=max_records,"source text style record budget");
  for(unsigned i=0;i<text_status.m_item_count;++i) { const auto* style=archive.ArchiveTextStyleFromArchiveTextStyleIndex(static_cast<int>(i)); require(style,"source skipped text style record"); out.component(*style); }
  table_receipt(out,archive,Table::text_style_table,"text_styles",text_status.m_item_count);
#undef OM9_SOURCE_TABLE
  operation_record="source render lights"; require(archive.BeginRead3dmLightTable(),"source begin light table"); size_t lights=0;
  for(;;) { ON_Light* pointer=nullptr; ON_3dmObjectAttributes attrs; const int rc=archive.Read3dmLight(&pointer,&attrs); std::unique_ptr<ON_Light> light(pointer);
    if(rc==0) { require(!light,"source end light table record"); break; } require(rc==1&&light,"source failed/skipped light record"); out.object(*light,attrs,Type::RenderLight); ++lights; }
  require(archive.EndRead3dmLightTable(),"source end light table"); table_receipt(out,archive,Table::light_table,"lights",lights);
  source_table<ON_HatchPattern>(out,archive,Table::hatchpattern_table,"hatch_patterns",&ON_BinaryArchive::BeginRead3dmHatchPatternTable,[](ON_BinaryArchive& a,ON_HatchPattern*& p){return a.Read3dmHatchPattern(&p);},&ON_BinaryArchive::EndRead3dmHatchPatternTable);
  source_table<ON_InstanceDefinition>(out,archive,Table::instance_definition_table,"instance_definitions",&ON_BinaryArchive::BeginRead3dmInstanceDefinitionTable,[](ON_BinaryArchive& a,ON_InstanceDefinition*& p){return a.Read3dmInstanceDefinition(&p);},&ON_BinaryArchive::EndRead3dmInstanceDefinitionTable);
  operation_record="source object table"; require(archive.BeginRead3dmObjectTable(),"source begin object table"); size_t objects=0;
  for(;;) {
    operation_record="source object record ordinal "+std::to_string(objects);
    ON_Object* pointer=nullptr; ON_3dmObjectAttributes attrs; const int rc=archive.Read3dmObject(&pointer,&attrs,0); std::unique_ptr<ON_Object> object(pointer);
    if(rc==0) { require(!object,"source end object table record"); break; }
    require(rc==1&&object,"source failed/skipped/unregistered native object record"); operation_record="source object "+id_string(attrs.m_uuid); out.object(*object,attrs,Type::ModelGeometry); ++objects;
  }
  require(archive.EndRead3dmObjectTable(),"source end object table"); table_receipt(out,archive,Table::object_table,"objects",objects);
  source_table<ON_HistoryRecord>(out,archive,Table::historyrecord_table,"history_records",&ON_BinaryArchive::BeginRead3dmHistoryRecordTable,[](ON_BinaryArchive& a,ON_HistoryRecord*& p){return a.Read3dmHistoryRecord(p);},&ON_BinaryArchive::EndRead3dmHistoryRecordTable);
  operation_record="source anonymous user tables"; size_t user_tables=0;
  for(;;) {
    ON__UINT32 tcode=0; ON__INT64 length=0; require(archive.PeekAt3dmBigChunkType(&tcode,&length),"source missing archive end mark"); if(tcode!=TCODE_USER_TABLE)break;
    require(++user_tables<=max_records&&++out.records<=max_records&&++source_record_count<=max_records,"source aggregate user table record budget"); ON_UUID application=ON_nil_uuid; bool goo=false; int uv=0; unsigned uov=0;
    require(archive.BeginRead3dmUserTable(application,&goo,&uv,&uov),"source malformed user table header"); ON_3dmGoo data;
    require(archive.Read3dmAnonymousUserTable(uv,uov,data)&&data.m_value>=0,"source malformed anonymous user table");
    out.result["user_tables"].push_back(J{{"application_id",nullable_id(application)},{"archive_version",uv},{"opennurbs_version",uov},
      {"last_saved_as_goo",goo},{"byte_length",data.m_value},{"payload_sha1",hash_bytes(data.m_goo,static_cast<size_t>(data.m_value))},{"opaque_references",true}});
    require(archive.EndRead3dmUserTable(),"source end user table"); const auto status=archive.Archive3dmTableStatus(Table::user_table);
    require(status.m_crc_error_count==0&&status.m_recoverable_error_count==0&&status.m_critical_error_count==0,"source user table errors");
  }
  out.result["table_receipts"].push_back(J{{"table","user_tables"},{"read_records",user_tables},{"retained_records",user_tables},{"derived_records",0},{"crc_errors",0},{"recoverable_errors",0},{"critical_errors",0}});
  operation_record="source end mark"; size_t endmark_bytes=0; require(archive.Read3dmEndMark(&endmark_bytes),"source complete archive end mark required"); table_receipt(out,archive,Table::end_mark,"end_mark",0);
  const auto final_bytes=retained_held_file?static_cast<uintmax_t>(_filelengthi64(_fileno(file))):std::filesystem::file_size(path);
  require(endmark_bytes==file_bytes&&archive.CurrentPosition()==file_bytes&&final_bytes==file_bytes,"source truncated/trailing/changed archive bytes");
  require(archive.BadCRCCount()==0&&archive.CriticalErrorCount()==0,"source native CRC/critical errors");
  require(ON_GetErrorCount()==initial_errors&&ON_GetWarningCount()==initial_warnings,"source native read warning/error or silent record repair");
  out.result["archive"]=J{{"version",version},{"opennurbs_version",ov},{"file_bytes",file_bytes},{"endmark_bytes",endmark_bytes},{"index_semantics","decoded_native"}};
  if(resolution) { operation_record="source held-byte SHA256 revalidation"; require(source_sha256(file,file_bytes)==source_hash,"source bytes changed during native read"); out.result["archive"]["source_sha256"]=source_hash; }
  out.finish_graph(); out.resolve_refs(out.result["settings"]["relations"]); out.result["complete_read"]=true;
  out.result["admission_required"]=true;
  size_t nodes=0; source_output_tree(out.result,nodes);
  if(retained_reference_map)*retained_reference_map=archive.ManifestMap();
  return out.result;
}
#include "retained_merge.hpp"
}
int wmain(int argc,wchar_t** argv) {
  try {
    require(argc>=2,"usage: om9-exchange-native --write packet.json private.3dm | --inspect model.3dm"); const std::wstring mode(argv[1]); J result;
    if(mode==L"--write") { require(argc==4,"--write requires packet and unique private output"); operation_record="packet preflight"; const auto p=packet(std::filesystem::path(argv[2])); Library library; Writer writer; result=writer.write(p,std::filesystem::path(argv[3])); }
    else if(mode==L"--inventory-source") { require(argc==3,"--inventory-source requires archive"); operation_record="source archive"; Library library; result=inventory_source(std::filesystem::path(argv[2])); }
    else if(mode==L"--project-source") { require(argc==4,"--project-source requires held archive and resolution packet"); operation_record="source resolution preflight"; const auto resolution=source_resolution(std::filesystem::path(argv[3])); Library library; result=inventory_source(std::filesystem::path(argv[2]),&resolution); }
    else if(mode==L"--prepare-retained-source") { require(argc==3,"--prepare-retained-source requires archive"); operation_record="retained source preparation"; Library library; result=prepare_retained_source(std::filesystem::path(argv[2])); }
    else if(mode==L"--write-retained") { require(argc==4,"--write-retained requires strict request and exclusive private output"); operation_record="retained request"; Library library; result=write_retained(std::filesystem::path(argv[2]),std::filesystem::path(argv[3])); }
    else if(mode==L"--inspect-retained-current") { require(argc==4,"--inspect-retained-current requires strict request and private output"); operation_record="cold retained current request"; Library library; result=inspect_retained_current(std::filesystem::path(argv[2]),std::filesystem::path(argv[3])); }
    else { require(mode==L"--inspect" && argc==3,"unsupported command/options"); operation_record="inspect archive"; Library library; result=inspect(std::filesystem::path(argv[2])); }
    operation_record="serialize owned result"; const auto bytes=result.dump(); require(bytes.size()<=8*1024*1024,"inventory stdout byte budget"); std::cout<<bytes<<'\n'; return 0;
  } catch(const std::bad_alloc&) { std::cerr<<"OM9 native exchange ("<<operation_record<<"): allocation refused\n"; return 1; }
  catch(const std::exception& error) { std::cerr<<"OM9 native exchange ("<<operation_record<<"): "<<error.what()<<'\n'; return 1; }
  catch(...) { std::cerr<<"OM9 native exchange ("<<operation_record<<"): native exception\n"; return 2; }
}
