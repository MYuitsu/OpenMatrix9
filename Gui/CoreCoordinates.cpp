// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreCoordinates.h"
#include "CoreCPlanes.h"
#include "CoreUnits.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <QApplication>
#include <QString>
#include <QThread>
#include <cstddef>
#include <stdexcept>
#include <type_traits>
#include <utility>

namespace {
// Mirrors Rust coordinate_input::CoordinateRequest. Every pointer is a borrowed
// length-delimited UTF-8 slice valid only for one synchronous call.
struct CoordinateRequest {
    std::uint32_t version,expected,source,has_previous;
    std::uint64_t document_owner,frame_revision;
    std::uint32_t logical_view_slot,reserved;
    double frame[12],previous[3],world[3];
    const std::uint8_t* original;std::size_t original_len;
    const std::uint8_t* unit_context;std::size_t unit_context_len;
    const std::uint8_t* document_uid;std::size_t document_uid_len;
};
struct CoordinateValue {
    std::uint32_t kind,reserved;
    double point[3],length_mm;
};
static_assert(std::is_standard_layout_v<CoordinateRequest>);
static_assert(offsetof(CoordinateRequest,frame)==40);
static_assert(offsetof(CoordinateRequest,original)==184);
static_assert(sizeof(CoordinateValue)==40);
constexpr std::size_t maxReportBytes=1024*1024;
}
extern "C" {
std::size_t om9_coordinate_resolve(const CoordinateRequest*,CoordinateValue*,std::uint8_t*,std::size_t,std::uint8_t*,std::size_t);
std::size_t om9_coordinate_report(const CoordinateRequest*,std::size_t,std::uint8_t*,std::size_t,std::uint8_t*,std::size_t);
bool om9_coordinate_verify_context(const CoordinateRequest*,std::uint64_t,const std::uint8_t*,std::size_t,const std::uint8_t*,std::size_t,std::uint8_t*,std::size_t);
}

