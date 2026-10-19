// SPDX-License-Identifier: LGPL-2.1-or-later
// Authored native dependency/format probe. Not an OM9 import/export capability.
// Upstream openNURBS ownership stays inside this dedicated process; no borrowed
// document/view/shape pointer crosses an ABI. General archive peak safety and
// process containment must be accepted before integrating a native operation.
#include <opennurbs.h>
#include <cmath>
#include <filesystem>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>

namespace {
struct Library {
  Library() { ON::Begin(); }
  ~Library() { ON::End(); }
  Library(const Library&) = delete;
  Library& operator=(const Library&) = delete;
};
void require(bool value, const char* operation) {
  if (!value) throw std::runtime_error(operation);
}
void point(std::ostream& out, const ON_3dPoint& p) {
  require(p.IsValid(), "nonfinite native point");
  out << '[' << p.x << ',' << p.y << ',' << p.z << ']';
}
void add(ONX_Model& model, const ON_Geometry& geometry, const wchar_t* name, const char* uuid) {
  require(geometry.IsValid(), "invalid authored fixture geometry");
  ON_3dmObjectAttributes attributes;
  attributes.m_name = name;
  attributes.m_uuid = ON_UuidFromString(uuid);
  attributes.m_layer_index = 0;
  require(!model.AddModelGeometryComponent(&geometry, &attributes, false).IsEmpty(), "native owned model copy");
}
void write_fixture(const std::filesystem::path& path) {
  ONX_Model model;
  model.m_settings.m_ModelUnitsAndTolerances.m_unit_system = ON::LengthUnitSystem::Millimeters;
  model.m_properties.m_Application.m_application_name = L"OpenMatrix9 standalone authored dependency probe";
  model.AddDefaultLayer(L"Authored", ON_Color::Black);
  const ON_Point p(ON_3dPoint(1,2,3));
  add(model,p,L"Point","10000000-0000-4000-8000-000000000001");
  const ON_LineCurve line(ON_3dPoint(0,0,0),ON_3dPoint(12,6,0));
  add(model,line,L"Line","10000000-0000-4000-8000-000000000002");
  ON_NurbsCurve arc(3,true,3,3);
  const double w=std::sqrt(0.5);
  require(arc.SetCV(0,ON_4dPoint(10,0,0,1)) && arc.SetCV(1,ON_4dPoint(10*w,10*w,0,w)) && arc.SetCV(2,ON_4dPoint(0,10,0,1)),"rational CV assignment");
  require(arc.SetKnot(0,2) && arc.SetKnot(1,2) && arc.SetKnot(2,4) && arc.SetKnot(3,4),"native compact knot assignment");
  add(model,arc,L"RationalArc","10000000-0000-4000-8000-000000000003");
  ON_Mesh mesh;
  require(mesh.SetVertex(0,ON_3dPoint(0,0,0)) && mesh.SetVertex(1,ON_3dPoint(10,0,0)) && mesh.SetVertex(2,ON_3dPoint(0,10,0)) && mesh.SetTriangle(0,0,1,2),"mesh assignment");
  add(model,mesh,L"Mesh","10000000-0000-4000-8000-000000000004");
  ON_PointCloud cloud;
  cloud.AppendPoint(ON_3dPoint(2,3,4));cloud.AppendPoint(ON_3dPoint(5,6,7));
  cloud.m_C.Append(ON_Color(255,0,0));cloud.m_C.Append(ON_Color(0,255,0));
  add(model,cloud,L"Cloud","10000000-0000-4000-8000-000000000005");
  ON_wString diagnostics;
  ON_TextLog log(diagnostics);
  require(model.Write(path.c_str(),5,&log),"native version-5 fixture writer");
}
std::string inventory(const std::filesystem::path& path) {
  // Fixture-only process probe. A file-byte limit alone does not certify the
  // reader's allocation bounds on hostile nested records; no app import gate
  // is enabled by these authored-fixture tests.
  require(std::filesystem::file_size(path)<=16*1024*1024,"probe file byte budget");
  ONX_Model model;ON_wString diagnostics;ON_TextLog log(diagnostics);
  require(model.Read(path.c_str(),&log),"native archive read");
  std::ostringstream out;out << std::setprecision(17);
  out << "{\"archive_version\":" << model.m_3dm_file_version << ",\"objects\":[";
  ONX_ModelComponentIterator objects(model,ON_ModelComponent::Type::ModelGeometry);
  bool first=true;unsigned count=0;
  for (const ON_ModelComponent* component=objects.FirstComponent();component;component=objects.NextComponent()) {
    require(++count<=32,"probe object budget");
    const auto* item=ON_ModelGeometryComponent::Cast(component);
    require(item!=nullptr,"native component type");
    const auto* geometry=item->Geometry(nullptr);const auto* attributes=item->Attributes(nullptr);
    require(geometry && attributes && geometry->IsValid(),"native geometry/attributes admission");
    // Names in this authored fixture are fixed ASCII. General UTF-8 strings
    // and escaping belong to the production adapter's bounded wire schema.
    const ON_String name(attributes->m_name);const std::string label(name.Array());
    require(label=="Point"||label=="Line"||label=="RationalArc"||label=="Mesh"||label=="Cloud","probe fixture label");
    if (!first) out << ',';first=false;
    out << "{\"name\":\"" << label << '"';
    if (const auto* p=ON_Point::Cast(geometry)) {out << ",\"type\":\"Point\",\"point\":";point(out,p->point);}
    else if (const auto* line=ON_LineCurve::Cast(geometry)) {out << ",\"type\":\"Line\",\"start\":";point(out,line->PointAtStart());out << ",\"end\":";point(out,line->PointAtEnd());}
    else if (const auto* arc=ON_NurbsCurve::Cast(geometry)) {const ON_Interval domain=arc->Domain();out << ",\"type\":\"NurbsCurve\",\"degree\":" << arc->Degree() << ",\"rational\":" << (arc->IsRational()?"true":"false") << ",\"cv_count\":" << arc->CVCount() << ",\"domain\":[" << domain[0] << ',' << domain[1] << "],\"midpoint\":";point(out,arc->PointAt(domain.Mid()));}
    else if (const auto* mesh=ON_Mesh::Cast(geometry)) {out << ",\"type\":\"Mesh\",\"faces\":[";for (int i=0;i<mesh->m_F.Count();++i) {if (i) out << ',';const auto& f=mesh->m_F[i];out << '[' << f.vi[0] << ',' << f.vi[1] << ',' << f.vi[2] << ',' << f.vi[3] << ']';}out << ']';}
    else if (const auto* cloud=ON_PointCloud::Cast(geometry)) {out << ",\"type\":\"Cloud\",\"points\":[";for (int i=0;i<cloud->m_P.Count();++i) {if (i) out << ',';point(out,cloud->m_P[i]);}out << "],\"colors\":[";for (int i=0;i<cloud->m_C.Count();++i) {if (i) out << ',';const auto& color=cloud->m_C[i];out << '[' << color.Red() << ',' << color.Green() << ',' << color.Blue() << ']';}out << ']';}
    else {throw std::runtime_error("unsupported probe geometry family");}
    out << '}';
  }
  out << "]}";return out.str();
}
}
int wmain(int argc,wchar_t** argv) {
  try {
    require(argc==3,"usage: om9-opennurbs-probe --write-fixture|--inspect path");
    Library library;
    const std::filesystem::path path(argv[2]);const std::wstring mode(argv[1]);
    if (mode==L"--write-fixture") write_fixture(path);
    else require(mode==L"--inspect","unsupported probe command");
    // Produce owned output before publishing any success bytes.
    const std::string report=inventory(path);std::cout << report << '\n';return 0;
  } catch(const std::exception& error) {std::cerr << "OM9 openNURBS probe: " << error.what() << '\n';return 1;}
  catch(...) {std::cerr << "OM9 openNURBS probe: native exception\n";return 2;}
}
