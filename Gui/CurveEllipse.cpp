// SPDX-License-Identifier: LGPL-2.1-or-later
// OM9-CURVE-006: GUI-thread FreeCAD/Part adapter. Rust owns geometry and session state.
#include "CurveEllipse.h"
#include "CoreUnits.h"
#include "SolidReferences.h"
#include "RustBridge.h"
#include "CoreLayers.h"
#include <Base/Interpreter.h>
#include <App/Document.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <algorithm>
#include <cmath>
#include <limits>
namespace {
using namespace OpenMatrix9Gui;
using Ref=CurvePyRef;
std::optional<SolidCurveReference> path;
struct Pick {QString name;SolidPoint point;};
std::optional<Pick> pickInfo(PyObject* info){
    if(!PyDict_Check(info))return {};
    auto* object=PyDict_GetItemString(info,"Object");auto* component=PyDict_GetItemString(info,"Component");
    if(!object||!component)return {};
    const char* name=PyUnicode_AsUTF8(object);const char* sub=PyUnicode_AsUTF8(component);
    if(!name||!sub||!std::string(sub).starts_with("Edge"))return {};
    Pick result;result.name=QString::fromUtf8(name)+"."+QString::fromUtf8(sub);
    const char* axes[]={"x","y","z"};
    for(unsigned i=0;i<3;++i){auto* coordinate=PyDict_GetItemString(info,axes[i]);if(!coordinate)throw std::runtime_error("Missing native Ellipse pick coordinate");result.point[i]=PyFloat_AsDouble(coordinate);if(PyErr_Occurred()||!std::isfinite(result.point[i]))throw std::runtime_error("Invalid native Ellipse pick coordinate");}
    return result;
}
unsigned selectReference(App::Document& doc,const QString& name){
    auto selected=solidCurveReference(doc,name.toStdString());
    verifySolidReferences(doc,{},selected);
    const auto effect=om9_ellipse_reference(0,0,0,0,0,0,1);
    if(effect==1)path=std::move(selected);
    return effect;
}
unsigned acceptCenter(App::Document& doc,const SolidPoint& seed,std::optional<double> fraction={}){
    if(!path)throw std::runtime_error("Reselect the Ellipse path");
    verifyEllipseReferences(doc);
    const auto [point,tangent]=solidOnCurve(*path,seed,fraction);
    verifyEllipseReferences(doc);
    return om9_ellipse_reference(point[0],point[1],point[2],tangent[0],tangent[1],tangent[2],2);
}
Ref vector(PyObject* type,const double* coordinates){return Ref(PyObject_CallFunction(type,"ddd",coordinates[0],coordinates[1],coordinates[2]));}
void preflight(PyObject* shape,bool wire){
    Ref valid(PyObject_CallMethod(shape,"isValid",nullptr));
    Ref empty(PyObject_CallMethod(shape,"isNull",nullptr));
    if(PyObject_IsTrue(valid.value)!=1||PyObject_IsTrue(empty.value)!=0)throw std::runtime_error("Part produced invalid Ellipse geometry");
    if(wire){
        Ref closed(PyObject_CallMethod(shape,"isClosed",nullptr)),edges(PyObject_GetAttrString(shape,"Edges"));
        if(PyObject_IsTrue(closed.value)!=1||PySequence_Size(edges.value)!=1)throw std::runtime_error("Ellipse must be a closed single-edge wire");
    }
}
void stringProperty(PyObject* object,const char* key,const char* value){
    Ref property(PyObject_CallMethod(object,"addProperty","sss","App::PropertyString",key,"OpenMatrix9"));
    Ref text(PyUnicode_FromString(value));
    if(PyObject_SetAttrString(object,key,text.value)<0)throw std::runtime_error("Cannot persist Ellipse metadata");
}
void applyLayer(App::Document& doc,PyObject* object,const CircleLayerSnapshot& layer){
    Ref name(PyObject_GetAttrString(object,"Name"));const char* nativeName=PyUnicode_AsUTF8(name.value);
    auto* native=nativeName?doc.getObject(nativeName):nullptr;
    if(!native)throw std::runtime_error("Ellipse output identity missing");
    CoreLayers::applyCircle(doc,*native,layer);
}
}
namespace OpenMatrix9Gui {
void clearEllipseReferences(){Base::PyGILStateLocker lock;path.reset();}
void verifyEllipseReferences(App::Document& doc){verifySolidReferences(doc,{},path);}
std::optional<unsigned> ellipseNativeInput(App::Document& doc,const QString& input){
    Base::PyGILStateLocker lock;const auto text=input.trimmed();
    // Undo keeps the edge while removing a center/axis; returning to edge-selection
    // clears it in ellipseNativeResult. Permit recovery from stale references.
    if(text.compare("Undo",Qt::CaseInsensitive)==0||text.compare("u",Qt::CaseInsensitive)==0||text.compare("Cancel",Qt::CaseInsensitive)==0||text.compare("Esc",Qt::CaseInsensitive)==0)return {};
    verifyEllipseReferences(doc);
    const auto mode=om9_ellipse_reference_mode();
    if(text.startsWith("Curve=",Qt::CaseInsensitive)&&mode==1)return selectReference(doc,text.mid(6).trimmed());
    if(text.startsWith("OnCurve=",Qt::CaseInsensitive)&&mode==2){
        bool ok=false;const double fraction=text.mid(8).toDouble(&ok);
        if(!ok)throw std::runtime_error("OnCurve requires a fraction within 0..1");
        return acceptCenter(doc,{0,0,0},fraction);
    }
    for(const auto* option:{"Center","Diameter","Corner","Vertical","FromFoci"})if(text.compare(option,Qt::CaseInsensitive)==0&&om9_curve_preview_count()==0){
        const auto effect=om9_curve_input(text.toUtf8().constData());
        if(effect==1&&om9_ellipse_reference_mode()==0)clearEllipseReferences();
        return effect;
    }
    return {};
}
std::optional<unsigned> ellipseNativePick(App::Document& doc,Gui::View3DInventor* view,const QPoint& position){
    const auto mode=om9_ellipse_reference_mode();if(mode!=1&&mode!=2)return {};
    Base::PyGILStateLocker lock;verifyEllipseReferences(doc);Ref nativeView(view->getPyObject());
    const auto pixel=view->getViewer()->fromQPoint(position);
    Ref infos(PyObject_CallMethod(nativeView.value,"getObjectsInfo","((ii))",int(pixel[0]),int(pixel[1])));
    if(PyList_Check(infos.value))for(Py_ssize_t i=0;i<PyList_Size(infos.value);++i){
        auto picked=pickInfo(PyList_GetItem(infos.value,i));if(!picked)continue;
        if(mode==1)return selectReference(doc,picked->name);
        if(!path)throw std::runtime_error("Reselect the Ellipse path");
        if(picked->name.toStdString()==path->input.name+"."+path->sub)return acceptCenter(doc,picked->point);
    }
    throw std::runtime_error(mode==1?"Pick a native curve edge, or enter Curve=Object.EdgeN":"Pick a center on the selected curve, or enter OnCurve=fraction");
}
std::optional<std::array<double,3>> ellipseNativeHover(App::Document& doc,Gui::View3DInventor* view,const QPoint& position){
    if(om9_ellipse_reference_mode()!=2||!path)return {};
    Base::PyGILStateLocker lock;verifyEllipseReferences(doc);Ref nativeView(view->getPyObject());
    const auto pixel=view->getViewer()->fromQPoint(position);
    Ref infos(PyObject_CallMethod(nativeView.value,"getObjectsInfo","((ii))",int(pixel[0]),int(pixel[1])));
    if(PyList_Check(infos.value))for(Py_ssize_t i=0;i<PyList_Size(infos.value);++i){
        auto picked=pickInfo(PyList_GetItem(infos.value,i));
        if(picked&&picked->name.toStdString()==path->input.name+"."+path->sub)return solidOnCurve(*path,picked->point).first;
    }
    return {};
}
void ellipseNativeResult(unsigned effect){if(effect==3||om9_ellipse_reference_mode()==1)clearEllipseReferences();}
void commitEllipse(App::Document& doc){
    Base::PyGILStateLocker lock;verifyEllipseReferences(doc);
    double plan[11];if(!om9_curve_ellipse_plan(plan))throw std::runtime_error("No committed Ellipse plan");
    Ref app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part")),vectorType(PyObject_GetAttrString(app.value,"Vector"));
    Ref center(vector(vectorType.value,plan)),major(vector(vectorType.value,plan+3)),normal(vector(vectorType.value,plan+6));
    Ref edge([&]()->PyObject*{
        if(om9_ellipse_deformable()){
            if(!om9_curve_spline_publish())throw std::runtime_error("No committed deformable Ellipse");
            auto spline=publishedSplineShape();return Py_NewRef(spline.value);
        }
        if(std::abs(plan[9]-plan[10])<=4*std::numeric_limits<double>::epsilon()*plan[9])return PyObject_CallMethod(part.value,"makeCircle","dOO",plan[9],center.value,normal.value);
        Ref type(PyObject_GetAttrString(part.value,"Ellipse"));
        Ref ellipse(PyObject_CallFunction(type.value,"Odd",center.value,plan[9],plan[10]));
        if(PyObject_SetAttrString(ellipse.value,"Axis",normal.value)<0||PyObject_SetAttrString(ellipse.value,"XAxis",major.value)<0)throw std::runtime_error("Cannot orient native Ellipse");
        return PyObject_CallMethod(ellipse.value,"toShape",nullptr);
    }());
    Ref edges(PyList_New(0));if(PyList_Append(edges.value,edge.value)<0)throw std::runtime_error("Cannot append Ellipse edge");
    Ref shape(PyObject_CallMethod(part.value,"Wire","O",edges.value));preflight(shape.value,true);
    // MarkFoci is enabled by Rust only for FromFoci. Prebuild both vertices before the transaction.
    std::vector<Ref> foci;
    if(om9_ellipse_mark_foci()){
        const double distance=std::sqrt(std::max(0.,(plan[9]-plan[10])*(plan[9]+plan[10])));
        for(const double sign:{-1.,1.}){
            const double point[3]={plan[0]+sign*distance*plan[3],plan[1]+sign*distance*plan[4],plan[2]+sign*distance*plan[5]};
            Ref position(vector(vectorType.value,point)),vertex(PyObject_CallMethod(part.value,"Vertex","O",position.value));
            preflight(vertex.value,false);
            Ref vertices(PyObject_GetAttrString(vertex.value,"Vertexes"));if(PySequence_Size(vertices.value)!=1)throw std::runtime_error("Ellipse focus must be a native vertex");
            foci.push_back(std::move(vertex));
        }
    }
    const auto layer=CoreLayers::captureCircle(doc);verifyEllipseReferences(doc);
    doc.openTransaction("Ellipse");
    try {
        auto object=createCurveFeature(doc,shape.value,"Ellipse");
        stringProperty(object.value,"OM9FeatureId","OM9-CURVE-006");stringProperty(object.value,"OM9Role","Ellipse");
        Ref kind(PyObject_CallMethod(object.value,"addProperty","sss","App::PropertyBool","EllipseDeformable","OpenMatrix9"));
        if(PyObject_SetAttrString(object.value,"EllipseDeformable",om9_ellipse_deformable()?Py_True:Py_False)<0)throw std::runtime_error("Cannot persist Ellipse output kind");
        Ref deviationProperty(PyObject_CallMethod(object.value,"addProperty","sss","App::PropertyLength","EllipseApproxDeviation","OpenMatrix9")),deviation(PyFloat_FromDouble(om9_ellipse_deviation()));
        if(PyObject_SetAttrString(object.value,"EllipseApproxDeviation",deviation.value)<0)throw std::runtime_error("Cannot persist Ellipse approximation deviation");
        applyLayer(doc,object.value,layer);
        const auto units=unitContextSnapshot(doc);
        if(!units.empty())stringProperty(object.value,"OM9UnitContext",units.c_str());
        for(std::size_t i=0;i<foci.size();++i){
            auto focusObject=createCurveFeature(doc,foci[i].value,i==0?"EllipseFocus1":"EllipseFocus2");
            stringProperty(focusObject.value,"OM9FeatureId","OM9-CURVE-006");stringProperty(focusObject.value,"OM9Role",i==0?"Focus1":"Focus2");
            Ref view(PyObject_GetAttrString(focusObject.value,"ViewObject")),color(Py_BuildValue("(ddd)",layer.color[0],layer.color[1],layer.color[2])),size(PyFloat_FromDouble(5.));
            if(PyObject_SetAttrString(view.value,"PointColor",color.value)<0||PyObject_SetAttrString(view.value,"PointSize",size.value)<0)throw std::runtime_error("Cannot style Ellipse focus");
            applyLayer(doc,focusObject.value,layer);
            if(!units.empty())stringProperty(focusObject.value,"OM9UnitContext",units.c_str());
        }
        doc.recompute();doc.commitTransaction();
    }catch(...){doc.abortTransaction();throw;}
}
}
