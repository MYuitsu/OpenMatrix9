// SPDX-License-Identifier: LGPL-2.1-or-later
// RCORE-01.T06 authored shared native definition; existing adapters only.
#include "ThreeDmArchive.h"
#include "ThreeDmMerge.h"
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <QJsonDocument>
#include <QJsonArray>
#include <QCryptographicHash>
#include <QFile>
#include <iostream>
#include <limits>
#include <cmath>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {
QJsonArray checks,cases;
constexpr double tolerance=1e-7;
using Points=std::array<std::array<double,3>,8>;
const Points expectedA{{{105,207,309},{107,207,309},{107,210,309},{105,210,309},{105,207,313},{107,207,313},{107,210,313},{105,210,313}}};
const Points expectedB{{{-1,20,44.5},{-1,16,44.5},{-10,16,44.5},{-10,20,44.5},{-1,20,46.5},{-1,16,46.5},{-10,16,46.5},{-10,20,46.5}}};
const std::array<double,16> ma{1,0,0,104,0,1,0,205,0,0,1,306,0,0,0,1},mb{0,-3,0,5,-2,0,0,22,0,0,.5,43,0,0,0,1},mm{0,-3,0,12,-2,0,0,11,0,0,.5,56,0,0,0,1};
void check(bool v,const char* name){checks.append(QJsonObject{{"name",name},{"passed",v}});if(!v)throw ExchangeError(name);}
template<class F> void rejected(F&& f,const char* name){QString error;try{f();}catch(const ExchangeError& e){error=QString::fromUtf8(e.what());}cases.append(QJsonObject{{"name",name},{"rejection",error}});check(error.contains("Invalid or singular")&&error.contains("transform"),name);}
QString uuid(ON_UUID id){char text[37]{};ON_UuidToString(id,text);return QString::fromLatin1(text);}
QJsonArray matrix(const std::array<double,16>& m){QJsonArray out;for(auto v:m)out.append(v);return out;}
QByteArray bytes(const std::filesystem::path& p){QFile f(QString::fromStdWString(p.wstring()));check(f.open(QIODevice::ReadOnly),"artifact opened");return f.readAll();}
QByteArray payload(const ON_Object& o){ON_Write3dmBufferArchive a(0,1024*1024,50,ON::Version());check(a.WriteObject(o),"native object serialized");return QByteArray(static_cast<const char*>(a.Buffer()),static_cast<qsizetype>(a.SizeOfArchive()));}
void saveBytes(const std::filesystem::path& p,const QByteArray& data){QFile f(QString::fromStdWString(p.wstring()));check(f.open(QIODevice::WriteOnly),"native payload diagnostic opened");check(f.write(data)==data.size(),"native payload diagnostic written");}
const ON_Object& object(const ONX_Model& m,ON_ModelComponent::Type type,const QString& id){auto c=m.ComponentFromId(type,ON_UuidFromString(id.toLatin1().constData())).ModelComponent();check(c!=nullptr,"exact component identity found");if(auto g=ON_ModelGeometryComponent::Cast(c))return *g->Geometry(nullptr);return *c;}
void points(const TopoDS_Shape& shape,const Points& expected,const char* name){
 TopTools_IndexedMapOfShape vertices;TopExp::MapShapes(shape,TopAbs_VERTEX,vertices);check(vertices.Extent()==8,"exact eight native corners");std::array<bool,8> used{};QJsonArray residuals,actual;
 for(int i=1;i<=vertices.Extent();++i){auto p=BRep_Tool::Pnt(TopoDS::Vertex(vertices(i)));actual.append(QJsonArray{p.X(),p.Y(),p.Z()});}
 cases.append(QJsonObject{{"name",name},{"actual_corners",actual}});
 for(auto p:expected){int matched=-1;double closest=std::numeric_limits<double>::infinity();
  for(int i=1;i<=vertices.Extent();++i){auto q=BRep_Tool::Pnt(TopoDS::Vertex(vertices(i)));double d=std::hypot(std::hypot(p[0]-q.X(),p[1]-q.Y()),p[2]-q.Z());closest=std::min(closest,d);if(d<=tolerance){check(matched==-1,"no ambiguous corner match");matched=i-1;}}
  check(matched>=0 && !used[matched],"bijective independent world corner");used[matched]=true;residuals.append(closest);
 }
 cases.append(QJsonObject{{"name",name},{"actual_corners",actual},{"corner_residuals",residuals}});
}
void readCorners(const std::filesystem::path& path,const Points& b,const QString& aId,const QString& bId){auto model=readArchive(path);check(model.items.size()==2,"two placements expand without canonical duplication");bool seenA=false,seenB=false;for(const auto& item:model.items){auto shape=std::get_if<TopoDS_Shape>(&item.geometry);check(shape&&!shape->IsNull(),"instance output is native CAD");auto root=QString::fromStdString(item.sourceRootUuid);check(root==aId||root==bId,"expanded leaf belongs to an exact authored root UUID");bool isA=root==aId;check(isA?!seenA:!seenB,"each authored root expands once");(isA?seenA:seenB)=true;points(*shape,isA?expectedA:b,item.name.c_str());}check(seenA&&seenB,"both exact root identities observed");}
void run(const std::filesystem::path& dir){
 std::filesystem::create_directories(dir);ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_Layer layer;layer.SetName(L"Fixture");model.AddModelComponent(layer);
 ON_3dPoint local[8]{ON_3dPoint(1,2,3),ON_3dPoint(3,2,3),ON_3dPoint(3,5,3),ON_3dPoint(1,5,3),ON_3dPoint(1,2,7),ON_3dPoint(3,2,7),ON_3dPoint(3,5,7),ON_3dPoint(1,5,7)};std::unique_ptr<ON_Brep> box(ON_BrepBox(local));check(box&&box->IsValid(),"authored local box valid");check(resolvedBrepOrientation(*box,tolerance)==1,"independently resolved box outward");box->SetSolidOrientationForExperts(1);
 ON_3dmObjectAttributes attributes;attributes.SetMode(ON::idef_object);attributes.m_name=L"Canonical";auto member=model.AddModelGeometryComponent(box.get(),&attributes);QString memberId=uuid(member.ModelComponent()->Id());
 ON_InstanceDefinition definition;definition.SetName(L"Shared");definition.SetInstanceDefinitionType(ON_InstanceDefinition::IDEF_UPDATE_TYPE::Static);definition.SetUnitSystem(ON::LengthUnitSystem::Millimeters);definition.AddInstanceGeometryId(member.ModelComponent()->Id());definition.SetBoundingBox(box->BoundingBox());auto def=model.AddModelComponent(definition);QString defId=uuid(def.ModelComponent()->Id());
 auto add=[&](const std::array<double,16>& values,const wchar_t* name){ON_InstanceRef r;r.m_instance_definition_uuid=def.ModelComponent()->Id();for(int i=0;i<16;++i)r.m_xform[i/4][i%4]=values[i];r.m_bbox=box->BoundingBox();r.m_bbox.Transform(r.m_xform);ON_3dmObjectAttributes a;a.m_layer_index=0;a.m_name=name;return model.AddModelGeometryComponent(&r,&a);};
 // The model member was cloned before the standalone box's bbox was computed.
 // Seed the actual model member's correct cache before recording source bytes;
 // the structural writer necessarily computes bounds for the definition.
 auto canonical=ON_ModelGeometryComponent::Cast(member.ModelComponent())->Geometry(nullptr);auto memberBounds=canonical->BoundingBox();
 check(memberBounds.m_min==ON_3dPoint(1,2,3)&&memberBounds.m_max==ON_3dPoint(3,5,7),"authored model member has exact valid local bounds before baseline");
 auto a=add(ma,L"A"),b=add(mb,L"B");QString aId=uuid(a.ModelComponent()->Id()),bId=uuid(b.ModelComponent()->Id());auto source=dir/"instance-source.3dm";check(model.Write(source.c_str(),5,nullptr),"authored source V5 written");const auto sourceBytes=bytes(source);readCorners(source,expectedB,aId,bId);
 auto inventory=inspectArchive(source);const auto& original=*inventory.nativeModel;auto originalMember=payload(object(original,ON_ModelComponent::Type::ModelGeometry,memberId));auto originalA=payload(object(original,ON_ModelComponent::Type::ModelGeometry,aId));auto originalDefinition=payload(object(original,ON_ModelComponent::Type::InstanceDefinition,defId));
 const QString ns="12345678-1234-1234-1234-123456789abc";
 auto request=[&](const std::array<double,16>& transform){return QJsonObject{{"schema_version",1},{"sources",QJsonArray{QJsonObject{{"namespace",ns},{"snapshot",QString::fromStdWString(source.wstring())},{"archive_sha256",inventory.document["archive_sha256"]},{"scale_mm",1.}}}},{"selected",QJsonArray{QJsonObject{{"host_id","A"},{"namespace",ns},{"source_uuid",aId},{"action","unchanged"}},QJsonObject{{"host_id","B"},{"namespace",ns},{"source_uuid",bId},{"action","instance"},{"instance_matrix",matrix(transform)}}}}};};
 auto output=dir/"instance-edited.3dm";writePreservedArchive(request(mm),output);Points moved=expectedB;for(auto& p:moved){p[0]+=7;p[1]-=11;p[2]+=13;}readCorners(output,moved,aId,bId);
 auto changed=inspectArchive(output);const auto& current=*changed.nativeModel;
 saveBytes(dir/"original-member.bin",originalMember);saveBytes(dir/"current-member.bin",payload(object(current,ON_ModelComponent::Type::ModelGeometry,memberId)));
 saveBytes(dir/"original-A.bin",originalA);saveBytes(dir/"current-A.bin",payload(object(current,ON_ModelComponent::Type::ModelGeometry,aId)));
 saveBytes(dir/"original-definition.bin",originalDefinition);saveBytes(dir/"current-definition.bin",payload(object(current,ON_ModelComponent::Type::InstanceDefinition,defId)));
 check(payload(object(current,ON_ModelComponent::Type::ModelGeometry,memberId))==originalMember,"B overlay leaves exact native canonical geometry payload");
 check(payload(object(current,ON_ModelComponent::Type::ModelGeometry,aId))==originalA,"B overlay leaves exact native sibling payload");
 check(payload(object(current,ON_ModelComponent::Type::InstanceDefinition,defId))==originalDefinition,"B overlay leaves exact native shared definition payload");
 check(bytes(source)==sourceBytes,"B overlay leaves input archive bytes unchanged");
 const auto destination=bytes(output);auto singular=mm;singular[0]=singular[4]=singular[8]=0;auto projective=mm;projective[12]=1;
 for(auto bad:{singular,projective}){rejected([&]{writePreservedArchive(request(bad),output);},bad==singular?"singular instance matrix explicitly rejected":"projective instance matrix explicitly rejected");check(bytes(output)==destination && bytes(source)==sourceBytes,"invalid writer preserves destination and source bytes");}
 for(double bad:{std::numeric_limits<double>::quiet_NaN(),std::numeric_limits<double>::infinity()}){ON_Point point(2,3,4);auto before=payload(point);ON_Xform transform=ON_Xform::IdentityTransformation;transform[0][0]=bad;rejected([&]{transformNativeGeometry(point,transform);},std::isnan(bad)?"actual native NaN transform rejected":"actual native Infinity transform rejected");check(payload(point)==before,"native invalid transform rejected before geometry mutation");}
 check(payload(object(original,ON_ModelComponent::Type::ModelGeometry,memberId))==originalMember && payload(object(original,ON_ModelComponent::Type::ModelGeometry,aId))==originalA,"retained original native owners unchanged after all calls");
 cases.append(QJsonObject{{"source",QString::fromStdWString(source.wstring())},{"output",QString::fromStdWString(output.wstring())},{"member_id",memberId},{"definition_id",defId},{"A_id",aId},{"B_id",bId},{"member_payload_sha256",QString::fromLatin1(QCryptographicHash::hash(originalMember,QCryptographicHash::Sha256).toHex())},{"A_payload_sha256",QString::fromLatin1(QCryptographicHash::hash(originalA,QCryptographicHash::Sha256).toHex())},{"definition_payload_sha256",QString::fromLatin1(QCryptographicHash::hash(originalDefinition,QCryptographicHash::Sha256).toHex())}});
}
}
int main(int argc,char** argv){ON::Begin();QString error;auto path=argc>1?std::filesystem::path(argv[1]):std::filesystem::temp_directory_path()/"instance-results.json";try{run(path.parent_path()/"instance-fixtures");}catch(const Standard_Failure& e){error=e.GetMessageString();}catch(const std::exception& e){error=e.what();}QJsonObject result{{"schema",1},{"fixture_length_unit","mm"},{"length_tolerance",tolerance},{"checks",checks},{"cases",cases},{"error",error}};QFile f(QString::fromStdWString(path.wstring()));if(!f.open(QIODevice::WriteOnly))return 2;f.write(QJsonDocument(result).toJson());std::cout<<checks.size()<<" checks; "<<(error.isEmpty()?"PASS":error.toStdString())<<"\n";return error.isEmpty()?0:1;}
