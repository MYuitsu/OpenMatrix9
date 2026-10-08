#include "ThreeDmArchive.h"
#include <BRepBndLib.hxx>
#include <Bnd_Box.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <BRepClass_FaceClassifier.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepLib.hxx>
#include <fstream>
#include <functional>
#include <limits>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static TopoDS_Face parameterFace(const ON_Brep& brep,int fi){
    auto& native=brep.m_F[fi];auto source=brep.m_S[native.m_si];auto surface=importSurface(*source);
    BRep_Builder builder;TopoDS_Face face;builder.MakeFace(face,surface,1e-7);
    auto rev=ON_RevSurface::Cast(source);bool transpose=rev&&rev->m_bTransposed;
    for(int li=0;li<native.m_li.Count();++li){auto& loop=brep.m_L[native.m_li[li]];BRepBuilderAPI_MakeWire wire;
        for(int ti=0;ti<loop.m_ti.Count();++ti){auto& trim=brep.m_T[loop.m_ti[ti]];if(trim.m_type==ON_BrepTrim::singular)continue;
            auto pc=importedSurfaceTrim(*source,trim);auto& sourceEdge=brep.m_E[trim.m_ei];auto d=sourceEdge.Domain();
            BRepBuilderAPI_MakeEdge edge(curve3d(sourceEdge),d.Min(),d.Max());auto e=edge.Edge();if(trim.m_bRev3d)pc->Reverse();
            builder.UpdateEdge(e,pc,surface,TopLoc_Location(),1e-7);builder.Range(e,surface,TopLoc_Location(),trim.Domain().Min(),trim.Domain().Max());
            builder.SameRange(e,false);builder.SameParameter(e,false);BRepLib::SameParameter(e,1e-7);if(trim.m_bRev3d)e.Reverse();wire.Add(e);
        }
        if(!wire.IsDone())throw std::runtime_error("Diagnostic parameter face wire failed");auto boundary=wire.Wire();if(transpose)boundary.Reverse();builder.Add(face,boundary);
    }
    return face;
}
static QJsonObject interiorExtrema(const ON_Brep& brep,double tolerance){
    double bounds[6]={1e100,1e100,1e100,-1e100,-1e100,-1e100};QJsonArray witnesses{QJsonValue(),QJsonValue(),QJsonValue(),QJsonValue(),QJsonValue(),QJsonValue()};int accepted=0;
    for(int fi=0;fi<brep.m_F.Count();++fi){
        auto face=parameterFace(brep,fi);auto surface=brep.m_S[brep.m_F[fi].m_si];
        auto du=surface->Domain(0),dv=surface->Domain(1);double lu=du.Length(),lv=dv.Length();
        for(int axis=0;axis<3;++axis)for(int i=0;i<=16;++i)for(int j=0;j<=16;++j){
            double u=double(i)/16,v=double(j)/16;bool stationary=false;
            for(int iter=0;iter<40;++iter){
                ON_3dPoint p;ON_3dVector a,b,aa,ab,bb;if(!surface->Ev2Der(du.ParameterAt(u),dv.ParameterAt(v),p,a,b,aa,ab,bb))break;
                double gu=a[axis]*lu,gv=b[axis]*lv;if(std::hypot(gu,gv)<1e-10){stationary=true;break;}
                double h11=aa[axis]*lu*lu,h12=ab[axis]*lu*lv,h22=bb[axis]*lv*lv,det=h11*h22-h12*h12;
                if(std::abs(det)<1e-15)break;
                double stepU=(h22*gu-h12*gv)/det,stepV=(h11*gv-h12*gu)/det;
                double scale=1;while(scale>1e-8&&(u-scale*stepU<0||u-scale*stepU>1||v-scale*stepV<0||v-scale*stepV>1))scale*=0.5;
                if(scale<=1e-8)break;u-=scale*stepU;v-=scale*stepV;
            }
            if(!stationary)continue;auto p=surface->PointAt(du.ParameterAt(u),dv.ParameterAt(v));
            auto uv=importedSurfaceParameters(*surface,du.ParameterAt(u),dv.ParameterAt(v));
            BRepClass_FaceClassifier classifier(face,gp_Pnt2d(uv[0],uv[1]),1e-7);if(classifier.State()!=TopAbs_IN&&classifier.State()!=TopAbs_ON)continue;
            ++accepted;auto witness=QJsonObject{{"face",fi},{"stationary_axis",axis},{"uv",QJsonArray{du.ParameterAt(u),dv.ParameterAt(v)}},{"point",QJsonArray{p.x,p.y,p.z}}};
            for(int k=0;k<3;++k){if(p[k]<bounds[k]){bounds[k]=p[k];witnesses[k]=witness;}if(p[k]>bounds[k+3]){bounds[k+3]=p[k];witnesses[k+3]=witness;}}
        }
    }
    QJsonArray values;for(double value:bounds)values.append(value);return QJsonObject{{"bounds",values},{"accepted_stationary_candidates",accepted},{"witnesses",witnesses},{"scope","Native derivative roots filtered in original trim UV; diagnostic search, not certified exhaustive extrema"}};
}
static QJsonArray boxValues(const ON_Brep& brep,bool surfaceTrim){
    double bounds[6]={1e100,1e100,1e100,-1e100,-1e100,-1e100};
    auto add=[&](const ON_3dPoint& p){for(int k=0;k<3;++k){bounds[k]=std::min(bounds[k],p[k]);bounds[k+3]=std::max(bounds[k+3],p[k]);}};
    auto optimize=[&](const std::function<ON_3dPoint(double)>& point,ON_Interval domain){
        const int steps=256;
        for(int i=0;i<=steps;++i)add(point(domain.ParameterAt(double(i)/steps)));
        for(int k=0;k<3;++k)for(int sign:{-1,1})for(int i=1;i<steps;++i){
            auto eval=[&](double t){return sign*point(t)[k];};
            double a=domain.ParameterAt(double(i-1)/steps),b=domain.ParameterAt(double(i+1)/steps),mid=(a+b)/2;
            if(eval(mid)>eval(a)||eval(mid)>eval(b))continue;
            for(int n=0;n<65;++n){double c=a+(b-a)*0.3819660112501051,d=b-(b-a)*0.3819660112501051;if(eval(c)<eval(d))b=d;else a=c;}
            add(point((a+b)/2));
        }
    };
    if(surfaceTrim){
        for(int fi=0;fi<brep.m_F.Count();++fi){auto& face=brep.m_F[fi];auto surface=brep.m_S[face.m_si];
            for(int li=0;li<face.m_li.Count();++li){auto& loop=brep.m_L[face.m_li[li]];
                for(int ti=0;ti<loop.m_ti.Count();++ti){const auto& trim=brep.m_T[loop.m_ti[ti]];
                    optimize([&](double t){auto uv=trim.PointAt(t);return surface->PointAt(uv.x,uv.y);},trim.Domain());
                }
            }
        }
    }else for(int ei=0;ei<brep.m_E.Count();++ei){const auto& edge=brep.m_E[ei];optimize([&](double t){return edge.PointAt(t);},edge.Domain());}
    QJsonArray result;for(double value:bounds)result.append(value);return result;
}
int main(){try{
    ON::Begin();ONX_Model model;if(!model.Read(std::filesystem::path(OM9_RING_PRECISION_SOURCE).c_str()))return 1;
    QJsonArray rows;
    for(auto uuid:{"502f1501-11ed-4233-950b-2f9f97a5fdcf","f82df7e7-7ec2-4edf-b880-3fadf26c3c70"}){
        auto component=ON_ModelGeometryComponent::Cast(model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid)).ModelComponent());
        auto brep=component?ON_Brep::Cast(component->Geometry(nullptr)):nullptr;if(!brep)return 2;
        auto shape=importBrep(*brep,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance);
        auto exported=exportBrep(shape,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance);
        auto tight=[](const ON_Brep& geometry){ON_BoundingBox box;if(!geometry.GetTightBoundingBox(box))throw std::runtime_error("SDK tight bounds failed");return QJsonArray{box.m_min.x,box.m_min.y,box.m_min.z,box.m_max.x,box.m_max.y,box.m_max.z};};
        ON_Brep shrunk(*exported);bool shrink=shrunk.ShrinkSurfaces();
        QJsonArray faceDomains;for(int fi=0;fi<brep->m_F.Count();++fi){const auto& a=brep->m_F[fi];const auto& b=exported->m_F[fi];auto sa=brep->m_S[a.m_si],sb=exported->m_S[b.m_si];faceDomains.append(QJsonObject{{"face",fi},{"source_class",sa->ClassId()->ClassName()},{"source_u",QJsonArray{sa->Domain(0).Min(),sa->Domain(0).Max()}},{"source_v",QJsonArray{sa->Domain(1).Min(),sa->Domain(1).Max()}},{"export_u",QJsonArray{sb->Domain(0).Min(),sb->Domain(0).Max()}},{"export_v",QJsonArray{sb->Domain(1).Min(),sb->Domain(1).Max()}}});}
        Bnd_Box box;BRepBndLib::AddOptimal(shape,box,false,false);double bounds[6];box.Get(bounds[0],bounds[1],bounds[2],bounds[3],bounds[4],bounds[5]);QJsonArray host;for(double value:bounds)host.append(value);
        rows.append(QJsonObject{{"uuid",uuid},{"source_edge_extrema",boxValues(*brep,false)},{"source_surface_trim_extrema",boxValues(*brep,true)},
            {"export_edge_extrema",boxValues(*exported,false)},{"export_surface_trim_extrema",boxValues(*exported,true)},{"host_optimal_bounds",host},
            {"source_sdk_tight",tight(*brep)},{"export_sdk_tight",tight(*exported)},{"shrink_valid",shrink&&shrunk.IsValid()},{"shrunk_sdk_tight",tight(shrunk)},{"faces",faceDomains},
            {"source_interior",interiorExtrema(*brep,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance)},
            {"export_interior",interiorExtrema(*exported,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance)}});
    }
    auto bytes=QJsonDocument(rows).toJson();std::ofstream output(OM9_BOUNDS_DIAGNOSTIC_OUTPUT);output<<bytes.constData();std::cout<<"Bounds diagnostic saved: "<<OM9_BOUNDS_DIAGNOSTIC_OUTPUT;return 0;
}catch(const std::exception& error){std::cerr<<error.what();return 1;}}
