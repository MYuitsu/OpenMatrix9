// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CurveConversionContract.h"
#include <QJsonArray>
#include <limits>
namespace OpenMatrix9Gui::ThreeDm::CurveConversion {
QueryError::QueryError(std::uint32_t code)
    : ExchangeError("Curve parameter query rejected (status " + std::to_string(code) + ")"), status(code) {}
namespace {
void require(std::uint32_t status) { if(status!=0)throw QueryError(status); }
Input input(const ON_Curve& source,std::array<double,2> target,int form,Options options) {
    auto d=source.Domain();
    return {std::uint32_t(form),1,1,0,{d.Min(),d.Max()},target,
            options.conversionTolerance,options.lengthTolerance,options.parameterTolerance};
}
std::array<double,3> point(const ON_3dPoint& p){return {p.x,p.y,p.z};}
std::array<double,3> point(const gp_Pnt& p){return {p.X(),p.Y(),p.Z()};}
template<class T> QJsonArray numbers(const T& values) {
    QJsonArray out;for(auto value:values)out.append(double(value));return out;
}
}
void Detail::preflight(const ON_Curve& source,Options options,std::span<const double> parameters) {
    auto d=source.Domain();
    // Provisional full-domain header validates options/count before conversion.
    // The actual native form/domain is validated again before any query results.
    auto provisional=input(source,{d.Min(),d.Max()},1,options);
    require(om9_curve_conversion_preflight(provisional,parameters.size()));
    for(double parameter:parameters) {
        auto action=om9_curve_conversion_request(provisional,parameter);
        if(action!=1&&action!=2)throw QueryError(action);
    }
}
Report Detail::observe(const ON_Curve& source,const ON_NurbsCurve& prepared,
    const Handle(Geom_Curve)& target,int nativeForm,std::array<double,2> originalDomain,
    bool beforeClampPeriodic,Options options,std::span<const double> parameters) {
    Report result;
    result.input=input(source,{target->FirstParameter(),target->LastParameter()},nativeForm,options);
    require(om9_curve_conversion_preflight(result.input,parameters.size()));
    result.getNurbFormDomain=originalDomain;
    result.beforeClampPeriodic=beforeClampPeriodic;
    result.afterClampPeriodic=prepared.IsPeriodic();
    result.targetBasis=BasisContract::checkedBasis(prepared);
    result.queries.reserve(parameters.size());
    for(double parameter:parameters) {
        auto action=om9_curve_conversion_request(result.input,parameter);
        if(action!=1&&action!=2)throw QueryError(action);
        double mapped=std::numeric_limits<double>::quiet_NaN();
        const bool forward=source.GetNurbFormParameterFromCurveParameter(parameter,&mapped);
        // False outputs are discarded even if the native API wrote finite data.
        require(om9_curve_conversion_mapped_parameter(result.input,forward?mapped:0.,forward,1));
        double inverse=std::numeric_limits<double>::quiet_NaN();
        const bool backward=source.GetCurveParameterFromNurbFormParameter(mapped,&inverse);
        require(om9_curve_conversion_mapped_parameter(result.input,backward?inverse:0.,backward,2));
        // This full-domain slice requires an in-domain raw diagnostic. It does
        // not extrapolate or pretend to support a different/subdomain lineage.
        require(om9_curve_conversion_mapped_parameter(result.input,parameter,1,1));
        Observation row{parameter,mapped,inverse,1,1,
            point(source.PointAt(parameter)),point(prepared.PointAt(mapped)),
            point(target->Value(mapped)),point(target->Value(parameter))};
        auto validated=om9_curve_conversion_observation(result.input,row);
        require(validated.status);
        result.queries.push_back(validated);
    }
    return result;
}
QJsonObject Report::json() const {
    QJsonArray rows;
    for(const auto& q:queries) {
        const auto& o=q.observation;
        rows.append(QJsonObject{{"schema_version",int(q.schema_version)},
            {"source_parameter",o.source_parameter},{"mapped_parameter",o.target_parameter},
            {"inverse_parameter",o.inverse_parameter},{"query_action",int(q.action)},
            {"raw_parameter_reusable",bool(q.raw_parameter_reusable)},
            {"source_point",numbers(o.source_point)},{"intermediate_point",numbers(o.intermediate_point)},
            {"native_point",numbers(o.native_point)},{"raw_native_point",numbers(o.raw_native_point)},
            {"sampled_residuals",numbers(q.residuals)}});
    }
    QJsonArray poles;
    for(const auto& p:targetBasis.poles)poles.append(numbers(p));
    QJsonObject basis{{"degree",double(targetBasis.axis.degree)},{"poles",poles},
        {"weights",numbers(targetBasis.weights)},{"knots",numbers(targetBasis.axis.knots)},
        {"multiplicities",numbers(targetBasis.axis.multiplicities)},
        {"domain",numbers(targetBasis.axis.domain)},{"periodic",targetBasis.axis.periodic},
        {"convention","openNURBS expanded structural view after ClampEnd"}};
    return {{"schema_version",1},{"native_form",int(input.form)},
        {"source_domain",numbers(input.source_domain)},{"target_domain",numbers(input.target_domain)},
        {"native_conversion_tolerance",input.conversion_tolerance},
        {"length_tolerance",input.length_tolerance},{"parameter_tolerance",input.parameter_tolerance},
        {"length_unit",QJsonValue::Null},{"parameter_unit","native source/target domain units"},
        {"frame","native_conversion_local"},{"method","ON_GetNurbForm_to_OCCT"},
        {"source_crc",double(sourceCrc)}, {"source_identity",QJsonValue::Null},
        {"get_nurb_form_domain",numbers(getNurbFormDomain)},
        {"before_clamp_periodic",beforeClampPeriodic},{"after_clamp_periodic",afterClampPeriodic},
        {"stages",QJsonArray{"GetNurbForm full domain","ClampEnd(2) owned buffer","OCCT rational B-spline reconstruction"}},
        {"target_basis",basis},{"queries",rows},
        {"measurement_scope","requested point samples only"},
        {"locus_preserved",QJsonValue::Null},{"parameterization_preserved",QJsonValue::Null},
        {"parameter_map",QJsonValue::Null},{"topology_preserved",QJsonValue::Null},
        {"attributes_preserved",QJsonValue::Null},{"global_deviation",QJsonValue::Null}};
}
}
