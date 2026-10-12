// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreGeometryContract.h"
#include "CoreObjects.h"
#include "NurbsBasisAdapter.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <Base/Exception.h>
#include <Base/Type.h>
#include <Mod/Part/App/PartFeature.h>
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

using ThreeDm::BasisContract::Axis;
using ThreeDm::BasisContract::checkedBasis;

QJsonValue unknown() { return QJsonValue(QJsonValue::Null); }
QString text(const std::string& value) { return QString::fromUtf8(value.data(), static_cast<int>(value.size())); }
QJsonArray pair(const std::array<double, 2>& values) { return {values[0], values[1]}; }
QJsonArray matrix(const std::array<double, 16>& values) {
    QJsonArray rows;
    for (unsigned row = 0; row < 4; ++row)
        rows.append(QJsonArray{values[row * 4], values[row * 4 + 1], values[row * 4 + 2], values[row * 4 + 3]});
    return rows;
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
QJsonObject axis(const Axis& data) {
    QJsonArray knots, multiplicities;
    for (double value : data.knots) knots.append(value);
    for (std::size_t value : data.multiplicities) multiplicities.append(static_cast<qint64>(value));
    return {{"degree", static_cast<qint64>(data.degree)}, {"knots", knots},
            {"multiplicities", multiplicities}, {"periodic", data.periodic}, {"full_domain", pair(data.domain)}};
}
template<class Basis> QJsonArray poles(const Basis& data) {
    QJsonArray values;
    for (const auto& point : data.poles) values.append(QJsonArray{point[0], point[1], point[2]});
    return values;
}
template<class Basis> QJsonValue weights(const Basis& data) {
    if (data.weights.empty()) return unknown();
    QJsonArray values;
    for (double value : data.weights) values.append(value);
    return values;
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
    double first = 0, last = 0;
    TopLoc_Location nativeLocation;
    auto curve = BRep_Tool::Curve(edge, nativeLocation, first, last);
    result["restriction_facts"] = QJsonObject{{"closed", BRep_Tool::IsClosed(edge)},
        {"closed_evidence", "native_topology_endpoints"}, {"periodic", unknown()}};
    result["brep_tool_location"] = location(nativeLocation);
    domains(input, first, last);
    if (curve.IsNull()) return;
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
            return;
        }
        const auto data = checkedBasis(spline);
        result["basis"] = QJsonObject{{"kind", "nurbs_curve"}, {"axis", axis(data.axis)},
            {"poles_xyz", poles(data)}, {"weights", weights(data)}, {"rational", spline->IsRational()}};
        result["basis_status"] = basisStatus("inspected", "copied_validated_basis");
        result["basis_frame"] = "native_local";
        return;
    }
    switch (GeomAdaptor_Curve(curve).GetType()) {
        case GeomAbs_Line: case GeomAbs_Circle: case GeomAbs_Ellipse:
        case GeomAbs_Hyperbola: case GeomAbs_Parabola: input.form = 1; break;
        default: break;
    }
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
        const auto data = checkedBasis(spline);
        result["basis"] = QJsonObject{{"kind", "nurbs_surface"}, {"axes", QJsonArray{axis(data.axes[0]), axis(data.axes[1])}},
            {"u_count", static_cast<qint64>(data.uCount)}, {"v_count", static_cast<qint64>(data.vCount)},
            {"pole_layout", "u_major"}, {"poles_xyz", poles(data)}, {"weights", weights(data)}, {"rational", rational}};
        result["basis_status"] = basisStatus("inspected", "copied_validated_basis");
        result["basis_frame"] = "native_local";
        return;
    }
    switch (GeomAdaptor_Surface(surface).GetType()) {
        case GeomAbs_Plane: case GeomAbs_Cylinder: case GeomAbs_Cone:
        case GeomAbs_Sphere: case GeomAbs_Torus: input.form = 1; break;
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
    DescriptorInput input{};
    input.stamp = snapshot.stamp;
    input.legacy_kind = 9;
    QJsonObject result{{"basis", unknown()}, {"basis_status", basisStatus("uninspected", "not_a_nurbs_support")},
                      {"basis_frame", unknown()}, {"brep_tool_location", unknown()}, {"restriction_facts", unknown()}};
    if (auto* feature = dynamic_cast<Part::Feature*>(object)) {
        const TopoDS_Shape shape = snapshot.subelement.empty() ? feature->Shape.getValue()
            : feature->Shape.getShape().getSubShape(snapshot.subelement.c_str());
        if (!snapshot.subelement.empty() && shape.IsNull()) throw std::runtime_error("Requested native subelement is missing");
        classify(shape, input);
        if (om9_geometry_operation(1, input.legacy_kind, 0, 0) != 0)
            throw std::runtime_error("Rust rejected descriptor inspection operation");
        if (input.representation == 2) curveFacts(TopoDS::Edge(shape), input, result);
        if (input.representation == 4) surfaceFacts(TopoDS::Face(shape), input, result);
    } else {
        if (!snapshot.subelement.empty()) throw std::runtime_error("No native subelement descriptor adapter for this representation");
        if (derived(*object, "Mesh::Feature")) { input.legacy_kind = 4; input.representation = 8; }
        else if (derived(*object, "Points::Feature")) { input.legacy_kind = 5; input.representation = 9; }
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
    // Supporting geometry location, shape placement and nested world placement
    // are separate observations. This reader does not certify their composition.
    result["coordinates"] = unknown(); result["local_to_world"] = unknown();
    result["form"] = summary.form == 1 ? QJsonValue("analytic") : summary.form == 2 ? QJsonValue("nurbs") : unknown();
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
    result["topology"] = QJsonObject{{"solid", summary.solid == 0 ? unknown() : QJsonValue(summary.solid == 2)},
        {"solid_evidence", summary.solid == 0 ? unknown() : QJsonValue("native_shape_type")},
        {"closed", unknown()}, {"manifold", unknown()}, {"solid_orientation", unknown()},
        {"faces", unknown()}, {"free_boundary_components", unknown()}};
    result["source_attributes"] = unknown();
    result["preservation"] = QJsonObject{{"locus", unknown()}, {"parameterization", unknown()}, {"topology", unknown()}, {"parameter_map", unknown()}};
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
void addGeometryContractMethods(PyObject* module) {
    static PyMethodDef methods[] = {
        {"geometryDescriptor", descriptorAPI, METH_VARARGS,
         "Read owned JSON geometry facts by document/object name and optional native subelement. GUI thread only; no native handles, properties, transactions or preservation claims."},
        {nullptr, nullptr, 0, nullptr}};
    if (PyModule_AddFunctions(module, methods) < 0) throw Base::RuntimeError("Cannot register geometry descriptor API");
}
}
