// SPDX-License-Identifier: LGPL-2.1-or-later
// RCORE-01.T10: standalone exact-lineage native queries, not durable History.
#include "CurveConversionContract.h"
#include <BRep_Tool.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <TopoDS.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QFile>
#include <cmath>
#include <iostream>
#include <limits>

using namespace OpenMatrix9Gui::ThreeDm;
using namespace OpenMatrix9Gui::ThreeDm::CurveConversion;
namespace {
constexpr double pi = 3.141592653589793238462643383279502884;
constexpr Options bounds{0., 1e-7, 1e-10};
QJsonArray checks, cases;
void check(bool value, const char* label) {
    checks.append(QJsonObject{{"name", label}, {"passed", value}});
    if (!value) throw std::runtime_error(label);
}
template<class F> void rejected(F&& f, std::uint32_t status, const char* label) {
    bool matched = false;
    try { f(); } catch (const QueryError& e) { matched = e.status == status; }
    check(matched, label);
}
double dist(const std::array<double,3>& a, const ON_3dPoint& b) {
    return std::hypot(std::hypot(a[0]-b.x, a[1]-b.y), a[2]-b.z);
}
void circle(double start, double end) {
    ON_ArcCurve source(ON_Circle(ON_Plane::World_xy, 10.));
    check(source.SetDomain(start,end), "circle domain authored");
    auto crc = source.DataCRC(0);
    std::array<double,8> fractions{0.,1./12.,1./6.,5./12.,0.193,0.617,0.893,1.};
    std::vector<double> requests;
    for (double fraction : fractions) requests.push_back(start+(end-start)*fraction);
    auto converted = importCurveObserved(source,bounds,requests);
    const auto& r = converted.report;
    check(r.input.form == 2, "native form2 is retained");
    check(r.input.source_domain == std::array<double,2>{start,end}, "source domain retained");
    check(r.input.target_domain == r.input.source_domain, "matching target domain is separate from mapping");
    check(BRepCheck_Analyzer(converted.shape).IsValid(), "actual converted edge valid");
    check(r.queries.size() == requests.size(), "complete requested batch only");
    check(r.targetBasis.weights.size() == r.targetBasis.poles.size(), "rational weights retained");
    bool nonunit = false;
    for (double weight : r.targetBasis.weights) nonunit |= std::abs(weight-1.)>1e-5;
    check(nonunit, "weights are not replaced with all1");
    for (std::size_t i=0;i<requests.size();++i) {
        const auto& q = r.queries[i];
        double angle = 2*pi*fractions[i];
        ON_3dPoint independent(10*std::cos(angle),10*std::sin(angle),0.);
        check(q.status == 0 && q.schema_version == 1 && q.action == 2, "validated native nonlinear query");
        check(dist(q.observation.native_point,independent) <= bounds.lengthTolerance, "mapped dependent point matches independent analytic point");
        check(dist(q.observation.intermediate_point,independent) <= bounds.lengthTolerance, "same prepared NURBS point matches independent analytic point");
        check(std::abs(q.observation.inverse_parameter-requests[i]) <= bounds.parameterTolerance, "inverse parameter meets declared bound");
        check(q.raw_parameter_reusable == 0, "form2 never authorizes raw parameter reuse");
    }
    const auto& decisive = r.queries[1];
    check(std::abs(decisive.observation.target_parameter-decisive.observation.source_parameter)>1e-8, "pi6 mapped parameter differs from raw");
    check(decisive.residuals[2]>1e-4, "raw pi6 point mismatch demonstrates dependency error");
    check(r.queries.front().observation.target_parameter == start && r.queries.back().observation.target_parameter == end, "seam endpoint sides stay distinct");
    double first,last;
    auto actual = BRep_Tool::Curve(TopoDS::Edge(converted.shape),first,last);
    double radiusResidual = 0.;
    constexpr int sampleCount = 1001;
    for (int i=0;i<sampleCount;++i) {
        auto p = actual->Value(first+(last-first)*double(i)/(sampleCount-1));
        radiusResidual = std::max(radiusResidual,std::abs(std::hypot(p.X(),p.Y())-10.));
    }
    check(radiusResidual <= bounds.lengthTolerance, "separate target-domain sampled locus bound");
    check(source.DataCRC(0) == crc && r.sourceCrc == crc, "original analytic source unchanged");
    auto json = r.json();
    check(json.value("parameter_map").isNull() && json.value("parameterization_preserved").isNull() && json.value("global_deviation").isNull(), "global correspondence stays unknown");
    json.insert("fixture_length_unit","mm");
    json.insert("locus_sample_count",sampleCount);
    json.insert("sampled_radius_residual",radiusResidual);
    json.insert("name",start == 0. ? "circle-r10-radians" : "circle-r10-domain-2-7");
    cases.append(json);
}
// Fault injection through the real synchronous native virtual-call boundary.
// These failure rows do not establish a successful kernel mapping.
struct FailingArc : ON_ArcCurve {
    mutable unsigned forwardCalls=0, inverseCalls=0;
    int mode;
    explicit FailingArc(int m) : ON_ArcCurve(ON_Circle(ON_Plane::World_xy,10.),0.,2*pi), mode(m) {}
    bool GetNurbFormParameterFromCurveParameter(double t,double* out) const override {
        ++forwardCalls;
        if (mode == 0) { *out = 4.5; return false; }
        if (mode == 1) { *out = std::numeric_limits<double>::quiet_NaN(); return true; }
        if (mode == 2) { *out = 99.; return true; }
        return ON_ArcCurve::GetNurbFormParameterFromCurveParameter(t,out);
    }
    bool GetCurveParameterFromNurbFormParameter(double t,double* out) const override {
        ++inverseCalls;
        if (mode == 3) { *out = 4.5; return false; }
        if (mode == 4) { *out = std::numeric_limits<double>::infinity(); return true; }
        if (mode == 5) { *out = 4.5; return true; }
        return ON_ArcCurve::GetCurveParameterFromNurbFormParameter(t,out);
    }
};
void failures() {
    std::array<double,1> request{pi/6};
    constexpr std::array<std::uint32_t,6> expected{108,107,106,109,107,110};
    for (int mode=0;mode<6;++mode) {
        FailingArc source(mode);
        auto crc=source.DataCRC(0);
        rejected([&]{importCurveObserved(source,bounds,request);},expected[mode],"bad native mapping publishes no result");
        check(source.DataCRC(0)==crc,"failed mapping leaves source unchanged");
        check(source.forwardCalls==1,"one native forward call for valid request");
        check(source.inverseCalls==(mode<3?0u:1u),"bad forward output is rejected before inverse native call");
    }
    FailingArc source(6);
    for (double bad : {-0.1,2*pi+0.1,std::numeric_limits<double>::quiet_NaN()}) {
        std::array<double,1> invalid{bad};
        rejected([&]{importCurveObserved(source,bounds,invalid);},std::isfinite(bad)?106:107,"invalid source parameter rejected before mapping");
    }
    check(source.forwardCalls==0,"invalid requests never call native mapping");
    std::vector<double> tooMany(om9_curve_conversion_query_limit()+1,pi/6);
    rejected([&]{importCurveObserved(source,bounds,tooMany);},105,"query budget rejects before conversion/query results");
    rejected([&]{importCurveObserved(source,{0.,0.,1e-10},request);},102,"undeclared length bound rejected");
    rejected([&]{importCurveObserved(source,{-1.,1e-7,1e-10},request);},102,"negative native conversion tolerance rejected");
}
struct GuardedBadNurbs : ON_NurbsCurve {
    mutable bool metadataRead=false;
    explicit GuardedBadNurbs(const ON_NurbsCurve& source):ON_NurbsCurve(source) {}
    ON_Interval Domain() const override {
        metadataRead=true;
        throw ExchangeError("malformed source metadata read before storage gate");
    }
    ON__UINT32 DataCRC(ON__UINT32) const override {
        metadataRead=true;
        throw ExchangeError("malformed source CRC read before storage gate");
    }
};
void malformedStoragePreflight() {
    ON_ArcCurve circle(ON_Circle(ON_Plane::World_xy,10.));
    ON_NurbsCurve basis;
    check(circle.GetNurbForm(basis)==2,"malformed-count control starts from valid NURBS");
    GuardedBadNurbs malformed(basis);
    const int count=malformed.m_cv_count;
    malformed.m_cv_count=1000001;
    bool rejected=false;
    try { importCurveObserved(malformed,bounds,{}); }
    catch(const ExchangeError&) { rejected=true; }
    malformed.m_cv_count=count;
    check(rejected && !malformed.metadataRead,"raw storage/count gate precedes Domain and CRC reads");
}
void identityAndLifetime() {
    Report copied;
    {
        ON_LineCurve line(ON_3dPoint(1,2,3),ON_3dPoint(11,22,33));
        check(line.SetDomain(2,7),"line domain authored");
        std::array<double,3> query{2.,3.37,7.};
        auto crc=line.DataCRC(0);
        auto result=importCurveObserved(line,bounds,query);
        check(result.report.input.form==1,"line native form1 retained");
        for (const auto& q : result.report.queries) {
            check(q.action==1 && q.raw_parameter_reusable==1,"line identity allowed at inspected query only");
            check(dist(q.observation.native_point,line.PointAt(q.observation.source_parameter))<=bounds.lengthTolerance,"line target actual point matches source");
        }
        check(line.DataCRC(0)==crc,"line source immutable");
        copied=result.report;
        ON_NurbsCurve rational;
        ON_ArcCurve arc(ON_Arc(ON_Circle(ON_Plane::World_xy,10.),pi/2));
        check(arc.GetNurbForm(rational)==2,"independent rational NURBS control authored");
        std::array<double,3> nq{rational.Domain().Min(),rational.Domain().ParameterAt(0.193),rational.Domain().Max()};
        auto ncrc=rational.DataCRC(0);
        auto nr=importCurveObserved(rational,bounds,nq);
        check(nr.report.input.form==1,"already NURBS form1 retained");
        for (const auto& q : nr.report.queries) check(q.raw_parameter_reusable==1,"already NURBS inspected identity");
        check(rational.DataCRC(0)==ncrc,"NURBS source immutable across clamp");
        cases.append(nr.report.json());
        auto empty=importCurveObserved(line,bounds,{});
        check(empty.report.queries.empty() && !empty.shape.IsNull(),"empty batch imports geometry without mapping promise");
    }
    check(copied.queries.size()==3 && copied.queries[1].observation.source_parameter==3.37,"copied report survives original native owners");
    check(copied.targetBasis.poles.size()==2,"copied target basis survives native owners");
    cases.append(copied.json());
}
}
int main(int argc,char** argv) {
    ON::Begin();
    int result=0;
    std::string error;
    try { circle(0.,2*pi); circle(2.,7.); identityAndLifetime(); failures(); malformedStoragePreflight(); }
    catch (const Standard_Failure& e) { result=1;error=e.GetMessageString(); }
    catch (const std::exception& e) { result=1;error=e.what(); }
    QJsonObject report{{"schema_version",1},{"fixture","RCORE-01.T10.native-query-slice"},{"passed",result==0},{"error",QString::fromStdString(error)},{"checks",checks},{"cases",cases}};
    if (argc>1) { QFile file(QString::fromLocal8Bit(argv[1]));if (!file.open(QIODevice::WriteOnly)) return 2;file.write(QJsonDocument(report).toJson()); }
    std::cout << (result==0?"PASS":"FAIL") << " checks=" << checks.size() << " " << error << '\n';
    ON::End();
    return result;
}
