// SPDX-License-Identifier: LGPL-2.1-or-later
// Authored native-only inventory fixtures. Run through om9-native-runner.
#include <opennurbs.h>
#include <filesystem>
#include <iomanip>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>

namespace {
void check(bool b, const char* m) { if (!b) throw std::runtime_error(m); }
ON_UUID id(unsigned n) { char s[37]{}; std::snprintf(s,sizeof(s),"20000000-0000-4000-8000-%012x",n); return ON_UuidFromString(s); }
}
class AuthoredUnknownPoint : public ON_Point {
  ON_OBJECT_DECLARE(AuthoredUnknownPoint);
public:
  AuthoredUnknownPoint() : ON_Point(ON_3dPoint(8,9,10)) {}
};
ON_OBJECT_IMPLEMENT(AuthoredUnknownPoint,ON_Point,"bcf4536b-64f6-4569-81b0-1ef3dbd3f51d");
class AuthoredUnknownLayer : public ON_Layer {
  ON_OBJECT_DECLARE(AuthoredUnknownLayer);
};
ON_OBJECT_IMPLEMENT(AuthoredUnknownLayer,ON_Layer,"265627f7-f9ce-4f96-825b-4c10b9f4c1e7");
class AuthoredOpaqueData : public ON_UserData {
  ON_OBJECT_DECLARE(AuthoredOpaqueData);
public:
  AuthoredOpaqueData() { m_userdata_uuid=id(1001); m_application_uuid=id(1002); m_userdata_copycount=1; }
  bool Archive() const override { return true; }
  bool Write(ON_BinaryArchive& a) const override { return a.WriteString(ON_wString(L"OM9 authored opaque attachment")); }
  bool Read(ON_BinaryArchive& a) override { ON_wString s; return a.ReadString(s); }
};
ON_OBJECT_IMPLEMENT(AuthoredOpaqueData,ON_UserData,"a041186b-6c27-4272-965d-0657f4073c85");

