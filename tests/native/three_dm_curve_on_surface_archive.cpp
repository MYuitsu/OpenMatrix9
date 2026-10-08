// OM9-FILE-012: exact native child archive recovery, not curve approximation.
#include "ThreeDmArchive.h"
#include "ThreeDmInventory.h"
#include "opennurbs_polyedgecurve.h"
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QDir>
#include <QProcess>
#include <QTemporaryDir>
#include <memory>
#include <filesystem>
#include <iostream>
#include <stdexcept>
static void require(bool v,const char* m){if(!v)throw std::runtime_error(m);}
static bool tag(const ON_Object* o,const wchar_t* value){ON_wString text;return o&&o->GetUserString(L"Child",text)&&text==value;}
static std::unique_ptr<ON_CurveOnSurface> fixture(int dimension,bool optional){
    auto s=new ON_NurbsSurface(dimension,false,2,2,2,2);
    for(int i=0;i<2;++i)for(int j=0;j<2;++j)s->SetCV(i,j,dimension==3?ON_3dPoint(10+i,20+j,30):ON_3dPoint(i,j,0));
    for(int i=0;i<2;++i){s->SetKnot(i,0,0);s->SetKnot(i,1,1);}s->SetUserString(L"Child",L"Surface");
    auto c2=new ON_ArcCurve(ON_Circle(ON_Plane(ON_3dPoint(.5,.5,0),ON_3dVector::ZAxis),.25),11,23);c2->ChangeDimension(2);c2->SetUserString(L"Child",L"Parameter");
    ON_Curve* c3=nullptr;
    if(optional){auto arc=new ON_ArcCurve(ON_Circle(ON_Plane(dimension==3?ON_3dPoint(10.5,20.5,30):ON_3dPoint(.5,.5,0),ON_3dVector::ZAxis),.25),11,23);arc->ChangeDimension(dimension);arc->SetUserString(L"Child",L"Approximation");c3=arc;}
    auto result=std::make_unique<ON_CurveOnSurface>(c2,c3,s);result->SetUserString(L"Root",L"Original surface curve");
    require(result->IsValid()&&result->IsClosed()&&result->Dimension()==dimension,"Valid closed fixture");return result;
}
static bool exact(const ON_CurveOnSurface& a,const ON_CurveOnSurface& b){
    if(!b.IsValid()||!b.IsClosed()||a.Dimension()!=b.Dimension()||a.Domain()!=b.Domain()||bool(a.m_c3)!=bool(b.m_c3))return false;
    if(!tag(b.m_c2,L"Parameter")||!tag(b.m_s,L"Surface")||(a.m_c3&&!tag(b.m_c3,L"Approximation")))return false;
    auto ac=ON_ArcCurve::Cast(a.m_c2),bc=ON_ArcCurve::Cast(b.m_c2);
    if(!ac||!bc||ac->m_dim!=bc->m_dim||ac->Domain()!=bc->Domain()||ac->m_arc.radius!=bc->m_arc.radius||ac->m_arc.plane.origin!=bc->m_arc.plane.origin||ac->m_arc.Domain()!=bc->m_arc.Domain())return false;
    if(a.m_c3){auto x=ON_ArcCurve::Cast(a.m_c3),y=ON_ArcCurve::Cast(b.m_c3);if(!x||!y||x->m_dim!=y->m_dim||x->Domain()!=y->Domain()||x->m_arc.radius!=y->m_arc.radius||x->m_arc.plane.origin!=y->m_arc.plane.origin||x->m_arc.Domain()!=y->m_arc.Domain())return false;}
    auto x=ON_NurbsSurface::Cast(a.m_s),y=ON_NurbsSurface::Cast(b.m_s);if(!x||!y||x->Dimension()!=y->Dimension())return false;
    for(int d=0;d<2;++d){if(x->Order(d)!=y->Order(d)||x->CVCount(d)!=y->CVCount(d)||x->Domain(d)!=y->Domain(d)||x->KnotCount(d)!=y->KnotCount(d))return false;for(int i=0;i<x->KnotCount(d);++i)if(x->Knot(d,i)!=y->Knot(d,i))return false;}
    for(int i=0;i<x->CVCount(0);++i)for(int j=0;j<x->CVCount(1);++j)for(int k=0;k<x->CVSize();++k)if(x->CV(i,j)[k]!=y->CV(i,j)[k])return false;
    for(double t:{11.,12.125,17.,22.75,23.})if(a.PointAt(t)!=b.PointAt(t))return false;
    return true;
}
static QByteArray bytes(const std::filesystem::path& p){QFile f(QString::fromStdWString(p.wstring()));require(f.open(QIODevice::ReadOnly),"Read native fixture bytes");return f.readAll();}
static QJsonArray strings(const ON_Object& o){ON_ClassArray<ON_UserString> a;o.GetUserStrings(a);QJsonArray result;for(int i=0;i<a.Count();++i){ON_String k(a[i].m_key),v(a[i].m_string_value);result.append(QJsonObject{{"key",QString::fromUtf8(k.Array())},{"value",QString::fromUtf8(v.Array())}});}return result;}
static QJsonValue arcFacts(const ON_Curve* c){if(!c)return QJsonValue::Null;auto a=ON_ArcCurve::Cast(c);require(a,"Expected source Arc");QJsonArray p;auto frame=a->m_arc.plane;for(auto v:{frame.origin,ON_3dPoint(frame.xaxis),ON_3dPoint(frame.yaxis),ON_3dPoint(frame.zaxis)})for(int i=0;i<3;++i)p.append(v[i]);for(double v:{frame.plane_equation.x,frame.plane_equation.y,frame.plane_equation.z,frame.plane_equation.d})p.append(v);return QJsonObject{{"class_name","ON_ArcCurve"},{"dimension",a->Dimension()},{"domain",QJsonArray{a->Domain()[0],a->Domain()[1]}},{"radius",a->m_arc.radius},{"plane",p},{"angle_domain",QJsonArray{a->m_arc.Domain()[0],a->m_arc.Domain()[1]}},{"user_strings",strings(*a)}};}
static QJsonObject surfaceFacts(const ON_Surface* s){auto n=ON_NurbsSurface::Cast(s);require(n,"Expected source NURBS surface");QJsonArray domains,knots,cvs;for(int d=0;d<2;++d){domains.append(QJsonArray{n->Domain(d)[0],n->Domain(d)[1]});QJsonArray k;for(int i=0;i<n->KnotCount(d);++i)k.append(n->Knot(d,i));knots.append(k);}for(int i=0;i<n->CVCount(0);++i){QJsonArray row;for(int j=0;j<n->CVCount(1);++j){QJsonArray cv;for(int k=0;k<n->CVSize();++k)cv.append(n->CV(i,j)[k]);row.append(cv);}cvs.append(row);}return {{"class_name","ON_NurbsSurface"},{"dimension",n->Dimension()},{"rational",n->IsRational()},{"orders",QJsonArray{n->Order(0),n->Order(1)}},{"domains",domains},{"knots",knots},{"cvs",cvs},{"user_strings",strings(*n)}};}
int main(){try{ON::Begin();QDir evidence(QStringLiteral(OM9_SURFACE_READ_EVIDENCE_DIR));require(QDir().mkpath(evidence.path()),"Evidence directory");QJsonArray cases;int valid=0,invalid=0;
    int legacy=0;
    for(int dimension:{2,3})for(bool optional:{false,true})for(int version:{50,60,70}){
        auto source=fixture(dimension,optional);ON_Write3dmBufferArchive raw(0,1024*1024,version,ON::Version());require(source->Write(raw),"Write raw native payload");
        ON_Read3dmBufferArchive reader(raw.SizeOfArchive(),raw.Buffer(),false,version,ON::Version());ON_CurveOnSurface decoded;
        require(decoded.Read(reader)&&exact(*source,decoded),"CurveOnSurface Read must preserve optional curve and required surface exactly");
        ON_Write3dmBufferArchive object(0,1024*1024,version,ON::Version());require(object.WriteObject(*source),"Write exact native class");ON_Read3dmBufferArchive objectReader(object.SizeOfArchive(),object.Buffer(),false,version,ON::Version());ON_Object* read=nullptr;require(objectReader.ReadObject(&read)==1,"Read native class through factory");std::unique_ptr<ON_Object> owned(read);auto curve=ON_CurveOnSurface::Cast(read);require(curve&&curve->ClassId()==&ON_CLASS_RTTI(ON_CurveOnSurface)&&exact(*source,*curve),"Whole object retains native class and children");
        ON_wString root;require(curve->GetUserString(L"Root",root)&&root==L"Original surface curve","Root user strings retained");
        ONX_Model model;ON_Layer layer;layer.SetName(L"Surface curves");model.AddModelComponent(layer);ON_3dmObjectAttributes attrs;attrs.m_name=L"Exact surface curve";auto added=model.AddModelGeometryComponent(source.get(),&attrs);require(!added.IsEmpty(),"Register model geometry");auto id=added.ModelComponent()->Id();
        const auto stem=QString("d%1-c3%2-v%3").arg(dimension).arg(optional?1:0).arg(version);auto input=std::filesystem::path(evidence.filePath(stem+".3dm").toStdWString());auto output=std::filesystem::path(evidence.filePath(stem+"-reread.3dm").toStdWString());require(model.Write(input.c_str(),version/10,nullptr),"Write full native model");auto immutable=bytes(input);ONX_Model modelRead;require(modelRead.Read(input.c_str(),nullptr),"Read full native model with optional child");auto component=ON_ModelGeometryComponent::Cast(modelRead.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,id).ModelComponent());auto native=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(native&&exact(*source,*native),"Native model child geometry exact");require(modelRead.Write(output.c_str(),version/10,nullptr),"Reencode full native model");ONX_Model final;require(final.Read(output.c_str(),nullptr),"Reread reencoded model");component=ON_ModelGeometryComponent::Cast(final.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,id).ModelComponent());native=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(native&&exact(*source,*native)&&bytes(input)==immutable,"Reencode exact native children and immutable input");
        auto inventory=OpenMatrix9Gui::ThreeDm::inspectArchive(input,1);auto records=inventory.document["records"].toArray();require(records.size()==1&&records[0].toObject()["capability"]=="retained","CurveOnSurface inventory must not advertise missing CAD adapter as editable");
        auto host=OpenMatrix9Gui::ThreeDm::readArchive(input,1,true);require(host.items.size()==1&&host.items[0].retained&&host.items[0].sourceClass=="ON_CurveOnSurface","Preservation import must retain complete native surface curve without CAD conversion");
#ifdef OM9_SURFACE_LEGACY_READER_EXE
        if(version==50){QTemporaryDir staging(evidence.filePath("legacy-XXXXXX"));require(staging.isValid(),"Fresh legacy staging");staging.setAutoRemove(false);auto outputPath=staging.filePath("output.3dm"),reportPath=staging.filePath("report.json");QProcess process;process.start(QStringLiteral(OM9_SURFACE_LEGACY_READER_EXE),{QString::fromStdWString(input.wstring()),outputPath,reportPath});require(process.waitForFinished(30000)&&process.exitStatus()==QProcess::NormalExit&&process.exitCode()==0,"Separate repaired SDK2013 reader exit0");QFile file(reportPath);require(file.open(QIODevice::ReadOnly),"Read independent report");auto report=QJsonDocument::fromJson(file.readAll()).object();require(report["sdk_version"]==201307115&&report["source_version"]==50&&report["read_repair"]=="OM9 CurveOnSurface::Read only","Declared independent reader revision and repair");auto records=report["curves"].toArray();require(records.size()==1,"Independent one native curve");auto row=records[0].toObject();char identity[37]{};ON_UuidToString(id,identity);require(row["uuid"]==identity&&row["class_name"]=="ON_CurveOnSurface"&&row["dimension"]==dimension&&row["parameter"]==arcFacts(source->m_c2)&&row["approximation"]==arcFacts(source->m_c3)&&row["surface"]==surfaceFacts(source->m_s)&&row["user_strings"].toArray()==strings(*source),"Independent exact child types/raw fields/root metadata");auto samples=row["samples"].toArray();int index=0;for(double t:{11.,12.125,17.,22.75,23.}){auto expected=source->PointAt(t);auto actual=samples[index++].toArray();require(actual.size()==3,"Independent sample dimension");for(int k=0;k<3;++k)require(std::abs(actual[k].toDouble()-expected[k])<=1e-12,"Independent composed sample agreement at declared rounding tolerance");}ONX_Model reread;require(reread.Read(std::filesystem::path(outputPath.toStdWString()).c_str(),nullptr),"Modern decode of independent reencode");auto component=ON_ModelGeometryComponent::Cast(reread.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,id).ModelComponent());auto curve=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(curve&&exact(*source,*curve)&&bytes(input)==immutable,"Independent reencode exact native fields and immutable source");++legacy;}
