// SPDX-License-Identifier: LGPL-2.1-or-later
#include "SurfaceGeometry.h"
#include "SurfaceLoft.h"
#include "SurfaceSeams.h"
#include "SurfaceRefit.h"
#include "SurfaceConstraints.h"
#include "SurfaceHistory.h"
#include "CurveGeometry.h"
#include "RustBridge.h"
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <Base/Interpreter.h>
#include <cmath>
#include <stdexcept>
#include <sstream>
#include <utility>
#include <algorithm>
#include <array>
namespace {
std::string pythonError() {
    PyObject *type=nullptr,*value=nullptr,*trace=nullptr;PyErr_Fetch(&type,&value,&trace);PyErr_NormalizeException(&type,&value,&trace);
    PyObject* text=value?PyObject_Str(value):nullptr;
    const char* message=text?PyUnicode_AsUTF8(text):nullptr;std::string result=message?message:"FreeCAD Part operation failed";
    Py_XDECREF(text);Py_XDECREF(type);Py_XDECREF(value);Py_XDECREF(trace);PyErr_Clear();return result;
}
struct Ref {
    PyObject* p;
    explicit Ref(PyObject* v):p(v){if(!p)throw std::runtime_error(pythonError());}
    ~Ref(){Py_XDECREF(p);}Ref(const Ref&)=delete;Ref& operator=(const Ref&)=delete;
    Ref(Ref&& other)noexcept:p(std::exchange(other.p,nullptr)){}
    PyObject* release(){return std::exchange(p,nullptr);}
};
bool flag(PyObject* shape,const char* method){Ref v(PyObject_CallMethod(shape,method,nullptr));const int value=PyObject_IsTrue(v.p);if(value<0)throw std::runtime_error(pythonError());return value==1;}
void set(PyObject* object,const char* key,PyObject* value){if(PyObject_SetAttrString(object,key,value)<0)throw std::runtime_error(pythonError());}
std::string string(PyObject* object,const char* attr){Ref v(PyObject_GetAttrString(object,attr));const char* text=PyUnicode_AsUTF8(v.p);if(!text)throw std::runtime_error(pythonError());return text;}
void property(PyObject* object,const char* type,const char* key,PyObject* value){Ref add(PyObject_CallMethod(object,"addProperty","sss",type,key,"OpenMatrix9"));set(object,key,value);}
Ref wireFromShape(PyObject* part,PyObject* shape){Ref edges(PyObject_GetAttrString(shape,"Edges"));return Ref(PyObject_CallMethod(part,"Wire","O",edges.p));}
void verifyCurves(PyObject* part,PyObject* wires,PyObject* surface) {
    for(Py_ssize_t i=0;i<PyList_Size(wires);++i){
        auto* wire=PyList_GetItem(wires,i);Ref length(PyObject_GetAttrString(wire,"Length"));
        const double tolerance=std::max(1e-4,PyFloat_AsDouble(length.p)*1e-7);
        Ref points(PyObject_CallMethod(wire,"discretize","i",65));
        for(Py_ssize_t j=0;j<PySequence_Size(points.p);++j){Ref point(PySequence_GetItem(points.p,j));Ref vertex(PyObject_CallMethod(part,"Vertex","O",point.p));Ref distance(PyObject_CallMethod(vertex.p,"distToShape","O",surface));
            Ref d(PySequence_GetItem(distance.p,0));const double value=PyFloat_AsDouble(d.p);
            if(!std::isfinite(value)||value>tolerance)throw std::runtime_error("Sweep 2 cannot satisfy both rails and all profiles with these inputs; adjust the profiles or rail directions");
        }
    }
}
std::array<double,3> coordinates(PyObject* point){
    std::array<double,3> p;const char* keys[]={"x","y","z"};for(unsigned i=0;i<3;++i){Ref v(PyObject_GetAttrString(point,keys[i]));p[i]=PyFloat_AsDouble(v.p);}return p;
}
Ref transportedSections(PyObject* part,PyObject* wires,bool maintainHeight,const std::vector<std::pair<double,double>>& slashes) {
    // A single profile is carried by two rails; rail separation scales the
    // profile. Never call OCCT's crashing ContactOnBorder path on this SDK.
    auto* rail=PyList_GetItem(wires,0);auto* auxiliary=PyList_GetItem(wires,1);auto* section=PyList_GetItem(wires,2);
    Ref app(PyImport_ImportModule("FreeCAD")),aPoints(PyObject_CallMethod(rail,"discretize","i",65)),bPoints(PyObject_CallMethod(auxiliary,"discretize","i",65));
    std::vector<std::array<double,3>> a,b;
    for(Py_ssize_t i=0;i<PySequence_Size(aPoints.p);++i){Ref p(PySequence_GetItem(aPoints.p,i));a.push_back(coordinates(p.p));}
    for(Py_ssize_t i=0;i<PySequence_Size(bPoints.p);++i){Ref p(PySequence_GetItem(bPoints.p,i));b.push_back(coordinates(p.p));}
    if(!slashes.empty()){
        std::vector<double> stations,pairs;for(unsigned i=0;i<=64;++i)stations.push_back(double(i)/64);
        for(const auto& [x,y]:slashes){stations.push_back(x);pairs.insert(pairs.end(),{x,y});}
        std::sort(stations.begin(),stations.end());stations.erase(std::unique(stations.begin(),stations.end()),stations.end());
        a.clear();b.clear();
        for(double x:stations){const double y=om9_surface_slash_parameter(pairs.data(),slashes.size(),x);
            if(!std::isfinite(y))throw std::runtime_error("Slash pairs must be strictly increasing interior positions on both rails");
            a.push_back(OpenMatrix9Gui::surfaceRailPoint(rail,x));b.push_back(OpenMatrix9Gui::surfaceRailPoint(auxiliary,y));}
    }
    if(a.size()!=b.size()||a.size()<3||flag(rail,"isClosed")!=flag(auxiliary,"isClosed"))throw std::runtime_error("Sweep2 rails need matching open/closed state");
    for(PyObject* points:{aPoints.p,bPoints.p}){Ref point(PySequence_GetItem(points,0)),vertex(PyObject_CallMethod(part,"Vertex","O",point.p)),distance(PyObject_CallMethod(vertex.p,"distToShape","O",section)),d(PySequence_GetItem(distance.p,0));
        if(PyFloat_AsDouble(d.p)>1e-4)throw std::runtime_error("Single-profile Sweep2: profile must meet both rail starts; align rail seams or use multiple profiles");}
    auto tangent=[&](std::size_t i){const auto before=i==0?0:i-1,after=i+1<a.size()?i+1:i;std::array<double,3> value;for(unsigned k=0;k<3;++k)value[k]=a[after][k]-a[before][k];return value;};
    Ref sections(PyList_New(0));const auto t0=tangent(0);const bool closed=flag(rail,"isClosed");
    for(std::size_t i=0;i<a.size()-(closed?1:0);++i){
        const auto t=tangent(i);double p[18],matrix[16];for(unsigned k=0;k<3;++k){p[k]=a[0][k];p[3+k]=b[0][k];p[6+k]=t0[k];p[9+k]=a[i][k];p[12+k]=b[i][k];p[15+k]=t[k];}
        if(!om9_surface_transport_height(p,maintainHeight,matrix))throw std::runtime_error("Sweep2 rails intersect or cannot define a section frame");
        Ref transform(PyObject_CallMethod(app.p,"Matrix",nullptr));for(unsigned row=0;row<4;++row)for(unsigned col=0;col<4;++col){const std::string key="A"+std::to_string(row+1)+std::to_string(col+1);Ref value(PyFloat_FromDouble(matrix[row*4+col]));set(transform.p,key.c_str(),value.p);}
        Ref moved(PyObject_CallMethod(section,"transformGeometry","O",transform.p));Ref wire=wireFromShape(part,moved.p);if(PyList_Append(sections.p,wire.p)<0)throw std::runtime_error(pythonError());
    }
    return sections;
}
}
namespace OpenMatrix9Gui {
PyObject* surfaceWire(App::Document& doc,const SurfaceInput& input) {
    auto* object=doc.getObject(input.object.c_str());if(!object)throw std::runtime_error("An input curve was deleted");
    Ref part(PyImport_ImportModule("Part"));Ref pyObject(object->getPyObject());Ref original(PyObject_GetAttrString(pyObject.p,"Shape"));Ref shape(PyObject_CallMethod(original.p,"copy","OO",Py_True,Py_False));
    if(!input.chain.empty()){
        Ref edges(PyList_New(0));
        for(const auto& [name,sub]:input.chain){Ref segment(surfaceWire(doc,{name,sub}));Ref items(PyObject_GetAttrString(segment.p,"Edges"));
            if(PySequence_Size(items.p)!=1)throw std::runtime_error("Each rail chain reference must contain exactly one edge");
            Ref edge(PySequence_GetItem(items.p,0));if(PyList_Append(edges.p,edge.p)<0)throw std::runtime_error(pythonError());}
        Ref joined(PyObject_CallMethod(part.p,"Wire","O",edges.p));Py_SETREF(shape.p,joined.release());
    }else if(PyObject_HasAttrString(pyObject.p,"Placement")&&PyObject_HasAttrString(pyObject.p,"getGlobalPlacement")){
        // Shape already carries local Placement. Apply only the parent's frame.
        Ref global(PyObject_CallMethod(pyObject.p,"getGlobalPlacement",nullptr)),local(PyObject_GetAttrString(pyObject.p,"Placement"));
        Ref inverse(PyObject_CallMethod(local.p,"inverse",nullptr)),parent(PyNumber_Multiply(global.p,inverse.p)),matrix(PyObject_CallMethod(parent.p,"toMatrix",nullptr));
        Ref transformed(PyObject_CallMethod(shape.p,"transformShape","OO",matrix.p,Py_False));
    }
    if(input.chain.empty()&&!input.sub.empty()) {
        if(input.sub.rfind("Edge",0)!=0&&input.sub.rfind("Wire",0)!=0)throw std::runtime_error("Select a curve or an edge, not a face or vertex");
        Ref element(PyObject_CallMethod(shape.p,"getElement","s",input.sub.c_str()));Py_SETREF(shape.p,element.release());
    }
    const auto type=string(shape.p,"ShapeType");
    if(type!="Wire"&&type!="Edge"&&type!="Compound")throw std::runtime_error("Select a curve object or a surface edge");
    Ref faces(PyObject_GetAttrString(shape.p,"Faces"));if(PySequence_Size(faces.p)!=0)throw std::runtime_error("Select individual surface edges, not the entire surface");
    Ref wire=wireFromShape(part.p,shape.p);
    if(!flag(wire.p,"isValid"))throw std::runtime_error("Input must form one valid connected wire");
    Ref length(PyObject_GetAttrString(wire.p,"Length"));const double l=PyFloat_AsDouble(length.p);
    if(PyErr_Occurred())throw std::runtime_error(pythonError());if(!std::isfinite(l)||l<=1e-7)throw std::runtime_error("Input curve has zero or invalid length");
    if(input.seam!=0) {
        if(!flag(wire.p,"isClosed")||!std::isfinite(input.seam)||input.seam<0||input.seam>=1)throw std::runtime_error("Seam needs a closed profile and a fraction between 0 and 1");
        Ref edges(PyObject_GetAttrString(wire.p,"Edges"));
        if(PySequence_Size(edges.p)!=1){Ref moved(rotateSurfaceWire(wire.p,input.seam));Py_SETREF(wire.p,moved.release());if(input.reverse){Ref reversed(PyObject_CallMethod(wire.p,"reverse",nullptr));}return wire.release();}
        Ref edge(PySequence_GetItem(edges.p,0));Ref splineShape(PyObject_CallMethod(edge.p,"toNurbs",nullptr));Ref splineEdges(PyObject_GetAttrString(splineShape.p,"Edges"));Ref splineEdge(PySequence_GetItem(splineEdges.p,0));Ref spline(PyObject_GetAttrString(splineEdge.p,"Curve"));
        if(!flag(spline.p,"isPeriodic")){Ref periodic(PyObject_CallMethod(spline.p,"setPeriodic",nullptr));}
        Ref first(PyObject_GetAttrString(spline.p,"FirstParameter")),last(PyObject_GetAttrString(spline.p,"LastParameter"));
        const double a=PyFloat_AsDouble(first.p),b=PyFloat_AsDouble(last.p);
        const double parameter=a+input.seam*(b-a);
        Ref inserted(PyObject_CallMethod(spline.p,"insertKnot","did",parameter,1,1e-9));
        Ref knots(PyObject_CallMethod(spline.p,"getKnots",nullptr));int originIndex=0;
        for(Py_ssize_t i=0;i<PySequence_Size(knots.p);++i){Ref knot(PySequence_GetItem(knots.p,i));if(std::abs(PyFloat_AsDouble(knot.p)-parameter)<1e-8){originIndex=int(i)+1;break;}}
        if(!originIndex)throw std::runtime_error("Cannot locate the requested seam parameter");
        Ref origin(PyObject_CallMethod(spline.p,"setOrigin","i",originIndex));
        Ref newEdge(PyObject_CallMethod(spline.p,"toShape",nullptr));Ref newWire=wireFromShape(part.p,newEdge.p);Py_SETREF(wire.p,newWire.release());
        // Rebuilding a periodic origin creates a forward edge. Preserve the
        // source traversal before applying the independent user Flip below.
        if(string(edge.p,"Orientation")=="Reversed"){Ref restored(PyObject_CallMethod(wire.p,"reverse",nullptr));}
    }
    if(input.reverse){Ref reversed(PyObject_CallMethod(wire.p,"reverse",nullptr));}
    return wire.release();
}
PyObject* buildSurface(App::Document& doc,const std::vector<SurfaceInput>& inputs,const SurfaceOptions& options) {
    // Spec: OM9-SURFACE-001, OM9-SURFACE-003, OM9-SURFACE-009.
    const unsigned rails=options.kind==1?1:options.kind==2?2:0;
    if(options.kind<1||options.kind>3||inputs.size()<rails+(rails?1:2))throw std::runtime_error("Not enough input curves");
    if(!om9_surface_options_valid(options.kind,options.style,options.closed))throw std::runtime_error("Unsupported surface options");
    if(options.closed&&inputs.size()<rails+(rails?2:3))throw std::runtime_error("Closed Sweep needs two profiles; Closed Loft needs three sections");
    if(options.maintainHeight&&(options.kind!=2||inputs.size()!=3))throw std::runtime_error("Maintain Height currently requires exactly one Sweep2 profile; clear it for multiple profiles");
    if(!options.slashes.empty()&&(options.kind!=2||inputs.size()!=3||inputs[0].closed||inputs[1].closed||options.continuityA||options.continuityB))
        throw std::runtime_error("Add Slash currently requires one Sweep2 profile and two open rails, without face continuity constraints");
    if(options.sectionMode>2||options.pointCount<2||options.pointCount>256||!std::isfinite(options.tolerance)||options.tolerance<=0||options.tolerance>1e6)throw std::runtime_error("Unsupported section fitting options");
    if(options.continuityA||options.continuityB||options.matchStart||options.matchEnd)
        return constrainedSurface(doc,inputs,options);
    Ref part(PyImport_ImportModule("Part")),wires(PyList_New(0));
    bool profileClosure=false;
    for(std::size_t i=0;i<inputs.size();++i){
        Ref wire(surfaceWire(doc,inputs[i]));
        const bool nativeClosed=flag(wire.p,"isClosed");
        if(i==rails)profileClosure=nativeClosed;
        if(i>rails&&nativeClosed!=profileClosure)throw std::runtime_error("Use either all open or all closed profiles");
        if(options.kind==2&&i>=rails){
            // Reject disconnected original data before OCCT's auxiliary-spine
            // solver: this SDK can crash instead of raising on such inputs.
            Ref length(PyObject_GetAttrString(wire.p,"Length"));
            const double tolerance=std::max(1e-4,PyFloat_AsDouble(length.p)*1e-7);
            for(unsigned rail=0;rail<2;++rail){
                Ref distance(PyObject_CallMethod(wire.p,"distToShape","O",PyList_GetItem(wires.p,rail)));
                Ref separation(PySequence_GetItem(distance.p,0));const double value=PyFloat_AsDouble(separation.p);
                if(!std::isfinite(value)||value>tolerance)throw std::runtime_error("Each original Sweep2 profile must meet both rails before construction");
            }
        }
        if(i>=rails&&options.sectionMode!=0){
            const bool closed=flag(wire.p,"isClosed");auto points=sampleCurve(wire.p,options.sectionMode==2?1025u:std::max(513u,4*options.pointCount+1));
            // The shared Rust rebuild removes the repeated closing sample
            // internally; keep it here so periodic input is validated once.
            std::vector<double> xyz;xyz.reserve(points.size()*3);
            for(const auto& p:points)xyz.insert(xyz.end(),p.begin(),p.end());
            const bool fitted=options.sectionMode==2?om9_surface_refit(xyz.data(),points.size(),options.tolerance,closed):
                om9_spline_rebuild(xyz.data(),points.size(),options.pointCount,std::min(3u,options.pointCount-1),closed);
            if(!fitted){
                char message[2048]={};om9_spline_message(message,sizeof(message));throw std::runtime_error(message);
            }
            auto fit=publishedSplineShape();Ref rebuilt=wireFromShape(part.p,fit.value);
            if(options.sectionMode==2)certifySurfaceRefit(wire.p,rebuilt.p,options.tolerance);
            Py_SETREF(wire.p,rebuilt.release());
        }
        if(PyList_Append(wires.p,wire.p)<0)throw std::runtime_error(pythonError());
    }
    if(options.closed&&rails){
        if(!flag(PyList_GetItem(wires.p,0),"isClosed")||(rails==2&&!flag(PyList_GetItem(wires.p,1),"isClosed")))
            throw std::runtime_error("Closed Sweep requires closed rails in this implementation");
    }
    if(!options.slashes.empty()&&(flag(PyList_GetItem(wires.p,0),"isClosed")||flag(PyList_GetItem(wires.p,1),"isClosed")))
        throw std::runtime_error("Add Slash requires open rails after source changes");
    PyObject* result=nullptr;
    if(options.kind==3) {
        if(options.style>=2)result=advancedLoft(wires.p,options.style,options.closed);
        else result=PyObject_CallMethod(part.p,"makeLoft","OOOO",wires.p,Py_False,options.style==1?Py_True:Py_False,options.closed?Py_True:Py_False);
    }else if(rails==2&&inputs.size()==3){
        // The SDK PipeShell is unstable for dense transformed curved sections
        // (65-station arch probe). Loft the same transported sections instead;
        // both rails still drive every frame and are checked on the result.
        Ref sections=transportedSections(part.p,wires.p,options.maintainHeight,options.slashes);
        result=PyObject_CallMethod(part.p,"makeLoft","OOOOi",sections.p,Py_False,Py_False,flag(PyList_GetItem(wires.p,0),"isClosed")?Py_True:Py_False,3);
    }else {
        Ref api(PyObject_GetAttrString(part.p,"BRepOffsetAPI"));Ref constructor(PyObject_GetAttrString(api.p,"MakePipeShell"));
        Ref pipe(PyObject_CallFunctionObjArgs(constructor.p,PyList_GetItem(wires.p,0),nullptr));
        Ref mode(PyObject_CallMethod(pipe.p,"setFrenetMode","O",options.frenet?Py_True:Py_False));
        if(rails==2){
            Ref auxiliary(PyObject_CallMethod(pipe.p,"setAuxiliarySpine","OOi",PyList_GetItem(wires.p,1),Py_True,int(om9_surface_sweep2_contact(inputs.size()-rails))));
        }
        for(std::size_t i=rails;i<inputs.size();++i){Ref added(PyObject_CallMethod(pipe.p,"add","OOO",PyList_GetItem(wires.p,i),Py_False,Py_False));}
        if(options.closed){Ref added(PyObject_CallMethod(pipe.p,"add","OOO",PyList_GetItem(wires.p,rails),Py_False,Py_False));}
        if(!flag(pipe.p,"isReady"))throw std::runtime_error("Sweep could not use these profiles");
        Ref built(PyObject_CallMethod(pipe.p,"build",nullptr));result=PyObject_CallMethod(pipe.p,"shape",nullptr);
    }
    Ref shape(result);Ref faces(PyObject_GetAttrString(shape.p,"Faces")),solids(PyObject_GetAttrString(shape.p,"Solids"));
    if(flag(shape.p,"isNull")||!flag(shape.p,"isValid")||PySequence_Size(faces.p)==0||PySequence_Size(solids.p)!=0)throw std::runtime_error("The kernel could not create a valid surface from these curves");
    if(options.kind==2)verifyCurves(part.p,wires.p,shape.p);
    return shape.release();
}
void commitSurface(App::Document& doc,PyObject* shape,const std::vector<SurfaceInput>& inputs,const SurfaceOptions& options) {
    const char* name=options.kind==1?"Sweep1":options.kind==2?"Sweep2":"Loft";
    const char* feature=options.kind==1?"OM9-SURFACE-001":options.kind==2?"OM9-SURFACE-003":"OM9-SURFACE-009";
    doc.openTransaction(name);
    try {
        if(options.history){
            auto* feature=createSurfaceHistory(doc,shape,inputs,options);Ref object(feature->getPyObject());
            Ref view(PyObject_GetAttrString(object.p,"ViewObject")),color(Py_BuildValue("(ddd)",0.0,130.0/255.0,85.0/255.0));set(view.p,"ShapeColor",color.p);set(view.p,"LineColor",color.p);
            doc.recompute();if(!feature->isValid()||feature->Shape.getShape().isNull())throw std::runtime_error("Surface History recompute failed before commit");
            doc.commitTransaction();return;
        }
        Ref pyDoc(doc.getPyObject()),object(PyObject_CallMethod(pyDoc.p,"addObject","ss","Part::Feature",name));set(object.p,"Shape",shape);
        Ref id(PyUnicode_FromString(feature)),command(PyUnicode_FromString(name));property(object.p,"App::PropertyString","OM9FeatureId",id.p);property(object.p,"App::PropertyString","OM9Command",command.p);
        Ref sources(PyList_New(0));std::ostringstream settings;settings.precision(17);settings<<"style="<<options.style<<";frenet="<<options.frenet<<";closed="<<options.closed<<";maintainHeight="<<options.maintainHeight<<";sectionMode="<<options.sectionMode<<";pointCount="<<options.pointCount<<";preview="<<options.preview<<";tolerance="<<options.tolerance<<";continuityA="<<options.continuityA<<";continuityB="<<options.continuityB<<";matchStart="<<options.matchStart<<";matchEnd="<<options.matchEnd;
        for(const auto& [a,b]:options.slashes)settings<<";slash="<<a<<","<<b;
        for(const auto& input:inputs){
            const auto references=input.chain.empty()?std::vector<std::pair<std::string,std::string>>{{input.object,input.sub}}:input.chain;
            settings<<";input="<<input.object<<"."<<input.sub<<":reverse="<<input.reverse<<",seam="<<input.seam<<",chain="<<input.chain.size();
            for(const auto& [name,sub]:references){auto* source=doc.getObject(name.c_str());if(!source)throw std::runtime_error("An input curve was deleted");Ref pySource(source->getPyObject());Ref subs(Py_BuildValue("[s]",sub.c_str()));Ref entry(PyTuple_Pack(2,pySource.p,subs.p));if(PyList_Append(sources.p,entry.p)<0)throw std::runtime_error(pythonError());}
        }
        property(object.p,"App::PropertyLinkSubList","SourceCurves",sources.p);Ref config(PyUnicode_FromString(settings.str().c_str()));property(object.p,"App::PropertyString","SurfaceOptions",config.p);
        Ref view(PyObject_GetAttrString(object.p,"ViewObject")),color(Py_BuildValue("(ddd)",0.0,130.0/255.0,85.0/255.0));set(view.p,"ShapeColor",color.p);set(view.p,"LineColor",color.p);
        doc.recompute();doc.commitTransaction();
    }catch(...){doc.abortTransaction();throw;}
}
}
