// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreShapeValidation.h"
#include "NativeInputTree.h"
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <Precision.hxx>
#include <TopExp_Explorer.hxx>
#include <QJsonArray>
#include <QSet>
#include <cstddef>
#include <stdexcept>
#include <type_traits>

namespace OpenMatrix9Gui {
namespace {
struct Input {
    std::uint32_t root_kind,native_valid,has_faces,edge_manifold,vertex_manifold,
        edge_closed,face_oriented,loose_geometry,duplicate_faces,native_orientation,shell_count;
};
struct Summary {
    std::uint32_t status,native_valid,manifold,closed,valid_solid,
        native_orientation,world_orientation,reasons;
};
static_assert(std::is_standard_layout_v<Input> && sizeof(Input)==44 && alignof(Input)==4 && offsetof(Input,shell_count)==40);
static_assert(std::is_standard_layout_v<Summary> && sizeof(Summary)==32 && alignof(Summary)==4 && offsetof(Summary,reasons)==28);
static_assert(TopAbs_SOLID==2 && TopAbs_VERTEX==7);
extern "C" Summary om9_shape_validation(Input,std::int32_t);
extern "C" std::uint32_t om9_shape_orientation(std::uint32_t,std::int32_t);
QJsonValue fact(std::uint32_t v) {return v?QJsonValue(v==2):QJsonValue(QJsonValue::Null);}
QJsonValue orientation(std::uint32_t v) {
    return v==1?QJsonValue("outward"):v==2?QJsonValue("inward"):QJsonValue(QJsonValue::Null);
}
}
QJsonObject shapeValidation(const TopoDS_Shape& shape,const QJsonObject& topology) {
    // Includes bounded depth/node/child totals, support owners and cycles before
    // native analyzer/classifier maps or geometry evaluation. No source repair.
    const auto bounded=ThreeDm::InputTree::validate(shape);
    const auto faces=topology.value("faces").toArray();
    const auto complex=topology.value("surface_complex").toObject();
    QSet<QString> faceEdges,faceVertices;
    for(const auto& f:faces)for(const auto& l:f.toObject().value("loops").toArray())
        for(const auto& u:l.toObject().value("edge_uses").toArray()) {
            const auto row=u.toObject();faceEdges.insert(row.value("edge_id").toString());
            faceVertices.insert(row.value("start_vertex_id").toString());
            faceVertices.insert(row.value("end_vertex_id").toString());
        }
    const bool loose=!faces.isEmpty() &&
        (faceEdges.size()!=topology.value("edges_3d").toArray().size() ||
         faceVertices.size()!=topology.value("vertices").toArray().size());
    std::uint32_t shells=0;
    for(TopExp_Explorer it(shape,TopAbs_SHELL);it.More();it.Next()) {
        if(++shells>16384)throw std::runtime_error("Native validation exceeds shell record budget");
    }
    BRepCheck_Analyzer analyzer(shape,true,false,true);
    const bool valid=analyzer.IsValid();
    std::uint32_t nativeOrientation=0;
    if(valid && shape.ShapeType()==TopAbs_SOLID && !faces.isEmpty() && shells) {
        BRepClass3d_SolidClassifier classifier(shape);
        classifier.PerformInfinitePoint(Precision::Confusion());
        if(classifier.State()==TopAbs_OUT)nativeOrientation=1;
        else if(classifier.State()==TopAbs_IN)nativeOrientation=2;
    }
    const Input input{static_cast<std::uint32_t>(shape.ShapeType()),valid?2u:1u,
        static_cast<std::uint32_t>(!faces.isEmpty()),static_cast<std::uint32_t>(topology.value("edge_manifold").toBool()),
        static_cast<std::uint32_t>(complex.value("vertex_manifold").toBool()),
        static_cast<std::uint32_t>(topology.value("edge_closed").toBool()),
        static_cast<std::uint32_t>(complex.value("face_oriented").toBool()),static_cast<std::uint32_t>(loose),
        static_cast<std::uint32_t>(topology.value("face_occurrences").toArray().size()!=faces.size()),nativeOrientation,shells};
    const auto checked=om9_shape_validation(input,1);
    if(checked.status)throw std::runtime_error("Rust rejected native shape-validation observations");
    static const char* reasons[]={"native_invalid","no_faces","edge_nonmanifold","vertex_nonmanifold",
        "open_boundary","inconsistent_face_orientation","loose_geometry","repeated_face_occurrences",
        "root_is_not_native_solid","native_validity_uninspected","solid_orientation_uninspected","no_shells"};
    QJsonArray diagnostics;
    for(unsigned n=0;n<std::size(reasons);++n)if(checked.reasons&(1u<<n))diagnostics.append(reasons[n]);
    return {{"schema_version",1},{"operation","InspectGeometry"},{"scope","native_shape_and_recorded_surface_complex"},
        {"native_valid",fact(checked.native_valid)},{"manifold",fact(checked.manifold)},
        {"closed",fact(checked.closed)},{"valid_solid",fact(checked.valid_solid)},
        {"native_solid_orientation",orientation(checked.native_orientation)},
        {"world_solid_orientation",orientation(checked.world_orientation)},
        {"native_orientation_code",int(checked.native_orientation)},{"reasons",diagnostics},
        {"shell_count",int(shells)},{"loose_geometry",loose},{"repeated_face_occurrences",bool(input.duplicate_faces)},
        {"native_method",QJsonObject{{"analyzer","BRepCheck_Analyzer"},{"geometry_controls",true},
            {"parallel",false},{"exact_curve_on_surface_requested",true},{"exact_curve_on_surface_scope","SameParameter_edges"},
            {"orientation_classifier","BRepClass3d_SolidClassifier.PerformInfinitePoint"},
            {"classifier_precision",Precision::Confusion()},{"precision_scope","native_algorithm_not_document_tolerance"}}},
        {"input_preflight",QJsonObject{{"nodes",static_cast<qint64>(bounded.nodes)},{"basis_bytes",static_cast<qint64>(bounded.bytes)}}},
        {"global_self_intersection_certificate",QJsonValue(QJsonValue::Null)},
        {"conversion_performed",false},{"source_modified",false}};
}
void reframeShapeValidation(QJsonObject& topology,std::int32_t sign) {
    if(!topology.value("validation").isObject())return;
    auto report=topology.value("validation").toObject();
    const int native=report.value("native_orientation_code").toInt(-1);
    const auto world=om9_shape_orientation(static_cast<std::uint32_t>(native),sign);
    if(world>2)throw std::runtime_error("Rust rejected solid-orientation frame");
    report["world_solid_orientation"]=orientation(world);
    report["world_orientation_sign"]=sign;
    topology["validation"]=report;
}
}
