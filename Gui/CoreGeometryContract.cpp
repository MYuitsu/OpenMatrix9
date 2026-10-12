// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreGeometryContract.h"
#include <Geom_OffsetCurve.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include "CoreTopologySnapshot.h"
#include "CoreShapeValidation.h"
#include "CoreSupportSnapshot.h"
#include "CoreSourceAttributes.h"
#include "CoreDefinitionSnapshot.h"
#include "CoreDiscreteSnapshot.h"
#include "CoreObjects.h"
#include "NurbsBasisAdapter.h"
#include "NativeInputTree.h"
#include "NativeSurfaceRequest.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/GeoFeatureGroupExtension.h>
#include <App/GroupExtension.h>
#include <App/Link.h>
#include <App/PropertyGeo.h>
#include <Base/Exception.h>
#include <Base/Type.h>
#include <Mod/Part/App/PartFeature.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Standard_Failure.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <QApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QThread>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iterator>
#include <stdexcept>
#include <type_traits>
#include <unordered_set>
#include <vector>

namespace OpenMatrix9Gui {
namespace {
// Matches the Rust repr(C) value-only gate; neither side retains pointers or
// frees the other's storage. Native exceptions stay here. Sources are read
// synchronously on the GUI thread and guarded before/after inspection.
struct DescriptorInput {
    ObjectStamp stamp;
    std::uint32_t legacy_kind, representation, known, values;
    double domains[4];
    std::uint32_t domain_count, form, facts_scope, domain_scope;
};
struct DescriptorSummary {
    std::uint32_t status, schema_version, representation, solid, known, values,
                  form, domain_count;
    double domains[4];
    std::uint32_t facts_scope, domain_scope;
};
static_assert(sizeof(std::uint64_t) == 8 && sizeof(double) == 8);
static_assert(std::is_standard_layout_v<ObjectStamp> && sizeof(ObjectStamp) == 40);
static_assert(std::is_standard_layout_v<DescriptorInput> && sizeof(DescriptorInput) == 104 &&
              alignof(DescriptorInput) == 8 && offsetof(DescriptorInput, stamp) == 0 &&
              offsetof(DescriptorInput, legacy_kind) == 40 && offsetof(DescriptorInput, representation) == 44 &&
              offsetof(DescriptorInput, known) == 48 && offsetof(DescriptorInput, values) == 52 &&
              offsetof(DescriptorInput, domains) == 56 && offsetof(DescriptorInput, domain_count) == 88 &&
              offsetof(DescriptorInput, form) == 92 && offsetof(DescriptorInput, facts_scope) == 96 &&
              offsetof(DescriptorInput, domain_scope) == 100);
static_assert(std::is_standard_layout_v<DescriptorSummary> && sizeof(DescriptorSummary) == 72 &&
              alignof(DescriptorSummary) == 8 && offsetof(DescriptorSummary, status) == 0 &&
              offsetof(DescriptorSummary, schema_version) == 4 && offsetof(DescriptorSummary, representation) == 8 &&
              offsetof(DescriptorSummary, solid) == 12 && offsetof(DescriptorSummary, known) == 16 &&
              offsetof(DescriptorSummary, values) == 20 && offsetof(DescriptorSummary, form) == 24 &&
              offsetof(DescriptorSummary, domain_count) == 28 && offsetof(DescriptorSummary, domains) == 32 &&
              offsetof(DescriptorSummary, facts_scope) == 64 && offsetof(DescriptorSummary, domain_scope) == 68);
extern "C" std::uint32_t om9_geometry_operation(std::uint32_t, std::uint32_t, std::uint32_t, std::uint32_t);
extern "C" DescriptorSummary om9_geometry_descriptor(DescriptorInput);
struct CompanionLimits {std::size_t max_poles,max_knots;};
static_assert(sizeof(CompanionLimits)==16&&alignof(CompanionLimits)==8);
extern "C" CompanionLimits om9_geometry_companion_limits();
extern "C" std::uint32_t om9_geometry_companion_counts(std::size_t,std::size_t,std::size_t,std::size_t,std::uint32_t);
struct FrameInput {std::array<double,16> parent,path,source,location;};
struct FrameSummary {std::uint32_t status,reserved;std::array<double,16> local_to_world;std::int32_t orientation_sign;std::uint32_t reserved2;};
static_assert(sizeof(FrameInput)==512&&alignof(FrameInput)==8&&offsetof(FrameInput,location)==384);
static_assert(sizeof(FrameSummary)==144&&alignof(FrameSummary)==8&&offsetof(FrameSummary,local_to_world)==8&&offsetof(FrameSummary,orientation_sign)==136);
extern "C" FrameSummary om9_geometry_frame(FrameInput);
constexpr std::array<double,16> identity{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1};

using ThreeDm::BasisContract::checkedBasis;

QJsonValue unknown() { return QJsonValue(QJsonValue::Null); }
QString text(const std::string& value) { return QString::fromUtf8(value.data(), static_cast<int>(value.size())); }
QJsonArray matrix(const std::array<double, 16>& values) {
    QJsonArray rows;
    for (unsigned row = 0; row < 4; ++row)
        rows.append(QJsonArray{values[row * 4], values[row * 4 + 1], values[row * 4 + 2], values[row * 4 + 3]});
    return rows;
}
std::array<double,16> nativeMatrix(const Base::Matrix4D& value) {
    std::array<double,16> result{};
    for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)result[r*4+c]=value[r][c];
    return result;
}
std::array<double,16> placement(App::DocumentObject& object) {
    auto* property=object.getPropertyByName<App::PropertyPlacement>("Placement");
    return property?nativeMatrix(property->getValue().toMatrix()):identity;
}
std::array<double,16> parentPlacement(App::DocumentObject& object) {
    // captureObject already bounded/preflighted the ancestor chain.
    auto* parent=App::GeoFeatureGroupExtension::getGroupOfObject(&object);
    return parent?nativeMatrix(parent->getExtensionByType<App::GeoFeatureGroupExtension>()->globalGroupPlacement().toMatrix()):identity;
}
std::array<double,16> observedMatrix(const QJsonValue& value) {
    if(!value.isArray()||value.toArray().size()!=4)throw std::runtime_error("Missing native support matrix");
    std::array<double,16> result{};const auto rows=value.toArray();
    for(int r=0;r<4;++r){if(!rows[r].isArray()||rows[r].toArray().size()!=4)throw std::runtime_error("Invalid native support matrix");
        const auto row=rows[r].toArray();for(int c=0;c<4;++c){if(!row[c].isDouble())throw std::runtime_error("Invalid native support matrix value");result[r*4+c]=row[c].toDouble();}
    }
    return result;
}
void frame(QJsonObject& result,FrameInput input,const char* coordinates,const char* locationKey=nullptr) {
    input.location=locationKey?observedMatrix(result[locationKey]):input.source;
    const auto checked=om9_geometry_frame(input);
    if(checked.status)throw std::runtime_error("Rust rejected native world frame (status "+std::to_string(checked.status)+")");
    result["local_to_world"]=matrix(checked.local_to_world);result["world_orientation_sign"]=checked.orientation_sign;
    result["coordinates"]=coordinates;
}
void reframe(QJsonObject& result,FrameInput input) {
    const auto representation=result["representation"].toString();
    if(representation=="unknown")return;
    if((representation=="point"||representation=="curve"||representation=="trimmed_face")&&result["brep_tool_location"].isArray())
        frame(result,input,"native_local","brep_tool_location");
    else if((representation=="mesh"||representation=="point_cloud")&&result.value("native_discrete_location").isArray())
        frame(result,input,"native_local","native_discrete_location");
    else frame(result,input,"object_local");
    auto topology=result["topology"].toObject();
    reframeShapeValidation(topology,result.value("world_orientation_sign").toInt());
    for(const char* key:{"faces","edges_3d","vertices"})if(topology.value(key).isArray()) {
        auto rows=topology.value(key).toArray();
        for(int i=0;i<rows.size();++i){auto row=rows[i].toObject();frame(row,input,"native_local","native_location");rows[i]=row;}
        topology[key]=rows;
    }
    result["topology"]=topology;
}
QJsonObject describe(App::Document&,const char*,const char*);
void reframeDefinitionLeaf(QJsonObject& result,App::DocumentObject& object,const std::array<double,16>& world) {
    reframe(result,FrameInput{identity,world,placement(object),identity});
}
QJsonArray location(const TopLoc_Location& value) {
    std::array<double, 16> values{};
    const auto& transform = value.Transformation();
    for (int row = 1; row <= 3; ++row) for (int col = 1; col <= 4; ++col) {
        const double coordinate = transform.Value(row, col);
        if (!std::isfinite(coordinate)) throw std::runtime_error("Nonfinite native support location");
        values[(row - 1) * 4 + col - 1] = coordinate;
    }
    values[15] = 1;
    return matrix(values);
}
QJsonObject basisStatus(const char* state, const char* reason) {
    const auto limits=om9_geometry_companion_limits();
    return {{"state", state}, {"reason", reason}, {"max_poles", static_cast<qint64>(limits.max_poles)},
            {"max_knots_per_axis", static_cast<qint64>(limits.max_knots)}};
}
void flag(DescriptorInput& input, unsigned bit, bool value) {
    input.known |= 1u << bit;
    if (value) input.values |= 1u << bit;
}
void domains(DescriptorInput& input, double a, double b, double c = 0, double d = 0) {
    const bool surface = input.representation == 4;
    if (std::isfinite(a) && std::isfinite(b) && a < b && std::isfinite(b - a) &&
        (!surface || (std::isfinite(c) && std::isfinite(d) && c < d && std::isfinite(d - c)))) {
        input.domains[0] = a; input.domains[1] = b;
        input.domains[2] = c; input.domains[3] = d;
        input.domain_count = surface ? 2 : 1;
        input.domain_scope = surface ? 3 : 2;
    }
}

// Scalar metadata can still be returned when the companion budget is exceeded.
// Check before checkedBasis allocates independent vectors, and before JSON arrays.
bool curveBudget(const Handle(Geom_BSplineCurve)& curve) {
    const int poleCount = curve->NbPoles(), knotCount = curve->NbKnots();
    if (poleCount <= 0 || knotCount <= 0) throw std::runtime_error("Invalid native curve basis counts");
    const auto status=om9_geometry_companion_counts(poleCount,0,knotCount,0,0);
    if(status>1)throw std::runtime_error("Rust rejected companion curve counts");
    return status==0;
}
bool surfaceBudget(const Handle(Geom_BSplineSurface)& surface) {
    const int u = surface->NbUPoles(), v = surface->NbVPoles();
    const int uk = surface->NbUKnots(), vk = surface->NbVKnots();
    if (u <= 0 || v <= 0 || uk <= 0 || vk <= 0) throw std::runtime_error("Invalid native surface basis counts");
    const auto status=om9_geometry_companion_counts(u,v,uk,vk,1);
    if(status>1)throw std::runtime_error("Rust rejected companion surface counts");
    return status==0;
}
void curveFacts(const TopoDS_Edge& edge, DescriptorInput& input, QJsonObject& result) {
    const auto edgeRows=result.value("topology").toObject().value("edges_3d").toArray();
    if(edgeRows.size()!=1)throw std::runtime_error("Invalid standalone edge support cardinality");
    const auto copiedEdge=edgeRows[0].toObject();
    const QJsonObject support{{"geometry_data",copiedEdge.value("geometry_data")},{"geometry_data_status",copiedEdge.value("geometry_data_status")}};
    // The owned edge row already selected the native location for absent
    // degenerate support. Do not rely on a Curve() out-parameter on that path.
    result["brep_tool_location"]=copiedEdge.value("native_location");
    for(auto it=support.begin();it!=support.end();++it)result[it.key()]=it.value();
    double first = 0, last = 0;
    TopLoc_Location nativeLocation;
    auto curve = BRep_Tool::Curve(edge, nativeLocation, first, last);
    result["restriction_facts"] = QJsonObject{{"closed", BRep_Tool::IsClosed(edge)},
        {"closed_evidence", "native_topology_endpoints"}, {"periodic", unknown()}};
    if (curve.IsNull()) return;
    domains(input, first, last);
    input.facts_scope = 1;
    // Do not apply native location or world placement to copied supporting poles.
    for (unsigned depth = 0; depth < 16; ++depth) {
        auto trimmed = Handle(Geom_TrimmedCurve)::DownCast(curve);
        if (trimmed.IsNull()) break;
        curve = trimmed->BasisCurve();
        if (depth == 15) throw std::runtime_error("Excessively nested trimmed curve support");
    }
    flag(input, 0, curve->IsClosed()); flag(input, 1, curve->IsPeriodic());
    auto spline = Handle(Geom_BSplineCurve)::DownCast(curve);
    if (!spline.IsNull()) {
        input.form = 2; flag(input, 2, spline->IsRational());
        if (!curveBudget(spline)) {
            result["basis_status"] = basisStatus("uninspected", "companion_limit");
            result["geometry_data_status"] = "basis_companion_limit";
            return;
        }
        for(auto it=support.begin();it!=support.end();++it)result[it.key()]=it.value();
        if(!support["geometry_data"].isObject()) {
            result["basis_status"] = basisStatus("uninspected", "companion_limit");
            return;
        }
        auto legacy=support["geometry_data"].toObject();legacy.remove("schema_version");
        legacy["rational"]=spline->IsRational();result["basis"]=legacy;
        result["support_payload_bytes"]=result.value("topology").toObject().value("support_payload_bytes");
        result["basis_status"] = basisStatus("inspected", "copied_validated_basis");
        result["basis_frame"] = "native_local";
        return;
    }
    if(!Handle(Geom_OffsetCurve)::DownCast(curve).IsNull())input.form=4;
    else switch (GeomAdaptor_Curve(curve).GetType()) {
        case GeomAbs_Line: case GeomAbs_Circle: case GeomAbs_Ellipse:
        case GeomAbs_Hyperbola: case GeomAbs_Parabola: input.form = 1; break;
        case GeomAbs_BezierCurve: input.form = 3; flag(input, 2, Handle(Geom_BezierCurve)::DownCast(curve)->IsRational()); break;
        default: break;
    }
    for(auto it=support.begin();it!=support.end();++it)result[it.key()]=it.value();
    result["support_payload_bytes"]=result.value("topology").toObject().value("support_payload_bytes");
}
void surfaceFacts(const TopoDS_Face& face, DescriptorInput& input, QJsonObject& result) {
    double u0 = 0, u1 = 0, v0 = 0, v1 = 0;
    BRepTools::UVBounds(face, u0, u1, v0, v1);
    domains(input, u0, u1, v0, v1);
    result["restriction_facts"] = QJsonObject{{"closed", QJsonArray{unknown(), unknown()}},
        {"periodic", QJsonArray{unknown(), unknown()}}};
    TopLoc_Location nativeLocation;
    auto surface = BRep_Tool::Surface(face, nativeLocation);
    result["brep_tool_location"] = location(nativeLocation);
    if (surface.IsNull()) return;
    const auto faces=result["topology"].toObject()["faces"].toArray();
    if(faces.size()!=1)throw std::runtime_error("Invalid face support snapshot cardinality");
    const auto support=faces[0].toObject();
    result["geometry_data"]=support["geometry_data"];
    result["geometry_data_status"]=support["geometry_data_status"];
    input.facts_scope = 1;
    for (unsigned depth = 0; depth < 16; ++depth) {
        auto trimmed = Handle(Geom_RectangularTrimmedSurface)::DownCast(surface);
        if (trimmed.IsNull()) break;
        surface = trimmed->BasisSurface();
        if (depth == 15) throw std::runtime_error("Excessively nested trimmed surface support");
    }
    flag(input, 0, surface->IsUClosed()); flag(input, 1, surface->IsVClosed());
    flag(input, 2, surface->IsUPeriodic()); flag(input, 3, surface->IsVPeriodic());
    auto spline = Handle(Geom_BSplineSurface)::DownCast(surface);
    if (!spline.IsNull()) {
        const bool rational = spline->IsURational() || spline->IsVRational();
        input.form = 2; flag(input, 4, rational);
        if (!surfaceBudget(spline)) {
            result["basis_status"] = basisStatus("uninspected", "companion_limit");
            return;
        }
        // A face already copied its support in the common topology capture.
        // Reuse that owned value; do not read/extract/serialize the basis again.
        if(!support["geometry_data"].isObject()) {
            result["basis_status"] = basisStatus("uninspected", "companion_limit");
            return;
        }
        auto legacy=support["geometry_data"].toObject();legacy.remove("schema_version");
        legacy["rational"]=rational;result["basis"]=legacy;
        result["basis_status"] = basisStatus("inspected", "copied_validated_basis");
        result["basis_frame"] = "native_local";
        return;
    }
    if(!Handle(Geom_OffsetSurface)::DownCast(surface).IsNull() || !Handle(Geom_SurfaceOfLinearExtrusion)::DownCast(surface).IsNull() || !Handle(Geom_SurfaceOfRevolution)::DownCast(surface).IsNull())input.form=4;
    else switch (GeomAdaptor_Surface(surface).GetType()) {
        case GeomAbs_Plane: case GeomAbs_Cylinder: case GeomAbs_Cone:
        case GeomAbs_Sphere: case GeomAbs_Torus: input.form = 1; break;
        case GeomAbs_BezierSurface: {
            const auto bezier=Handle(Geom_BezierSurface)::DownCast(surface);
            input.form=3;flag(input,4,bezier->IsURational()||bezier->IsVRational());break;
        }
        default: break;
    }
}
void classify(const TopoDS_Shape& shape, DescriptorInput& input) {
    if (shape.IsNull()) return;
    switch (shape.ShapeType()) {
        case TopAbs_VERTEX: input.legacy_kind = 1; input.representation = 1; break;
        case TopAbs_EDGE: input.legacy_kind = 2; input.representation = 2; break;
        case TopAbs_WIRE: input.legacy_kind = 2; input.representation = 3; break;
        case TopAbs_FACE: input.legacy_kind = 3; input.representation = 4; break;
        case TopAbs_SHELL: input.legacy_kind = 3; input.representation = 5; break;
        case TopAbs_SOLID: input.legacy_kind = 3; input.representation = 6; break;
        default: input.legacy_kind = 3; input.representation = 7; break;
    }
}
bool derived(const App::DocumentObject& object, const char* name) {
    const auto type = Base::Type::fromName(name);
    return !type.isBad() && object.getTypeId().isDerivedFrom(type);
}
QJsonValue boolean(const DescriptorSummary& summary, unsigned bit) {
    return summary.known & (1u << bit) ? QJsonValue((summary.values & (1u << bit)) != 0) : unknown();
}
QJsonObject describe(App::Document& doc, const char* objectName, const char* subelement) {
    const auto snapshot = captureObject(doc, objectName, subelement);
    verifyObject(doc, snapshot);
    auto* object = doc.getObject(objectName);
    if (!object) throw std::runtime_error("Source object disappeared during descriptor capture");
    if(object->isLink()||object->hasExtension(App::LinkBaseExtension::getExtensionClassTypeId())||(!dynamic_cast<Part::Feature*>(object)&&!derived(*object,"Mesh::Feature")&&
        !derived(*object,"Points::Feature")&&object->hasExtension(App::GroupExtension::getExtensionClassTypeId())))
        return definitionSnapshot(*object,snapshot,DefinitionReaders{describe,reframeDefinitionLeaf});
    DescriptorInput input{};
    input.stamp = snapshot.stamp;
    input.legacy_kind = 9;
    QJsonObject result{{"basis", unknown()}, {"basis_status", basisStatus("uninspected", "not_a_nurbs_support")},
                      {"basis_frame", unknown()}, {"brep_tool_location", unknown()}, {"restriction_facts", unknown()},
                      {"geometry_data", unknown()}, {"geometry_data_status", "uninspected_support_type"}};
    if (auto* feature = dynamic_cast<Part::Feature*>(object)) {
        const TopoDS_Shape shape = snapshot.subelement.empty() ? feature->Shape.getValue()
            : feature->Shape.getShape().getSubShape(snapshot.subelement.c_str());
        if (!snapshot.subelement.empty() && shape.IsNull()) throw std::runtime_error("Requested native subelement is missing");
        classify(shape, input);
        static const char* orientations[]={"forward","reversed","internal","external"};
        const auto side=static_cast<unsigned>(shape.Orientation());
        if(side>=std::size(orientations))throw std::runtime_error("Unknown native geometry orientation");
        result["geometry_orientation"]=orientations[side];
        if (om9_geometry_operation(1, input.legacy_kind, 0, 0) != 0)
            throw std::runtime_error("Rust rejected descriptor inspection operation");
        if (input.representation >= 1 && input.representation <= 7) {
            auto topology=topologySnapshot(shape);
            auto validation=shapeValidation(shape,topology);
            validation["input_identity"]=QJsonObject{{"document_uid",text(snapshot.documentUid)},
                {"object_name",text(snapshot.objectName)},{"subelement",text(snapshot.subelement)},
                {"generation",QString::number(static_cast<qulonglong>(snapshot.stamp.generation))},
                {"revision",QString::number(static_cast<qulonglong>(snapshot.stamp.revision))}};
            topology["validation"]=validation;
            topology["native_shape_is_solid"]=shape.ShapeType()==TopAbs_SOLID;
            result["topology"]=topology;
        }
        if(input.representation==1){
            const auto vertices=result.value("topology").toObject().value("vertices").toArray();
            if(vertices.size()!=1)throw std::runtime_error("Invalid standalone native point cardinality");
            const auto vertex=vertices[0].toObject();
            result["geometry_data"]=QJsonObject{{"kind","point"},{"point_xyz",vertex.value("point_xyz")},{"tolerance",vertex.value("tolerance")}};
            result["geometry_data_status"]="copied_validated_native_vertex";
            result["brep_tool_location"]=vertex.value("native_location");
        }
        if (input.representation == 2) curveFacts(TopoDS::Edge(shape), input, result);
        if (input.representation == 4) surfaceFacts(TopoDS::Face(shape), input, result);
    } else {
        if (!snapshot.subelement.empty()) throw std::runtime_error("No native subelement descriptor adapter for this representation");
        const bool mesh=derived(*object,"Mesh::Feature"),cloud=derived(*object,"Points::Feature");
        if(mesh||cloud){
            input.legacy_kind=mesh?4:5;input.representation=mesh?8:9;
            const auto copied=discreteSnapshot(*object,mesh);
            for(auto it=copied.constBegin();it!=copied.constEnd();++it)result.insert(it.key(),it.value());
        }
        result["basis_status"] = basisStatus("uninspected", "unsupported_basis_representation");
    }
    if (om9_geometry_operation(1, input.legacy_kind, 0, 0) != 0)
        throw std::runtime_error("Rust rejected descriptor inspection operation");
    const auto summary = om9_geometry_descriptor(input);
    if (summary.status != 0) throw std::runtime_error("Rust rejected native descriptor facts (status " + std::to_string(summary.status) + ")");
    static const char* names[] = {"unknown", "point", "curve", "polycurve", "trimmed_face", "shell", "solid", "brep", "mesh", "point_cloud"};
    if (summary.representation >= std::size(names)) throw std::runtime_error("Unsupported Rust descriptor representation");
    result["schema_version"] = static_cast<int>(summary.schema_version);
    result["representation"] = names[summary.representation];
    result["legacy_kind"] = static_cast<int>(input.legacy_kind);
    // Decimal strings preserve the complete unsigned identity domain in JSON.
    const auto& stamp = snapshot.stamp;
    result["stamp"] = QJsonObject{{"document", QString::number(static_cast<qulonglong>(stamp.document))},
        {"object", QString::number(static_cast<qulonglong>(stamp.object))},
        {"generation", QString::number(static_cast<qulonglong>(stamp.generation))},
        {"revision", QString::number(static_cast<qulonglong>(stamp.revision))},
        {"epoch", QString::number(static_cast<qulonglong>(stamp.epoch))}};
    result["object_name"] = text(snapshot.objectName);
    result["document_uid"] = text(snapshot.documentUid);
    result["subelement"] = text(snapshot.subelement);
    result["unit_context_snapshot"] = text(snapshot.unitContext);
    result["world_placement_provenance"] = matrix(snapshot.world);
    result["coordinates"] = unknown(); result["local_to_world"] = unknown();
    result["form"] = summary.form == 1 ? QJsonValue("analytic") : summary.form == 2 ? QJsonValue("nurbs") : summary.form == 3 ? QJsonValue("bezier") : summary.form == 4 ? QJsonValue("derived") : unknown();
    result["facts_scope"] = summary.facts_scope == 1 ? QJsonValue("supporting_geometry") :
        summary.facts_scope == 2 ? QJsonValue("restricted_geometry") : unknown();
    result["domain_scope"] = summary.domain_scope == 1 ? QJsonValue("supporting_geometry") :
        summary.domain_scope == 2 ? QJsonValue("edge_restriction") :
        summary.domain_scope == 3 ? QJsonValue("face_uv_restriction") : unknown();
    QJsonArray observedDomains;
    for (unsigned axisIndex = 0; axisIndex < summary.domain_count; ++axisIndex)
        observedDomains.append(QJsonArray{summary.domains[axisIndex * 2], summary.domains[axisIndex * 2 + 1]});
    result["domains"] = summary.domain_count ? QJsonValue(observedDomains) : unknown();
    QJsonObject facts{{"closed", unknown()}, {"periodic", unknown()}, {"rational", unknown()}};
    if (summary.representation == 2) {
        facts["closed"] = boolean(summary, 0); facts["periodic"] = boolean(summary, 1); facts["rational"] = boolean(summary, 2);
    } else if (summary.representation == 4) {
        facts["closed"] = QJsonArray{boolean(summary, 0), boolean(summary, 1)};
        facts["periodic"] = QJsonArray{boolean(summary, 2), boolean(summary, 3)};
        facts["rational"] = boolean(summary, 4);
    }
    result["facts"] = facts;
    QJsonObject topology = result.contains("topology") ? result["topology"].toObject() :
        QJsonObject{{"closed", unknown()}, {"manifold", unknown()}, {"solid_orientation", unknown()},
                    {"faces", unknown()}, {"free_boundary_components", unknown()}};
    topology["solid"] = summary.solid == 0 ? unknown() : QJsonValue(summary.solid == 2);
    topology["solid_evidence"] = summary.solid == 0 ? unknown() :
        QJsonValue(summary.representation == 8 || summary.representation == 9 ? "native_representation_type" : "native_shape_type");
    result["topology"] = topology;
    result["source_attributes"] = sourceAttributes(*object,snapshot);
    result["preservation"] = QJsonObject{{"locus", unknown()}, {"parameterization", unknown()}, {"topology", unknown()}, {"parameter_map", unknown()}};
    const auto source=placement(*object);
    reframe(result,FrameInput{parentPlacement(*object),source,source,identity});
    verifyObject(doc, snapshot);
    return result;
}
PyObject* descriptorAPI(PyObject*, PyObject* args) {
    const char *documentName = nullptr, *objectName = nullptr, *subelement = "";
    if (!PyArg_ParseTuple(args, "ss|s", &documentName, &objectName, &subelement)) return nullptr;
    try {
        if (!qApp || QThread::currentThread() != qApp->thread()) throw std::runtime_error("Geometry descriptors require the GUI thread");
        if (std::strlen(documentName) > 1024) throw std::runtime_error("Invalid descriptor document reference");
        auto* doc = App::GetApplication().getDocument(documentName);
        if (!doc || doc->testStatus(App::Document::Restoring)) throw std::runtime_error("Descriptor document is missing or restoring");
        const auto raw = QJsonDocument(describe(*doc, objectName, subelement)).toJson(QJsonDocument::Compact);
        return PyUnicode_DecodeUTF8(raw.constData(), raw.size(), "strict");
    } catch (const Standard_Failure& error) {
        PyErr_SetString(PyExc_RuntimeError, error.GetMessageString() ? error.GetMessageString() : "Native geometry inspection failed");
    } catch (const Base::Exception& error) {
        PyErr_SetString(PyExc_RuntimeError, error.what());
    } catch (const std::exception& error) {
        PyErr_SetString(PyExc_ValueError, error.what());
    } catch (...) {
        PyErr_SetString(PyExc_RuntimeError, "Native geometry descriptor operation failed");
    }
    return nullptr;
}
}
void validateNativePartShape(PyObject* shape,const std::string& operation,const std::string& identity) {
    const auto context=operation+" input "+identity+": ";
    if(!qApp||QThread::currentThread()!=qApp->thread())
        throw std::runtime_error("KernelFailure: "+context+"native input requires the GUI thread");
    if(!shape||!PyObject_TypeCheck(shape,&Part::TopoShapePy::Type))
        throw std::runtime_error("UnsupportedRepresentation: "+context+"no native Part shape owner");
    auto* wrapper=static_cast<Part::TopoShapePy*>(shape);
    if(!wrapper->Base::PyObjectBase::isValid()||!wrapper->getTopoShapePtr())
        throw std::runtime_error("StaleGeometry: "+context+"native Part shape is no longer live");
    try {
        ThreeDm::InputTree::validate(wrapper->getTopoShapePtr()->getShape());
    }catch(const ThreeDm::GeometryRepresentationUnavailable& error){
        throw std::runtime_error("UnsupportedRepresentation: "+context+error.what());
    }catch(const Standard_Failure& error){
        throw std::runtime_error("KernelFailure: "+context+
            (error.GetMessageString()?error.GetMessageString():"native input inspection failed"));
    }catch(const std::exception& error){
        throw std::runtime_error("InvalidBasis: "+context+error.what());
    }
}
extern "C" unsigned om9_native_edit_capability(unsigned,unsigned,unsigned,unsigned);
extern "C" unsigned om9_native_rebuild_capability(unsigned,unsigned,std::size_t,std::size_t,std::size_t,unsigned);
extern "C" unsigned om9_native_curve_capability(unsigned,unsigned);
extern "C" unsigned om9_native_solid_capability(unsigned,unsigned);
void validateCurrentSolidCapability(unsigned representation,unsigned stage,
    const std::string& operation,const std::string& identity){
    const auto status=om9_native_solid_capability(representation,stage);
    if(status==4)return;
    const char* code=status==101?"UnsupportedRepresentation":status==102?"InvalidBasis":
        status==103?"KernelFailure":"UnsupportedOperation";
    throw std::runtime_error(std::string(code)+": "+operation+" input "+identity+": current Solid request is unsupported");
}
static PyObject* solidCapabilityAPI(PyObject*,PyObject* args){
    unsigned representation=0,stage=0;if(!PyArg_ParseTuple(args,"II",&representation,&stage))return nullptr;
    const auto status=om9_native_solid_capability(representation,stage);
    const auto raw=QJsonDocument(QJsonObject{{"schema_version",1},{"status",static_cast<int>(status)},
        {"allowed",status==4},{"backend",status==4?"occt_edit":""}}).toJson(QJsonDocument::Compact);
    return PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict");
}
void validateCurrentCurveCapability(unsigned representation,unsigned stage,
    const std::string& operation,const std::string& identity){
    const auto status=om9_native_curve_capability(representation,stage);
    if(status==4)return;
    const char* code=status==101?"UnsupportedRepresentation":status==102?"InvalidBasis":
        status==103?"KernelFailure":"UnsupportedOperation";
    throw std::runtime_error(std::string(code)+": "+operation+" input "+identity+": current Curve request is unsupported");
}
static PyObject* curveCapabilityAPI(PyObject*,PyObject* args){
    unsigned representation=0,stage=0;if(!PyArg_ParseTuple(args,"II",&representation,&stage))return nullptr;
    const auto status=om9_native_curve_capability(representation,stage);
    const auto raw=QJsonDocument(QJsonObject{{"schema_version",1},{"status",static_cast<int>(status)},
        {"allowed",status==4},{"backend",status==4?"occt_edit":""}}).toJson(QJsonDocument::Compact);
    return PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict");
}
static PyObject* rebuildCapabilityAPI(PyObject*,PyObject* args){
    unsigned representation=0,stage=0,flags=0;Py_ssize_t count=0,poles=0,degree=0;
    if(!PyArg_ParseTuple(args,"IInnnI",&representation,&stage,&count,&poles,&degree,&flags))return nullptr;
    if(count<0||poles<0||degree<0){PyErr_SetString(PyExc_ValueError,"Negative Rebuild request count/degree");return nullptr;}
    const auto status=om9_native_rebuild_capability(representation,stage,static_cast<std::size_t>(count),
        static_cast<std::size_t>(poles),static_cast<std::size_t>(degree),flags);
    const auto raw=QJsonDocument(QJsonObject{{"schema_version",1},{"status",static_cast<int>(status)},
        {"allowed",status==4},{"backend",status==4?"occt_edit":""}}).toJson(QJsonDocument::Compact);
    return PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict");
}
static PyObject* editCapabilityAPI(PyObject*,PyObject* args){
    unsigned kind=0,representation=0,flags=0,mode=0;
    if(!PyArg_ParseTuple(args,"IIII",&kind,&representation,&flags,&mode))return nullptr;
    const auto status=om9_native_edit_capability(kind,representation,flags,mode);
    const char* error=status==100?"UnsupportedOperation":status==101?"UnsupportedRepresentation":status==102?"UnsupportedOptions":status==103?"KernelFailure":"";
    const char* backend=status==4?"occt_edit":status==5?"mesh_polyhedral_occt_edit":status==6?"native_explode_adapters":"";
    const auto raw=QJsonDocument(QJsonObject{{"schema_version",1},{"status",static_cast<int>(status)},{"allowed",status<100},{"backend",backend},{"reason",error}}).toJson(QJsonDocument::Compact);
    return PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict");
}
static PyObject* surfaceCapabilityAPI(PyObject*,PyObject* args){
  try {
    NativeSurfaceRequest request{};Py_ssize_t count=0;PyObject* slashObject=nullptr;
    if(!PyArg_ParseTuple(args,"IIIIIIIdIInIO",&request.kind,&request.representation,&request.stage,&request.flags,
        &request.style,&request.sectionMode,&request.pointCount,&request.tolerance,&request.continuityA,&request.continuityB,
        &count,&request.railClosed,&slashObject))return nullptr;
    if(count<0){PyErr_SetString(PyExc_ValueError,"Negative Surface input count");return nullptr;}
    if(!PyList_CheckExact(slashObject)&&!PyTuple_CheckExact(slashObject)){PyErr_SetString(PyExc_TypeError,"Slash pairs must be a plain sequence");return nullptr;}
    request.inputCount=static_cast<std::size_t>(count);request.slashCount=static_cast<std::size_t>(PySequence_Size(slashObject));
    std::vector<double> pairs;
    if(request.slashCount<=64){
        pairs.reserve(request.slashCount*2);
        for(std::size_t i=0;i<request.slashCount;++i){
            PyObject* pair=PySequence_GetItem(slashObject,static_cast<Py_ssize_t>(i));
            if(!pair)return nullptr;
            if((!PyTuple_CheckExact(pair)&&!PyList_CheckExact(pair))||PySequence_Size(pair)!=2){Py_DECREF(pair);PyErr_SetString(PyExc_ValueError,"Each slash pair needs two scalar values");return nullptr;}
            for(unsigned j=0;j<2;++j){PyObject* value=PySequence_GetItem(pair,j);if(!value){Py_DECREF(pair);return nullptr;}
                const auto number=PyFloat_AsDouble(value);Py_DECREF(value);
                if(PyErr_Occurred()){Py_DECREF(pair);return nullptr;}pairs.push_back(number);}
            Py_DECREF(pair);
        }
    }
    request.slashPairs=pairs.empty()?nullptr:pairs.data();
    const auto status=om9_native_surface_capability(request);
    const auto raw=QJsonDocument(QJsonObject{{"schema_version",1},{"status",static_cast<int>(status)},
        {"allowed",status==4},{"backend",status==4?"occt_edit":""}}).toJson(QJsonDocument::Compact);
    return PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict");
  }catch(const Standard_Failure& error){
    PyErr_SetString(PyExc_RuntimeError,error.GetMessageString()?error.GetMessageString():"KernelFailure: Surface request binding failed");
  }catch(const std::exception& error){
    PyErr_SetString(PyExc_RuntimeError,error.what());
  }catch(...){PyErr_SetString(PyExc_RuntimeError,"KernelFailure: Surface request binding failed");}
  return nullptr;
}
void addGeometryContractMethods(PyObject* module) {
    static PyMethodDef methods[] = {
        {"geometrySolidCapability",solidCapabilityAPI,METH_VARARGS,
         "Read the existing Rust Box/Sphere session decision; no native geometry owner or caller storage is retained."},
        {"geometryCurveCapability",curveCapabilityAPI,METH_VARARGS,
         "Read the existing Rust Curve session operation/type/options decision; no geometry owner or caller storage is retained."},
        {"geometryRebuildCapability",rebuildCapabilityAPI,METH_VARARGS,
         "Read the shared scalar Rebuild request decision; no geometry owner or caller storage is retained."},
        {"geometrySurfaceCapability",surfaceCapabilityAPI,METH_VARARGS,
         "Read the shared bounded Surface scalar request decision; no document/geometry owner is inspected or retained."},
        {"geometryEditCapability",editCapabilityAPI,METH_VARARGS,
         "Read the shared value-only native Edit operation/type/options decision; does not inspect or mutate a document."},
        {"geometryDescriptor", descriptorAPI, METH_VARARGS,
         "Read owned JSON geometry facts by document/object name and optional native subelement. GUI thread only; no native handles, properties, transactions or preservation claims."},
        {nullptr, nullptr, 0, nullptr}};
    if (PyModule_AddFunctions(module, methods) < 0) throw Base::RuntimeError("Cannot register geometry descriptor API");
}
}
