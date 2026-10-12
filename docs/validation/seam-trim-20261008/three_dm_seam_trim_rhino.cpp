#include "ThreeDmInventory.h"
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QCryptographicHash>
#include <iostream>
#include <map>
using namespace OpenMatrix9Gui::ThreeDm;
static int checks=0;
static void require(bool value,const char* text){++checks;if(!value)throw ExchangeError(text);}
static QByteArray bytes(QString path){QFile f(path);require(f.open(QIODevice::ReadOnly),"Read frozen actual target bytes");return f.readAll();}
static QString sha(QString path){return QString::fromLatin1(QCryptographicHash::hash(bytes(path),QCryptographicHash::Sha256).toHex());}
static bool indices(const ON_SimpleArray<int>& a,const ON_SimpleArray<int>& b){if(a.Count()!=b.Count())return false;for(int i=0;i<a.Count();++i)if(a[i]!=b[i])return false;return true;}
static QJsonArray refinements;
static void equivalentUVBasis(const ON_NurbsCurve& source,const ON_NurbsCurve& saved,int index){
    require(source.m_dim==2&&saved.m_dim==2&&source.m_is_rat==saved.m_is_rat&&source.m_order==saved.m_order&&source.Domain()==saved.Domain(),"UV refinement retains dimension,rationality,degree and domain");
    ON_NurbsCurve a(source),b(saved);std::map<double,int> ak,bk,common;
    for(int i=0;i<a.KnotCount();++i)++ak[a.Knot(i)];for(int i=0;i<b.KnotCount();++i)++bk[b.Knot(i)];common=ak;for(const auto& [t,m]:bk)common[t]=std::max(common[t],m);
    for(const auto& [t,m]:common){
        if(t==a.Domain().Min()||t==a.Domain().Max())require(ak[t]==bk[t],"UV refinement retains clamped endpoint multiplicities");
        else{if(ak[t]<m)require(a.InsertKnot(t,m),"Refine source UV basis on an independent copy");if(bk[t]<m)require(b.InsertKnot(t,m),"Refine actual saved UV basis on an independent copy");}
    }
    require(a.m_cv_count==b.m_cv_count&&a.KnotCount()==b.KnotCount(),"UV refinement produces a complete common basis");
    for(int i=0;i<a.KnotCount();++i)require(a.Knot(i)==b.Knot(i),"All common-basis UV knots exact");
    double maximum=0;
    for(int i=0;i<a.m_cv_count;++i){ON_4dPoint x,y;a.GetCV(i,x);b.GetCV(i,y);require(x.w==y.w&&x.z==y.z,"UV common-basis homogeneous weights/unused coordinate exact");const double delta=std::hypot(x.x-y.x,x.y-y.y);maximum=std::max(maximum,delta);require(std::isfinite(delta)&&delta<=1e-12,"Every common-basis UV control point agrees at numerical precision; no sampling substitute");}
    refinements.append(QJsonObject{{"c2_index",index},{"source_cv_count",source.m_cv_count},{"saved_cv_count",saved.m_cv_count},{"common_cv_count",a.m_cv_count},{"maximum_uv_control_difference",maximum},{"uv_numerical_threshold",1e-12}});
}
static void graph(const ON_Brep& a,const ON_Brep& b){
    require(a.m_V.Count()==b.m_V.Count()&&a.m_E.Count()==b.m_E.Count()&&a.m_T.Count()==b.m_T.Count()&&a.m_L.Count()==b.m_L.Count()&&a.m_F.Count()==b.m_F.Count()&&a.m_C2.Count()==b.m_C2.Count()&&a.m_C3.Count()==b.m_C3.Count()&&a.m_S.Count()==b.m_S.Count(),"All native BRep topology/curve/surface table counts exact");
    require(a.DataCRC(0)==b.DataCRC(0),"Native BRep geometry CRC agrees; individualC2 basis checked separately because BRep CRC omitsC2");
    for(int i=0;i<a.m_V.Count();++i)require(a.m_V[i].point==b.m_V[i].point&&indices(a.m_V[i].m_ei,b.m_V[i].m_ei)&&a.m_V[i].m_tolerance==b.m_V[i].m_tolerance,"Decoded native vertices,edge adjacency and tolerances exact");
    for(int i=0;i<a.m_E.Count();++i){const auto& x=a.m_E[i];const auto& y=b.m_E[i];require(x.m_c3i==y.m_c3i&&x.m_vi[0]==y.m_vi[0]&&x.m_vi[1]==y.m_vi[1]&&indices(x.m_ti,y.m_ti)&&x.Domain()==y.Domain()&&x.ProxyCurveDomain()==y.ProxyCurveDomain()&&x.ProxyCurveIsReversed()==y.ProxyCurveIsReversed()&&x.m_tolerance==y.m_tolerance,"Decoded edge references,negative proxy domain,reversal,adjacency and tolerance exact");}
    for(int i=0;i<a.m_T.Count();++i){const auto& x=a.m_T[i];const auto& y=b.m_T[i];require(x.m_c2i==y.m_c2i&&x.m_ei==y.m_ei&&x.m_vi[0]==y.m_vi[0]&&x.m_vi[1]==y.m_vi[1]&&x.m_li==y.m_li&&x.m_type==y.m_type&&x.m_iso==y.m_iso&&x.m_bRev3d==y.m_bRev3d&&x.Domain()==y.Domain()&&x.ProxyCurveDomain()==y.ProxyCurveDomain()&&x.ProxyCurveIsReversed()==y.ProxyCurveIsReversed(),"Decoded seam/singular trim topology,UV proxy domains and winding exact");}
    for(int i=0;i<a.m_L.Count();++i)require(indices(a.m_L[i].m_ti,b.m_L[i].m_ti)&&a.m_L[i].m_fi==b.m_L[i].m_fi&&a.m_L[i].m_type==b.m_L[i].m_type,"Decoded native loop graph exact");
    for(int i=0;i<a.m_F.Count();++i)require(a.m_F[i].m_si==b.m_F[i].m_si&&indices(a.m_F[i].m_li,b.m_F[i].m_li)&&a.m_F[i].m_bRev==b.m_F[i].m_bRev,"Decoded native face/surface graph and orientation exact");
    for(int dim=0;dim<2;++dim){const auto& ca=dim?a.m_C3:a.m_C2;const auto& cb=dim?b.m_C3:b.m_C2;for(int i=0;i<ca.Count();++i){
        require(ca[i]->ClassId()==cb[i]->ClassId()&&ca[i]->Domain()==cb[i]->Domain(),"Native curve class and domain retained");
        const bool identical=ca[i]->DataCRC(0)==cb[i]->DataCRC(0);
        if(!identical){auto x=ON_NurbsCurve::Cast(ca[i]),y=ON_NurbsCurve::Cast(cb[i]);require(dim==0&&x&&y,"Only C2 NURBS knot refinement may differ;3D curves remain exact");equivalentUVBasis(*x,*y,i);}
        else for(int j=0;j<=16;++j)require(ca[i]->PointAt(ca[i]->Domain().ParameterAt(j/16.))==cb[i]->PointAt(cb[i]->Domain().ParameterAt(j/16.)),"Exact-basis native2D/3D curve interior values exact");
    }}
    for(int i=0;i<a.m_S.Count();++i){require(a.m_S[i]->ClassId()==b.m_S[i]->ClassId()&&a.m_S[i]->Domain(0)==b.m_S[i]->Domain(0)&&a.m_S[i]->Domain(1)==b.m_S[i]->Domain(1)&&a.m_S[i]->DataCRC(0)==b.m_S[i]->DataCRC(0),"Native surface class/domains/payload exact");for(int u=0;u<=4;++u)for(int v=0;v<=4;++v)require(a.m_S[i]->PointAt(a.m_S[i]->Domain(0).ParameterAt(u/4.),a.m_S[i]->Domain(1).ParameterAt(v/4.))==b.m_S[i]->PointAt(b.m_S[i]->Domain(0).ParameterAt(u/4.),b.m_S[i]->Domain(1).ParameterAt(v/4.)),"Decoded native surface grid points exact");}
}
int main(){try{ON::Begin();QDir root(QStringLiteral(OM9_SEAM_TRIM_RHINO));auto oracle=QJsonDocument::fromJson(bytes(root.filePath("oracle.json"))).object(),actual=QJsonDocument::fromJson(bytes(root.filePath("rhino5-gui-profile-results.json"))).object();require(actual["ok"].toBool()&&actual["cases"].toArray().size()==4&&oracle["cases"].toArray().size()==4,"Actual four-case GUI completion");require(actual["oracle_sha256"]==sha(root.filePath("oracle.json"))&&actual["probe_sha256"]==sha(root.filePath("rhino5-seam-trim-probe.py")),"Actual target binds independent oracle/probe");QJsonArray cases;
    for(int i=0;i<4;++i){auto wanted=oracle["cases"].toArray()[i].toObject(),row=actual["cases"].toArray()[i].toObject();auto source=root.filePath(row["fixture"].toString()),saved=root.filePath(QFileInfo(row["saved_fixture"].toString()).fileName());require(row["fixture"]==wanted["fixture"]&&row["passed"].toBool()&&row["bad_objects"].toInt()==0&&row["source_unchanged"].toBool(),"Actual GUI source/SaveAs/reread valid with no bad objects");auto sourceHash=sha(source),savedHash=sha(saved);require(sourceHash==wanted["source_sha256"]&&row["source_sha256"]==wanted["source_sha256"]&&savedHash==row["saved_sha256"],"Source and actual saved byte bindings");auto before=inspectArchive(std::filesystem::path(source.toStdWString())),after=inspectArchive(std::filesystem::path(saved.toStdWString()));require(before.document["issues"].toArray().isEmpty()&&after.document["issues"].toArray().isEmpty()&&after.document["source_version"]==50&&after.document["records"].toArray().size()==1,"Complete target native graph decoded");auto expected=wanted["objects"].toArray()[0].toObject();auto id=ON_UuidFromString(expected["uuid"].toString().toLatin1().constData());auto get=[&](ArchiveInventory& inv){auto component=ON_ModelGeometryComponent::Cast(inv.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,id).ModelComponent());return component?ON_Brep::Cast(component->Geometry(nullptr)):nullptr;};
        auto a=get(before),b=get(after);require(a&&b&&a->IsValid()&&b->IsValid()&&a->IsSolid()&&b->IsSolid()&&(a->SolidOrientation()==1||a->SolidOrientation()==2)&&(b->SolidOrientation()==1||b->SolidOrientation()==2),"Native UUID and closed valid solid retained; SDK1/2 winding checked by independent signed CAD volume below");refinements=QJsonArray{};graph(*a,*b);
        for(auto brep:{a,b}){auto shape=importBrep(*brep,1e-7);require(!shape.IsNull()&&BRepCheck_Analyzer(shape).IsValid(),"Actual source/saved BRep has editable valid CAD representation");GProp_GProps mass;BRepGProp::VolumeProperties(shape,mass,1e-13);require(std::abs(mass.Mass()-expected["volume_mm3"].toDouble())<=1e-6,"Independent analytic signed volume after actual Rhino5 at unchanged threshold");}
        require(sha(source)==sourceHash&&sha(saved)==savedHash,"Read-only target decode/conversion leaves both source and saved files immutable");cases.append(QJsonObject{{"fixture",wanted["fixture"]},{"source_sha256",sourceHash},{"saved_sha256",savedHash},{"uv_knot_refinements",refinements},{"passed",true}});
    }
    QFile result(QStringLiteral(OM9_SEAM_TRIM_RHINO_RESULTS));require(result.open(QIODevice::WriteOnly),"Write actual target native graph evidence");const auto reportHash=sha(root.filePath("rhino5-gui-profile-results.json"));result.write(QJsonDocument(QJsonObject{{"ok",true},{"checks",checks},{"rhino_report_sha256",reportHash},{"cases",cases}}).toJson());std::cout<<checks<<" actual BRep seam/pole/edit checks passed\n";return 0;
}catch(const std::exception& e){std::cerr<<"Check "<<checks<<": "<<e.what()<<"\n";return 1;}}
