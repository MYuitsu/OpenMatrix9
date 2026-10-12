#include "CoreSnapGeometry.h"
#include "SnapObjectInfo.h"
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <BRep_Tool.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepTools.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <cmath>
extern "C" bool om9_modeling_snap_allowed(unsigned,bool,bool,unsigned);
namespace OpenMatrix9Gui {
SnapCandidates boundedSnapCandidates(const App::DocumentObject* object,unsigned mode,std::size_t budget) {
    SnapCandidates result;
    const auto info=classifySnapObject(object);
    if(!om9_modeling_snap_allowed(info.kind,info.native_cad,info.preview,mode))return result;
    budget=std::min(budget,std::size_t(2048));
    if(!budget){result.complete=false;return result;}
    TopExp_Explorer edges(info.shape,TopAbs_EDGE);
    if(mode==8&&edges.More())return result;
    if(mode==2&&!edges.More())return result;
    auto append=[&](const gp_Pnt& point){
        if(!std::isfinite(point.X())||!std::isfinite(point.Y())||!std::isfinite(point.Z()))return;
        Base::Vector3d value;
        info.global_transform.multVec(Base::Vector3d(point.X(),point.Y(),point.Z()),value);
        if(std::isfinite(value.x)&&std::isfinite(value.y)&&std::isfinite(value.z))result.points.push_back(value);
    };
    auto visit=[&](){
        ++result.visited_topology;
        if(result.visited_topology>budget||result.points.size()>=budget){result.complete=false;return false;}
        return true;
    };
    if(mode==2||mode==8) {
        for(TopExp_Explorer vertices(info.shape,TopAbs_VERTEX);vertices.More();vertices.Next()) {
            if(!visit())break;
            append(BRep_Tool::Pnt(TopoDS::Vertex(vertices.Current())));
        }
    } else if(mode==4) {
        for(;edges.More();edges.Next()) {
            if(!visit())return result;
            try {
                BRepAdaptor_Curve curve(TopoDS::Edge(edges.Current()));
                const double first=curve.FirstParameter(),last=curve.LastParameter();
                if(!std::isfinite(first)||!std::isfinite(last)||last<=first)continue;
                const double length=GCPnts_AbscissaPoint::Length(curve,first,last);
                if(!std::isfinite(length)||length<=0)continue;
                GCPnts_AbscissaPoint half(curve,length/2.,first);
                if(half.IsDone())append(curve.Value(half.Parameter()));
            }catch(const Standard_Failure&){result.complete=false;return result;}
        }
        for(TopExp_Explorer faces(info.shape,TopAbs_FACE);faces.More();faces.Next()) {
            if(!visit())return result;
            const auto face=TopoDS::Face(faces.Current());double u0,u1,v0,v1;
            BRepTools::UVBounds(face,u0,u1,v0,v1);
            if(!std::isfinite(u0)||!std::isfinite(u1)||!std::isfinite(v0)||!std::isfinite(v1))continue;
            gp_Pnt2d uv(u0/2.+u1/2.,v0/2.+v1/2.);
            BRepClass_FaceClassifier classifier(face,uv,1e-7);
            if(classifier.State()==TopAbs_IN||classifier.State()==TopAbs_ON)
                append(BRepAdaptor_Surface(face).Value(uv.X(),uv.Y()));
        }
    }
    return result;
}
std::vector<Base::Vector3d> endCandidates(const App::DocumentObject* object){return boundedSnapCandidates(object,2,2048).points;}
std::vector<Base::Vector3d> midCandidates(const App::DocumentObject* object){return boundedSnapCandidates(object,4,2048).points;}
std::vector<Base::Vector3d> pointCandidates(const App::DocumentObject* object){return boundedSnapCandidates(object,8,2048).points;}
}
