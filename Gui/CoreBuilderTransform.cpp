// SPDX-License-Identifier: LGPL-2.1-or-later
// Same OCCT operators as FreeCAD TopoShape, retaining actual native history.
#include "CoreBuilderTransform.h"
#include "NativeInputTree.h"
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRep_Tool.hxx>
#include <BRepTools.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <GeomAPI_ProjectPointOnSurf.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Surface.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <Precision.hxx>
#include <Standard_Failure.hxx>
#include <QJsonArray>
#include <array>
#include <stdexcept>
namespace OpenMatrix9Gui {
namespace {
using ShapeMap=NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>;
extern "C" bool om9_builder_trace_counts(std::size_t,std::size_t,std::size_t,std::size_t);
void budget(std::size_t v,std::size_t e,std::size_t f,std::size_t s=0){
    if(!om9_builder_trace_counts(v,e,f,s))throw std::runtime_error("Builder native observation budget exceeded");
}
gp_GTrsf generalized(const Base::Matrix4D& m){gp_GTrsf t;for(int r=1;r<=3;++r)for(int c=1;c<=4;++c)t.SetValue(r,c,m[r-1][c-1]);return t;}
template<class Operator> void apply(BuilderTrace& trace,Operator& op){
    const auto result=op.Shape();
    for(auto& pair:trace.pairs){auto mapped=op.ModifiedShape(pair.current);
        if(mapped.IsNull()||mapped.ShapeType()!=pair.current.ShapeType())throw std::runtime_error("Native ModifiedShape lost or changed a template subshape");
        pair.current=mapped;
    }
    trace.shape=Part::TopoShape(result);
}
void copy(BuilderTrace& trace){BRepBuilderAPI_Copy op(trace.shape.getShape(),true,false);apply(trace,op);}
void affinity(BuilderTrace& trace,const Base::Matrix4D& matrix,bool duplicate){
    // Preserve TopoShape::transformGShape's extra copy before GTransform.
    copy(trace);BRepBuilderAPI_GTransform op(trace.shape.getShape(),generalized(matrix),duplicate);apply(trace,op);
}
void location(BuilderTrace& trace,const Base::Matrix4D& matrix){
    const auto old=trace.shape.getShape().Location();auto relocated=trace.shape;relocated.setTransform(matrix);
    const auto delta=relocated.getShape().Location()*old.Inverted();
    for(auto& pair:trace.pairs)pair.current=pair.current.Located(delta*pair.current.Location());
    trace.shape=relocated;
}
QJsonArray point(const gp_Pnt& p){return {p.X(),p.Y(),p.Z()};}
gp_Pnt point(const Base::Vector3d& p){return {p.x,p.y,p.z};}
Base::Vector3d vector(const gp_Pnt& p){return Base::Vector3d(p.X(),p.Y(),p.Z());}
unsigned ordinal(TopAbs_ShapeEnum type){if(type==TopAbs_VERTEX)return 0;if(type==TopAbs_EDGE)return 1;if(type==TopAbs_FACE)return 2;throw std::runtime_error("Unknown trace subshape kind");}
std::array<ShapeMap,3> maps(const TopoDS_Shape& shape){std::array<ShapeMap,3> result;
    for(const auto type:{TopAbs_VERTEX,TopAbs_EDGE,TopAbs_FACE})TopExp::MapShapes(shape,type,result[ordinal(type)]);
    return result;
}
}
BuilderTrace builderWorldTrace(const Part::TopoShape& source,const double* values,const Base::Placement& frame){
    ThreeDm::InputTree::validate(source.getShape());
    // Count before MapShapes allocates native maps; source is immutable here.
    std::size_t counts[3]={};
    for(const auto type:{TopAbs_VERTEX,TopAbs_EDGE,TopAbs_FACE})for(TopExp_Explorer it(source.getShape(),type);it.More();it.Next()){
        ++counts[ordinal(type)];budget(counts[0],counts[1],counts[2]);
    }
    BuilderTrace trace{source,source,{}};const auto sourceMaps=maps(source.getShape());
    for(unsigned kind=0;kind<3;++kind)for(int i=1;i<=sourceMaps[kind].Extent();++i)trace.pairs.push_back({sourceMaps[kind](i),sourceMaps[kind](i),unsigned(i)});
    copy(trace);Base::Matrix4D affine;
    for(unsigned i=0;i<3;++i){affine[i][i]=values[i];affine[i][3]=values[i+3];}
    affinity(trace,affine,false);
    // Existing builder uses transformShape(frame,true), a rigid copy transform.
    gp_Trsf rigid;const auto matrix=frame.toMatrix();rigid.SetValues(matrix[0][0],matrix[0][1],matrix[0][2],matrix[0][3],matrix[1][0],matrix[1][1],matrix[1][2],matrix[1][3],matrix[2][0],matrix[2][1],matrix[2][2],matrix[2][3]);
    BRepBuilderAPI_Transform op(trace.shape.getShape(),rigid,true);apply(trace,op);
    ThreeDm::InputTree::validate(trace.shape.getShape());return trace;
}
BuilderTrace builderLocalTrace(const BuilderTrace& world,const Base::Placement& global,const Base::Placement& own){
    auto trace=world;copy(trace);const auto embedded=trace.shape.getTransform();
    location(trace,Base::Matrix4D());affinity(trace,global.inverse().toMatrix()*embedded,true);
    location(trace,own.toMatrix());ThreeDm::InputTree::validate(trace.shape.getShape());return trace;
}
QJsonObject builderTraceObservations(const BuilderTrace& trace,const QJsonObject& inputSnapshot,
    const double* affine,const Base::Placement& sourceWorld,const Base::Placement& outputParent){
    const auto outputMaps=maps(trace.shape.getShape());QJsonArray pairs;std::size_t samples=0;
    const auto expected=[&](const gp_Pnt& p){auto v=vector(p);for(unsigned i=0;i<3;++i)v[i]=v[i]*affine[i]+affine[i+3];return point(outputParent.inverse().toMatrix()*(sourceWorld.toMatrix()*v));};
    const auto sample=[&](const gp_Pnt& source,const gp_Pnt& output){budget(outputMaps[0].Extent(),outputMaps[1].Extent(),outputMaps[2].Extent(),++samples);return QJsonObject{{"source_point",point(source)},{"output_point",point(output)}};};
    for(const auto& pair:trace.pairs){
        const unsigned kind=ordinal(pair.source.ShapeType());const auto index=outputMaps[kind].FindIndex(pair.current);
        if(!index)throw std::runtime_error("ModifiedShape is not in the published native output");
        QJsonObject row{{"kind",kind==0?"vertex":kind==1?"edge":"face"},{"source_index",int(pair.sourceIndex)},{"output_index",index}};QJsonArray observations,failures;
        if(kind==0)observations.append(sample(BRep_Tool::Pnt(TopoDS::Vertex(pair.source)),BRep_Tool::Pnt(TopoDS::Vertex(pair.current))));
        else if(kind==1){
            TopLoc_Location aLocation,bLocation;double a0=0,a1=0,b0=0,b1=0;
            const auto a=BRep_Tool::Curve(TopoDS::Edge(pair.source),aLocation,a0,a1),b=BRep_Tool::Curve(TopoDS::Edge(pair.current),bLocation,b0,b1);
            if(a.IsNull()!=b.IsNull())throw std::runtime_error("Native transform changed presence of an edge support");
            row["source_class"]=a.IsNull()?QJsonValue(QJsonValue::Null):QJsonValue(a->DynamicType()->Name());row["output_class"]=b.IsNull()?QJsonValue(QJsonValue::Null):QJsonValue(b->DynamicType()->Name());
            if(!a.IsNull()){
                row["source_domain"]=QJsonArray{a0,a1};row["output_domain"]=QJsonArray{b0,b1};
                for(unsigned i=0;i<17;++i){const auto parameter=a0+(a1-a0)*(double(i)/16.);const auto p=a->Value(parameter).Transformed(aLocation.Transformation());
                    const auto projected=expected(p).Transformed(bLocation.Inverted().Transformation());
                    double mapped=parameter;int solutions=0;bool evaluated=false;const char* unavailable=nullptr;gp_Pnt q;
                    // Query the actual output before invoking a distance-based
                    // projector, whose squared distances can overflow on large
                    // valid geometry. This certifies only this recorded query.
                    try{
                        if(parameter>=b0&&parameter<=b1){const auto candidate=b->Value(parameter);
                            evaluated=candidate.X()==projected.X()&&candidate.Y()==projected.Y()&&candidate.Z()==projected.Z();}
                        if(!evaluated){GeomAPI_ProjectPointOnCurve query(projected,b,b0,b1);
                            if(!query.NbPoints())unavailable="no_projection_solution";
                            else{mapped=query.LowerDistanceParameter();solutions=query.NbPoints();}}
                        if(!unavailable)q=b->Value(mapped).Transformed(bLocation.Transformation());
                    }catch(const Standard_Failure&){unavailable="native_projection_failure";}
                    if(unavailable){
                        budget(outputMaps[0].Extent(),outputMaps[1].Extent(),outputMaps[2].Extent(),++samples);
                        failures.append(QJsonObject{{"source_parameter",parameter},{"source_point",point(p)},
                            {"error","KernelFailure"},{"reason",unavailable}});continue;
                    }
                    auto value=sample(p,q);value["source_parameter"]=parameter;value["output_parameter"]=mapped;
                    value["query_method"]=evaluated?"verified_native_evaluation":"native_projection";
                    // Multiple solutions do not prove a unique global mapping.
                    value["projection_solution_count"]=evaluated?QJsonValue(QJsonValue::Null):QJsonValue(solutions);observations.append(value);
                }
            }
        }else{
            const auto aFace=TopoDS::Face(pair.source),bFace=TopoDS::Face(pair.current);TopLoc_Location aLocation,bLocation;
            const auto a=BRep_Tool::Surface(aFace,aLocation),b=BRep_Tool::Surface(bFace,bLocation);
            if(a.IsNull()||b.IsNull())throw std::runtime_error("Native face mapping lost a surface support");
            row["source_class"]=a->DynamicType()->Name();row["output_class"]=b->DynamicType()->Name();
            double u0,u1,v0,v1,bu0,bu1,bv0,bv1;BRepTools::UVBounds(aFace,u0,u1,v0,v1);BRepTools::UVBounds(bFace,bu0,bu1,bv0,bv1);
            row["source_uv_domain"]=QJsonArray{u0,u1,v0,v1};row["output_uv_domain"]=QJsonArray{bu0,bu1,bv0,bv1};
            for(unsigned i=0;i<5;++i)for(unsigned j=0;j<5;++j){const double u=u0+(u1-u0)*(i+.5)/5.,v=v0+(v1-v0)*(j+.5)/5.;
                BRepClass_FaceClassifier inside(aFace,gp_Pnt2d(u,v),Precision::PConfusion());if(inside.State()!=TopAbs_IN&&inside.State()!=TopAbs_ON)continue;
                const auto p=a->Value(u,v).Transformed(aLocation.Transformation());const auto projected=expected(p).Transformed(bLocation.Inverted().Transformation());
                const char* unavailable=nullptr;double mappedU=0,mappedV=0;gp_Pnt q;
                try{
                    GeomAPI_ProjectPointOnSurf query(projected,b,bu0,bu1,bv0,bv1);
                    if(!query.NbPoints())unavailable="no_projection_solution";
                    else{query.LowerDistanceParameters(mappedU,mappedV);
                        BRepClass_FaceClassifier targetInside(bFace,gp_Pnt2d(mappedU,mappedV),Precision::PConfusion());
                        if(targetInside.State()!=TopAbs_IN&&targetInside.State()!=TopAbs_ON)unavailable="projected_point_outside_trim";
                        else q=b->Value(mappedU,mappedV).Transformed(bLocation.Transformation());}
                }catch(const Standard_Failure&){unavailable="native_projection_failure";}
                if(unavailable){
                    budget(outputMaps[0].Extent(),outputMaps[1].Extent(),outputMaps[2].Extent(),++samples);
                    failures.append(QJsonObject{{"source_uv",QJsonArray{u,v}},{"source_point",point(p)},
                        {"error","KernelFailure"},{"reason",unavailable}});continue;
                }
                auto value=sample(p,q);
                value["source_uv"]=QJsonArray{u,v};value["output_uv"]=QJsonArray{mappedU,mappedV};observations.append(value);
            }
        }
        row["samples"]=observations;if(!failures.isEmpty())row["query_failures"]=failures;pairs.append(row);
    }
    return {{"schema_version",1},{"method","composed_occt_modified_shape"},{"input_snapshot",inputSnapshot},{"pairs",pairs}};
}
}