#endif
        cases.append(QJsonObject{{"dimension",dimension},{"optional_child",optional},{"version",version},{"exact_native_class_children",true},{"source_immutable",true},{"source_retained_host",true}});++valid;
    }
    // An archive decoder must retain known unresolved reference records; their
    // owning model/adapter resolves them after the native child payload is read.
    int deferred=0;
    for(int child=0;child<2;++child){
        auto source=fixture(3,true);const auto referenceId=ON_UuidFromString("00000000-0000-4000-8000-000000000143");
        std::unique_ptr<ON_Curve> target(child==0?source->m_c2:source->m_c3);auto reference=new ON_PolyEdgeCurve;require(reference->Create(target.get(),referenceId),"Create resolved reference child");
        if(child==0)source->m_c2=reference;else source->m_c3=reference;
        require(source->IsValid(),"Resolved reference source valid");ON_Write3dmBufferArchive raw(0,1024*1024,50,ON::Version());require(source->Write(raw),"Write resolved child reference");ON_Read3dmBufferArchive reader(raw.SizeOfArchive(),raw.Buffer(),false,50,ON::Version());ON_CurveOnSurface decoded;
        require(decoded.Read(reader),"Read must retain known unresolved PolyEdge references for later model resolution");
        auto ref=ON_PolyEdgeCurve::Cast(child==0?decoded.m_c2:decoded.m_c3);auto segment=ref?ON_PolyEdgeSegment::Cast(ref->SegmentCurve(0)):nullptr;
        require(segment&&ref->Dimension()==0&&segment->m_object_id==referenceId&&tag(decoded.m_s,L"Surface"),"Reference identity and required surface retained");++deferred;
    }
    // Wrong types, malformed presence flags and every truncated payload must
    // leave an already valid object's children untouched, with staged ownership.
    auto original=fixture(3,true);auto sentinel=fixture(3,true);
    auto rejects=[&](ON_Write3dmBufferArchive& raw){ON_Read3dmBufferArchive read(raw.SizeOfArchive(),raw.Buffer(),false,50,ON::Version());require(!sentinel->Read(read)&&exact(*original,*sentinel),"Failed child decode must not replace valid existing children");++invalid;};
    ON_Point wrong(ON_3dPoint(1,2,3));
    for(int phase=0;phase<4;++phase){ON_Write3dmBufferArchive raw(0,1024*1024,50,ON::Version());
        if(phase==0){raw.WriteObject(wrong);}else{raw.WriteObject(*original->m_c2);raw.WriteInt(phase==1?2:1);if(phase==2)raw.WriteObject(wrong);else if(phase==3){raw.WriteObject(*original->m_c3);raw.WriteObject(wrong);}}
        rejects(raw);
    }
    ON_Write3dmBufferArchive full(0,1024*1024,50,ON::Version());require(original->Write(full),"Full truncation input");
    for(size_t size=0;size<full.SizeOfArchive();size+=17){ON_Read3dmBufferArchive read(size,full.Buffer(),false,50,ON::Version());require(!sentinel->Read(read)&&exact(*original,*sentinel),"Truncated payload must leave existing native children intact");++invalid;}
    QFile report(evidence.filePath("results.json"));require(report.open(QIODevice::WriteOnly),"Open native report");auto data=QJsonDocument(QJsonObject{{"ok",true},{"valid_cases",valid},{"invalid_cases",invalid},{"deferred_reference_cases",deferred},{"legacy_cases",legacy},{"cases",cases}}).toJson();require(report.write(data)==data.size(),"Write native report");require(valid==12&&invalid>20&&deferred==2,"Required valid, deferred and malformed cases");std::cout<<"CurveOnSurface exact archive recovery12 valid,2 deferred references,"<<legacy<<" independent legacy and "<<invalid<<" malformed/truncated cases PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
