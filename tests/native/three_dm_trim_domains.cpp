#include "ThreeDmInventory.h"
#include "ThreeDmNativeReferences.h"
#include "opennurbs_polyedgecurve.h"
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <iostream>
#include <limits>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool condition,const char* message){if(!condition)throw std::runtime_error(message);}
static void reverseUnderlying(ON_Curve& curve,ON_CurveProxy& proxy){
    const auto domain=curve.Domain();require(curve.Reverse()&&curve.SetDomain(domain),"Reverse real curve with original parameter interval");
    const auto evaluation=proxy.Domain();require(proxy.ON_CurveProxy::Reverse()&&proxy.SetDomain(evaluation),"Reverse proxy without altering native topology");proxy.DestroyRuntimeCache();
}
int main(){try{ON::Begin();QDir dir(QStringLiteral(OM9_TRIM_DOMAIN_DIR));require(QDir().mkpath(dir.path()),"Evidence directory");QJsonArray cases;
    for(int shape=0;shape<2;++shape)for(bool edgeReversed:{false,true})for(bool trimReversed:{false,true})for(bool segmentReversed:{false,true})for(bool consistent:{false,true}){
        ON_NurbsSurface surface(3,false,3,2,3,2);surface.SetKnot(0,0,0);surface.SetKnot(0,1,0);surface.SetKnot(0,2,1);surface.SetKnot(0,3,1);surface.SetKnot(1,0,0);surface.SetKnot(1,1,1);
        for(int i=0;i<3;++i)for(int j=0;j<2;++j)surface.SetCV(i,j,ON_3dPoint(i,3*j+(i==1?1:0),shape&&i==1?2:0));
        require(surface.IsValid(),"Real quadratic surface valid");auto brep=new ON_Brep;require(brep->NewFace(surface)&&brep->IsValid(),"Valid curved native face topology");
        auto& trim=brep->m_T[0];auto edge=trim.Edge();require(edge&&edge->EdgeCurveOf()&&trim.TrimCurveOf(),"Actual trim/edge curves");
        // Topological reversal updates connected trim direction; a separate
        // reversed C3/proxy pair preserves that geometry and topology.
        if(trimReversed){require(edge->Reverse(),"Reverse topological edge and connected trim directions");auto c3=brep->m_C3[edge->m_c3i];require(c3->SetDomain(0,1),"Restore independent C3 domain");edge->SetProxyCurve(c3);}
        auto c2=brep->m_C2[trim.m_c2i];require(c2->SetDomain(101,149),"Reparameterize UV curve without changing geometry");trim.SetProxyCurve(c2);require(edge->SetDomain(11,23)&&trim.SetDomain(101,149),"Set independent edge/trim domains");
        if(edgeReversed)reverseUnderlying(*brep->m_C3[edge->m_c3i],*edge);
        brep->DestroyRuntimeCache();ON_wString validation;ON_TextLog log(validation);if(!brep->IsValid(&log)){std::wcerr<<validation.Array();throw std::runtime_error("Independent reversed C3/C2 and edge/trim domains stay valid");}
        require(trim.m_bRev3d==trimReversed,"Trim directions relative to reversed topological edge");
        ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance=1e-7;ON_Layer layer;layer.SetName(L"Curved trim domains");model.AddModelComponent(layer);auto added=model.AddManagedModelGeometryComponent(brep,nullptr);require(!added.IsEmpty(),"Own real curved Brep");
        auto ref=new ON_PolyEdgeCurve;require(ref->Create(edge->EdgeCurveOf(),added.ModelComponent()->Id()),"Create reference without defective SDK trim factory");auto segment=ref->SegmentCurve(0);segment->m_component_index=trim.ComponentIndex();segment->m_edge_domain=ON_Interval(14,20);segment->m_trim_domain=consistent?ON_Interval(113,137):ON_Interval(101,149);
        auto first=edge->RealCurveParameter(14),last=edge->RealCurveParameter(20);if(first>last)std::swap(first,last);require(segment->SetProxyCurveDomain(ON_Interval(first,last)),"Corresponding strict C3 interval");if(segmentReversed)require(segment->Reverse(),"Reverse reference evaluation");segment->SetDomain(31,47);ref->SetDomain(31,47);
        auto uv=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(1,1,0));uv->ChangeDimension(2);uv->SetDomain(31,47);auto root=new ON_CurveOnSurface(uv,ref,new ON_PlaneSurface(ON_xy_plane));require(root->IsValid(),"Live native reference valid even with wrong trim interval");model.AddManagedModelGeometryComponent(root,nullptr);
        auto name=QString("shape-%1-edge-%2-trim-%3-segment-%4-consistent-%5.3dm").arg(shape).arg(edgeReversed).arg(trimReversed).arg(segmentReversed).arg(consistent);auto path=std::filesystem::path(dir.filePath(name).toStdWString());require(model.Write(path.c_str(),5,nullptr),"Write real independent domain fixture");auto archive=inspectArchive(path,1);QJsonObject row;for(auto value:archive.document["records"].toArray())if(value.toObject()["class_name"]=="ON_CurveOnSurface")row=value.toObject();auto analysis=row["native_reference_analysis"].toObject();
        require(analysis["status"]==(consistent?"linked":"invalid"),"Trim subdomain endpoints must correspond to the referenced edge portion");
        if(consistent){auto graph=archive.nativeReferences->graphs.at(row["source_uuid"].toString());auto linked=ON_CurveOnSurface::Cast(graph->geometry.at(graph->rootUuid).get());auto actual=ON_PolyEdgeCurve::Cast(linked->m_c3)->SegmentCurve(0);
            const bool reverseAlongEdge=edgeReversed!=segmentReversed;require(actual->ReversedEdgeDir()==reverseAlongEdge&&actual->ReversedTrimDir()==(reverseAlongEdge!=trimReversed),"Native direction helpers account for independent proxy reversals");
            for(int i=0;i<=16;++i){double f=reverseAlongEdge?1.-i/16.:i/16.;double u=.25+.5*f;if(trimReversed)u=1-u;auto point=actual->PointAt(31+i);auto expected=ON_3dPoint(2*u,2*u*(1-u),shape?4*u*(1-u):0);require(point.DistanceTo(expected)<1e-12,"Quadratic boundary matches independent analytic surface equation");require(std::abs(actual->EdgeParameter(31+i)-(14+6*f))<1e-12,"Independent native edge parameter domains agree");}
            auto refs=analysis["references"].toArray();auto binding=refs[0].toObject()["trim_domain_analysis"].toObject();require(binding["status"]=="endpoints_and_interior_samples_verified"&&binding["interior_mapping"]=="projected_samples_verified","Diagnostics expose verified physical interior correspondence without claiming certified global mapping");
            require(binding["interior_samples"].toArray().size()==15,"Fixed bounded interior projection samples");
            require(binding["maximum_endpoint_deviation"].toDouble()<1e-12&&binding["source_tolerance"].toDouble()==1e-7,"Source-unit endpoint deviations and model tolerance exposed");
            std::weak_ptr<ONX_Model> source=archive.nativeModel;archive={};require(!source.expired()&&actual->BrepTrim()&&actual->PointAt(39).DistanceTo(ON_3dPoint(1,.5,shape?1:0))<1e-12,"Real curved trim lifetime retained after inventory release");
        }else require(!archive.nativeReferences->graphs.contains(row["source_uuid"].toString())&&analysis["message"].toString().contains("trim"),"Wrong but in-bounds trim interval exposes no linked graph");
        cases.append(QJsonObject{{"fixture",name},{"shape",shape},{"edge_reversed",edgeReversed},{"trim_reversed",trimReversed},{"segment_reversed",segmentReversed},{"consistent",consistent},{"passed",true}});
    }
    QJsonArray tolerances;
    for(int kind=0;kind<5;++kind){
        auto path=std::filesystem::path(dir.filePath("shape-0-edge-0-trim-0-segment-0-consistent-1.3dm").toStdWString());auto archive=inspectArchive(path,1);auto key=archive.nativeReferences->graphs.begin()->first;
        auto component=ON_ModelGeometryComponent::Cast(archive.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(key.toLatin1().constData())).ModelComponent());auto root=const_cast<ON_CurveOnSurface*>(ON_CurveOnSurface::Cast(component->Geometry(nullptr)));auto raw=ON_PolyEdgeCurve::Cast(root->m_c3)->SegmentCurve(0);raw->m_trim_domain=ON_Interval(113.001,137.001);
        archive.nativeModel->m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance=kind==0?.01:kind==3?-1:kind==4?std::numeric_limits<double>::quiet_NaN():1e-8;
        if(kind==2){auto owner=ON_ModelGeometryComponent::Cast(archive.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,raw->m_object_id).ModelComponent());auto brep=const_cast<ON_Brep*>(ON_Brep::Cast(owner->Geometry(nullptr)));brep->m_E[brep->m_T[raw->m_component_index.m_index].m_ei].m_tolerance=.001;}
        const bool accepted=kind==0||kind==2;auto resolution=resolveNativeReferences(archive.nativeModel,archive.document["records"].toArray());auto analysis=resolution->diagnostics[key].toObject();require(analysis["status"]==(accepted?"linked":"invalid"),"Trim endpoint validation respects model/edge tolerance and rejects malformed tolerance");
        if(accepted){auto binding=analysis["references"].toArray()[0].toObject()["trim_domain_analysis"].toObject();double deviation=binding["maximum_endpoint_deviation"].toDouble();require(deviation>1e-5&&deviation<1e-4&&deviation<=binding["effective_tolerance"].toDouble(),"Small UV shift accepted only within effective physical tolerance");}
        else require(!resolution->graphs.contains(key),"Out-of-tolerance and malformed tolerance graphs never exposed");
        tolerances.append(QJsonObject{{"case",kind},{"accepted",accepted},{"passed",true}});
    }
    QFile report(dir.filePath("results.json"));require(report.open(QIODevice::WriteOnly),"Evidence report");report.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",cases},{"valid_cases",16},{"invalid_cases",16},{"tolerance_cases",5},{"tolerances",tolerances}}).toJson());std::cout<<"Trim domains PASS32 domain +5 tolerance cases\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
