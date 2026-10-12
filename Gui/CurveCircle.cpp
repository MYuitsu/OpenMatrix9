// SPDX-License-Identifier: LGPL-2.1-or-later
// OM9-CURVE-005: host references and bounded exact native tangent solving.
#include "CurveCircle.h"
#include "SolidReferences.h"
#include "RustBridge.h"
#include "CircleHistory.h"
#include "CircleHistoryAdapter.h"
#include "CircleTangent.h"
#include "HistoryFeature.h"
#include "CoreCurveNativeInput.h"
#include <Base/Interpreter.h>
#include <App/Document.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Mod/Part/App/TopoShape.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <BRepBuilderAPI_Copy.hxx>
#include <Standard_Failure.hxx>
#include <QJsonArray>
#include <algorithm>
#include <cmath>
#include <memory>
namespace {
using namespace OpenMatrix9Gui;
std::map<std::size_t,SolidCurveReference> refs;
std::optional<SolidCurveReference> path;
using Ref=CurvePyRef;
// Native exception: OCCT geometry and FreeCAD identity guards cannot be kept
// by Rust. These immutable snapshots contain no Python-owned references; Rust
// alone decides which opaque input contexts survive. Geometry is copied once
// per captured reference and shared across option/point contexts.
struct NativeReferenceSnapshot {
    std::string name,signature,sub;
    ObjectSnapshot guard;
    TopoDS_Shape shape,edge;
    std::shared_ptr<const CurveNativeSource> evidence;
};
using NativeReference=std::shared_ptr<const NativeReferenceSnapshot>;
struct CircleReferenceContext {
    std::map<std::size_t,NativeReference> refs;
    std::optional<NativeReference> path;
    std::vector<CircleHistorySource> sources;
};
constexpr std::size_t contextLimit=1025;
std::map<std::uint64_t,CircleReferenceContext> contexts;
std::vector<std::weak_ptr<const NativeReferenceSnapshot>> referenceSnapshots;
bool sameStamp(const ObjectStamp& a,const ObjectStamp& b){
    return a.document==b.document&&a.object==b.object&&a.generation==b.generation&&a.revision==b.revision&&a.epoch==b.epoch;
}
bool sameGuard(const ObjectSnapshot& a,const ObjectSnapshot& b){
    return sameStamp(a.stamp,b.stamp)&&a.objectName==b.objectName&&a.subelement==b.subelement&&a.documentUid==b.documentUid&&a.unitContext==b.unitContext&&a.world==b.world&&a.ancestors.size()==b.ancestors.size()&&std::equal(a.ancestors.begin(),a.ancestors.end(),b.ancestors.begin(),sameStamp);
}
TopoDS_Shape copyShape(const TopoDS_Shape& shape){
    try{return BRepBuilderAPI_Copy(shape,true,true).Shape();}
    catch(const Standard_Failure& error){throw std::runtime_error(std::string("Cannot snapshot Circle geometry: ")+(error.GetMessageString()?error.GetMessageString():"native copy failed"));}
}
TopoDS_Shape nativeShape(const EditShape& shape){
    if(!shape||!PyObject_TypeCheck(shape->value,&Part::TopoShapePy::Type))throw std::runtime_error("Circle reference is not native BRep geometry");
    return copyShape(static_cast<Part::TopoShapePy*>(shape->value)->getTopoShapePtr()->getShape());
}
NativeReference snapshotReference(const SolidCurveReference& ref){
    for(auto it=referenceSnapshots.begin();it!=referenceSnapshots.end();){
        const auto snapshot=it->lock();if(!snapshot){it=referenceSnapshots.erase(it);continue;}
        if(snapshot->name==ref.input.name&&snapshot->signature==ref.input.signature&&snapshot->sub==ref.sub&&sameGuard(snapshot->guard,ref.input.guard))return snapshot;
        ++it;
    }
    auto snapshot=std::make_shared<const NativeReferenceSnapshot>(NativeReferenceSnapshot{ref.input.name,ref.input.signature,ref.sub,ref.input.guard,nativeShape(ref.input.shape),nativeShape(ref.edge),ref.curveEvidence});
    referenceSnapshots.push_back(snapshot);return snapshot;
}
EditShape restoredShape(const TopoDS_Shape& shape){
    Part::TopoShape native(copyShape(shape));Ref wrapper(native.getPyObject());
    return std::make_shared<Ref>(std::move(wrapper));
}
SolidCurveReference restoredReference(const NativeReference& snapshot){
    EditInput input{snapshot->name,snapshot->signature,restoredShape(snapshot->shape),false,{},snapshot->guard};
    return {std::move(input),restoredShape(snapshot->edge),snapshot->sub,snapshot->evidence};
}
void restoreContext(const CircleReferenceContext& context){
    std::map<std::size_t,SolidCurveReference> restoredRefs;for(const auto& [index,ref]:context.refs)restoredRefs.emplace(index,restoredReference(ref));
    std::optional<SolidCurveReference> restoredPath;if(context.path)restoredPath=restoredReference(*context.path);
    auto restoredSources=context.sources;
    refs=std::move(restoredRefs);path=std::move(restoredPath);restoreCircleHistoryReferenceSnapshot(std::move(restoredSources));
}
std::uint64_t retainContexts(){
    std::array<std::uint64_t,contextLimit> keys{};const auto count=om9_circle_input_context_keys(keys.data(),keys.size());
    if(count==0||count>keys.size())throw std::runtime_error("Circle input context retention is unavailable or exceeds its bound");
    const auto end=keys.begin()+count;const auto key=om9_circle_input_context_key();
    if(std::find(keys.begin(),end,key)==end)throw std::runtime_error("Circle input context identity is unavailable");
    for(auto it=contexts.begin();it!=contexts.end();)if(std::find(keys.begin(),end,it->first)==end)it=contexts.erase(it);else ++it;
    referenceSnapshots.erase(std::remove_if(referenceSnapshots.begin(),referenceSnapshots.end(),[](const auto& snapshot){return snapshot.expired();}),referenceSnapshots.end());
    return key;
}
struct Pick {QString name;SolidPoint p;};
std::optional<Pick> pickInfo(PyObject* info){
    if(!PyDict_Check(info))return {};
    auto* name=PyDict_GetItemString(info,"Object");auto* edge=PyDict_GetItemString(info,"Component");
    if(!name||!edge)return {};const char* n=PyUnicode_AsUTF8(name);const char* e=PyUnicode_AsUTF8(edge);
    if(!n||!e||!std::string(e).starts_with("Edge"))return {};
    Pick p;p.name=QString::fromUtf8(n)+"."+QString::fromUtf8(e);const char* axes[]={"x","y","z"};
    for(unsigned i=0;i<3;++i){auto* a=PyDict_GetItemString(info,axes[i]);if(!a)throw std::runtime_error("Missing native curve point");p.p[i]=PyFloat_AsDouble(a);if(PyErr_Occurred()||!std::isfinite(p.p[i]))throw std::runtime_error("Invalid native curve point");}return p;
}
unsigned reference(App::Document& doc,const QString& name,SolidPoint seed,bool hasSeed,const QString& original={}){
    const auto admission=curveNativeAdmission(doc,"Circle");
    auto ref=solidCurveReference(doc,name.toStdString(),"Circle");const auto mode=om9_circle_reference_mode();
    if(mode==1){
        ref.curveEvidence=captureCurveNativeSource(doc,ref.input.name,ref.sub,{},"Circle");
        const auto e=acceptCurveNativeInput(doc,admission,"reference",1,{0,0,0},{0,0,0},{},original,{{ref.curveEvidence,"path",0,0,0}});if(e!=1)return e;path=std::move(ref);
        // A seeded path pick contains two accepted Rust events. Capture the
        // selected-path context before the center event can advance its key.
        captureCircleReferenceContext();
        if(hasSeed){const auto next=curveNativeAdmission(doc,"Circle");const auto [p,n]=solidOnCurve(*path,seed);const auto fraction=solidCurveFraction(*path,p);verifyCircleReferences(doc);return acceptCurveNativeInput(doc,next,"reference",2,p,n,{},original,{},fraction);}return e;
    }
    if(mode!=3)throw std::runtime_error("Native curve pick requires AroundCurve or Tangent curve-selection step");
    const auto [p,n]=solidOnCurve(ref,seed);const auto index=om9_circle_constraints(nullptr);
    ref.curveEvidence=captureCurveNativeSource(doc,ref.input.name,ref.sub,{p},"Circle");
    auto e=acceptCurveNativeInput(doc,admission,"reference",3,p,n,{},original,{{ref.curveEvidence,"tangent",index,0,0}},solidCurveFraction(ref,p));if(e==1)refs.insert_or_assign(index,std::move(ref));return e;
}
}
namespace OpenMatrix9Gui {
App::DocumentObject* circleHistoryObject(App::Document& doc,PyObject* shape){
    if(!om9_circle_history()||!historyRecordingEnabled(doc))return nullptr;
    return createCircleHistoryForCommand(doc,shape,refs,path);
}
void clearCircleReferences(){Base::PyGILStateLocker lock;refs.clear();path.reset();clearCircleHistoryReferences();contexts.clear();referenceSnapshots.clear();}
void captureCircleReferenceContext(){
    Base::PyGILStateLocker lock;const auto key=retainContexts();if(contexts.contains(key))return;
    CircleReferenceContext context;for(const auto& [index,ref]:refs)context.refs.emplace(index,snapshotReference(ref));if(path)context.path=snapshotReference(*path);context.sources=circleHistoryReferenceSnapshot();
    if(contexts.size()>=contextLimit)throw std::runtime_error("Circle native context storage exceeds its bound");
    contexts.emplace(key,std::move(context));
}
void verifyCircleReferences(App::Document& doc){verifySolidReferences(doc,refs,path);for(const auto& [index,ref]:refs)if(ref.curveEvidence)verifyCurveNativeSource(doc,*ref.curveEvidence,"Circle");if(path&&path->curveEvidence)verifyCurveNativeSource(doc,*path->curveEvidence,"Circle");verifyCircleHistoryReferences(doc);}
std::optional<unsigned> circleNativeInput(App::Document& doc,const QString& text){
    Base::PyGILStateLocker lock;captureCircleReferenceContext();const auto mode=om9_circle_reference_mode();const auto t=text.trimmed();
    // Undo must remain available for recovering from a changed/deleted reference.
    if(t.compare("Undo",Qt::CaseInsensitive)!=0&&t.compare("u",Qt::CaseInsensitive)!=0&&t.compare("Cancel",Qt::CaseInsensitive)!=0&&t.compare("Esc",Qt::CaseInsensitive)!=0)verifyCircleReferences(doc);
    if(auto history=circleHistoryNativeInput(doc,text))return history;
    if(t.startsWith("Curve=",Qt::CaseInsensitive)&&(mode==1||mode==3)){
        const auto value=t.mid(6);const auto split=value.indexOf('@');const auto name=split<0?value:value.left(split);SolidPoint seed{0,0,0};
        if(split>=0){const auto coords=value.mid(split+1).split(',');if(coords.size()!=3)throw std::runtime_error("Use Curve=Object.EdgeN@worldX,worldY,worldZ");for(unsigned i=0;i<3;++i){bool ok=false;seed[i]=coords[i].toDouble(&ok);if(!ok||!std::isfinite(seed[i]))throw std::runtime_error("Curve seed coordinates must be finite");}}
        return reference(doc,name,seed,split>=0,text);
    }
    if(mode==2&&t.startsWith("OnCurve=",Qt::CaseInsensitive)){
        if(!path)throw std::runtime_error("Reselect the path");bool ok=false;const double f=t.mid(8).toDouble(&ok);if(!ok)throw std::runtime_error("OnCurve requires a fraction within 0..1");
        const auto admission=curveNativeAdmission(doc,"Circle");const auto [p,n]=solidOnCurve(*path,{0,0,0},f);verifyCircleReferences(doc);return acceptCurveNativeInput(doc,admission,"reference",2,p,n,{},text,{},f);
    }
    if(mode==4&&((t.isEmpty()&&om9_curve_preview_count()==0)||t.compare("Selection",Qt::CaseInsensitive)==0)){
        const auto points=solidSelectedFitPoints(doc,true,om9_curve_preview_count());if(points.empty())throw std::runtime_error("Select at least three native vertices, poles, mesh vertices or point-cloud points");
        std::vector<double> flat;for(const auto& p:points)flat.insert(flat.end(),p.begin(),p.end());const auto e=om9_circle_fit_batch(flat.data(),points.size());if(e!=1)return e;
        if(!t.isEmpty())return e;
    }
    return {};
}
std::optional<unsigned> circleNativePick(App::Document& doc,Gui::View3DInventor* view,const QPoint& pos){
    Base::PyGILStateLocker lock;captureCircleReferenceContext();verifyCircleReferences(doc);const auto mode=om9_circle_reference_mode();if(mode!=1&&mode!=2&&mode!=3)return {};
    Ref v(view->getPyObject());const auto pixel=view->getViewer()->fromQPoint(pos);
    Ref infos(PyObject_CallMethod(v.value,"getObjectsInfo","((ii))",int(pixel[0]),int(pixel[1])));
    if(PyList_Check(infos.value))for(Py_ssize_t i=0;i<PyList_Size(infos.value);++i){auto pick=pickInfo(PyList_GetItem(infos.value,i));if(!pick)continue;
        if(mode==2){if(!path)throw std::runtime_error("Reselect the path");if(pick->name.toStdString()!=path->input.name+"."+path->sub)continue;const auto admission=curveNativeAdmission(doc,"Circle");const auto [p,n]=solidOnCurve(*path,pick->p);const auto fraction=solidCurveFraction(*path,p);verifyCircleReferences(doc);return acceptCurveNativeInput(doc,admission,"reference",2,p,n,{},QString(),{},fraction);}
        return reference(doc,pick->name,pick->p,false);
    }
    throw std::runtime_error(mode==2?"Pick a center on the selected curve, or enter OnCurve=fraction":"Pick a native curve edge, or enter Curve=Object.EdgeN@x,y,z");
}
std::optional<std::array<double,3>> circleNativeHover(App::Document& doc,Gui::View3DInventor* view,const QPoint& pos){
    if(om9_circle_reference_mode()!=3)return {};
    Base::PyGILStateLocker lock;verifyCircleReferences(doc);Ref v(view->getPyObject());const auto pixel=view->getViewer()->fromQPoint(pos);
    Ref info(PyObject_CallMethod(v.value,"getObjectInfo","((ii))",int(pixel[0]),int(pixel[1])));auto pick=pickInfo(info.value);if(!pick)return {};
    auto ref=solidCurveReference(doc,pick->name.toStdString());return solidOnCurve(ref,pick->p).first;
}
unsigned circleNativeResult(App::Document& doc,unsigned effect){
    Base::PyGILStateLocker lock;if(effect==3){clearCircleReferences();return effect;}
    const auto key=retainContexts();if(const auto found=contexts.find(key);found!=contexts.end())restoreContext(found->second);
    pruneCircleHistoryReferences(effect);
    const auto count=om9_circle_constraints(nullptr);for(auto it=refs.begin();it!=refs.end();)if(it->first>=count)it=refs.erase(it);else ++it;
    if(om9_circle_reference_mode()==1||om9_circle_construction()!=5)path.reset();
    if(om9_circle_construction()!=7)refs.clear();
    // Existing contexts are immutable. In particular, an Undo restoration must
    // never replace its snapshot with resources pruned for a newer input.
    captureCircleReferenceContext();
    if(effect==2)verifyCircleReferences(doc);
    if(effect!=1||om9_circle_reference_mode()!=5)return effect;
    verifyCircleReferences(doc);double b[12],frame[13];om9_circle_constraints(b);if(!om9_circle_frame(frame))throw std::runtime_error("No tangent plane");
    const auto admission=curveNativeAdmission(doc,"Circle");std::array<double,3> fractions{};
    const auto plan=circleSolveNative(refs,b,count,frame,om9_circle_solution_index(),om9_circle_from_first(),om9_circle_tangent_vertical(),&fractions);
    verifyCircleReferences(doc);QJsonArray contacts;for(std::size_t i=0;i<count;++i)contacts.append(refs.contains(i)?QJsonValue(fractions[i]):QJsonValue());
    return acceptCircleNativeSolution(doc,admission,plan,contacts);
}
}
