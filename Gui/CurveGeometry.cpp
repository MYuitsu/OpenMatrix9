#include "CurveGeometry.h"
#include "RustBridge.h"
#include <App/Document.h>
#include "CoreGeometryContract.h"
#include "CorePreparedSnapshot.h"
#include "CorePreparedCad.h"
#include "CoreTopologySnapshot.h"
#include "CoreShapeValidation.h"
#include <Mod/Part/App/TopoShapePy.h>
#include <QJsonDocument>
#include <QJsonArray>
#include <QJsonParseError>
#include <Standard_Failure.hxx>
#include <Base/Exception.h>
#include <cmath>
namespace OpenMatrix9Gui {
std::string curveConstructionEvidence(){
    const auto query=om9_curve_construction_json(0,nullptr,0);
    if(query.status||query.reserved||!query.generation||!query.needed||query.needed>om9_prepared_result_limit())
        throw std::runtime_error("Cannot capture completed Curve construction");
    std::string bytes(query.needed,'\0');
    const auto copy=om9_curve_construction_json(query.generation,reinterpret_cast<unsigned char*>(bytes.data()),bytes.size());
    if(copy.status||copy.reserved||copy.generation!=query.generation||copy.needed!=query.needed)
        throw std::runtime_error("Curve construction changed during capture");
    return bytes;
}
void persistCurveConstruction(PyObject* object,const std::string& evidence){
    CurvePyRef property(PyObject_CallMethod(object,"addProperty","sss","App::PropertyString","OM9CurveConstruction","OpenMatrix9"));
    CurvePyRef value(PyUnicode_DecodeUTF8(evidence.data(),Py_ssize_t(evidence.size()),"strict"));
    if(PyObject_SetAttrString(object,"OM9CurveConstruction",value.value)<0)
        throw std::runtime_error("Cannot persist Curve construction");
    CurvePyRef mode(PyObject_CallMethod(object,"setEditorMode","si","OM9CurveConstruction",1));
}
void persistCurveGeometryResult(PyObject* object,const std::string& result){
    if(!PyObject_HasAttrString(object,"OM9GeometryResult")){
        CurvePyRef property(PyObject_CallMethod(object,"addProperty","sss","App::PropertyString","OM9GeometryResult","OpenMatrix9"));
    }
    CurvePyRef value(PyUnicode_DecodeUTF8(result.data(),Py_ssize_t(result.size()),"strict"));
    if(PyObject_SetAttrString(object,"OM9GeometryResult",value.value)<0)throw std::runtime_error("KernelFailure: ConstructCurve result metadata assignment failed");
    CurvePyRef mode(PyObject_CallMethod(object,"setEditorMode","si","OM9GeometryResult",1));
}
std::string curveGeometryResult(App::Document& doc,PyObject* shape,const std::string& construction,const std::string& units,const std::string& outputName,const std::string& outputRole){
    std::string command="ConstructCurve";const char* failure="InvalidBasis";
    try {
        QJsonParseError error;const auto parsed=QJsonDocument::fromJson(QByteArray::fromStdString(construction),&error);
        if(error.error!=QJsonParseError::NoError||!parsed.isObject())throw std::runtime_error("Invalid owned Curve construction JSON");
        const auto recipe=parsed.object();command=recipe.value("command").toString().toStdString();
        validateNativePartShape(shape,command,outputName);
        const auto& native=static_cast<Part::TopoShapePy*>(shape)->getTopoShapePtr()->getShape();
        failure="InvalidTrimTopology";const auto topology=topologySnapshot(native),validation=shapeValidation(native,topology);
        const QJsonObject attributes{{"schema_version",1},{"availability","not_applicable_constructed_output"}};
        const QJsonObject identity{{"document_name",doc.getName()},{"name",QString::fromStdString(outputName)}};
        QJsonObject context{{"schema_version",1},{"operation","ConstructCurve"},{"construction",recipe},
            {"attribute_role","source_evidence_in_input_snapshots_output_attributes_unavailable"},
            {"output_parent_to_world",QJsonArray{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1}}};
        if(!outputRole.empty())context["output_role"]=QString::fromStdString(outputRole);
        failure="InvalidBasis";const auto result=preparedCadResult(native,topology,validation,attributes,identity,units,context);
        return QJsonDocument(result).toJson(QJsonDocument::Compact).toStdString();
    }catch(const PreparedCadFailure&){throw;}
    catch(const Standard_Failure& e){throw std::runtime_error("KernelFailure: "+command+" output "+outputName+": "+(e.GetMessageString()?e.GetMessageString():"native result capture failed"));}
    catch(const Base::Exception& e){throw std::runtime_error("KernelFailure: "+command+" output "+outputName+": "+e.what());}
    catch(const std::exception& e){
        const std::string detail=e.what();
        for(const char* type:{"UnsupportedOperation","UnsupportedRepresentation","InvalidBasis","InvalidTrimTopology","NonFiniteGeometry","StaleGeometry","SingularTransform","KernelFailure"})
            if(detail.rfind(std::string(type)+":",0)==0)throw;
        if(detail.find("budget")!=std::string::npos)failure="InvalidBasis";
        throw std::runtime_error(std::string(failure)+": "+command+" output "+outputName+": "+detail);
    }
}
CurvePyRef publishedSplineShape() {
    // OM9-CURVE-003/009: Rust fits poles; OCCT constructs the native B-spline.
    const auto count=om9_spline_pole_count(), degree=om9_spline_degree(), knots=om9_spline_knot_count();
    if(count<=degree||count>256||!degree||knots<2)throw std::runtime_error("Invalid Rust spline output");
    CurvePyRef app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part"));
    CurvePyRef vector(PyObject_GetAttrString(app.value,"Vector"));
    CurvePyRef poles(PyList_New(0)),mults(PyList_New(0)),values(PyList_New(0));
    for(std::size_t i=0;i<count;++i){
        const double x=om9_spline_pole(i,0),y=om9_spline_pole(i,1),z=om9_spline_pole(i,2);
        if(!std::isfinite(x)||!std::isfinite(y)||!std::isfinite(z))throw std::runtime_error("Nonfinite spline pole");
        CurvePyRef p(PyObject_CallFunction(vector.value,"ddd",x,y,z));
        if(PyList_Append(poles.value,p.value)<0)throw std::runtime_error("Cannot append spline pole");
    }
    for(std::size_t i=0;i<knots;++i){
        CurvePyRef u(PyFloat_FromDouble(om9_spline_knot(i))),m(PyLong_FromSize_t(om9_spline_multiplicity(i)));
        if(PyList_Append(values.value,u.value)<0||PyList_Append(mults.value,m.value)<0)throw std::runtime_error("Cannot append spline knot");
    }
    CurvePyRef type(PyObject_GetAttrString(part.value,"BSplineCurve")),curve(PyObject_CallNoArgs(type.value));
    CurvePyRef built(PyObject_CallMethod(curve.value,"buildFromPolesMultsKnots","OOOOn",poles.value,mults.value,values.value,om9_spline_periodic()?Py_True:Py_False,Py_ssize_t(degree)));
    CurvePyRef shape(PyObject_CallMethod(curve.value,"toShape",nullptr));
    CurvePyRef valid(PyObject_CallMethod(shape.value,"isValid",nullptr));
    if(PyObject_IsTrue(valid.value)!=1)throw std::runtime_error("Part produced invalid spline geometry");
    return shape;
}
CurvePyRef createCurveFeature(App::Document& doc,PyObject* shape,const char* name) {
    CurvePyRef pyDoc(doc.getPyObject()),object(PyObject_CallMethod(pyDoc.value,"addObject","ss","Part::Feature",name));
    if(PyObject_SetAttrString(object.value,"Shape",shape)<0)throw std::runtime_error("Cannot assign curve shape");
    CurvePyRef view(PyObject_GetAttrString(object.value,"ViewObject")),color(Py_BuildValue("(ddd)",0.0,130.0/255.0,85.0/255.0));
    if(PyObject_SetAttrString(view.value,"LineColor",color.value)<0)throw std::runtime_error("Cannot assign curve color");
    return object;
}
std::vector<std::array<double,3>> sampleCurve(PyObject* edge,std::size_t count) {
    CurvePyRef method(PyObject_GetAttrString(edge,"discretize")),args(PyTuple_New(0)),kwargs(Py_BuildValue("{s:n}","Number",Py_ssize_t(count)));
    CurvePyRef points(PyObject_Call(method.value,args.value,kwargs.value));
    const auto n=PySequence_Size(points.value);
    if(n!=Py_ssize_t(count))throw std::runtime_error("Curve cannot be sampled with the requested count");
    std::vector<std::array<double,3>> output;output.reserve(count);
    for(Py_ssize_t i=0;i<n;++i){
        CurvePyRef p(PySequence_GetItem(points.value,i));std::array<double,3> xyz;
        for(int j=0;j<3;++j){CurvePyRef v(PyObject_GetAttrString(p.value,j==0?"x":j==1?"y":"z"));xyz[j]=PyFloat_AsDouble(v.value);if(PyErr_Occurred()||!std::isfinite(xyz[j]))throw std::runtime_error("Invalid sampled coordinate");}
        output.push_back(xyz);
    }
    return output;
}
}
