// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreTopologySnapshot.h"
#include "CoreSupportSnapshot.h"
#include "CoreEdgeSnapshot.h"
#include "NativeInputTree.h"
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Curve.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <QJsonArray>
#include <cstddef>
#include <cstdint>
#include <map>
#include <stdexcept>
#include <type_traits>
#include <vector>

namespace OpenMatrix9Gui {
namespace {
using ShapeMap = NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>;
struct Face {std::uint64_t surface_id;std::uint32_t orientation,reserved;std::size_t first_loop,loop_count;};
struct Loop {std::uint32_t role,reserved;std::size_t first_use,use_count;};
struct Use {
    std::uint64_t edge_id,pcurve_id,start_vertex,end_vertex;
    std::uint32_t orientation,seam,degenerate,reserved;
    double domain[2],uv_start[2],uv_end[2];
};
struct Summary {
    std::uint32_t status,edge_manifold,edge_closed,faces,loops,edge_uses,edge_identities,boundary_components,degenerate_uses,
        vertex_manifold,face_oriented,vertex_count,boundary_vertex_count;
};
static_assert(std::is_standard_layout_v<Face> && sizeof(Face)==32 && alignof(Face)==8 && offsetof(Face,first_loop)==16);
static_assert(std::is_standard_layout_v<Loop> && sizeof(Loop)==24 && alignof(Loop)==8 && offsetof(Loop,first_use)==8);
static_assert(std::is_standard_layout_v<Use> && sizeof(Use)==96 && alignof(Use)==8 && offsetof(Use,domain)==48);
static_assert(sizeof(Summary)==52 && alignof(Summary)==4 && offsetof(Summary,vertex_manifold)==36);
extern "C" std::uint32_t om9_topology_counts(std::size_t,std::size_t,std::size_t);
extern "C" std::uint32_t om9_topology_domain(double,double);
extern "C" std::uint32_t om9_edge_geometry_counts(std::size_t,std::size_t);
extern "C" std::uint32_t om9_edge_tolerance(double);
extern "C" Summary om9_topology_inspect(const Face*,std::size_t,const Loop*,std::size_t,const Use*,std::size_t,std::uint32_t*,std::size_t);
void budget(std::size_t f,std::size_t l,std::size_t u) {
    if(om9_topology_counts(f,l,u)) throw std::runtime_error("Topology snapshot exceeds Rust record budget");
}
std::uint32_t orientation(TopAbs_Orientation value) {
    if(value==TopAbs_FORWARD)return 1;
    if(value==TopAbs_REVERSED)return 2;
    throw std::runtime_error("Unsupported internal/external orientation in topology snapshot");
}
QString identity(std::uint64_t value) {return QString::number(static_cast<qulonglong>(value));}
QJsonObject useJson(const Use& u,const QJsonObject& pcurve) {
    return {{"edge_id",identity(u.edge_id)},{"pcurve_id",identity(u.pcurve_id)},
        {"start_vertex_id",identity(u.start_vertex)},{"end_vertex_id",identity(u.end_vertex)},
        {"orientation",u.orientation==1?"forward":"reversed"},{"seam",bool(u.seam)},
        {"degenerate",bool(u.degenerate)},{"pcurve",pcurve}};
}
}

QJsonObject topologySnapshot(const TopoDS_Shape& shape) {
    if(shape.IsNull())throw std::runtime_error("Cannot inspect topology of a null shape");
    // Gate support trees before UV value queries. This does not certify raw
    // decoder allocation or full native kernel validity/manifoldness.
    ThreeDm::InputTree::validate(shape);
    std::vector<Face> faces;
    std::vector<Loop> loops;
    std::vector<Use> uses;
    std::vector<QJsonObject> pcurves;
    std::vector<QJsonObject> faceSupports;
    std::vector<double> faceTolerances;
    std::size_t supportPayloadBytes=0;
    ShapeMap nativeFaces,edges,vertices;
    QJsonArray faceOccurrences;
    for(TopExp_Explorer fi(shape,TopAbs_FACE);fi.More();fi.Next()) {
        budget(static_cast<std::size_t>(faceOccurrences.size())+1,loops.size(),uses.size());
        const auto original=TopoDS::Face(fi.Current());
        const auto faceOrientation=orientation(original.Orientation());
        const int existing=nativeFaces.FindIndex(original);
        const int faceId=existing?existing:nativeFaces.Add(original);
        faceOccurrences.append(QJsonObject{{"face_id",identity(faceId)},
            {"orientation",faceOrientation==1?"forward":"reversed"}});
        // FreeCAD's Faces contains unique TShape+location identities. Record
        // repeated compound references separately; they must not add fictitious
        // adjacency and turn a sheet's free boundaries into a closed shell.
        if(existing)continue;
        // Face orientation belongs to the face row. Forward support traversal
        // keeps loop orientation and seam pcurve side independent of it.
        const auto face=TopoDS::Face(original.Oriented(TopAbs_FORWARD));
        const double tolerance=BRep_Tool::Tolerance(face);
        if(om9_edge_tolerance(tolerance))throw std::runtime_error("Invalid native face tolerance");
        faceTolerances.push_back(tolerance);
        faceSupports.push_back(faceSupportSnapshot(face,supportPayloadBytes));
        Face row{static_cast<std::uint64_t>(faceId),faceOrientation,0,loops.size(),0};
        const auto outer=BRepTools::OuterWire(face);
        for(TopExp_Explorer wi(face,TopAbs_WIRE);wi.More();wi.Next()) {
            budget(static_cast<std::size_t>(faceOccurrences.size()),loops.size()+1,uses.size());
            const auto wire=TopoDS::Wire(wi.Current());
            Loop loop{wire.IsSame(outer)?1u:2u,0,uses.size(),0};
            // WireExplorer can silently omit disconnected/branched/internal
            // occurrences. Compare its entire oriented multiset to direct
            // native children, rather than certify its partial traversal.
            std::map<std::pair<int,std::uint32_t>,std::size_t> remaining;
            std::size_t directCount=0;
            for(TopoDS_Iterator child(wire);child.More();child.Next()) {
                budget(static_cast<std::size_t>(faceOccurrences.size()),loops.size()+1,uses.size()+ ++directCount);
                if(child.Value().ShapeType()!=TopAbs_EDGE)throw std::runtime_error("Unsupported non-edge wire child");
                const auto e=TopoDS::Edge(child.Value());
                ++remaining[{edges.Add(e),orientation(e.Orientation())}];
            }
            for(BRepTools_WireExplorer ei(wire,face);ei.More();ei.Next()) {
                budget(static_cast<std::size_t>(faceOccurrences.size()),loops.size()+1,uses.size()+1);
                const auto edge=ei.Current();
                const auto side=orientation(edge.Orientation());
                const int edgeId=edges.FindIndex(edge);
                auto found=remaining.find({edgeId,side});
                if(found==remaining.end()||!found->second)throw std::runtime_error("Invalid ordered wire occurrence");
                --found->second;
                double first=0,last=0;
                auto pc=BRep_Tool::CurveOnSurface(edge,face,first,last);
                if(pc.IsNull())throw std::runtime_error("Missing face pcurve in topology snapshot");
                if(om9_topology_domain(first,last))throw std::runtime_error("Invalid pcurve restriction domain in topology snapshot");
                ThreeDm::InputTree::validate(pc);
                const auto a=pc->Value(side==1?first:last),b=pc->Value(side==1?last:first);
                TopoDS_Vertex start,end;
                TopExp::Vertices(edge,start,end,true);
                if(start.IsNull()||end.IsNull())throw std::runtime_error("Missing trim endpoint topology identity");
                Use use{static_cast<std::uint64_t>(edgeId),uses.size()+1,
                    static_cast<std::uint64_t>(vertices.Add(start)),static_cast<std::uint64_t>(vertices.Add(end)),
                    side,static_cast<std::uint32_t>(BRep_Tool::IsClosed(edge,face)),
                    static_cast<std::uint32_t>(BRep_Tool::Degenerated(edge)),0,
                    {first,last},{a.X(),a.Y()},{b.X(),b.Y()}};
                uses.push_back(use);
                QJsonObject pcurve{{"native_class",pc->DynamicType()->Name()},
                    {"domain",QJsonArray{first,last}},{"uv_start",QJsonArray{a.X(),a.Y()}},
                    {"uv_end",QJsonArray{b.X(),b.Y()}},{"coordinates","face_uv"},
                    {"support_scope","supporting_geometry"}};
                const auto support=curveSupportSnapshot(pc,supportPayloadBytes);
                for(auto it=support.begin();it!=support.end();++it)pcurve[it.key()]=it.value();
                pcurves.push_back(pcurve);
            }
            for(const auto& [key,count]:remaining)if(count)throw std::runtime_error("Incomplete ordered wire traversal in topology snapshot");
            loop.use_count=uses.size()-loop.first_use;
            loops.push_back(loop);
        }
        row.loop_count=loops.size()-row.first_loop;
        faces.push_back(row);
    }
    // Retain loose edges/vertices in mixed compounds and standalone Part roots,
    // without adding them to face incidence or altering existing use identities.
    for(TopExp_Explorer it(shape,TopAbs_EDGE);it.More();it.Next())if(!edges.Contains(it.Current())){
        if(om9_edge_geometry_counts(edges.Extent()+1,vertices.Extent()))throw std::runtime_error("Edge/vertex snapshot exceeds Rust record budget");edges.Add(it.Current());}
    for(TopExp_Explorer it(shape,TopAbs_VERTEX);it.More();it.Next())if(!vertices.Contains(it.Current())){
        if(om9_edge_geometry_counts(edges.Extent(),vertices.Extent()+1))throw std::runtime_error("Edge/vertex snapshot exceeds Rust record budget");vertices.Add(it.Current());}
    const auto geometry=edgeGeometrySnapshot(shape,edges,vertices,supportPayloadBytes);
    std::vector<std::uint32_t> componentIds(uses.size());
    const auto summary=om9_topology_inspect(faces.data(),faces.size(),loops.data(),loops.size(),
        uses.data(),uses.size(),componentIds.data(),componentIds.size());
    if(summary.status)throw std::runtime_error("Rust rejected topology snapshot (status "+std::to_string(summary.status)+")");
    QJsonArray jsonFaces;
    ShapeMap enumeratedFaces;TopExp::MapShapes(shape,TopAbs_FACE,enumeratedFaces);
    for(std::size_t faceIndex=0;faceIndex<faces.size();++faceIndex) {
        const auto& face=faces[faceIndex];
        QJsonArray jsonLoops;
        for(std::size_t li=face.first_loop;li<face.first_loop+face.loop_count;++li) {
            const auto& l=loops[li];QJsonArray jsonUses;
            for(std::size_t ui=l.first_use;ui<l.first_use+l.use_count;++ui)jsonUses.append(useJson(uses[ui],pcurves[ui]));
            jsonLoops.append(QJsonObject{{"role",l.role==1?"outer":"hole"},{"edge_uses",jsonUses}});
        }
        auto jsonFace=faceSupports[faceIndex];
        jsonFace["face_id"]=identity(face.surface_id);jsonFace["surface_id"]=identity(face.surface_id);
        const auto nativeIndex=enumeratedFaces.FindIndex(nativeFaces(static_cast<int>(face.surface_id)));
        if(!nativeIndex)throw std::runtime_error("Native face enumeration identity changed");
        jsonFace["native_topology_index"]=identity(nativeIndex);
        jsonFace["orientation"]=face.orientation==1?"forward":"reversed";jsonFace["loops"]=jsonLoops;
        jsonFace["tolerance"]=faceTolerances[faceIndex];
        jsonFaces.append(jsonFace);
    }
    std::vector<QJsonArray> groups(summary.boundary_components);
    for(std::size_t i=0;i<uses.size();++i)if(componentIds[i])groups.at(componentIds[i]-1).append(useJson(uses[i],pcurves[i]));
    QJsonArray boundaries;
    for(const auto& group:groups)boundaries.append(QJsonObject{{"edge_uses",group},{"order","connectivity_component"}});
    return {{"faces",jsonFaces},{"face_occurrences",faceOccurrences},{"free_boundary_components",boundaries},
        {"edges_3d",geometry["edges_3d"]},{"vertices",geometry["vertices"]},
        {"edge_use_count",int(summary.edge_uses)},{"edge_identity_count",int(summary.edge_identities)},
        {"degenerate_use_count",int(summary.degenerate_uses)},
        {"support_payload_bytes",static_cast<qint64>(supportPayloadBytes)},
        {"edge_manifold",bool(summary.edge_manifold)},{"edge_closed",bool(summary.edge_closed)},
        {"surface_complex",QJsonObject{{"vertex_manifold",bool(summary.vertex_manifold)},{"face_oriented",bool(summary.face_oriented)},
            {"vertex_count",int(summary.vertex_count)},{"boundary_vertex_count",int(summary.boundary_vertex_count)},
            {"manifold",bool(summary.edge_manifold&&summary.vertex_manifold)},{"closed",bool(summary.edge_closed)},
            {"scope","recorded_unique_native_face_complex"},{"geometry_validity",QJsonValue(QJsonValue::Null)}}},
        {"incidence_scope","recorded_face_edge_uses"},{"identity_scope","snapshot_local_native_shape_and_location"},
        {"closed",QJsonValue(QJsonValue::Null)},{"manifold",QJsonValue(QJsonValue::Null)},
        {"solid_orientation",QJsonValue(QJsonValue::Null)}};
}
}
