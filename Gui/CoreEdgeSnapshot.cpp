// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreEdgeSnapshot.h"
#include "CoreSupportSnapshot.h"
#include <BRep_Tool.hxx>
#include <Geom_Curve.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonParseError>
#include <array>
#include <cstdint>
#include <stdexcept>
#include <vector>
namespace OpenMatrix9Gui {
namespace {
using Matrix=std::array<double,16>;
struct Edge {std::uint64_t edge_id,start_vertex,end_vertex;double domain[2],tolerance;std::uint32_t curve_present,degenerate,same_parameter,same_range;Matrix location;};
struct Vertex {std::uint64_t vertex_id;double point[3],tolerance;Matrix location;};
struct Input {const Edge* edges;std::size_t edge_count;const Vertex* vertices;std::size_t vertex_count;};
struct JsonSummary {std::uint32_t status,reserved;std::size_t needed;};
static_assert(sizeof(Edge)==192&&alignof(Edge)==8&&offsetof(Edge,location)==64);
static_assert(sizeof(Vertex)==168&&alignof(Vertex)==8&&offsetof(Vertex,location)==40);
static_assert(sizeof(Input)==32&&alignof(Input)==8&&sizeof(JsonSummary)==16);
extern "C" std::uint32_t om9_edge_geometry_counts(std::size_t,std::size_t);
extern "C" JsonSummary om9_edge_geometry_json(Input,std::uint8_t*,std::size_t);
extern "C" std::uint32_t om9_support_payload(std::size_t,std::size_t);
Matrix matrix(const TopLoc_Location& value){Matrix out{};const auto& t=value.Transformation();for(int r=1;r<=3;++r)for(int c=1;c<=4;++c)out[(r-1)*4+c-1]=t.Value(r,c);out[15]=1;return out;}
QJsonValue unknown(){return QJsonValue(QJsonValue::Null);}
}
QJsonObject edgeGeometrySnapshot(const TopoDS_Shape& shape,const TopologyShapeMap& edges,const TopologyShapeMap& vertices,std::size_t& used){
 if(om9_edge_geometry_counts(edges.Extent(),vertices.Extent()))throw std::runtime_error("Edge/vertex snapshot exceeds Rust record/retained budget");
 // MapShapes is bounded by the already collected unique rows of this immutable
 // source. Source indices are native enumeration metadata, separate from stable
 // within-snapshot edge/vertex IDs used by the face/pcurve occurrence graph.
 TopologyShapeMap nativeEdges,nativeVertices;TopExp::MapShapes(shape,TopAbs_EDGE,nativeEdges);TopExp::MapShapes(shape,TopAbs_VERTEX,nativeVertices);
 std::vector<Edge> es;std::vector<Vertex> vs;es.reserve(edges.Extent());vs.reserve(vertices.Extent());
 std::vector<Handle(Geom_Curve)> curves;curves.reserve(edges.Extent());
 for(int i=1;i<=vertices.Extent();++i){const auto vertex=TopoDS::Vertex(vertices(i));const auto p=BRep_Tool::Pnt(TopoDS::Vertex(vertex.Located(TopLoc_Location())));
  vs.push_back({static_cast<std::uint64_t>(i),{p.X(),p.Y(),p.Z()},BRep_Tool::Tolerance(vertex),matrix(vertex.Location())});}
 for(int i=1;i<=edges.Extent();++i){const auto edge=TopoDS::Edge(edges(i).Oriented(TopAbs_FORWARD));TopoDS_Vertex a,b;TopExp::Vertices(edge,a,b,true);
  if(a.IsNull()||b.IsNull())throw std::runtime_error("Missing native3D edge endpoint identity");
  const auto start=vertices.FindIndex(a),end=vertices.FindIndex(b);if(!start||!end)throw std::runtime_error("Missing native3D edge endpoint owner");
  double first=0,last=0;TopLoc_Location location;auto curve=BRep_Tool::Curve(edge,location,first,last);curves.push_back(curve);
  if(curve.IsNull()){first=0;last=0;location=edge.Location();}
  es.push_back({static_cast<std::uint64_t>(i),static_cast<std::uint64_t>(start),static_cast<std::uint64_t>(end),{first,last},BRep_Tool::Tolerance(edge),
   static_cast<std::uint32_t>(!curve.IsNull()),static_cast<std::uint32_t>(BRep_Tool::Degenerated(edge)),static_cast<std::uint32_t>(BRep_Tool::SameParameter(edge)),static_cast<std::uint32_t>(BRep_Tool::SameRange(edge)),matrix(location)});}
 const Input input{es.data(),es.size(),vs.data(),vs.size()};const auto query=om9_edge_geometry_json(input,nullptr,0);
 if(query.status||query.reserved||!query.needed)throw std::runtime_error("Rust rejected native3D edge/vertex geometry (status "+std::to_string(query.status)+")");
 if(om9_support_payload(used,query.needed))throw std::runtime_error("Edge/vertex numeric payload exceeds shared support budget");
 QByteArray raw(static_cast<qsizetype>(query.needed),'\0');const auto copied=om9_edge_geometry_json(input,reinterpret_cast<std::uint8_t*>(raw.data()),query.needed);
 if(copied.status||copied.reserved||copied.needed!=query.needed)throw std::runtime_error("Rust rejected native3D edge/vertex JSON output");
 QJsonParseError error;const auto parsed=QJsonDocument::fromJson(raw,&error);if(error.error!=QJsonParseError::NoError||!parsed.isObject())throw std::runtime_error("Invalid Rust edge/vertex geometry JSON");
 used+=query.needed;auto result=parsed.object();auto jsonEdges=result["edges_3d"].toArray();auto jsonVertices=result["vertices"].toArray();
 for(int i=0;i<jsonEdges.size();++i){auto row=jsonEdges[i].toObject();const int index=nativeEdges.FindIndex(edges(i+1));if(!index)throw std::runtime_error("Native edge enumeration identity changed");
  row["native_topology_index"]=QString::number(index);const auto& curve=curves[i];
  if(curve.IsNull()){row["native_class"]=unknown();row["geometry_data"]=unknown();row["geometry_data_status"]="absent_degenerate_curve3d";}
  else{row["native_class"]=curve->DynamicType()->Name();const auto support=curveSupportSnapshot(curve,used);for(auto it=support.constBegin();it!=support.constEnd();++it)row.insert(it.key(),it.value());}
  jsonEdges[i]=row;}
 for(int i=0;i<jsonVertices.size();++i){auto row=jsonVertices[i].toObject();const int index=nativeVertices.FindIndex(vertices(i+1));if(!index)throw std::runtime_error("Native vertex enumeration identity changed");row["native_topology_index"]=QString::number(index);jsonVertices[i]=row;}
 result["edges_3d"]=jsonEdges;result["vertices"]=jsonVertices;return result;
}
}
