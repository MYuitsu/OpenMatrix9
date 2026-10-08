#include "ThreeDmArchive.h"
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <Geom_Surface.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <BRepBndLib.hxx>
#include <Bnd_Box.hxx>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <fstream>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool v,const char* msg){if(!v)throw std::runtime_error(msg);}
static double volume(const TopoDS_Shape& shape){GProp_GProps p;BRepGProp::VolumeProperties(shape,p,1e-10);return p.Mass();}
int main(int argc,char** argv){try{
    ON::Begin();QJsonArray rows;ONX_Model model;require(model.Read(std::filesystem::path(OM9_RING_PRECISION_SOURCE).c_str()),"Original source read");
    for(auto uuid:{"2cbc0748-a4cf-42b4-a298-abe8a2a15ead","4af47cae-cdfa-4235-9b1d-f54d7dc7254d","502f1501-11ed-4233-950b-2f9f97a5fdcf","53ae3f58-728e-4bff-9239-801419688715"}){
        auto ref=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid));auto component=ON_ModelGeometryComponent::Cast(ref.ModelComponent());require(component,"Source component");auto brep=ON_Brep::Cast(component->Geometry(nullptr));require(brep,"Source Brep");auto shape=importBrep(*brep,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance);auto exported=exportBrep(shape,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance);QJsonArray faces;
        for(int fi=0;fi<brep->m_F.Count();++fi){auto& a=brep->m_F[fi];auto& b=exported->m_F[fi];auto sa=brep->m_S[a.m_si],sb=exported->m_S[b.m_si];double maxSurface=0,maxTrim=0,maxImportSurface=0;QJsonArray trims;auto importedSurface=importSurface(*sa);ON_NurbsSurface nurbs;int form=sa->GetNurbForm(nurbs);
            for(int u=0;u<=10;++u)for(int v=0;v<=10;++v){double x=sa->Domain(0).ParameterAt(u/10.0),y=sa->Domain(1).ParameterAt(v/10.0);auto uv=importedSurfaceParameters(*sa,x,y);auto p=importedSurface->Value(uv[0],uv[1]);maxImportSurface=std::max(maxImportSurface,sa->PointAt(x,y).DistanceTo(ON_3dPoint(p.X(),p.Y(),p.Z())));maxSurface=std::max(maxSurface,sa->PointAt(x,y).DistanceTo(sb->PointAt(x,y)));}
            for(int li=0;li<a.m_li.Count();++li){auto& la=brep->m_L[a.m_li[li]];auto& lb=exported->m_L[b.m_li[li]];for(int ti=0;ti<la.m_ti.Count();++ti){auto& ta=brep->m_T[la.m_ti[ti]];auto& tb=exported->m_T[lb.m_ti[ti]];double error=0;for(int sample=0;sample<=32;++sample){auto qa=ta.PointAt(ta.Domain().ParameterAt(sample/32.0)),qb=tb.PointAt(tb.Domain().ParameterAt(sample/32.0));error=std::max(error,sa->PointAt(qa.x,qa.y).DistanceTo(sb->PointAt(qb.x,qb.y)));}maxTrim=std::max(maxTrim,error);trims.append(QJsonObject{{"source_c2",ta.m_c2i},{"exported_c2",tb.m_c2i},{"max_corresponding_sample_mm",error},{"source_reverse",ta.m_bRev3d},{"exported_reverse",tb.m_bRev3d}});}}
            QJsonObject revInfo;if(auto rev=ON_RevSurface::Cast(sa)){revInfo=QJsonObject{{"profile",rev->m_curve->ClassId()->ClassName()},{"angle",QJsonArray{rev->m_angle.Min(),rev->m_angle.Max()}},{"domain",QJsonArray{rev->m_t.Min(),rev->m_t.Max()}},{"profile_domain",QJsonArray{rev->m_curve->Domain().Min(),rev->m_curve->Domain().Max()}},{"transposed",rev->m_bTransposed}};if(auto arc=ON_ArcCurve::Cast(rev->m_curve))revInfo["profile_angles"]=QJsonArray{arc->m_arc.Domain().Min(),arc->m_arc.Domain().Max()};}
            faces.append(QJsonObject{{"face",fi},{"source_surface",sa->ClassId()->ClassName()},{"revolution",revInfo},{"nurb_form",form},{"import_surface_uv_deviation",maxImportSurface},{"surface",maxSurface},{"trim",maxTrim},{"trims",trims}});
        }
        double nativeBoundaryDistance=0;
        for(int fi=0;fi<brep->m_F.Count();++fi){const auto& face=brep->m_F[fi];for(int li=0;li<face.m_li.Count();++li){auto& loop=brep->m_L[face.m_li[li]];for(int ti=0;ti<loop.m_ti.Count();++ti){auto& trim=brep->m_T[loop.m_ti[ti]];for(int i=0;i<=4;++i){
            auto uv=trim.PointAt(trim.Domain().ParameterAt(i/4.0));auto point=brep->m_S[face.m_si]->PointAt(uv.x,uv.y);
            BRepExtrema_DistShapeShape distance(BRepBuilderAPI_MakeVertex(gp_Pnt(point.x,point.y,point.z)).Vertex(),shape);
            require(distance.IsDone(),"Independent native boundary distance must resolve");nativeBoundaryDistance=std::max(nativeBoundaryDistance,distance.Value());
        }}}}
        Bnd_Box before,after;BRepBndLib::AddOptimal(shape,before,false,false);BRepBndLib::AddOptimal(importBrep(*exported,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance),after,false,false);double a[6],z[6],maxBounds=0;before.Get(a[0],a[1],a[2],a[3],a[4],a[5]);after.Get(z[0],z[1],z[2],z[3],z[4],z[5]);for(int i=0;i<6;++i)maxBounds=std::max(maxBounds,std::abs(a[i]-z[i]));
        rows.append(QJsonObject{{"uuid",uuid},{"source_orientation",brep->SolidOrientation()},{"host_signed_volume",volume(shape)},{"native_boundary_host_distance",nativeBoundaryDistance},{"roundtrip_bounds_deviation",maxBounds},{"faces",faces}});
    }
    auto output=std::filesystem::path(OM9_RING_PRECISION_OUTPUT);std::ofstream stream(output,std::ios::binary);auto bytes=QJsonDocument(rows).toJson();stream.write(bytes.data(),bytes.size());stream.close();
    if(argc>1&&std::string(argv[1])=="--diagnose"){std::cout<<"diagnostics "<<output<<'\n';return 0;}
    if(argc>1&&std::string(argv[1])=="--uv"){
        for(auto row:rows)require(row.toObject()["roundtrip_bounds_deviation"].toDouble()<1e-3,"Real revolution BRep roundtrip must retain physical bounds");
        for(auto row:rows)require(row.toObject()["native_boundary_host_distance"].toDouble()<model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance,"Canonical real BRep must retain independently evaluated native boundaries within source model tolerance");
        for(auto row:rows)for(auto face:row.toObject()["faces"].toArray())require(face.toObject()["import_surface_uv_deviation"].toDouble()<1e-10,"Imported revolution must preserve native parameter evaluation, not only surface locus");
        int paths=0;
        for(bool transpose:{false,true})for(bool arcProfile:{false,true})for(int span=0;span<3;++span){
            ON_RevSurface surface;surface.m_axis=ON_Line(ON_3dPoint(2,-3,5),ON_3dPoint(2,-3,6));
            if(arcProfile){ON_Plane plane(ON_3dPoint(5,-3,5),ON_3dVector(1,0,0),ON_3dVector(0,0,1));auto arc=new ON_ArcCurve(ON_Arc(ON_Circle(plane,0.4),ON_Interval(0.2,1.9)));arc->SetDomain(11,29);surface.m_curve=arc;}
            else{auto line=new ON_LineCurve(ON_3dPoint(5,-3,5),ON_3dPoint(5,-3,7));line->SetDomain(11,29);surface.m_curve=line;}
            surface.m_angle=ON_Interval(0.37,span==0?1.1:(span==1?4.7:0.37+2*ON_PI));surface.m_t=ON_Interval(-7,13);surface.m_bTransposed=transpose;
            require(surface.IsValid(),"Synthetic revolution fixture");auto imported=importSurface(surface);
            double umin,umax,vmin,vmax;imported->Bounds(umin,umax,vmin,vmax);auto profileDomain=arcProfile?ON_ArcCurve::Cast(surface.m_curve)->m_arc.Domain():surface.m_curve->Domain();
            require(std::abs(umin-surface.m_angle.Min())<1e-10&&std::abs(umax-surface.m_angle.Max())<1e-10&&std::abs(vmin-profileDomain.Min())<1e-10&&std::abs(vmax-profileDomain.Max())<1e-10,"Imported analytic surface must retain mapped native domain bounds");
            for(int i=0;i<=16;++i)for(int j=0;j<=16;++j){double u=surface.Domain(0).ParameterAt(i/16.0),v=surface.Domain(1).ParameterAt(j/16.0);auto uv=importedSurfaceParameters(surface,u,v);auto actual=imported->Value(uv[0],uv[1]);require(surface.PointAt(u,v).DistanceTo(ON_3dPoint(actual.X(),actual.Y(),actual.Z()))<1e-10,"Independent revolution/arc/scaled/transposed evaluation");}
            ON_NurbsCurve trim(2,true,3,3);trim.SetCV(0,ON_4dPoint(surface.Domain(0).Min(),surface.Domain(1).Min(),0,1));trim.SetCV(1,ON_4dPoint(surface.Domain(0).Mid()*0.7,surface.Domain(1).Mid()*0.7,0,0.7));trim.SetCV(2,ON_4dPoint(surface.Domain(0).Max(),surface.Domain(1).Max(),0,1));trim.SetKnot(0,0);trim.SetKnot(1,0);trim.SetKnot(2,1);trim.SetKnot(3,1);require(trim.IsValid(),"Synthetic rational trim");auto mapped=importedSurfaceTrim(surface,trim);
            for(int i=0;i<=32;++i){auto point=trim.PointAt(i/32.0);auto expected=surface.PointAt(point.x,point.y);auto q=mapped->Value(i/32.0);auto actual=imported->Value(q.X(),q.Y());require(expected.DistanceTo(ON_3dPoint(actual.X(),actual.Y(),actual.Z()))<1e-10,"Mapped rational trim must retain surface evaluation");}
            if(span<2){
                std::unique_ptr<ON_Brep> native(surface.BrepForm());require(native&&native->IsValid()&&ON_RevSurface::Cast(native->m_S[0]),"Synthetic trimmed native revolution BRep");
                auto host=importBrep(*native,1e-6);GProp_GProps area;BRepGProp::SurfaceProperties(host,area,1e-10);
                auto expected=surface.m_angle.Length()*(arcProfile?0.4*(3*(1.9-0.2)+0.4*(std::sin(1.9)-std::sin(0.2))):6);
                std::cerr.precision(17);std::cerr<<"Analytic area transpose="<<transpose<<" arc="<<arcProfile<<" span="<<span<<" expected="<<expected<<" actual="<<area.Mass()<<'\n';
                require(std::abs(area.Mass()-expected)<1e-6,"Transposed/native trim must retain independently computed surface area within model tolerance");
                TopExp_Explorer face(host,TopAbs_FACE);require(face.More(),"Imported trimmed face");auto center=importedSurfaceParameters(surface,surface.Domain(0).Mid(),surface.Domain(1).Mid());
                auto interior=imported->Value(center[0],center[1]);BRepClass_FaceClassifier inside(TopoDS::Face(face.Current()),interior,1e-7);require(inside.State()==TopAbs_IN,"Mapped loop must enclose native physical interior");
                // Compare physical native points with the final canonical BRep,
                // independently of its reparameterized NURBS UV and roundtrip.
                for(int i=1;i<=5;++i)for(int j=1;j<=5;++j){
                    auto point=surface.PointAt(surface.Domain(0).ParameterAt(i/6.0),surface.Domain(1).ParameterAt(j/6.0));
                    BRepExtrema_DistShapeShape distance(BRepBuilderAPI_MakeVertex(gp_Pnt(point.x,point.y,point.z)).Vertex(),host);
                    require(distance.IsDone()&&distance.Value()<1e-6,"Canonical BRep must retain independent native physical samples");
                }
                auto exterior=imported->Value(surface.m_angle.Max()+0.1,center[1]);
                // FaceClassifier projects physical points onto the finite surface;
                // its UV classification alone cannot establish physical exclusion.
                BRepExtrema_DistShapeShape outside(BRepBuilderAPI_MakeVertex(exterior).Vertex(),host);
                require(outside.IsDone()&&outside.Value()>1e-3,"Mapped loop must exclude physical point outside trim");
                auto roundtrip=exportBrep(host,1e-6);GProp_GProps after;BRepGProp::SurfaceProperties(importBrep(*roundtrip,1e-6),after,1e-10);
                std::cerr.precision(17);std::cerr<<"BRep path transpose="<<transpose<<" arc="<<arcProfile<<" span="<<span<<" expected="<<expected<<" import="<<area.Mass()<<" reimport="<<after.Mass()<<'\n';
                // Export constructs trim endpoints at the requested 1e-6 model
                // tolerance; native analytic evaluation is checked separately.
                require(std::abs(after.Mass()-expected)<1e-6,"Trimmed revolution export/reimport area within fixture model tolerance");
            }
            ++paths;
        }
        require(paths==12,"Synthetic paths count");std::cout<<"Native revolution parameter evaluation PASS: 12 source revolution faces, 12 synthetic scaled/transposed line/arc surfaces and rational trims\n";return 0;
    }
    auto outward=exportBrep(BRepPrimAPI_MakeBox(2,3,4).Shape(),1e-7);require(volume(importBrep(*outward,1e-7))>23.999,"Outward source volume");
    outward->Flip();outward->SetSolidOrientationForExperts(-1);require(outward->IsValid()&&outward->IsSolid(),"Inward source valid solid");auto inward=importBrep(*outward,1e-7);
    require(volume(inward)<-23.999,"Import must retain inward solid orientation and signed volume");
    auto copy=exportBrep(inward,1e-7);require(volume(importBrep(*copy,1e-7))<-23.999,"Export/reimport must retain inward orientation");
    for(int orientation:{-1,1})for(int reflection:{-1,1}){
        auto native=exportBrep(BRepPrimAPI_MakeBox(2,3,4).Shape(),1e-7);if(orientation==-1){native->Flip();native->SetSolidOrientationForExperts(-1);}
        ON_Xform affine=ON_Xform::IdentityTransformation;affine[0][0]=reflection*2;affine[1][1]=3;affine[2][2]=4;affine[0][1]=0.25;affine[0][3]=19;
        transformNativeGeometry(*native,affine);require(native->SolidOrientation()==orientation*reflection,"Declared orientation through nonuniform/sheared/reflected affine map");
        require(std::abs(volume(importBrep(*native,1e-7))-orientation*reflection*576)<1e-7,"Affine signed volume agrees independent determinant");
    }
    // Independent SDK construction has no Rhino orientation cache. Face winding,
    // not a blanket outward convention, determines its signed solid direction.
    ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(2,0,0),ON_3dPoint(2,3,0),ON_3dPoint(0,3,0),ON_3dPoint(0,0,4),ON_3dPoint(2,0,4),ON_3dPoint(2,3,4),ON_3dPoint(0,3,4)};
    for(int orientation:{-1,1})for(int reflection:{-1,1}){
        std::unique_ptr<ON_Brep> unknown(ON_BrepBox(corners));if(orientation==-1)unknown->Flip();
        require(unknown->IsValid()&&unknown->SolidOrientation()==2,"Independent solid must exercise SDK +2");
        auto crc=unknown->DataCRC(0);auto raw=importBrep(*unknown,1e-7);
        require(std::abs(volume(raw)-orientation*24)<1e-7,"Unknown solid import preserves independent native winding");
        require(unknown->SolidOrientation()==2&&unknown->DataCRC(0)==crc,"Unknown import does not change native source");
        ON_Brep transformed(*unknown);ON_Xform affine=ON_Xform::IdentityTransformation;
        affine[0][0]=reflection*2;affine[1][1]=3;affine[2][2]=4;affine[0][1]=0.25;affine[0][3]=19;
        transformNativeGeometry(transformed,affine);
        require(transformed.SolidOrientation()==orientation*reflection,"Unknown sign resolved before affine transformation");
        require(std::abs(volume(importBrep(transformed,1e-7))-orientation*reflection*576)<1e-7,"Unknown affine signed volume agrees independent determinant");
        ON_Xform mirror=ON_Xform::IdentityTransformation;mirror[0][0]=-1;transformNativeGeometry(transformed,mirror);
        require(std::abs(volume(importBrep(transformed,1e-7))+orientation*reflection*576)<1e-7,"Second reflection restores signed parity");
        auto roundtrip=exportBrep(importBrep(transformed,1e-7),1e-7);
        require(std::abs(volume(importBrep(*roundtrip,1e-7))+orientation*reflection*576)<1e-7,"Resolved unknown sign survives native export and reimport");
        require(unknown->SolidOrientation()==2&&unknown->DataCRC(0)==crc,"Affine working copy keeps original unknown source immutable");
    }
    std::cout<<"Signed solid orientation PASS including unknown SDK +2\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
