// SPDX-License-Identifier: LGPL-2.1-or-later
// OM9-CURVE-005: host references and bounded exact native tangent solving.
#include "CurveCircle.h"
#include "SolidReferences.h"
#include "RustBridge.h"
#include "CircleHistory.h"
#include "CircleHistoryAdapter.h"
#include "CircleTangent.h"
#include "HistoryFeature.h"
#include <Base/Interpreter.h>
#include <App/Document.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <cmath>
namespace {
using namespace OpenMatrix9Gui;
std::map<std::size_t,SolidCurveReference> refs;
std::optional<SolidCurveReference> path;
using Ref=CurvePyRef;
struct Pick {QString name;SolidPoint p;};
std::optional<Pick> pickInfo(PyObject* info){
    if(!PyDict_Check(info))return {};
    auto* name=PyDict_GetItemString(info,"Object");auto* edge=PyDict_GetItemString(info,"Component");
    if(!name||!edge)return {};const char* n=PyUnicode_AsUTF8(name);const char* e=PyUnicode_AsUTF8(edge);
    if(!n||!e||!std::string(e).starts_with("Edge"))return {};
    Pick p;p.name=QString::fromUtf8(n)+"."+QString::fromUtf8(e);const char* axes[]={"x","y","z"};
    for(unsigned i=0;i<3;++i){auto* a=PyDict_GetItemString(info,axes[i]);if(!a)throw std::runtime_error("Missing native curve point");p.p[i]=PyFloat_AsDouble(a);if(PyErr_Occurred()||!std::isfinite(p.p[i]))throw std::runtime_error("Invalid native curve point");}return p;
}
unsigned reference(App::Document& doc,const QString& name,SolidPoint seed,bool hasSeed){
    auto ref=solidCurveReference(doc,name.toStdString());const auto mode=om9_circle_reference_mode();
    if(mode==1){
        const auto e=om9_circle_reference(0,0,0,0,0,0,1);if(e!=1)return e;path=std::move(ref);
        if(hasSeed){const auto [p,n]=solidOnCurve(*path,seed);return om9_circle_reference(p[0],p[1],p[2],n[0],n[1],n[2],2);}return e;
    }
    if(mode!=3)throw std::runtime_error("Native curve pick requires AroundCurve or Tangent curve-selection step");
    const auto [p,n]=solidOnCurve(ref,seed);const auto index=om9_circle_constraints(nullptr);
    auto e=om9_circle_reference(p[0],p[1],p[2],n[0],n[1],n[2],3);if(e==1)refs.insert_or_assign(index,std::move(ref));return e;
}
}
namespace OpenMatrix9Gui {
App::DocumentObject* circleHistoryObject(App::Document& doc,PyObject* shape){
    if(!om9_circle_history()||!historyRecordingEnabled(doc))return nullptr;
    return createCircleHistoryForCommand(doc,shape,refs,path);
}
void clearCircleReferences(){Base::PyGILStateLocker lock;refs.clear();path.reset();clearCircleHistoryReferences();}
void verifyCircleReferences(App::Document& doc){verifySolidReferences(doc,refs,path);verifyCircleHistoryReferences(doc);}
std::optional<unsigned> circleNativeInput(App::Document& doc,const QString& text){
    Base::PyGILStateLocker lock;const auto mode=om9_circle_reference_mode();const auto t=text.trimmed();
    // Undo must remain available for recovering from a changed/deleted reference.
    if(t.compare("Undo",Qt::CaseInsensitive)!=0&&t.compare("u",Qt::CaseInsensitive)!=0&&t.compare("Cancel",Qt::CaseInsensitive)!=0&&t.compare("Esc",Qt::CaseInsensitive)!=0)verifyCircleReferences(doc);
    if(auto history=circleHistoryNativeInput(doc,t))return history;
    if(t.startsWith("Curve=",Qt::CaseInsensitive)&&(mode==1||mode==3)){
        const auto value=t.mid(6);const auto split=value.indexOf('@');const auto name=split<0?value:value.left(split);SolidPoint seed{0,0,0};
        if(split>=0){const auto coords=value.mid(split+1).split(',');if(coords.size()!=3)throw std::runtime_error("Use Curve=Object.EdgeN@worldX,worldY,worldZ");for(unsigned i=0;i<3;++i){bool ok=false;seed[i]=coords[i].toDouble(&ok);if(!ok||!std::isfinite(seed[i]))throw std::runtime_error("Curve seed coordinates must be finite");}}
        return reference(doc,name,seed,split>=0);
    }
    if(mode==2&&t.startsWith("OnCurve=",Qt::CaseInsensitive)){
        if(!path)throw std::runtime_error("Reselect the path");bool ok=false;const double f=t.mid(8).toDouble(&ok);if(!ok)throw std::runtime_error("OnCurve requires a fraction within 0..1");
        const auto [p,n]=solidOnCurve(*path,{0,0,0},f);return om9_circle_reference(p[0],p[1],p[2],n[0],n[1],n[2],2);
    }
    if(mode==4&&((t.isEmpty()&&om9_curve_preview_count()==0)||t.compare("Selection",Qt::CaseInsensitive)==0)){
        const auto points=solidSelectedFitPoints(doc,true);if(points.empty())throw std::runtime_error("Select at least three native vertices, poles, mesh vertices or point-cloud points");
        std::vector<double> flat;for(const auto& p:points)flat.insert(flat.end(),p.begin(),p.end());const auto e=om9_circle_fit_batch(flat.data(),points.size());if(e!=1)return e;
        if(!t.isEmpty())return e;
    }
    return {};
}
std::optional<unsigned> circleNativePick(App::Document& doc,Gui::View3DInventor* view,const QPoint& pos){
    const auto mode=om9_circle_reference_mode();if(mode!=1&&mode!=2&&mode!=3)return {};
    Base::PyGILStateLocker lock;verifyCircleReferences(doc);Ref v(view->getPyObject());const auto pixel=view->getViewer()->fromQPoint(pos);
    Ref infos(PyObject_CallMethod(v.value,"getObjectsInfo","((ii))",int(pixel[0]),int(pixel[1])));
    if(PyList_Check(infos.value))for(Py_ssize_t i=0;i<PyList_Size(infos.value);++i){auto pick=pickInfo(PyList_GetItem(infos.value,i));if(!pick)continue;
        if(mode==2){if(!path)throw std::runtime_error("Reselect the path");if(pick->name.toStdString()!=path->input.name+"."+path->sub)continue;const auto [p,n]=solidOnCurve(*path,pick->p);return om9_circle_reference(p[0],p[1],p[2],n[0],n[1],n[2],2);}
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
    Base::PyGILStateLocker lock;pruneCircleHistoryReferences(effect);
    const auto count=om9_circle_constraints(nullptr);for(auto it=refs.begin();it!=refs.end();)if(it->first>=count)it=refs.erase(it);else ++it;
    if(om9_circle_reference_mode()==1||om9_circle_construction()!=5)path.reset();
    if(om9_circle_construction()!=7)refs.clear();
    if(effect!=1||om9_circle_reference_mode()!=5)return effect;
    verifyCircleReferences(doc);double b[12],frame[13];om9_circle_constraints(b);if(!om9_circle_frame(frame))throw std::runtime_error("No tangent plane");
    const auto plan=circleSolveNative(refs,b,count,frame,om9_circle_solution_index(),om9_circle_from_first(),om9_circle_tangent_vertical());
    return om9_circle_solution_accept_normal(plan[0],plan[1],plan[2],plan[3],plan[4],plan[5],plan[6]);
}
}