namespace {
ON_3dmObjectAttributes* add(ONX_Model& model, ON_Geometry* geometry, unsigned n, const wchar_t* name, bool member=false, int layer=0) {
  auto* a=new ON_3dmObjectAttributes(); a->m_uuid=id(n); a->m_name=name; a->m_layer_index=layer;
  a->m_material_index=0; a->SetMaterialSource(ON::material_from_object); a->AddToGroup(0);
  a->SetColorSource(ON::color_from_object); a->m_color=ON_Color(11,22,33,55);
  if(member) a->SetMode(ON::idef_object);
  check(!model.AddManagedModelGeometryComponent(geometry,a,false).IsEmpty(),"authored geometry add");
  return a;
}
// This mutates only the fixture author's uniquely owned in-memory model. It
// never runs on an imported model or inside the production exchange program.
void clean_model(ONX_Model& m,double millimeters_per_unit,bool mixed,bool unresolved) {
  using T=ON_ModelComponent::Type;
  for(const unsigned n:{20u,21u,22u,24u,25u,26u,34u,42u})check(!m.RemoveModelComponent(T::ModelGeometry,id(n)).IsEmpty(),"clean fixture geometry removal");
  if(!unresolved) { check(!m.RemoveModelComponent(T::ModelGeometry,id(44)).IsEmpty(),"clean orphan point removal"); check(!m.RemoveModelComponent(T::InstanceDefinition,id(33)).IsEmpty(),"clean orphan definition removal"); }
  ONX_ModelComponentIterator objects(m,T::ModelGeometry);
  const auto scale=ON_Xform::DiagonalTransformation(1/millimeters_per_unit);
  for(const auto* c=objects.FirstComponent();c;c=objects.NextComponent()) {
    const auto* item=ON_ModelGeometryComponent::Cast(c); check(item,"clean fixture geometry component");
    auto* g=const_cast<ON_Geometry*>(item->Geometry(nullptr)); auto* a=const_cast<ON_3dmObjectAttributes*>(item->Attributes(nullptr)); check(g&&a,"clean fixture owned geometry/attrs");
    const auto identity=a->m_uuid; const auto name=a->m_name; const bool member=a->IsInstanceDefinitionObject();
    *a=ON_3dmObjectAttributes(); a->m_uuid=identity; a->m_name=name; a->m_layer_index=0; if(member)a->SetMode(ON::idef_object);
    if(identity==id(10))ON_Point::Cast(g)->point=ON_3dPoint(25.4,50.8,0);
    if(auto* poly=ON_PolyCurve::Cast(g))for(int i=0;i<poly->Count();++i)check(poly->SegmentCurve(i)->SetDomain(poly->SegmentDomain(i)[0],poly->SegmentDomain(i)[1]),"clean exact segment domains");
    if(auto* instance=ON_InstanceRef::Cast(g)) { for(int i=0;i<3;++i)instance->m_xform[i][3]/=millimeters_per_unit; }
    else check(g->Transform(scale),"clean physical geometry normalization");
    if(auto* cloud=ON_PointCloud::Cast(g))for(int i=0;i<cloud->m_N.Count();++i)check(cloud->m_N[i].Unitize(),"clean cloud normal");
  }
  ONX_ModelComponentIterator defs(m,T::InstanceDefinition);
  for(const auto* c=defs.FirstComponent();c;c=defs.NextComponent()) {
    auto* d=const_cast<ON_InstanceDefinition*>(ON_InstanceDefinition::Cast(c)); check(d,"clean fixture owned definition");
    auto bbox=d->BoundingBox(); bbox.Transform(scale); d->SetBoundingBox(bbox);
    d->SetUnitSystem(m.m_settings.m_ModelUnitsAndTolerances.m_unit_system);
    if(d->Id()==id(30)) { ON_SimpleArray<ON_UUID> members; members.Append(id(40)); d->SetInstanceGeometryIdList(members); }
    if(d->Id()==id(33))d->SetUnitSystem(ON::LengthUnitSystem::None);
    if(mixed && (d->Id()==id(30)||d->Id()==id(32))) {
      const double local=d->Id()==id(30)?25.4:2.5;
      d->SetUnitSystem(d->Id()==id(30)?ON_UnitSystem(ON::LengthUnitSystem::Inches):ON_UnitSystem::CreateCustomUnitSystem(L"OM9 clean definition unit",0.0025));
      auto b=d->BoundingBox(); b.Transform(ON_Xform::DiagonalTransformation(1/local)); d->SetBoundingBox(b);
      auto* member=const_cast<ON_Geometry*>(m.ModelGeometryComponentFromId(id(d->Id()==id(30)?40:43)).Geometry(nullptr));
      check(member&&member->Transform(ON_Xform::DiagonalTransformation(1/local)),"clean definition-local geometry");
      for(const auto* object=objects.FirstComponent();object;object=objects.NextComponent()) {
        const auto* item=ON_ModelGeometryComponent::Cast(object); auto* instance=const_cast<ON_InstanceRef*>(ON_InstanceRef::Cast(item->Geometry(nullptr)));
        if(instance&&instance->m_instance_definition_uuid==d->Id())for(int i=0;i<3;++i)for(int k=0;k<3;++k)instance->m_xform[i][k]*=local;
      }
    }
  }
  if(unresolved) { auto* instance=new ON_InstanceRef(); instance->m_instance_definition_uuid=id(33); instance->m_xform=ON_Xform::IdentityTransformation; instance->m_xform[0][3]=11;
    auto* attributes=add(m,instance,28,L"Unresolved definition instance"); *attributes=ON_3dmObjectAttributes(); attributes->m_uuid=id(28); attributes->m_name=L"Unresolved definition instance"; attributes->m_layer_index=0; }
  ONX_ModelComponentIterator layers(m,T::Layer);
  for(const auto* c=layers.FirstComponent();c;c=layers.NextComponent())if(c->Id()==id(1)) {
    auto* layer=const_cast<ON_Layer*>(ON_Layer::Cast(c)); check(layer,"clean owned layer"); layer->SetRenderMaterialIndex(-1); layer->SetLinetypeIndex(-1);
  }
  check(!m.RemoveModelComponent(T::Layer,id(2)).IsEmpty(),"clean child layer removal");
  for(const auto entry:std::initializer_list<std::pair<T,unsigned>>{{T::RenderMaterial,3},{T::LinePattern,4},{T::Group,5},{T::DimStyle,6},{T::HatchPattern,7}})
    check(!m.RemoveModelComponent(entry.first,id(entry.second)).IsEmpty(),"clean unsupported table removal");
  m.m_settings.SetCurrentDimensionStyleId(ON_nil_uuid);
  for(int i=0;i<m.m_userdata_table.Count();++i)delete m.m_userdata_table[i]; m.m_userdata_table.Destroy();
}
void create(const std::wstring& requested_mode, const std::filesystem::path& path) {
  const bool clean=requested_mode.rfind(L"clean-",0)==0;
  const bool mixed=requested_mode==L"clean-mixed",unresolved=requested_mode==L"clean-unresolved";
  const bool color_parent=requested_mode==L"clean-color-parent",color_material=requested_mode==L"clean-color-material";
  const std::wstring mode=clean?(mixed||unresolved||color_parent||color_material?L"mm":requested_mode.substr(6)):requested_mode;
  if(clean)check(mode==L"mm"||mode==L"inch"||mode==L"custom"||mode==L"unitless"||mode==L"unset","clean fixture mode");
  check(!std::filesystem::exists(path),"fixture destination exists");
  ONX_Model m;
  auto& units=m.m_settings.m_ModelUnitsAndTolerances;
  units.m_unit_system=ON_UnitSystem::Millimeters;
  if(mode==L"inch") units.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Inches);
  else if(mode==L"custom") units.m_unit_system=ON_UnitSystem::CreateCustomUnitSystem(L"OM9 authored unit",0.0025);
  else if(mode==L"unitless") units.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::None);
  else if(mode==L"unset") units.m_unit_system=ON_UnitSystem::Unset;
  else check(mode==L"mm" || mode==L"unknown-class" || mode==L"unknown-table" || mode==L"record-budget" || mode==L"polycurve-point-budget" || mode==L"duplicate-id" || mode==L"missing-dependency" || mode==L"definition-cycle" || mode==L"invalid-tolerance" || mode==L"attribute-userdata","fixture mode");
  units.m_absolute_tolerance=0.002; units.m_angle_tolerance=0.02; units.m_relative_tolerance=0.005;
  if(mode==L"invalid-tolerance")units.m_absolute_tolerance=0;
  m.m_settings.m_PageUnitsAndTolerances.m_unit_system=mode==L"custom"
    ? ON_UnitSystem::CreateCustomUnitSystem(L"OM9 paper unit",0.007)
    : ON_UnitSystem(ON::LengthUnitSystem::Inches);
  m.m_settings.m_model_basepoint=ON_3dPoint(3,4,5);
  ON_Layer layer; layer.SetId(id(1)); layer.SetName(L"Source Parent"); layer.SetIndex(0); layer.SetColor(ON_Color(100,110,120)); layer.SetRenderMaterialIndex(0);
  check(!m.AddModelComponent(layer,false).IsEmpty(),"layer add");
  ON_Layer child; child.SetId(id(2)); child.SetName(L"Source Child"); child.SetIndex(1); child.SetParentLayerId(id(1)); child.SetVisible(false); child.SetLocked(true);
  check(!m.AddModelComponent(child,false).IsEmpty(),"child layer add");
  m.m_settings.SetCurrentLayerId(id(1));
  ON_Material material; material.SetId(id(3)); material.SetName(L"Source material"); material.SetIndex(0);
  check(!m.AddModelComponent(material,false).IsEmpty(),"material add");
  ON_Linetype pattern; pattern.SetId(id(4)); pattern.SetName(L"Source line pattern"); pattern.SetIndex(0); pattern.AppendSegment(ON_LinetypeSegment(2.0,ON_LinetypeSegment::eSegType::stLine));
  check(!m.AddModelComponent(pattern,false).IsEmpty(),"line pattern add");
  ON_Group group; group.SetId(id(5)); group.SetName(L"Source group"); group.SetIndex(0);
  check(!m.AddModelComponent(group,false).IsEmpty(),"group add");
  ON_DimStyle dim; dim.SetId(id(6)); dim.SetName(L"Source nondefault dimension style"); dim.SetIndex(0); dim.SetTextHeight(2.75); dim.SetLengthResolution(5);
  check(!m.AddModelComponent(dim,false).IsEmpty(),"dimstyle add");
  m.m_settings.SetCurrentDimensionStyleId(id(6));
  ON_HatchPattern hatch; hatch.SetId(id(7)); hatch.SetName(L"Source hatch"); hatch.SetIndex(0);
  check(!m.AddModelComponent(hatch,false).IsEmpty(),"hatch pattern add");
  add(m,new ON_Point(ON_3dPoint(1,2,3)),10,L"Exact point");
  auto* line=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(3,4,0)); check(line->SetDomain(7,9),"line domain"); auto* line_attributes=add(m,line,11,L"Exact line");
  auto* circle=new ON_ArcCurve(ON_Circle(ON_xy_plane,6)); check(circle->SetDomain(2,8),"circle domain"); add(m,circle,12,L"Exact circle");
  const ON_Circle phased_circle(ON_Plane(ON_3dPoint(2,3,4),ON_3dVector(0,1,0),ON_3dVector(0,0,1)),6);
  const ON_Arc phased_arc(phased_circle,ON_Interval(1,1+2*ON_PI));
  add(m,new ON_ArcCurve(phased_arc,11,17),29,L"Exact phase and custom parameter circle");
  add(m,new ON_ArcCurve(ON_Arc(ON_Circle(ON_xy_plane,3),ON_Interval(0,2*ON_PI-1e-13)),11,17),34,L"Retained near-full arc");
  auto* curve=new ON_NurbsCurve(3,true,3,3); const double w=std::sqrt(0.5);
  curve->SetCV(0,ON_4dPoint(10,0,0,1)); curve->SetCV(1,ON_4dPoint(10*w,10*w,0,w)); curve->SetCV(2,ON_4dPoint(0,10,0,1));
  curve->SetKnot(0,2); curve->SetKnot(1,2); curve->SetKnot(2,4); curve->SetKnot(3,4); add(m,curve,13,L"Exact rational curve");
  auto* surface=new ON_NurbsSurface(3,true,2,2,2,3);
  for(int u=0;u<2;++u)for(int v=0;v<3;++v) { const double weight=1.0+u+v; surface->SetCV(u,v,ON_4dPoint(5.0*u*weight,static_cast<double>(v)*weight,static_cast<double>(u+v)*weight,weight)); }
  surface->SetKnot(0,0,0); surface->SetKnot(0,1,2); surface->SetKnot(1,0,3); surface->SetKnot(1,1,4); surface->SetKnot(1,2,5); add(m,surface,14,L"Exact rational surface");
  auto* mesh=new ON_Mesh(); mesh->m_V.Append(ON_3fPoint(0,0,0)); mesh->m_V.Append(ON_3fPoint(2,0,0)); mesh->m_V.Append(ON_3fPoint(2,2,0)); mesh->m_V.Append(ON_3fPoint(0,2,0));
  ON_MeshFace face{}; face.vi[0]=0; face.vi[1]=1; face.vi[2]=2; face.vi[3]=3; mesh->m_F.Append(face);
  for(int i=0;i<4;++i) { mesh->m_N.Append(ON_3fVector(0,0,1)); mesh->m_C.Append(ON_Color(50,100,150,45)); }
  auto* uv=new ON_Mesh(*mesh); for(int i=0;i<4;++i) uv->m_T.Append(ON_2fPoint(static_cast<float>(i%2),static_cast<float>(i/2)));
  add(m,mesh,15,L"Exact mesh"); add(m,uv,25,L"Retained UV mesh");
  auto* cloud=new ON_PointCloud(); cloud->m_P.Append(ON_3dPoint(2,3,4)); cloud->m_P.Append(ON_3dPoint(5,6,7));
  cloud->m_N.Append(ON_3dVector(0,0,1)); cloud->m_N.Append(ON_3dVector(0,1,0)); cloud->m_C.Append(ON_Color(255,0,0)); cloud->m_C.Append(ON_Color(0,255,0,127));
  auto* values=new ON_PointCloud(*cloud); values->m_V.Append(1.5); values->m_V.Append(2.5);
  add(m,cloud,16,L"Exact cloud"); add(m,values,26,L"Retained cloud scalar channel");
  auto* poly=new ON_PolyCurve(); poly->Append(new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(1,0,0))); poly->Append(new ON_LineCurve(ON_3dPoint(1,0,0),ON_3dPoint(1,1,0))); add(m,poly,17,L"Exact polycurve");
  const ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(1,0,0),ON_3dPoint(1,1,0),ON_3dPoint(0,1,0),ON_3dPoint(0,0,1),ON_3dPoint(1,0,1),ON_3dPoint(1,1,1),ON_3dPoint(0,1,1)};
  add(m,ON_BrepBox(corners),20,L"Retained Brep box");
  add(m,new ON_ArcCurve(ON_Arc(ON_Circle(ON_xy_plane,3),ON_PI)),21,L"Retained partial arc");
  auto* annotation=new ON_Text(); check(annotation->Create(L"Authored annotation",&dim,ON_xy_plane,false,0,0),"authored annotation create"); annotation->SetDimensionStyleId(id(6)); add(m,annotation,24,L"Retained annotation relation");
  auto* opaque=new ON_Point(ON_3dPoint(4,5,6)); check(opaque->AttachUserData(new AuthoredOpaqueData()),"opaque attachment"); add(m,opaque,22,L"Retained opaque attachment");
  add(m,new ON_Point(ON_3dPoint(1,2,3)),40,L"Definition point",true);
  add(m,ON_BrepBox(corners),42,L"Retained CAD definition member",true);
  auto* nested=new ON_InstanceRef(); nested->m_instance_definition_uuid=id(mode==L"definition-cycle"?31:mode==L"missing-dependency"?99:30); nested->m_xform=ON_Xform::TranslationTransformation(ON_3dVector(0,5,0)); add(m,nested,41,L"Nested definition instance",true);
  ON_InstanceDefinition master; master.SetId(id(30)); master.SetName(L"Shared master"); master.SetUnitSystem(units.m_unit_system); master.SetBoundingBox(ON_BoundingBox(ON_3dPoint(0,0,0),ON_3dPoint(1,2,3))); ON_SimpleArray<ON_UUID> members; members.Append(id(40)); members.Append(id(42)); master.SetInstanceGeometryIdList(members);
  check(!m.AddModelComponent(master,false).IsEmpty(),"master definition");
  ON_InstanceDefinition outer; outer.SetId(id(31)); outer.SetName(L"Nested master"); outer.SetUnitSystem(units.m_unit_system); outer.SetBoundingBox(ON_BoundingBox(ON_3dPoint(1,7,3),ON_3dPoint(1,7,3))); members.SetCount(0); members.Append(id(41)); outer.SetInstanceGeometryIdList(members);
  check(!m.AddModelComponent(outer,false).IsEmpty(),"nested definition");
  auto* root=new ON_InstanceRef(); root->m_instance_definition_uuid=id(31); root->m_xform=ON_Xform::IdentityTransformation; root->m_xform[0][0]=-2; root->m_xform[1][1]=3; root->m_xform[2][2]=4; root->m_xform[0][3]=10; add(m,root,18,L"Mirrored nested instance");
  auto* shared=new ON_InstanceRef(); shared->m_instance_definition_uuid=id(30); shared->m_xform=ON_Xform::IdentityTransformation; add(m,shared,19,L"Shared instance");
  add(m,new ON_Point(ON_3dPoint(1,2,3)),43,L"Unrelated exact definition point",true);
  ON_InstanceDefinition good; good.SetId(id(32)); good.SetName(L"Unrelated projection master"); good.SetUnitSystem(units.m_unit_system); good.SetBoundingBox(ON_BoundingBox(ON_3dPoint(1,2,3),ON_3dPoint(1,2,3))); members.SetCount(0); members.Append(id(43)); good.SetInstanceGeometryIdList(members); check(!m.AddModelComponent(good,false).IsEmpty(),"unrelated definition");
  auto* unrelated=new ON_InstanceRef(); unrelated->m_instance_definition_uuid=id(32); unrelated->m_xform=ON_Xform::IdentityTransformation; unrelated->m_xform[0][3]=7; add(m,unrelated,27,L"Unrelated exact instance");
  add(m,new ON_Point(ON_3dPoint(1,2,3)),44,L"Orphan different-unit definition point",true);
  ON_InstanceDefinition orphan; orphan.SetId(id(33)); orphan.SetName(L"Orphan different-unit master"); orphan.SetUnitSystem(ON::LengthUnitSystem::Meters); orphan.SetBoundingBox(ON_BoundingBox(ON_3dPoint(1,2,3),ON_3dPoint(1,2,3))); members.SetCount(0); members.Append(id(44)); orphan.SetInstanceGeometryIdList(members); check(!m.AddModelComponent(orphan,false).IsEmpty(),"orphan definition");
  auto* data=new ONX_Model_UserData(); data->m_uuid=id(1003); data->m_usertable_3dm_version=50; data->m_usertable_opennurbs_version=ON::Version(); data->m_goo.m_typecode=TCODE_USER_RECORD; data->m_goo.m_value=8; data->m_goo.m_goo=static_cast<unsigned char*>(onmemdup("OM9 goo!",8)); m.m_userdata_table.Append(data);
  if(mode==L"unknown-class") add(m,new AuthoredUnknownPoint(),23,L"Unregistered cold class");
  if(mode==L"unknown-table") { auto* unknown=new AuthoredUnknownLayer(); unknown->SetId(id(1004)); unknown->SetName(L"Unregistered cold layer"); unknown->SetIndex(2); check(!m.AddModelComponentForExperts(unknown,true,false,true).IsEmpty(),"unknown table fixture"); }
  if(mode==L"duplicate-id")line_attributes->m_uuid=id(10);
  if(mode==L"attribute-userdata")check(line_attributes->AttachUserData(new AuthoredOpaqueData()),"authored attribute attachment");
  if(mode==L"polycurve-point-budget") {
    auto* p=new ON_PolyCurve();
    for(int segment=0;segment<2;++segment) { auto* c=new ON_NurbsCurve(3,false,2,34000); const double start=33999.0*segment;
      for(int i=0;i<34000;++i) { c->SetCV(i,ON_3dPoint(start+i,0,0)); c->SetKnot(i,start+i); }
      check(p->Append(c),"large native polycurve segment"); }
    add(m,p,28,L"Aggregate polycurve point budget");
  }
  if(mode==L"record-budget") for(unsigned n=2000;n<6100;++n) add(m,new ON_Point(ON_3dPoint(0,0,0)),n,L"Record budget point");
  if(clean) { const auto tag=units.m_unit_system.UnitSystem(); const double declared=units.m_unit_system.MillimetersPerUnit(ON_DBL_QNAN);
    clean_model(m,tag==ON::LengthUnitSystem::None||tag==ON::LengthUnitSystem::Unset||!std::isfinite(declared)?25.4:declared,mixed,unresolved);
    if(color_parent||color_material) {
      auto* a=const_cast<ON_3dmObjectAttributes*>(m.ModelGeometryComponentFromId(id(10)).Attributes(nullptr));
      check(a,"clean color selector attributes");
      a->SetColorSource(color_parent?ON::color_from_parent:ON::color_from_material);
    }
    if(mode==L"unitless"||mode==L"unset")m.m_settings.m_PageUnitsAndTolerances.m_unit_system=units.m_unit_system; }
  ON_wString log_text; ON_TextLog log(log_text); check(m.Write(path.c_str(),50,&log),"fixture native write");
}
// This independent cold oracle reads authored test archives only. Its answers
// come from ON_ArcCurve::Evaluate, not the exchange JSON projection.
void circle_oracle(const std::filesystem::path& path,unsigned object_id) {
  ONX_Model model; ON_wString text; ON_TextLog log(text);
  check(model.Read(path.c_str(),&log),"oracle read authored archive");
  const ON_ArcCurve* circle=nullptr;
  ONX_ModelComponentIterator objects(model,ON_ModelComponent::Type::ModelGeometry);
  for(const auto* c=objects.FirstComponent();c;c=objects.NextComponent()) {
    const auto* item=ON_ModelGeometryComponent::Cast(c); check(item,"oracle native object");
    const auto* attributes=item->Attributes(nullptr); check(attributes,"oracle native attributes");
    if(attributes->m_uuid==id(object_id)) { circle=ON_ArcCurve::Cast(item->Geometry(nullptr)); break; }
  }
  check(circle&&circle->IsValid(),"oracle actual arc missing or invalid");
  const auto domain=circle->Domain(),angles=circle->m_arc.DomainRadians();
  std::cout<<std::setprecision(17)<<"{\"status\":\"native_circle_oracle\",\"domain\":["<<domain[0]<<','<<domain[1]
    <<"],\"angle_domain\":["<<angles[0]<<','<<angles[1]<<"],\"native_is_circle\":"<<(circle->IsCircle()?"true":"false")<<",\"samples\":[";
  for(int i=0;i<3;++i) {
    const double u=i==0?domain[0]:i==1?domain.Mid():domain[1]; double values[12]{};
    check(circle->Evaluate(u,3,3,values),"oracle actual native D0-D3 evaluation");
    if(i)std::cout<<',';
    std::cout<<"{\"u\":"<<u<<",\"derivatives\":[";
    for(int d=0;d<4;++d) { if(d)std::cout<<','; std::cout<<'['<<values[3*d]<<','<<values[3*d+1]<<','<<values[3*d+2]<<']'; }
    std::cout<<"]}";
  }
  std::cout<<"]}\n";
}
}
int wmain(int argc,wchar_t** argv) {
  try {
    check(argc==3||argc==4,"fixture arguments: mode private.3dm | --oracle-circle authored.3dm object_number"); ON::Begin();
    if(argc==4) { check(std::wstring(argv[1])==L"--oracle-circle","oracle arguments"); const auto n=std::stoul(argv[3]); check(n<=65535,"oracle object id"); circle_oracle(std::filesystem::path(argv[2]),static_cast<unsigned>(n)); }
    else { create(argv[1],std::filesystem::path(argv[2])); std::cout<<"{\"status\":\"authored_native_fixture\"}\n"; }
    ON::End(); return 0;
  }
  catch(const std::exception& e) { std::cerr<<"OM9 authored source fixture: "<<e.what()<<'\n'; return 1; }
}
