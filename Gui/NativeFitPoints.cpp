// SPDX-License-Identifier: LGPL-2.1-or-later
#include "NativeFitPoints.h"
#include "NativeInputTree.h"
#include "CoreGeometryContract.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyGeo.h>
#include <App/ComplexGeoData.h>
#include <Mod/Part/App/TopoShape.h>
#include <Mod/Mesh/App/Mesh.h>
#include <Mod/Points/App/Points.h>
#include <BRep_Tool.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <Standard_Failure.hxx>
#include <QApplication>
#include <QThread>
#include <limits>
namespace OpenMatrix9Gui {
namespace {
extern "C" unsigned om9_fit_point_count(std::size_t,std::size_t);
extern "C" unsigned om9_fit_point_valid(unsigned,double,double,double);
extern "C" unsigned om9_prepared_cad_budget(std::size_t,std::size_t,std::size_t,std::size_t);
using Ref=CurvePyRef;
bool exact(const Base::BaseClass& value,const char* name){
    const auto type=Base::Type::fromName(name);return !type.isBad()&&value.getTypeId()==type;
}
struct Reader {
    std::size_t used;
    unsigned kind;
    std::string context;
    std::vector<SolidPoint> result;
    Ref parent;
    Ref app;
    void fail(const char* code,const std::string& message)const{
        throw std::runtime_error(std::string(code)+": "+context+message);
    }
    void count(std::size_t added)const{
        if(const auto status=om9_fit_point_count(used,added))fail(status==5?"KernelFailure":"InvalidBasis","FitPoints supports at most 1024 points in the request");
    }
    void append(double x,double y,double z){
        count(result.size()+1);
        Ref point(PyObject_CallMethod(app.value,"Vector","ddd",x,y,z));
        Ref moved(PyObject_CallMethod(parent.value,"multVec","O",point.value));
        double values[3];const char* axes[]={"x","y","z"};
        for(unsigned i=0;i<3;++i){Ref v(PyObject_GetAttrString(moved.value,axes[i]));values[i]=PyFloat_AsDouble(v.value);
            if(PyErr_Occurred()){PyErr_Clear();fail("KernelFailure","cannot read transformed native point");}}
        const auto status=om9_fit_point_valid(kind,values[0],values[1],values[2]);
        if(status)fail(status==5?"KernelFailure":status==4?"NonFiniteGeometry":"InvalidBasis","native point violates the existing coordinate bounds");
        result.push_back({values[0],values[1],values[2]});
    }
    void append(gp_Pnt p,const TopLoc_Location& location){
        p.Transform(location.Transformation());append(p.X(),p.Y(),p.Z());
    }
    template<class Geometry> void basis(const Geometry& geometry){
        try{ThreeDm::InputTree::validate(geometry);}
        catch(const Standard_Failure& e){fail("KernelFailure",e.GetMessageString()?e.GetMessageString():"native basis inspection failed");}
        catch(const std::exception& e){fail("InvalidBasis",e.what());}
    }
    bool surface(const TopoDS_Shape& face){
        TopLoc_Location location;const auto geometry=BRep_Tool::Surface(TopoDS::Face(face),location);
        auto spline=Handle(Geom_BSplineSurface)::DownCast(geometry);
        auto bezier=Handle(Geom_BezierSurface)::DownCast(geometry);
        if(spline.IsNull()&&bezier.IsNull())return false;
        const std::size_t u=spline.IsNull()?bezier->NbUPoles():spline->NbUPoles();
        const std::size_t v=spline.IsNull()?bezier->NbVPoles():spline->NbVPoles();
        if(v && u>std::numeric_limits<std::size_t>::max()/v)fail("InvalidBasis","surface pole count overflow");
        count(u*v);basis(geometry);result.reserve(u*v);
        for(std::size_t i=1;i<=u;++i)for(std::size_t j=1;j<=v;++j)
            append(spline.IsNull()?bezier->Pole(int(i),int(j)):spline->Pole(int(i),int(j)),location);
        return true;
    }
    bool curve(const TopoDS_Shape& edge){
        TopLoc_Location location;double first,last;
        const auto geometry=BRep_Tool::Curve(TopoDS::Edge(edge),location,first,last);
        auto spline=Handle(Geom_BSplineCurve)::DownCast(geometry);
        auto bezier=Handle(Geom_BezierCurve)::DownCast(geometry);
        if(spline.IsNull()&&bezier.IsNull())return false;
        const std::size_t n=spline.IsNull()?bezier->NbPoles():spline->NbPoles();
        count(n);basis(geometry);result.reserve(n);
        for(std::size_t i=1;i<=n;++i)append(spline.IsNull()?bezier->Pole(int(i)):spline->Pole(int(i)),location);
        return true;
    }
    TopoDS_Shape single(const TopoDS_Shape& shape,TopAbs_ShapeEnum type){
        TopTools_IndexedMapOfShape seen;std::size_t visits=0;
        for(TopExp_Explorer it(shape,type);it.More();it.Next()){
            if(om9_prepared_cad_budget(0,0,++visits,0))fail("InvalidBasis","native topology traversal budget exceeded");
            seen.Add(it.Current());if(seen.Extent()>1)return {};
        }
        return seen.Extent()==1?seen(1):TopoDS_Shape();
    }
    void part(const Part::TopoShape& owner,const std::string& sub,bool surfacePoles){
        if(!sub.empty()){
            if(!sub.starts_with("Vertex")&&!(surfacePoles&&sub.starts_with("Face")))
                fail("UnsupportedRepresentation","FitPoints subelements must be vertices or supported surface faces");
            const auto selected=owner.getSubShape(sub.c_str());
            if(selected.ShapeType()==TopAbs_VERTEX){const auto p=BRep_Tool::Pnt(TopoDS::Vertex(selected));append(p.X(),p.Y(),p.Z());return;}
            if(!surface(selected))fail("UnsupportedRepresentation","selected face has no surface control points");
            return;
        }
        const auto& shape=owner.getShape();
        if(surfacePoles){const auto face=single(shape,TopAbs_FACE);if(!face.IsNull()&&surface(face))return;}
        const auto edge=single(shape,TopAbs_EDGE);if(!edge.IsNull()&&curve(edge))return;
        TopTools_IndexedMapOfShape vertices;std::size_t visits=0;
        for(TopExp_Explorer it(shape,TopAbs_VERTEX);it.More();it.Next()){
            if(om9_prepared_cad_budget(0,0,++visits,0))fail("InvalidBasis","native vertex traversal budget exceeded");
            if(vertices.Contains(it.Current()))continue;
            count(result.size()+1);vertices.Add(it.Current());
            const auto p=BRep_Tool::Pnt(TopoDS::Vertex(it.Current()));append(p.X(),p.Y(),p.Z());
        }
    }
};
}
std::vector<SolidPoint> nativeFitPoints(App::Document& doc,const std::string& name,
    const std::string& sub,bool surfacePoles,std::size_t used,const char* operation,bool currentCurve,bool currentSolid,const std::function<void(unsigned)>& independentAdmission){
    const std::string context=std::string(operation)+" input "+name+(sub.empty()?"":"."+sub)+": ";
    auto fail=[&](const char* code,const std::string& message){throw std::runtime_error(std::string(code)+": "+context+message);};
    if(!qApp||QThread::currentThread()!=qApp->thread()||!PyGILState_Check())fail("KernelFailure","native FitPoints requires GUI thread/GIL");
    auto* object=doc.getObject(name.c_str());
    if(!object||!object->isAttachedToDocument()||!object->isValid())fail("StaleGeometry","source is missing or invalid");
    if(object->isDerivedFrom(Base::Type::fromName("App::Link")))fail("UnsupportedRepresentation","FitPoints does not accept linked references");
    try {
        Ref py(object->getPyObject());
        Ref global(PyObject_CallMethod(py.value,"getGlobalPlacement",nullptr)),local(PyObject_GetAttrString(py.value,"Placement")),
            inverse(PyObject_CallMethod(local.value,"inverse",nullptr)),parent(PyNumber_Multiply(global.value,inverse.value));
        Reader reader{used,surfacePoles?1U:2U,context,{},std::move(parent),Ref(PyImport_ImportModule("FreeCAD"))};
        auto admit=[&](unsigned representation){const auto identity=name+(sub.empty()?"":"."+sub);if(independentAdmission)independentAdmission(representation);else if(currentCurve)validateCurrentCurveCapability(representation,2,operation,identity);else if(currentSolid)validateCurrentSolidCapability(representation,2,operation,identity);};
        if(auto* shapeProperty=object->getPropertyByName<App::PropertyComplexGeoData>("Shape")){
            if(!exact(*shapeProperty,"Part::PropertyPartShape"))reader.fail("UnsupportedRepresentation","no builtin native shape property owner");
            const auto* value=shapeProperty->getComplexData();if(!value)reader.fail("StaleGeometry","native shape owner is unavailable");
            const auto& shape=static_cast<const Part::TopoShape*>(value)->getShape();
            const unsigned representation=sub.starts_with("Vertex")?1U:sub.starts_with("Face")?4U:
                shape.ShapeType()==TopAbs_VERTEX?1U:shape.ShapeType()==TopAbs_EDGE?2U:shape.ShapeType()==TopAbs_WIRE?3U:
                shape.ShapeType()==TopAbs_FACE?4U:shape.ShapeType()==TopAbs_SHELL?5U:shape.ShapeType()==TopAbs_SOLID?6U:7U;
            admit(representation);
            reader.part(*static_cast<const Part::TopoShape*>(value),sub,surfacePoles);
        }else{
            auto* property=object->getPropertyByName<App::PropertyComplexGeoData>("Mesh");const bool mesh=property!=nullptr;
            if(!property)property=object->getPropertyByName<App::PropertyComplexGeoData>("Points");
            if(!property||!exact(*property,mesh?"Mesh::PropertyMeshKernel":"Points::PropertyPointKernel"))
                reader.fail("UnsupportedRepresentation","no builtin native mesh/cloud property owner");
            // Sphere's existing cloud reader ignores subelement labels and
            // fits the whole cloud; Circle requires a whole-cloud selection.
            if(!sub.empty()&&(mesh||surfacePoles))reader.fail("UnsupportedRepresentation","select the whole mesh or point cloud");
            const auto* value=property->getComplexData();if(!value)reader.fail("StaleGeometry","native discrete owner is unavailable");
            admit(mesh?8U:9U);
            if(mesh&&!exact(*value,"Mesh::MeshObject"))reader.fail("UnsupportedRepresentation","native mesh owner has an unsupported type");
            const auto matrix=value->getTransform();
            auto append=[&](const auto& p){const auto moved=matrix*Base::Vector3d(p.x,p.y,p.z);reader.append(moved.x,moved.y,moved.z);};
            if(mesh){const auto& vertices=static_cast<const Mesh::MeshObject*>(value)->getKernel().GetPoints();
                reader.count(vertices.size());reader.result.reserve(vertices.size());for(const auto& p:vertices)append(p);
            }else{const auto& vertices=static_cast<const Points::PointKernel*>(value)->getBasicPoints();
                reader.count(vertices.size());reader.result.reserve(vertices.size());for(const auto& p:vertices)append(p);}
        }
        return std::move(reader.result);
    }catch(const Standard_Failure& e){fail("KernelFailure",e.GetMessageString()?e.GetMessageString():"native FitPoints inspection failed");}
    return {};
}
}