namespace OpenMatrix9Gui {
namespace {
void guiThread() {
    if(!qApp||QThread::currentThread()!=qApp->thread())
        throw std::runtime_error("Coordinate host operations require the GUI thread");
}
const std::uint8_t* bytes(const std::string& value) {
    return reinterpret_cast<const std::uint8_t*>(value.data());
}
void pointValues(double* out,const Base::Vector3d& point) {
    out[0]=point.x;out[1]=point.y;out[2]=point.z;
}
CoordinateRequest contextRequest(const CoordinateContext& context) {
    CoordinateRequest result{};
    result.version=1;result.document_owner=context.documentOwner;
    result.frame_revision=context.frameRevision;result.logical_view_slot=context.logicalViewSlot;
    for(std::size_t i=0;i<context.frame.size();++i)result.frame[i]=context.frame[i];
    result.unit_context=bytes(context.unitContext);result.unit_context_len=context.unitContext.size();
    result.document_uid=bytes(context.documentUid);result.document_uid_len=context.documentUid.size();
    return result;
}
CoordinateRequest inputRequest(const CoordinateInput& input) {
    auto result=contextRequest(input.context);
    result.expected=static_cast<std::uint32_t>(input.expected);
    result.source=static_cast<std::uint32_t>(input.source);
    result.original=bytes(input.original);result.original_len=input.original.size();
    if(input.previous){result.has_previous=1;pointValues(result.previous,*input.previous);}
    pointValues(result.world,input.point);
    return result;
}
[[noreturn]] void failure(const std::array<std::uint8_t,512>& error) {
    const auto* message=reinterpret_cast<const char*>(error.data());
    throw std::runtime_error(*message?message:"Coordinate input adapter failed");
}
CoordinateInput resolved(CoordinateInput input) {
    const auto request=inputRequest(input);
    std::array<std::uint8_t,512> error{};CoordinateValue result{};
    const auto count=om9_coordinate_resolve(&request,nullptr,nullptr,0,error.data(),error.size());
    if(!count||count>maxReportBytes)failure(error);
    std::vector<std::uint8_t> buffer(count);
    if(om9_coordinate_resolve(&request,&result,buffer.data(),buffer.size(),error.data(),error.size())!=count)
        failure(error);
    input.kind=static_cast<CoordinateValueKind>(result.kind);
    input.point=Base::Vector3d(result.point[0],result.point[1],result.point[2]);
    input.lengthMm=result.length_mm;
    input.provenance.assign(reinterpret_cast<const char*>(buffer.data()),count-1);
    return input;
}
}

CoordinateContext captureCoordinateContext(App::Document& doc,Gui::View3DInventor* view) {
    guiThread();
    const auto plane=captureConstructionPlane(doc,view);
    CoordinateContext result;
    result.documentOwner=plane.documentOwner;result.frameRevision=plane.revision;
    result.logicalViewSlot=plane.logicalViewSlot;result.frame=plane.frame;result.plane=plane.plane;
    result.documentUid=doc.Uid.getValueStr();result.unitContext=unitContextSnapshot(doc);
    verifyCoordinateContext(doc,result);
    return result;
}
void verifyCoordinateContext(const App::Document& doc,const CoordinateContext& context) {
    guiThread();
    const auto owner=constructionPlaneDocumentOwner(doc);
    const std::string uid=doc.Uid.getValueStr();
    const auto units=unitContextSnapshot(doc);
    const auto request=contextRequest(context);std::array<std::uint8_t,512> error{};
    if(!om9_coordinate_verify_context(&request,owner,bytes(uid),uid.size(),bytes(units),units.size(),error.data(),error.size()))
        failure(error);
}
CoordinateInput parseCoordinateInput(const CoordinateContext& context,const QString& text,
    CoordinateExpected expected,const std::optional<Base::Vector3d>& previous) {
    CoordinateInput input;input.context=context;input.expected=expected;
    input.original=text.toUtf8().toStdString();input.previous=previous;
    return resolved(std::move(input));
}
CoordinateInput worldCoordinateInput(const CoordinateContext& context,const Base::Vector3d& point,
    CoordinateSource source,const std::optional<Base::Vector3d>& previous) {
    // Typed input belongs to parseCoordinateInput; invalid remaining enum values
    // are rejected by Rust before output is returned.
    if(source==CoordinateSource::Typed)throw std::runtime_error("World-coordinate input requires a world source");
    CoordinateInput input;input.context=context;input.source=source;input.point=point;input.previous=previous;
    return resolved(std::move(input));
}
std::string coordinateInputReport(const std::vector<CoordinateInput>& inputs) {
    if(inputs.size()>64)throw std::runtime_error("Coordinate report exceeds 64 inputs");
    std::vector<CoordinateRequest> requests;requests.reserve(inputs.size());
    for(const auto& input:inputs)requests.push_back(inputRequest(input));
    std::array<std::uint8_t,512> error{};
    const auto count=om9_coordinate_report(requests.data(),requests.size(),nullptr,0,error.data(),error.size());
    if(!count||count>maxReportBytes)failure(error);
    std::vector<std::uint8_t> buffer(count);
    if(om9_coordinate_report(requests.data(),requests.size(),buffer.data(),buffer.size(),error.data(),error.size())!=count)
        failure(error);
    return std::string(reinterpret_cast<const char*>(buffer.data()),count-1);
}
void persistCoordinateProvenance(App::DocumentObject& object,const std::vector<CoordinateInput>& inputs) {
    guiThread();
    auto* doc=object.getDocument();
    if(!doc||!doc->hasPendingTransaction())throw std::runtime_error("Coordinate provenance requires the caller's document transaction");
    for(const auto& input:inputs)verifyCoordinateContext(*doc,input.context);
    const auto report=coordinateInputReport(inputs); // Validate before property mutation.
    auto* existing=object.getPropertyByName("OM9CoordinateReport");
    auto* property=dynamic_cast<App::PropertyString*>(existing);
    if(existing&&!property)throw std::runtime_error("OM9CoordinateReport has an incompatible property type");
    if(!property)property=dynamic_cast<App::PropertyString*>(object.addDynamicProperty(
        "App::PropertyString","OM9CoordinateReport","OpenMatrix9",
        "Accepted coordinate inputs and command-start frame/unit provenance",App::Prop_ReadOnly));
    if(!property)throw std::runtime_error("Cannot store coordinate provenance");
    property->setValue(report);
}
}
