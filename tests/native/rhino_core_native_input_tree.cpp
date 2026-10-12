// SPDX-License-Identifier: LGPL-2.1-or-later
// Actual converter entry hooks are enabled only in ThreeDmInputTreeFixture.
#include "ThreeDmGeometry.h"
#include "ThreeDmArchive.h"
#include "ThreeDmExplode.h"
#include "CurveConversionContract.h"
#include "NurbsBasisAdapter.h"
#include "NativeInputTree.h"
#include "ThreeDmPointCloud.h"
#include <Geom_BezierCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColgp_Array1OfPnt2d.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <QJsonArray>
#include <QJsonObject>
#include <QJsonDocument>
#include <QByteArray>
#include <QCryptographicHash>
#include <fstream>
#include <filesystem>
#include <iostream>
#include <map>
#include <limits>
#include <functional>
#include <chrono>
#include <vector>
using namespace OpenMatrix9Gui::ThreeDm;
class UnverifiedNurbsOwner : public ON_NurbsCurve {ON_OBJECT_DECLARE(UnverifiedNurbsOwner);};
ON_OBJECT_IMPLEMENT(UnverifiedNurbsOwner,ON_NurbsCurve,"ec83398c-ec93-4070-905a-c7c7575e8f77");
namespace {
std::map<std::string,int> entries;
QJsonArray cases;int failures=0;
void check(const char* clause,bool passed){cases.append(QJsonObject{{"clause",clause},{"passed",passed}});if(!passed){++failures;std::cerr<<clause<<'\n';}}
template<class T> struct Restore {T& field;T original;explicit Restore(T& value):field(value),original(value){}~Restore(){field=original;}};
struct AdjacencyRestore {
    ON_SimpleArray<int>& values;ON_SimpleArray<int> original;
    explicit AdjacencyRestore(ON_SimpleArray<int>& input):values(input),original(std::move(input)){}
    ~AdjacencyRestore(){values=std::move(original);}
};
struct DomainProbe : ON_CurveProxy {mutable int domainReads=0;ON_Interval Domain() const override{++domainReads;return ON_CurveProxy::Domain();}};
ON_NurbsCurve line(int dim=3){ON_NurbsCurve c(dim,true,2,2);c.SetCV(0,ON_4dPoint(2,0,0,1));c.SetCV(1,ON_4dPoint(2,0,1,1));c.SetKnot(0,0);c.SetKnot(1,1);return c;}
ON_NurbsSurface plane(){ON_NurbsSurface s(3,true,2,2,2,2);for(int u=0;u<2;++u)for(int v=0;v<2;++v)s.SetCV(u,v,ON_4dPoint(u,v,0,1));for(int d=0;d<2;++d){s.SetKnot(d,0,0);s.SetKnot(d,1,1);}return s;}
QByteArray bytes(const ON_NurbsCurve& c){return QByteArray(reinterpret_cast<const char*>(c.m_cv),c.CVCapacity()*sizeof(double))+QByteArray(reinterpret_cast<const char*>(c.m_knot),c.KnotCapacity()*sizeof(double));}
QByteArray bytes(const ON_NurbsSurface& s){return QByteArray(reinterpret_cast<const char*>(s.m_cv),s.m_cv_capacity*sizeof(double))+QByteArray(reinterpret_cast<const char*>(s.m_knot[0]),s.m_knot_capacity[0]*sizeof(double))+QByteArray(reinterpret_cast<const char*>(s.m_knot[1]),s.m_knot_capacity[1]*sizeof(double));}
QByteArray bytes(const ON_SimpleArray<int>& a){const int count=a.Count(),capacity=a.Capacity();return QByteArray(reinterpret_cast<const char*>(&count),sizeof(count))+QByteArray(reinterpret_cast<const char*>(&capacity),sizeof(capacity))+(count>0?QByteArray(reinterpret_cast<const char*>(a.Array()),count*sizeof(int)):QByteArray());}
template<class Source,class Action> void rejected(const char* name,const Source& source,Action action){
    const auto before=bytes(source);entries.clear();bool refused=false;std::string reason;
    try{action();}catch(const ExchangeError& error){refused=true;reason=error.what();}catch(const Standard_Failure& error){reason=std::string("unexpected native exception: ")+error.GetMessageString();}
    QJsonObject trace;int total=0;for(const auto& [entry,count]:entries){trace[QString::fromStdString(entry)]=count;total+=count;}
    check(name,refused&&total==0&&bytes(source)==before);
    auto record=cases.last().toObject();record["refused"]=refused;record["reason"]=QString::fromStdString(reason);record["output_entry_count"]=total;record["entries"]=trace;
    record["source_bytes_equal"]=bytes(source)==before;record["source_sha256"]=QString(QCryptographicHash::hash(before,QCryptographicHash::Sha256).toHex());cases[cases.size()-1]=record;
}
void invalidOwners(){
    auto c=line();c.m_cv[0]=1e151;
    rejected("direct curve rejects before conversion",c,[&]{curve3d(c);});
    ON_CurveProxy proxy;proxy.SetProxyCurve(&c);
    rejected("CurveProxy hidden basis rejects before conversion",c,[&]{curve3d(proxy);});
    DomainProbe probe;probe.SetProxyCurve(&c);probe.domainReads=0;
    rejected("plain import rejects before source Domain argument",c,[&]{importCurve(probe,1e-7);});check("plain import source domain not read on invalid basis",probe.domainReads==0);
    const double t=.5;
    rejected("observed proxy rejects before CRC/domain conversion",c,[&]{CurveConversion::importCurveObserved(proxy,{},std::span<const double>(&t,1));});
    for(double value:{std::numeric_limits<double>::quiet_NaN(),std::numeric_limits<double>::infinity(),-1.0,0.0}){Restore weight(c.m_cv[c.m_dim]);c.m_cv[0]=2;c.m_cv[c.m_dim]=value;rejected("proxy rational weight rejected before GetNurbForm",c,[&]{curve3d(proxy);});}c.m_cv[0]=1e151;
    {Restore count(c.m_cv_count);c.m_cv_count=std::numeric_limits<int>::max();entries.clear();bool refused=false;try{curve3d(proxy);}catch(const ExchangeError&){refused=true;}check("proxy enormous CV count rejects before buffer access",refused&&entries.empty());}
    ON_PolyCurve nested;auto* child=new ON_NurbsCurve(line());nested.Append(child);child->m_cv[0]=1e151;
    rejected("PolyCurve hidden basis rejects before conversion",*child,[&]{curve3d(nested);});
    ON_PolyCurve forest;auto* first=new ON_NurbsCurve(line());auto* last=new ON_NurbsCurve(line());last->SetCV(0,ON_4dPoint(2,0,1,1));last->SetCV(1,ON_4dPoint(2,0,2,1));check("multi-child forest setup",forest.Append(first)&&forest.Append(last));last->m_cv[0]=1e151;rejected("whole PolyCurve forest rejects before first output",*last,[&]{curve3d(forest);});
    auto s=plane();s.m_cv[0]=1e151;
    rejected("direct surface rejects before conversion",s,[&]{importSurface(s);});
    ON_SurfaceProxy sp;sp.SetProxySurface(&s);
    rejected("SurfaceProxy hidden basis rejects before conversion",s,[&]{importSurface(sp);});
    {Restore u(s.m_cv_count[0]);Restore v(s.m_cv_count[1]);s.m_cv_count[0]=s.m_cv_count[1]=std::numeric_limits<int>::max();entries.clear();bool refused=false;try{importSurface(sp);}catch(const ExchangeError&){refused=true;}check("proxy oversized surface grid rejects before buffer access",refused&&entries.empty());}
    ON_RevSurface rev;auto* profile=new ON_NurbsCurve(line());auto* profileProxy=new ON_CurveProxy;profileProxy->SetProxyCurve(profile);rev.m_curve=profileProxy;
    rev.m_axis=ON_Line(ON_3dPoint(0,0,0),ON_3dPoint(0,0,1));rev.m_angle=rev.m_t=ON_Interval(0,2*ON_PI);profile->m_cv[0]=1e151;
    rejected("revolution proxy profile rejects before conversion",*profile,[&]{importSurface(rev);});
    profileProxy->SetProxyCurve(nullptr);delete profile;
    ON_SumSurface sum;sum.m_curve[0]=new ON_NurbsCurve(line());sum.m_curve[1]=new ON_NurbsCurve(line());auto* sumChild=ON_NurbsCurve::Cast(sum.m_curve[1]);sumChild->m_cv[0]=1e151;
    rejected("SumSurface hidden second curve rejects before conversion",*sumChild,[&]{importSurface(sum);});
    ON_Extrusion extrusion;extrusion.m_profile_count=1;extrusion.m_profile=new ON_NurbsCurve(line(2));auto* ep=ON_NurbsCurve::Cast(extrusion.m_profile);ep->m_cv[0]=1e151;
    rejected("Extrusion hidden profile rejects before conversion",*ep,[&]{importSurface(extrusion);});
    ON_CurveOnSurface cos(new ON_NurbsCurve(line(2)),new ON_NurbsCurve(line()),new ON_NurbsSurface(plane()));auto* cs=ON_NurbsSurface::Cast(cos.m_s);cs->m_cv[0]=1e151;
    rejected("CurveOnSurface hidden support rejects before conversion",*cs,[&]{curve3d(cos);});
    ON_Brep b;auto* owned=new ON_NurbsSurface(plane());check("BRep fixture setup",b.Create(owned)&&b.IsValid());auto* support=ON_NurbsSurface::Cast(b.m_S[0]);
    {Restore field(support->m_cv[0]);support->m_cv[0]=1e151;rejected("BRep surface rejects before whole source copy",*support,[&]{importBrep(b,1e-7);});rejected("BRep orientation rejects before whole source copy",*support,[&]{resolvedBrepOrientation(b,1e-7);});}
    for(int dim: {2,3}){auto* input=new ON_NurbsCurve(line(dim));(dim==2?b.m_C2:b.m_C3).Append(input);input->m_cv[0]=1e151;rejected(dim==2?"BRep unused C2 rejects before source copy":"BRep unused C3 rejects before source copy",*input,[&]{importBrep(b,1e-7);});input->m_cv[0]=2;}
    auto oc=Handle(Geom_BSplineCurve)::DownCast(curve3d(line()));Handle(Geom_TrimmedCurve) trim=new Geom_TrimmedCurve(oc,0,1);auto hidden=Handle(Geom_BSplineCurve)::DownCast(trim->BasisCurve());hidden->SetPole(1,gp_Pnt(1e151,0,0));
    entries.clear();bool refused=false;try{curveON(trim,0,1);}catch(const ExchangeError&){refused=true;}check("OCCT trimmed curve rejects before copy/GeomConvert",refused&&entries.empty());
    auto os=Handle(Geom_BSplineSurface)::DownCast(importSurface(plane()));Handle(Geom_RectangularTrimmedSurface) st=new Geom_RectangularTrimmedSurface(os,0.,1.,0.,1.);auto hs=Handle(Geom_BSplineSurface)::DownCast(st->BasisSurface());hs->SetPole(1,1,gp_Pnt(1e151,0,0));
    entries.clear();refused=false;try{exportSurface(st);}catch(const ExchangeError&){refused=true;}check("OCCT trimmed surface rejects before GeomConvert",refused&&entries.empty());
}
void positiveOwners(){
    ON_NurbsCurve circle;ON_Circle(ON_Plane::World_xy,17).GetNurbForm(circle);const auto before=bytes(circle);ON_CurveProxy proxy;proxy.SetProxyCurve(&circle,ON_Interval(0,ON_PI));proxy.Reverse();
    entries.clear();auto target=curve3d(proxy);check("positive rational reversed proxy reaches real converter",!target.IsNull()&&!entries.empty());double maxError=0;for(int i=0;i<=32;++i){const double t=proxy.Domain().ParameterAt(i/32.);const auto a=proxy.PointAt(t);maxError=std::max(maxError,target->Value(t).Distance(gp_Pnt(a.x,a.y,a.z)));}check("proxy reversed subdomain locus unchanged",maxError<=1e-7&&bytes(circle)==before);
    const ON_3dPoint poles[]={ON_3dPoint(0,0,0),ON_3dPoint(1,2,0),ON_3dPoint(2,0,0)};ON_NurbsCurve periodic;periodic.CreatePeriodicUniformNurbs(3,3,3,poles);const auto pb=bytes(periodic);ON_CurveProxy pp;pp.SetProxyCurve(&periodic);check("periodic proxy reuses working conversion",!curve3d(pp).IsNull()&&periodic.IsPeriodic()&&bytes(periodic)==pb);
    auto s=plane();const auto sb=bytes(s);ON_SurfaceProxy sp;sp.SetProxySurface(&s);check("rational SurfaceProxy remains supported",!importSurface(sp).IsNull()&&bytes(s)==sb);
    ON_Brep b;auto* owned=new ON_NurbsSurface(plane());check("positive trimmed BRep setup",b.Create(owned)&&b.IsValid());const auto crc=b.DataCRC(0);entries.clear();auto shape=importBrep(b,1e-7);check("positive BRep retains source and reaches actual copy",!shape.IsNull()&&b.DataCRC(0)==crc&&entries["brep.Copy"]==1);
}
void vertexAdjacency(){
    ON_Brep b;auto* owned=new ON_NurbsSurface(plane());check("vertex adjacency fixture setup",b.Create(owned)&&b.IsValid()&&b.m_V.Count()>0);
    auto& adjacency=b.m_V[0].m_ei;
    {
        AdjacencyRestore restore(adjacency);
        // Public SDK expert API; actual allocation holds both initialized ints.
        // Only the declared capacity is inconsistent, so the RED copy is readable.
        auto* storage=static_cast<int*>(onmalloc(2*sizeof(int)));storage[0]=storage[1]=0;
        adjacency.SetArray(storage,2,1);
        rejected("vertex adjacency malformed envelope rejects before BRep copy",adjacency,[&]{importBrep(b,1e-7);});
        rejected("vertex adjacency malformed envelope rejects before orientation copy",adjacency,[&]{resolvedBrepOrientation(b,1e-7);});
    }
    {
        AdjacencyRestore restore(adjacency);
        constexpr int count=16777217; // 64 MiB plus one live initialized int.
        adjacency.Reserve(count);adjacency.SetCount(count);std::fill_n(adjacency.Array(),count,0);
        rejected("oversized live vertex adjacency rejects before BRep copy",adjacency,[&]{importBrep(b,1e-7);});
        rejected("oversized live vertex adjacency rejects before orientation copy",adjacency,[&]{resolvedBrepOrientation(b,1e-7);});
    }
    check("vertex adjacency restored source remains valid",b.IsValid());
}
void brepTopologyReferences(){
    ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(1,0,0),ON_3dPoint(1,1,0),ON_3dPoint(0,1,0),ON_3dPoint(0,0,1),ON_3dPoint(1,0,1),ON_3dPoint(1,1,1),ON_3dPoint(0,1,1)};
    std::unique_ptr<ON_Brep> b(ON_BrepBox(corners));
    check("BRep cross-index fixture setup",b&&b->IsValid()&&b->m_E.Count()>0&&b->m_E[0].m_ti.Count()==2);
    if(!b||b->m_E.Count()==0||b->m_E[0].m_ti.Count()!=2)return;
    auto& otherTrim=b->m_E[0].m_ti[1];Restore<int> restore(otherTrim);otherTrim=b->m_T.Count();
    entries.clear();bool preflightRejected=false;
    try{(void)InputTree::validate(*b);}catch(const ExchangeError&){preflightRejected=true;}
    check("BRep out-of-range edge trim reference rejects in preflight",preflightRejected);
    if(preflightRejected){
        entries.clear();bool refused=false;
        try{(void)resolvedBrepOrientation(*b,1e-7);}catch(const ExchangeError&){refused=true;}
        check("BRep invalid trim reference blocks orientation copy",refused&&entries.empty());
    }
    ON_3dPoint faceCorners[8]={ON_3dPoint(0,0,0),ON_3dPoint(1,0,0),ON_3dPoint(1,1,0),ON_3dPoint(0,1,0),ON_3dPoint(0,0,1),ON_3dPoint(1,0,1),ON_3dPoint(1,1,1),ON_3dPoint(0,1,1)};
    std::unique_ptr<ON_Brep> badFace(ON_BrepBox(faceCorners));
    check("BRep invalid face-loop fixture setup",badFace&&badFace->m_F.Count()>0&&badFace->m_F[0].m_li.Count()>0);
    if(!badFace||badFace->m_F.Count()==0||badFace->m_F[0].m_li.Count()==0)return;
    auto& loopIndex=badFace->m_F[0].m_li[0];Restore<int> restoreLoop(loopIndex);loopIndex=badFace->m_L.Count();
    entries.clear();bool refused=false;
    try{(void)resolvedBrepOrientation(*badFace,1e-7);}catch(const ExchangeError&){refused=true;}
    check("BRep copy topology validation rejects invalid face loop before orientation query",refused&&entries["brep.orientation.Copy"]==1&&entries["brep.orientation.Query"]==0);
}
void deletedTopology(){
    ON_Brep b;auto* owned=new ON_NurbsSurface(plane());check("deleted topology fixture setup",b.Create(owned)&&b.IsValid());
    const int edgeCount=b.m_E.Count(),trimCount=b.m_T.Count();
    b.DeleteEdge(b.NewEdge(0),false);b.DeleteTrim(b.NewTrim(0),false);
    ON_wString validityText;ON_TextLog validity(validityText);const bool valid=b.IsValid(&validity);
    ON_Brep legacyWorkingCopy(b);ON_wString copyText;ON_TextLog copyLog(copyText);
    check("non-compacted deleted edge and trim storage retained",b.m_E.Count()==edgeCount+1&&b.m_T.Count()==trimCount+1&&b.m_E[edgeCount].m_edge_index==-1&&b.m_T[trimCount].m_trim_index==-1);
    // DeleteEdge/DeleteTrim clear both the proxy and component back pointer.
    // In this pinned SDK, IsCorrupt rejects those retained slots on back-pointer
    // consistency before IsValid reaches its later deleted-index checks. The
    // existing importer copied and ran the same validation, so it did not accept
    // this state; skipping the tombstone here would only defer the same failure.
    check("pinned SDK rejects DeleteEdge/DeleteTrim tombstones at IsCorrupt",!valid&&!legacyWorkingCopy.IsValid(&copyLog));
    auto sdkRecord=cases.last().toObject();sdkRecord["source_validity"]=QString::fromWCharArray(validityText.Array());sdkRecord["legacy_copy_validity"]=QString::fromWCharArray(copyText.Array());cases[cases.size()-1]=sdkRecord;
    rejected("unsupported deleted topology rejects before BRep copy",*ON_NurbsSurface::Cast(b.m_S[0]),[&]{importBrep(b,1e-7);});
    rejected("unsupported deleted topology rejects before orientation copy",*ON_NurbsSurface::Cast(b.m_S[0]),[&]{resolvedBrepOrientation(b,1e-7);});
    b.Compact();check("SDK-compacted topology is valid",b.IsValid()&&b.m_E.Count()==edgeCount&&b.m_T.Count()==trimCount);
    const auto crc=b.DataCRC(0);entries.clear();bool imported=false;std::string reason;
    try{auto shape=importBrep(b,1e-7);GProp_GProps area;BRepGProp::SurfaceProperties(shape,area);imported=!shape.IsNull()&&std::abs(area.Mass()-1)<1e-7&&entries["brep.Copy"]==1;}catch(const ExchangeError& e){reason=e.what();}
    check("compacted topology imports the unchanged unit-area face",imported&&b.DataCRC(0)==crc&&b.m_E.Count()==edgeCount&&b.m_T.Count()==trimCount);
    auto record=cases.last().toObject();record["reason"]=QString::fromStdString(reason);cases[cases.size()-1]=record;
    entries.clear();bool oriented=false;try{oriented=resolvedBrepOrientation(b,1e-7)==0&&entries["brep.orientation.Copy"]==1;}catch(const ExchangeError&){}
    check("compacted topology keeps open BRep orientation",oriented&&b.DataCRC(0)==crc);
    ON_Brep activeEdge;owned=new ON_NurbsSurface(plane());activeEdge.Create(owned);activeEdge.NewEdge(-1);
    rejected("active null edge proxy still rejects before whole source copy",*ON_NurbsSurface::Cast(activeEdge.m_S[0]),[&]{importBrep(activeEdge,1e-7);});
    ON_Brep activeTrim;owned=new ON_NurbsSurface(plane());activeTrim.Create(owned);activeTrim.NewTrim(-1);
    rejected("active null trim proxy still rejects before orientation copy",*ON_NurbsSurface::Cast(activeTrim.m_S[0]),[&]{resolvedBrepOrientation(activeTrim,1e-7);});
}
void archiveStaging(){
    const auto root=std::filesystem::temp_directory_path()/"om9-native-input-tree-archive";
    std::filesystem::create_directories(root);
    ON_3dmObjectAttributes attributes;
    ONX_Model block;block.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    auto invalidCurve=line();invalidCurve.m_cv[0]=1e151;
    auto memberAttributes=attributes;memberAttributes.SetMode(ON::idef_object);
    auto member=block.AddModelGeometryComponent(&invalidCurve,&memberAttributes);
    ON_InstanceDefinition definition;definition.SetName(L"Invalid transformed curve");definition.AddInstanceGeometryId(member.ModelComponent()->Id());
    auto definitionRef=block.AddModelComponent(definition);
    ON_InstanceRef instance;instance.m_instance_definition_uuid=definitionRef.ModelComponent()->Id();instance.m_xform=ON_Xform::TranslationTransformation(ON_3dVector(5,0,0));
    block.AddModelGeometryComponent(&instance,&attributes);
    const auto blockPath=root/"invalid-transformed-member.3dm";
    const bool blockWritten=block.Write(blockPath.c_str(),5,nullptr);
    entries.clear();bool blockRejected=false;std::string blockReason;
    try{(void)readArchive(blockPath);}catch(const ExchangeError& error){blockRejected=true;blockReason=error.what();}
    check("archive transformed block member rejects at native input gate before Duplicate",blockWritten&&blockRejected&&blockReason.find("Native NURBS basis rejected")!=std::string::npos&&entries["archive.geometry.Duplicate"]==0&&entries["curve.GetNurbForm"]==0);
    auto record=cases.last().toObject();record["reason"]=QString::fromStdString(blockReason);record["entries"]=QJsonObject{{"archive.geometry.Duplicate",entries["archive.geometry.Duplicate"]},{"curve.GetNurbForm",entries["curve.GetNurbForm"]}};cases[cases.size()-1]=record;

    ONX_Model surfaceModel;surfaceModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    auto invalidSurface=plane();invalidSurface.m_cv[0]=1e151;surfaceModel.AddModelGeometryComponent(&invalidSurface,&attributes);
    const auto surfacePath=root/"invalid-surface-before-brepform.3dm";
    const bool surfaceWritten=surfaceModel.Write(surfacePath.c_str(),5,nullptr);
    entries.clear();bool surfaceRejected=false;std::string surfaceReason;
    try{(void)readArchive(surfacePath);}catch(const ExchangeError& error){surfaceRejected=true;surfaceReason=error.what();}
    check("archive surface rejects at native input gate before BrepForm",surfaceWritten&&surfaceRejected&&surfaceReason.find("Native NURBS basis rejected")!=std::string::npos&&entries["archive.surface.BrepForm"]==0);
    record=cases.last().toObject();record["reason"]=QString::fromStdString(surfaceReason);record["entries"]=QJsonObject{{"archive.surface.BrepForm",entries["archive.surface.BrepForm"]}};cases[cases.size()-1]=record;

    ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(1,0,0),ON_3dPoint(1,1,0),ON_3dPoint(0,1,0),ON_3dPoint(0,0,1),ON_3dPoint(1,0,1),ON_3dPoint(1,1,1),ON_3dPoint(0,1,1)};
    std::unique_ptr<ON_Brep> invalidBrep(ON_BrepBox(corners));
    auto* support=invalidBrep&&invalidBrep->m_S.Count()>0?ON_NurbsSurface::Cast(invalidBrep->m_S[0]):nullptr;
    if(support)support->m_cv[0]=1e151;
    ONX_Model signedModel;signedModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    auto signedMemberAttributes=attributes;signedMemberAttributes.SetMode(ON::idef_object);
    ON_UUID signedMemberId=ON_nil_uuid;
    if(support){auto signedMember=signedModel.AddModelGeometryComponent(invalidBrep.get(),&signedMemberAttributes);if(!signedMember.IsEmpty())signedMemberId=signedMember.ModelComponent()->Id();}
    ON_InstanceDefinition signedDefinition;signedDefinition.SetName(L"Invalid signed placement");
    if(signedMemberId!=ON_nil_uuid)signedDefinition.AddInstanceGeometryId(signedMemberId);
    auto signedDefinitionRef=signedModel.AddModelComponent(signedDefinition);
    ON_InstanceRef signedInstance;signedInstance.m_instance_definition_uuid=signedDefinitionRef.ModelComponent()->Id();
    auto signedAttributes=attributes;signedAttributes.SetUserString(L"OpenMatrix9.InwardSolidInstance",L"1");
    signedModel.AddModelGeometryComponent(&signedInstance,&signedAttributes);
    const auto signedPath=root/"invalid-signed-placement-brep.3dm";
    const bool signedWritten=invalidBrep!=nullptr&&support&&signedMemberId!=ON_nil_uuid&&signedModel.Write(signedPath.c_str(),5,nullptr);
    entries.clear();bool signedRejected=false;std::string signedReason;
    try{(void)readArchive(signedPath);}catch(const ExchangeError& error){signedRejected=true;signedReason=error.what();}
    check("archive signed placement BRep rejects at native input gate before orientation",signedWritten&&signedRejected&&signedReason.find("Native NURBS basis rejected")!=std::string::npos&&entries["archive.brep.SignedPlacementOrientation"]==0);
    record=cases.last().toObject();record["reason"]=QString::fromStdString(signedReason);record["entries"]=QJsonObject{{"archive.brep.SignedPlacementOrientation",entries["archive.brep.SignedPlacementOrientation"]}};cases[cases.size()-1]=record;

    ONX_Model reuseModel;reuseModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    ON_Mesh reusableMesh;reusableMesh.m_V.Append(ON_3fPoint(0,0,0));reusableMesh.m_V.Append(ON_3fPoint(1,0,0));reusableMesh.m_V.Append(ON_3fPoint(0,1,0));reusableMesh.m_F.Append(ON_MeshFace{{0,1,2,2}});
    reuseModel.AddModelGeometryComponent(&reusableMesh,&attributes);ON_PointCloud reusableCloud;reusableCloud.m_P.Append(ON_3dPoint(2,3,4));reuseModel.AddModelGeometryComponent(&reusableCloud,&attributes);
    const auto reusePath=root/"mesh-cloud-existing-adapters.3dm";const bool reuseWritten=reuseModel.Write(reusePath.c_str(),5,nullptr);
    entries.clear();std::size_t reusedCount=0;try{reusedCount=readArchive(reusePath).items.size();}catch(const ExchangeError&){}
    check("archive retains existing Mesh and PointCloud adapters after preflight",reuseWritten&&reusedCount==2&&entries["mesh.IsValid"]==1&&entries["archive.pointcloud.Copy"]==1);
}
void explodeStaging(){
    const auto root=std::filesystem::temp_directory_path()/("om9-native-input-tree-explode-"+std::to_string(std::chrono::steady_clock::now().time_since_epoch().count()));
    std::filesystem::create_directories(root);
    ON_3dmObjectAttributes attributes;
    ONX_Model block;block.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    auto invalidCurve=line();invalidCurve.m_cv[0]=1e151;
    auto memberAttributes=attributes;memberAttributes.SetMode(ON::idef_object);
    auto member=block.AddModelGeometryComponent(&invalidCurve,&memberAttributes);
    ON_InstanceDefinition definition;definition.SetName(L"Invalid Explode curve");definition.AddInstanceGeometryId(member.ModelComponent()->Id());
    auto definitionRef=block.AddModelComponent(definition);
    ON_InstanceRef instance;instance.m_instance_definition_uuid=definitionRef.ModelComponent()->Id();instance.m_xform=ON_Xform::IdentityTransformation;
    auto rootRef=block.AddModelGeometryComponent(&instance,&attributes);
    const auto blockPath=root/"invalid-exploded-member.3dm";
    const bool blockWritten=block.Write(blockPath.c_str(),5,nullptr);
    const auto blockId=[](const ON_UUID& value){char text[37]{};ON_UuidToString(value,text);return std::string(text);}(rootRef.ModelComponent()->Id());
    entries.clear();bool blockRejected=false;std::string blockReason;
    try{(void)explodeArchiveRecord(blockPath,blockId);}catch(const ExchangeError& error){blockRejected=true;blockReason=error.what();}
    check("Explode block member rejects before Duplicate",blockWritten&&blockRejected&&blockReason.find("Native NURBS basis rejected")!=std::string::npos&&entries["explode.geometry.Duplicate"]==0&&entries["curve.GetNurbForm"]==0);
    auto record=cases.last().toObject();record["reason"]=QString::fromStdString(blockReason);record["entries"]=QJsonObject{{"explode.geometry.Duplicate",entries["explode.geometry.Duplicate"]},{"curve.GetNurbForm",entries["curve.GetNurbForm"]}};cases[cases.size()-1]=record;

    ONX_Model cageModel;cageModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    ON_NurbsCage invalidCage(ON_BoundingBox(ON_3dPoint(0,0,0),ON_3dPoint(1,1,1)),2,2,2,2,2,2);
    invalidCage.m_cv[0]=std::numeric_limits<double>::quiet_NaN();
    auto cageRef=cageModel.AddModelGeometryComponent(&invalidCage,&attributes);
    const auto cagePath=root/"invalid-exploded-cage.3dm";
    const bool cageWritten=cageModel.Write(cagePath.c_str(),5,nullptr);
    const auto cageId=[](const ON_UUID& value){char text[37]{};ON_UuidToString(value,text);return std::string(text);}(cageRef.ModelComponent()->Id());
    entries.clear();bool cageRejected=false;std::string cageReason;
    try{(void)explodeArchiveRecord(cagePath,cageId);}catch(const ExchangeError& error){cageRejected=true;cageReason=error.what();}
    check("Explode cage rejects before IsValid and boundary evaluation",cageWritten&&cageRejected&&cageReason.find("Invalid native NURBS cage")!=std::string::npos&&entries["explode.cage.IsValid"]==0&&entries["explode.cage.IsoSurface"]==0);
    record=cases.last().toObject();record["reason"]=QString::fromStdString(cageReason);record["entries"]=QJsonObject{{"explode.cage.IsValid",entries["explode.cage.IsValid"]},{"explode.cage.IsoSurface",entries["explode.cage.IsoSurface"]}};cases[cases.size()-1]=record;

    ONX_Model extremeModel;extremeModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    ON_NurbsCage extremeCage(ON_BoundingBox(ON_3dPoint(0,0,0),ON_3dPoint(1,1,1)),2,2,2,2,2,2);
    extremeCage.m_knot[0][0]=-1e308;extremeCage.m_knot[0][1]=1e308;
    auto extremeRef=extremeModel.AddModelGeometryComponent(&extremeCage,&attributes);
    const auto extremePath=root/"overflowing-finite-cage-domain.3dm";
    const bool extremeWritten=extremeModel.Write(extremePath.c_str(),5,nullptr);
    const auto extremeId=[](const ON_UUID& value){char text[37]{};ON_UuidToString(value,text);return std::string(text);}(extremeRef.ModelComponent()->Id());
    entries.clear();bool extremeRejected=false;std::string extremeReason;
    try{(void)explodeArchiveRecord(extremePath,extremeId);}catch(const ExchangeError& error){extremeRejected=true;extremeReason=error.what();}
    check("Explode cage rejects finite knot overflow before evaluation",extremeWritten&&extremeRejected&&extremeReason.find("Invalid native NURBS cage")!=std::string::npos&&entries["explode.cage.IsValid"]==0&&entries["explode.cage.IsoSurface"]==0);
    record=cases.last().toObject();record["reason"]=QString::fromStdString(extremeReason);record["entries"]=QJsonObject{{"explode.cage.IsValid",entries["explode.cage.IsValid"]},{"explode.cage.IsoSurface",entries["explode.cage.IsoSurface"]}};cases[cases.size()-1]=record;

    ONX_Model invalidMeshModel;invalidMeshModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    ON_Mesh invalidMesh;invalidMesh.m_V.Append(ON_3fPoint(0,0,0));invalidMesh.m_V.Append(ON_3fPoint(1,0,0));invalidMesh.m_V.Append(ON_3fPoint(0,1,0));invalidMesh.m_F.Append(ON_MeshFace{{0,1,2,2}});invalidMesh.m_F[0].vi[0]=std::numeric_limits<int>::max();
    auto invalidMeshAttributes=attributes;invalidMeshAttributes.SetMode(ON::idef_object);auto invalidMeshRef=invalidMeshModel.AddModelGeometryComponent(&invalidMesh,&invalidMeshAttributes);
    ON_InstanceDefinition invalidMeshDefinition;invalidMeshDefinition.SetName(L"Invalid mesh block");invalidMeshDefinition.AddInstanceGeometryId(invalidMeshRef.ModelComponent()->Id());auto invalidMeshDefinitionRef=invalidMeshModel.AddModelComponent(invalidMeshDefinition);
    ON_InstanceRef invalidMeshInstance;invalidMeshInstance.m_instance_definition_uuid=invalidMeshDefinitionRef.ModelComponent()->Id();auto invalidMeshRoot=invalidMeshModel.AddModelGeometryComponent(&invalidMeshInstance,&attributes);
    const auto invalidMeshPath=root/"invalid-exploded-mesh.3dm";const bool invalidMeshWritten=invalidMeshModel.Write(invalidMeshPath.c_str(),5,nullptr);
    const auto invalidMeshId=[](const ON_UUID& value){char text[37]{};ON_UuidToString(value,text);return std::string(text);}(invalidMeshRoot.ModelComponent()->Id());
    entries.clear();bool invalidMeshRejected=false;std::string invalidMeshReason;try{(void)explodeArchiveRecord(invalidMeshPath,invalidMeshId);}catch(const ExchangeError& error){invalidMeshRejected=true;invalidMeshReason=error.what();}
    check("Explode malformed mesh rejects before Duplicate or SDK IsValid",invalidMeshWritten&&invalidMeshRejected&&invalidMeshReason.find("Invalid native mesh face index")!=std::string::npos&&entries["explode.geometry.Duplicate"]==0&&entries["mesh.IsValid"]==0);
    record=cases.last().toObject();record["reason"]=QString::fromStdString(invalidMeshReason);record["entries"]=QJsonObject{{"explode.geometry.Duplicate",entries["explode.geometry.Duplicate"]},{"mesh.IsValid",entries["mesh.IsValid"]}};cases[cases.size()-1]=record;

    ONX_Model cloudModel;cloudModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_PointCloud pointCloud;pointCloud.m_P.Append(ON_3dPoint(1,2,3));auto cloudAttributes=attributes;cloudAttributes.SetMode(ON::idef_object);auto cloudRef=cloudModel.AddModelGeometryComponent(&pointCloud,&cloudAttributes);
    ON_InstanceDefinition cloudDefinition;cloudDefinition.SetName(L"PointCloud block");cloudDefinition.AddInstanceGeometryId(cloudRef.ModelComponent()->Id());auto cloudDefinitionRef=cloudModel.AddModelComponent(cloudDefinition);ON_InstanceRef cloudInstance;cloudInstance.m_instance_definition_uuid=cloudDefinitionRef.ModelComponent()->Id();auto cloudRoot=cloudModel.AddModelGeometryComponent(&cloudInstance,&attributes);
    const auto cloudPath=root/"unsupported-exploded-cloud.3dm";const bool cloudWritten=cloudModel.Write(cloudPath.c_str(),5,nullptr);
    const auto cloudId=[](const ON_UUID& value){char text[37]{};ON_UuidToString(value,text);return std::string(text);}(cloudRoot.ModelComponent()->Id());
    entries.clear();bool cloudRejected=false;std::string cloudReason;try{(void)explodeArchiveRecord(cloudPath,cloudId);}catch(const GeometryRepresentationUnavailable& error){cloudRejected=true;cloudReason=error.what();}
    check("Explode unsupported PointCloud rejects before Duplicate",cloudWritten&&cloudRejected&&cloudReason.find("PointCloud Explode")!=std::string::npos&&entries["explode.geometry.Duplicate"]==0);
}
void traversalPolicy(){
    UnverifiedNurbsOwner custom;static_cast<ON_NurbsCurve&>(custom)=line();entries.clear();bool unsupported=false;try{curve3d(custom);}catch(const ExchangeError& e){unsupported=std::string(e.what()).find("Unverified native curve owner")!=std::string::npos;}check("unknown registered derived owner cannot bypass extraction whitelist",unsupported&&entries.empty());
    auto leaf=line();ON_CurveProxy proxy;proxy.SetProxyCurve(&leaf);
    const auto result=InputTree::validate(proxy);check("borrowed traversal visits proxy and its original basis",result.nodes==2&&result.bytes>=96);
    ON_CurveOnSurface root(new ON_NurbsCurve(line(2)),new ON_NurbsCurve(line()),new ON_SumSurface);
    auto* sum=ON_SumSurface::Cast(root.m_s);sum->m_curve[0]=new ON_NurbsCurve(line());sum->m_curve[1]=new ON_NurbsCurve(line());
    bool refused=false;{Restore original(sum->m_curve[0]);sum->m_curve[0]=&root;try{InputTree::validate(root);}catch(const ExchangeError&){refused=true;}}
    check("native curve/surface owner cycle rejects before child evaluation",refused);
    std::vector<std::unique_ptr<ON_CurveProxy>> chain;const ON_Curve* current=&leaf;
    for(int i=0;i<63;++i){auto item=std::make_unique<ON_CurveProxy>();item->SetProxyCurve(current);current=item.get();chain.push_back(std::move(item));}
    check("64-deep live proxy tree accepted",InputTree::validate(*current).nodes==64);
    auto excess=std::make_unique<ON_CurveProxy>();excess->SetProxyCurve(current);refused=false;try{InputTree::validate(*excess);}catch(const ExchangeError& e){refused=std::string(e.what()).find("status 2")!=std::string::npos;}check("65-deep live proxy tree rejects before SDK recursion",refused);
    ON_PolyCurve wide;auto& children=const_cast<ON_CurveArray&>(wide.SegmentCurves());for(int i=0;i<16384;++i)children.Append(nullptr);refused=false;try{InputTree::validate(wide);}catch(const ExchangeError& e){refused=std::string(e.what()).find("status 3")!=std::string::npos;}check("oversized child container rejects before reading child pointers",refused);
    ON_NurbsCurve large(3,true,2,800000);for(int i=0;i<large.CVCount();++i){large.SetCV(i,ON_4dPoint(i,0,0,1));large.SetKnot(i,i);}check("aggregate budget live basis setup",large.IsValid());
    ON_SumSurface shared;auto* pa=new ON_CurveProxy;auto* pb=new ON_CurveProxy;pa->SetProxyCurve(&large);pb->SetProxyCurve(&large);shared.m_curve[0]=pa;shared.m_curve[1]=pb;entries.clear();refused=false;try{importSurface(shared);}catch(const ExchangeError& e){refused=std::string(e.what()).find("status 4")!=std::string::npos;}check("repeated live basis aggregate rejects before output allocation",refused&&entries.empty());
    ON_PolylineCurve pl;pl.m_pline.Append(ON_3dPoint(0,0,0));pl.m_pline.Append(ON_3dPoint(1e151,1,0));pl.m_t.Append(0);pl.m_t.Append(1);entries.clear();refused=false;try{curve3d(pl);}catch(const ExchangeError&){refused=true;}check("polyline values reject before GetNurbForm",refused&&entries.empty());
    ON_NurbsCage cage(ON_BoundingBox(ON_3dPoint(0,0,0),ON_3dPoint(1,1,1)),2,2,2,2,2,2);
    check("valid cage passes bounded input preflight",InputTree::validate(cage).nodes==1);
    {Restore count(cage.m_cv_count[0]);cage.m_cv_count[0]=std::numeric_limits<int>::max();refused=false;try{(void)InputTree::validate(cage);}catch(const ExchangeError&){refused=true;}check("oversized cage grid rejects before CV or knot access",refused);}
    {Restore stride(cage.m_cv_stride[0]);cage.m_cv_stride[0]=std::numeric_limits<int>::max();refused=false;try{(void)InputTree::validate(cage);}catch(const ExchangeError&){refused=true;}check("overflowing cage stride rejects before CV access",refused);}
    TColgp_Array1OfPnt p(1,3);p.SetValue(1,gp_Pnt(0,0,0));p.SetValue(2,gp_Pnt(1,2,0));p.SetValue(3,gp_Pnt(2,0,0));Handle(Geom_BezierCurve) bez=new Geom_BezierCurve(p);bez->SetPole(1,gp_Pnt(1e151,0,0));entries.clear();refused=false;try{curveON(bez,0,1);}catch(const ExchangeError&){refused=true;}check("Bezier original rational basis policy precedes conversion",refused&&entries.empty());
}
void meshAndPointCloud(){
    ON_Mesh mesh;mesh.m_V.Append(ON_3fPoint(0,0,0));mesh.m_V.Append(ON_3fPoint(1,0,0));mesh.m_V.Append(ON_3fPoint(0,1,0));
    ON_MeshFace face;face.vi[0]=0;face.vi[1]=1;face.vi[2]=2;face.vi[3]=2;mesh.m_F.Append(face);
    entries.clear();bool valid=false,refused=false;try{valid=InputTree::validate(mesh).nodes==1;}catch(const ExchangeError&){}
    check("bounded mesh owner passes native preflight",valid);
    entries.clear();try{auto converted=importMesh(mesh);valid=converted.vertices.size()==3&&converted.faces.size()==1;}catch(const ExchangeError&){valid=false;}
    check("mesh positive control reaches instrumented SDK validity",valid&&entries["mesh.IsValid"]==1);
    {Restore<int> faceIndex(mesh.m_F[0].vi[0]);mesh.m_F[0].vi[0]=std::numeric_limits<int>::max();entries.clear();refused=false;
        try{(void)importMesh(mesh);}catch(const ExchangeError&){refused=true;}
        check("mesh bad face index rejects before SDK IsValid",refused&&entries["mesh.IsValid"]==0);}

    for(int i=0;i<3;++i)mesh.m_K.Append(ON_SurfaceCurvature::Nan);entries.clear();valid=false;
    try{valid=InputTree::validate(mesh).nodes==1;}catch(const ExchangeError&){}
    check("mesh accepts the documented paired NaN curvature sentinel",valid);
    {Restore<ON_SurfaceCurvature> curvature(mesh.m_K[0]);mesh.m_K[0].k2=1.0;entries.clear();refused=false;
        try{(void)importMesh(mesh);}catch(const ExchangeError&){refused=true;}
        check("mesh rejects malformed partial-NaN curvature before SDK IsValid",refused&&entries["mesh.IsValid"]==0);}
    mesh.m_K.Empty();
    for(int i=0;i<3;++i)mesh.m_K.Append(ON_SurfaceCurvature::CreateFromPrincipalCurvatures(0.25,0.5));entries.clear();valid=false;
    try{valid=InputTree::validate(mesh).nodes==1;}catch(const ExchangeError&){}
    check("mesh accepts finite set principal curvatures",valid);mesh.m_K.Empty();

    {
        ON_MeshNgonBuffer buffer;const unsigned int vertices[]{0,1,2,2};
        auto ngon=const_cast<ON_MeshNgon*>(buffer.CreateFromQuad(vertices,0));
        mesh.m_Ngon.Append(ngon);entries.clear();refused=false;std::string reason;
        try{(void)InputTree::validate(mesh);}catch(const ExchangeError& error){refused=true;reason=error.what();}
        check("mesh rejects unmanaged N-gon before reading index buffers",ngon&&ngon->Capacity()==0&&refused&&reason.find("N-gon storage capacity")!=std::string::npos);
        auto record=cases.last().toObject();record["ngon_capacity"]=ngon?static_cast<double>(ngon->Capacity()):-1.0;record["ngon_vertices"]=ngon?static_cast<double>(ngon->m_Vcount):-1.0;record["ngon_faces"]=ngon?static_cast<double>(ngon->m_Fcount):-1.0;record["reason"]=QString::fromStdString(reason);record["refused"]=refused;cases[cases.size()-1]=record;
        mesh.m_Ngon.Empty();
    }

    ON_PointCloud cloud;cloud.m_P.Append(ON_3dPoint(1,2,3));
    entries.clear();valid=false;try{valid=InputTree::validate(cloud).nodes==1;}catch(const ExchangeError&){}
    check("bounded PointCloud owner passes native preflight",valid);
    entries.clear();try{validatePointCloud(cloud);valid=true;}catch(const ExchangeError&){valid=false;}
    check("PointCloud positive control reaches instrumented value traversal",valid&&entries["cloud.values"]==1);
    auto* storage=static_cast<ON_3dPoint*>(onmalloc(2*sizeof(ON_3dPoint)));storage[0]=ON_3dPoint(1,2,3);storage[1]=ON_3dPoint(4,5,6);
    cloud.m_P.SetArray(storage,2,1);entries.clear();refused=false;
    try{validatePointCloud(cloud);}catch(const ExchangeError&){refused=true;}
    check("PointCloud malformed capacity rejects before value traversal",refused&&entries["cloud.values"]==0);

    ON_PointCloud numericCloud;numericCloud.m_P.Append(ON_3dPoint(1,2,3));numericCloud.m_N.Append(ON_3dVector(0,0,1));numericCloud.m_V.Append(4.0);
    {Restore<ON_3dPoint> point(numericCloud.m_P[0]);numericCloud.m_P[0].x=std::numeric_limits<double>::quiet_NaN();entries.clear();refused=false;
        try{validatePointCloud(numericCloud);}catch(const ExchangeError&){refused=true;}
        check("PointCloud rejects nonfinite point before value traversal",refused&&entries["cloud.values"]==0);}
    {Restore<ON_3dVector> normal(numericCloud.m_N[0]);numericCloud.m_N[0].z=std::numeric_limits<double>::infinity();entries.clear();refused=false;
        try{validatePointCloud(numericCloud);}catch(const ExchangeError&){refused=true;}
        check("PointCloud rejects nonfinite normal before value traversal",refused&&entries["cloud.values"]==0);}
    {Restore<double> intensity(numericCloud.m_V[0]);numericCloud.m_V[0]=std::numeric_limits<double>::quiet_NaN();entries.clear();refused=false;
        try{validatePointCloud(numericCloud);}catch(const ExchangeError&){refused=true;}
        check("PointCloud rejects nonfinite intensity before value traversal",refused&&entries["cloud.values"]==0);}

    ON_PointCloud inactivePlane;inactivePlane.m_P.Append(ON_3dPoint(1,2,3));
    inactivePlane.m_plane.origin.x=std::numeric_limits<double>::quiet_NaN();
    auto plane=pointCloudFields(inactivePlane)["plane"].toArray();bool canonical=plane.size()==16;
    for(const auto& value:plane)canonical=canonical&&value.isDouble()&&value.toDouble()==0.0;
    auto summaryPlane=pointCloudSummary(inactivePlane)["plane"].toArray();
    for(const auto& value:summaryPlane)canonical=canonical&&value.isDouble()&&value.toDouble()==0.0;
    check("inactive invalid PointCloud plane is omitted from fields and summary",canonical);

    const auto fieldsPath=std::filesystem::temp_directory_path()/("om9-point-cloud-overlay-"+std::to_string(std::chrono::steady_clock::now().time_since_epoch().count())+".json");
    ON_PointCloud overlay;overlay.m_P.Append(ON_3dPoint(7,8,9));
    const auto fieldsHash=writePointCloudFields(overlay,fieldsPath);
    ON_PointCloud emptySource;ON_Geometry& emptyGeometry=emptySource;entries.clear();bool applied=false;
    try{applyPointCloudFields(emptyGeometry,fieldsPath,fieldsHash);applied=emptySource.m_P.Count()==1&&emptySource.m_P[0]==ON_3dPoint(7,8,9);}catch(const ExchangeError&){}
    check("PointCloud overlay preflights and permits an empty native source",applied&&entries["pointcloud.apply.copy"]==1);

    ON_PointCloud malformedSource;auto* malformedStorage=static_cast<ON_3dPoint*>(onmalloc(2*sizeof(ON_3dPoint)));
    malformedStorage[0]=ON_3dPoint(1,2,3);malformedStorage[1]=ON_3dPoint(4,5,6);malformedSource.m_P.SetArray(malformedStorage,2,1);
    ON_Geometry& malformedGeometry=malformedSource;entries.clear();refused=false;
    try{applyPointCloudFields(malformedGeometry,fieldsPath,fieldsHash);}catch(const ExchangeError&){refused=true;}
    check("PointCloud overlay rejects malformed source storage before copy",refused&&entries["pointcloud.apply.copy"]==0);
    std::filesystem::remove(fieldsPath);
}
}
namespace OpenMatrix9Gui::ThreeDm::InputTrace {void entered(const char* name){++entries[name];}}
int main(int argc,char** argv){ON::Begin();std::string fatal;try{invalidOwners();positiveOwners();vertexAdjacency();brepTopologyReferences();deletedTopology();traversalPolicy();archiveStaging();explodeStaging();meshAndPointCloud();}catch(const Standard_Failure& error){fatal=error.GetMessageString();++failures;}catch(const std::exception& error){fatal=error.what();++failures;}
    QJsonObject report{{"ok",failures==0},{"checks",cases},{"failure_count",failures},{"fatal",QString::fromStdString(fatal)},{"instrumentation","fixture-only actual converter copy/GetNurbForm/output entries; input setup and extraction are excluded"}};
    if(argc>1){std::ofstream stream(argv[1]);stream<<QJsonDocument(report).toJson().constData();}
    std::cout<<cases.size()<<" checks; "<<failures<<" failures\n";ON::End();return failures?1:0;}
