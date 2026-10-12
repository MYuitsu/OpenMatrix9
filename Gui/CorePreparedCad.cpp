// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CorePreparedCad.h"
#include "CorePreparedSnapshot.h"
#include "CoreDiscreteSnapshot.h"
#include "CoreUnits.h"
#include "CoreTopologySnapshot.h"
#include "CoreShapeValidation.h"
#include "NativeInputTree.h"
#include <Mod/Part/App/TopoShapePy.h>
#include <Mod/Mesh/App/MeshPy.h>
#include <BRepBuilderAPI_Copy.hxx>
#include <Standard_Failure.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <App/Application.h>
#include <App/Document.h>
#include <algorithm>
#include <cstdint>
#include <memory>
#include <limits>
#include <vector>

namespace OpenMatrix9Gui {
namespace {
extern "C" unsigned om9_prepared_cad_admit(unsigned,unsigned,unsigned);
extern "C" unsigned om9_prepared_cad_budget(std::size_t,std::size_t,std::size_t,std::size_t);
extern "C" unsigned om9_prepared_metadata(std::size_t,std::size_t,std::size_t);
extern "C" unsigned om9_prepared_number(double);
extern "C" unsigned om9_prepared_binding_field(unsigned,unsigned,double,std::size_t);
extern "C" unsigned om9_discrete_counts(unsigned,std::size_t,std::size_t);
void decref(PyObject* p){Py_XDECREF(p);}
using Owner=std::unique_ptr<PyObject,decltype(&decref)>;
Owner own(PyObject* p){if(!p)throw std::runtime_error("Cannot allocate owned prepared data");return Owner(p,decref);}
Owner retain(PyObject* p){Py_INCREF(p);return Owner(p,decref);}
const char* code(unsigned c){
    static const char* names[]={"","UnsupportedRepresentation","InvalidBasis","InvalidTrim",
        "NonFiniteGeometry","StaleGeometry","UnsupportedTransform","KernelFailure","ConversionLoss"};
    return c< std::size(names)?names[c]:"KernelFailure";
}
QJsonValue unknown(){return QJsonValue(QJsonValue::Null);}
class Preparation {
    std::size_t nodes=0,bytes=0,nativeNodes=0,nativeBytes=0;
    std::vector<PyObject*> path;
    QJsonObject input;
    QJsonArray results;
    std::string unitContext;
    void reject(unsigned c,const std::string& reason){throw PreparedCadFailure(c,"CommitImport3dm",input,reason);}
    void budget(std::size_t n=0,std::size_t b=0){
        if(n>std::numeric_limits<std::size_t>::max()-nodes || b>std::numeric_limits<std::size_t>::max()-bytes)
            reject(2,"Prepared logical count/byte overflow");
        if(om9_prepared_metadata(nodes+n,bytes+b,path.size()+1))reject(2,"Prepared plain metadata exceeds its Rust depth/node/byte budget");
        if(om9_prepared_cad_budget(nodes+n,bytes+b,nativeNodes,nativeBytes))reject(2,"Prepared batch exceeds cumulative Rust logical geometry/metadata budget");
    }
    QString string(PyObject* p){
        if(!p||!PyUnicode_CheckExact(p))reject(1,"Prepared string requires a plain Unicode value");
        Py_ssize_t length=0;const auto* utf8=PyUnicode_AsUTF8AndSize(p,&length);
        if(!utf8||length<0)reject(1,"Invalid prepared UTF-8 string");
        budget(0,static_cast<std::size_t>(length));
        return QString::fromUtf8(utf8,length);
    }
    Owner freeze(PyObject* p){
        budget(1);++nodes;
        if(p==Py_None||PyBool_Check(p))return retain(p);
        if(PyUnicode_CheckExact(p)){
            Py_ssize_t length=0;if(!PyUnicode_AsUTF8AndSize(p,&length)||length<0)reject(1,"Invalid metadata UTF-8");
            budget(0,static_cast<std::size_t>(length));bytes+=static_cast<std::size_t>(length);return retain(p);
        }
        if(PyLong_CheckExact(p)){
            int overflow=0;(void)PyLong_AsLongLongAndOverflow(p,&overflow);
            if(overflow||PyErr_Occurred())reject(1,"Plain metadata integer exceeds signed64-bit range");
            budget(0,8);bytes+=8;return retain(p);
        }
        if(PyFloat_CheckExact(p)){
            if(auto c=om9_prepared_number(PyFloat_AS_DOUBLE(p)))reject(c,"Nonfinite prepared metadata number");
            budget(0,8);bytes+=8;return retain(p);
        }
        if(PyBytes_CheckExact(p)){
            const auto count=static_cast<std::size_t>(PyBytes_GET_SIZE(p));budget(0,count);bytes+=count;return retain(p);
        }
        const bool dictionary=PyDict_CheckExact(p),list=PyList_CheckExact(p),tuple=PyTuple_CheckExact(p);
        if(!dictionary&&!list&&!tuple)reject(1,"Prepared metadata requires plain immutable values and containers");
        if(std::find(path.begin(),path.end(),p)!=path.end())reject(1,"Cyclic prepared plain metadata");
        const auto count=static_cast<std::size_t>(dictionary?PyDict_Size(p):list?PyList_GET_SIZE(p):PyTuple_GET_SIZE(p));
        budget(dictionary?count*2:count); // count bound precedes container allocation.
        const auto storage=count*sizeof(PyObject*)*(dictionary?2:1);budget(0,storage);bytes+=storage;
        path.push_back(p);
        struct Pop {std::vector<PyObject*>& path;~Pop(){path.pop_back();}} pop{path};
        if(dictionary){
            auto out=own(PyDict_New());Py_ssize_t position=0;PyObject *key=nullptr,*value=nullptr;
            while(PyDict_Next(p,&position,&key,&value)){
                if(!PyUnicode_CheckExact(key))reject(1,"Prepared dictionary keys require plain Unicode");
                auto k=freeze(key),v=freeze(value);
                if(PyDict_SetItem(out.get(),k.get(),v.get())<0)throw std::runtime_error("Cannot freeze metadata dictionary");
            }
            return out;
        }
        auto out=own(list?PyList_New(static_cast<Py_ssize_t>(count)):PyTuple_New(static_cast<Py_ssize_t>(count)));
        for(std::size_t n=0;n<count;++n){auto value=freeze(list?PyList_GET_ITEM(p,n):PyTuple_GET_ITEM(p,n));
            if(list)PyList_SET_ITEM(out.get(),n,value.release());else PyTuple_SET_ITEM(out.get(),n,value.release());}
        return out;
    }
    unsigned fact(const QJsonValue& v){return v.isBool()?(v.toBool()?2u:1u):0u;}
    void bindingField(unsigned field,PyObject* value,const char* name){
        unsigned kind=0;double number=0;std::size_t count=0;
        if(value){
            if(PyUnicode_CheckExact(value))kind=1;
            else if(PyBool_Check(value))kind=3;
            else if(PyFloat_CheckExact(value)){kind=2;number=PyFloat_AS_DOUBLE(value);}
            else if(PyLong_CheckExact(value)){kind=6;number=PyLong_AsDouble(value);if(PyErr_Occurred())reject(2,"Invalid prepared binding integer");}
            else if(PyList_CheckExact(value)||PyTuple_CheckExact(value)){
                kind=4;count=static_cast<std::size_t>(PyList_CheckExact(value)?PyList_GET_SIZE(value):PyTuple_GET_SIZE(value));
            }else kind=5;
        }
        if(auto status=om9_prepared_binding_field(field,kind,number,count))
            reject(status,std::string("Invalid or missing prepared binding field: ")+name);
    }
    void bindingMetadata(PyObject* meta){
        const char* fields[]={"name","layer","tolerance","locked","visible","color"};
        for(unsigned n=0;n<std::size(fields);++n)bindingField(n,PyDict_GetItemString(meta,fields[n]),fields[n]);
        auto* color=PyDict_GetItemString(meta,"color");
        const auto count=PyList_CheckExact(color)?PyList_GET_SIZE(color):PyTuple_GET_SIZE(color);
        for(Py_ssize_t n=0;n<count;++n)bindingField(6,PyList_CheckExact(color)?PyList_GET_ITEM(color,n):PyTuple_GET_ITEM(color,n),"color channel");
        if(auto* density=PyDict_GetItemString(meta,"wire_density"))bindingField(7,density,"wire_density");
        // Snapshot capture reads these optional fields even for non-working CAD.
        // Validate their exact types before any native getter can set PyErr.
        if(auto* kind=PyDict_GetItemString(meta,"geometry_kind"))bindingField(7,kind,"geometry_kind");
        for(const char* key:{"representation","class_name"})
            if(auto* value=PyDict_GetItemString(meta,key))bindingField(8,value,key);
        if(auto* working=PyDict_GetItemString(meta,"working_mode")){
            bindingField(3,working,"working_mode");
            if(working==Py_True){
                bindingField(7,PyDict_GetItemString(meta,"geometry_kind"),"geometry_kind");
                bindingField(8,PyDict_GetItemString(meta,"representation"),"representation");
            }
        }
    }
    void set(PyObject* dict,const char* key,PyObject* value){if(PyDict_SetItemString(dict,key,value)<0)throw std::runtime_error("Cannot freeze prepared result field");}
    QJsonObject attributes(PyObject* metadata){
        // Exact frozen binding fields only. No Python encoder or conversion
        // callbacks; arbitrary properties/userdata remain explicitly uninspected.
        QJsonObject out{{"schema_version",1},{"coverage",QJsonObject{{"other_metadata","uninspected"},{"rhino_user_data","uninspected"}}}};
        for(const char* key:{"name","layer","source_uuid","class_name","representation"})
            if(auto* value=PyDict_GetItemString(metadata,key))out[key]=string(value);
        for(const char* key:{"locked","visible","working_mode"})
            if(auto* value=PyDict_GetItemString(metadata,key))out[key]=value==Py_True;
        for(const char* key:{"tolerance","wire_density","geometry_kind"})
            if(auto* value=PyDict_GetItemString(metadata,key))out[key]=PyFloat_CheckExact(value)?PyFloat_AS_DOUBLE(value):PyLong_AsDouble(value);
        auto* color=PyDict_GetItemString(metadata,"color");const auto count=PyList_CheckExact(color)?PyList_GET_SIZE(color):PyTuple_GET_SIZE(color);
        QJsonArray channels;for(Py_ssize_t n=0;n<count;++n){auto* value=PyList_CheckExact(color)?PyList_GET_ITEM(color,n):PyTuple_GET_ITEM(color,n);
            channels.append(PyFloat_CheckExact(value)?PyFloat_AS_DOUBLE(value):PyLong_AsDouble(value));}
        out["color"]=channels;return out;
    }
    QJsonValue plainJson(PyObject* value){
        if(value==Py_None)return unknown();
        if(PyBool_Check(value))return value==Py_True;
        if(PyUnicode_CheckExact(value))return string(value);
        if(PyFloat_CheckExact(value))return PyFloat_AS_DOUBLE(value);
        if(PyLong_CheckExact(value)){const auto number=PyLong_AsDouble(value);if(PyErr_Occurred())reject(2,"Invalid prepared JSON integer");return number;}
        if(PyList_CheckExact(value)||PyTuple_CheckExact(value)){
            const auto count=PyList_CheckExact(value)?PyList_GET_SIZE(value):PyTuple_GET_SIZE(value);QJsonArray rows;
            for(Py_ssize_t n=0;n<count;++n)rows.append(plainJson(PyList_CheckExact(value)?PyList_GET_ITEM(value,n):PyTuple_GET_ITEM(value,n)));
            return rows;
        }
        if(PyDict_CheckExact(value)){
            QJsonObject object;Py_ssize_t position=0;PyObject *key=nullptr,*item=nullptr;
            while(PyDict_Next(value,&position,&key,&item))object[string(key)]=plainJson(item);
            return object;
        }
        reject(1,"Prepared geometry JSON requires exact plain fields");return unknown();
    }
    void bindReport(PyObject* metadata,const QJsonObject& report){
        const auto raw=QJsonDocument(report).toJson(QJsonDocument::Compact);budget(0,raw.size());bytes+=raw.size();
        auto encoded=own(PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict"));
        set(metadata,"_om9_geometry_result",encoded.get());results.append(report);
    }
    Owner discrete(PyObject* original,PyObject* metadata){
        QJsonObject data,validation{{"native_valid",unknown()},{"valid_solid",false}};
        Owner geometry(nullptr,decref);const bool cloud=PyDict_GetItemString(metadata,"point_cloud_fields");
        if(cloud){
            if(!PyDict_CheckExact(original))reject(1,"Prepared cloud requires exact plain current fields");
            geometry=freeze(original);const auto current=plainJson(geometry.get()).toObject();
            // Existing host binding uses IntegerList for RGBA: admit exact
            // integers before native/Python property getters can set errors.
            if(auto* colors=PyDict_GetItemString(geometry.get(),"colors")){
                if(PyList_CheckExact(colors)||PyTuple_CheckExact(colors)){
                    const auto count=PyList_CheckExact(colors)?PyList_GET_SIZE(colors):PyTuple_GET_SIZE(colors);
                    for(Py_ssize_t n=0;n<count;++n)bindingField(7,PyList_CheckExact(colors)?PyList_GET_ITEM(colors,n):PyTuple_GET_ITEM(colors,n),"PointCloud RGBA integer");
                }
            }
            bool planeValid=false;const auto plane=current.value("plane").toArray();
            if(plane.size()==16&&std::all_of(plane.begin(),plane.end(),[](const QJsonValue& v){return v.isDouble();})){
                ON_Plane nativePlane;
                nativePlane.origin=ON_3dPoint(plane[0].toDouble(),plane[1].toDouble(),plane[2].toDouble());
                nativePlane.xaxis=ON_3dVector(plane[3].toDouble(),plane[4].toDouble(),plane[5].toDouble());
                nativePlane.yaxis=ON_3dVector(plane[6].toDouble(),plane[7].toDouble(),plane[8].toDouble());
                nativePlane.zaxis=ON_3dVector(plane[9].toDouble(),plane[10].toDouble(),plane[11].toDouble());
                for(int n=0;n<4;++n)nativePlane.plane_equation[n]=plane[12+n].toDouble();
                planeValid=nativePlane.IsValid();
            }
            validation["native_plane_valid"]=planeValid;
            data={{"representation","point_cloud"},{"coordinates","native_local"},{"current_fields",current},{"native_storage_precision","float64"}};
        }else{
            // Mesh is delay-loaded. Its exported static Type is a data import,
            // which MSVC cannot delay-load; obtain the SDK type from a bounded
            // empty native wrapper instead, after native module initialization.
            auto module=own(PyImport_ImportModule("Mesh"));
            if(Base::Type::fromName("Mesh::MeshObject").isBad())reject(1,"Native Mesh module is not initialized");
            auto prototype=own(Mesh::MeshPy::PyMake(nullptr,nullptr,nullptr));
            if(Py_TYPE(original)!=Py_TYPE(prototype.get()))reject(1,"Prepared mesh requires exact native Mesh.Mesh ownership");
            auto* wrapper=static_cast<Mesh::MeshPy*>(original);
            if(!wrapper->Base::PyObjectBase::isValid())reject(5,"Prepared native mesh is no longer live");
            auto* owner=wrapper->getMeshObjectPtr();if(!owner)reject(5,"Prepared mesh owner is missing");
            MeshInputBudget bounded;
            try{bounded=meshInputBudget(*owner);}catch(const MeshInputFailure& e){reject(e.status,e.what());}
            if(bounded.nodes>std::numeric_limits<std::size_t>::max()-nativeNodes||bounded.bytes>std::numeric_limits<std::size_t>::max()-nativeBytes)reject(2,"Prepared mesh cumulative budget overflow");
            nativeNodes+=bounded.nodes;nativeBytes+=bounded.bytes;budget();
            QJsonObject captured;
            try{captured=meshSnapshot(*owner);}catch(const std::exception& error){reject(3,error.what());}
            data=captured.value("geometry_data").toObject();data["representation"]="mesh";data["coordinates"]="native_local_with_locations";
            data["native_discrete_location"]=captured.value("native_discrete_location");
            auto* vertices=PyDict_GetItemString(metadata,"vertices");auto* faces=PyDict_GetItemString(metadata,"faces");
            if(!vertices||!faces)reject(1,"Prepared mesh requires source vertices/faces fields");
            data["source_archive_mesh"]=QJsonObject{{"vertices",plainJson(vertices)},{"faces",plainJson(faces)}};
            // Serialize/admit before this geometry-owning native constructor.
        }
        const QJsonObject snapshot{{"ownership",cloud?"private_plain_copy":"private_native_copy"},{"geometry_data",data},
            {"topology_data",QJsonObject{{"discrete_kind",cloud?"point_cloud":"mesh"},{"manifold_certificate",unknown()}}}};
        bindReport(metadata,preparedGeometryResult(snapshot,validation,attributes(metadata),input,unitContext));
        if(!cloud){auto* owner=static_cast<Mesh::MeshPy*>(original)->getMeshObjectPtr();geometry=own(new Mesh::MeshPy(new Mesh::MeshObject(*owner),Py_TYPE(original)));}
        return geometry;
    }
    Owner rows(PyObject* source,const QString& preview){
        if(!source||(!PyList_CheckExact(source)&&!PyTuple_CheckExact(source)))reject(1,"Prepared geometry requires a plain list or tuple");
        const auto count=PyList_CheckExact(source)?PyList_GET_SIZE(source):PyTuple_GET_SIZE(source);
        if(count>16384)reject(2,"Prepared geometry exceeds Rust native row limit");
        budget(static_cast<std::size_t>(count)*2,static_cast<std::size_t>(count)*2*sizeof(PyObject*));
        nodes+=static_cast<std::size_t>(count)*2;bytes+=static_cast<std::size_t>(count)*2*sizeof(PyObject*);
        auto out=own(PyList_New(count));
        for(Py_ssize_t n=0;n<count;++n){
            input.remove("name");input.remove("source_uuid");
            input["row_index"]=static_cast<qint64>(n);input["preview_identity"]=preview;
            auto* pair=PyList_CheckExact(source)?PyList_GET_ITEM(source,n):PyTuple_GET_ITEM(source,n);
            if(!PyTuple_CheckExact(pair)&&!PyList_CheckExact(pair))reject(1,"Prepared geometry requires plain metadata/native-geometry pairs");
            const auto size=PyTuple_CheckExact(pair)?PyTuple_GET_SIZE(pair):PyList_GET_SIZE(pair);
            if(size!=2)reject(1,"Prepared geometry requires two-item plain pairs");
            auto* meta=PyTuple_CheckExact(pair)?PyTuple_GET_ITEM(pair,0):PyList_GET_ITEM(pair,0);
            auto* original=PyTuple_CheckExact(pair)?PyTuple_GET_ITEM(pair,1):PyList_GET_ITEM(pair,1);
            if(!PyDict_CheckExact(meta))reject(1,"Prepared metadata requires a plain dictionary");
            Py_ssize_t keyPosition=0;PyObject *metaKey=nullptr,*metaValue=nullptr;
            while(PyDict_Next(meta,&keyPosition,&metaKey,&metaValue))
                if(!PyUnicode_CheckExact(metaKey))reject(1,"Prepared dictionary keys require plain Unicode");
            // Read a name only if it is already an exact immutable string.
            auto* name=PyDict_GetItemString(meta,"name");
            input["name"]=name&&PyUnicode_CheckExact(name)?string(name):QString();
            if(auto* uuid=PyDict_GetItemString(meta,"source_uuid")){
                if(PyUnicode_CheckExact(uuid))input["source_uuid"]=string(uuid);
                else reject(1,"Prepared source identity requires plain Unicode");
            }
            auto frozen=freeze(meta);
            bindingMetadata(frozen.get());
            auto geometry=retain(original);
            if(PyDict_GetItemString(frozen.get(),"brep")){
                if(!PyObject_TypeCheck(original,&Part::TopoShapePy::Type))reject(1,"Prepared CAD requires a native Part Shape");
                auto* wrapper=static_cast<Part::TopoShapePy*>(original);
                if(!wrapper->Base::PyObjectBase::isValid())reject(5,"Prepared native Part Shape is no longer live");
                const auto& shape=wrapper->getTopoShapePtr()->getShape();
                ThreeDm::InputTree::Summary bounded;
                try{bounded=ThreeDm::InputTree::validate(shape);}
                catch(const ThreeDm::GeometryRepresentationUnavailable& e){reject(1,e.what());}
                catch(const std::exception& e){reject(2,e.what());}
                nativeNodes+=bounded.nodes;nativeBytes+=bounded.bytes;budget();
                // Gate native owners/basis before a geometry-owning copy. The
                // caller's mutable shape is never passed to document observers.
                BRepBuilderAPI_Copy copied(shape,true,false);
                const auto privateShape=copied.Shape();
                QJsonObject topology,validation;
                try{topology=topologySnapshot(privateShape);validation=shapeValidation(privateShape,topology);}
                catch(const ThreeDm::GeometryRepresentationUnavailable& e){reject(1,e.what());}
                catch(const std::exception& e){reject(3,e.what());}
                const auto status=om9_prepared_cad_admit(static_cast<unsigned>(privateShape.ShapeType()),
                    fact(validation.value("native_valid")),fact(validation.value("valid_solid")));
                if(status)reject(status,"Rust rejected prepared native geometry/topology result");
                const auto report=preparedCadResult(privateShape,topology,validation,attributes(frozen.get()),input,unitContext);
                bindReport(frozen.get(),report);
                geometry=own(new Part::TopoShapePy(new Part::TopoShape(privateShape)));
            }else geometry=discrete(original,frozen.get());
            auto ownedPair=own(PyTuple_New(2));PyTuple_SET_ITEM(ownedPair.get(),0,frozen.release());PyTuple_SET_ITEM(ownedPair.get(),1,geometry.release());
            PyList_SET_ITEM(out.get(),n,ownedPair.release());
        }
        return out;
    }
public:
    explicit Preparation(const char* document){input["document_name"]=QString::fromUtf8(document);}
    Owner run(PyObject* source){
        auto* document=App::GetApplication().getDocument(input.value("document_name").toString().toUtf8().constData());
        if(!document)reject(5,"Prepared target document disappeared");
        try{unitContext=unitContextSnapshot(*document);}
        catch(const std::exception& error){reject(2,error.what());}
        if(!PyDict_Check(source))return rows(source,QString());
        if(!PyDict_CheckExact(source))reject(1,"Prepared archive requires a plain dictionary");
        auto out=own(PyDict_New());Py_ssize_t position=0;PyObject *key=nullptr,*value=nullptr;
        budget(static_cast<std::size_t>(PyDict_Size(source))*2);
        while(PyDict_Next(source,&position,&key,&value)){
            auto ownedKey=freeze(key);const auto name=string(ownedKey.get());Owner frozen(nullptr,decref);
            if(name=="host_geometry")frozen=rows(value,QString());
            else if(name=="host_previews"){
                if(!PyDict_CheckExact(value))reject(1,"Prepared previews require a plain dictionary");
                budget(static_cast<std::size_t>(PyDict_Size(value))*2);
                frozen=own(PyDict_New());Py_ssize_t at=0;PyObject *identity=nullptr,*members=nullptr;
                while(PyDict_Next(value,&at,&identity,&members)){auto k=freeze(identity);auto r=rows(members,string(k.get()));
                    if(PyDict_SetItem(frozen.get(),k.get(),r.get())<0)throw std::runtime_error("Cannot freeze preview rows");}
            }else if(name=="_om9_geometry_result")continue;
            else frozen=freeze(value);
            if(PyDict_SetItem(out.get(),ownedKey.get(),frozen.get())<0)throw std::runtime_error("Cannot freeze prepared archive");
        }
        if(!PyDict_GetItemString(out.get(),"host_geometry"))reject(1,"Prepared archive requires host_geometry");
        const auto raw=QJsonDocument(QJsonObject{{"schema_version",1},{"status","accepted"},{"operation","CommitImport3dm"},
            {"scope","prepared_geometry_rows_and_previews"},{"results",results}}).toJson(QJsonDocument::Compact);
        budget(0,raw.size());bytes+=raw.size();auto encoded=own(PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict"));
        set(out.get(),"_om9_geometry_result",encoded.get());return out;
    }
    const QJsonObject& identity() const{return input;}
};
}
PreparedCadFailure::PreparedCadFailure(unsigned c,const char* operation,const QJsonObject& identity,const std::string& reason)
    :std::runtime_error(std::string(code(c))+" "+operation+": "+reason),
     result{{"schema_version",1},{"status","rejected"},{"output_snapshots",QJsonArray{}},
        {"conversion_reports",QJsonArray{}},{"topology_map",unknown()},
        {"diagnostics",QJsonArray{QJsonObject{{"code",code(c)},{"operation",operation},{"input_identity",identity},
            {"reason",QString::fromUtf8(reason.data(),reason.size())}}}}} {}
PyObject* prepareOwnedCad(PyObject* input,const char* document){
    Preparation preparation(document);
    try{return preparation.run(input).release();}
    catch(const PreparedCadFailure&){throw;}
    catch(const Standard_Failure& e){throw PreparedCadFailure(7,"CommitImport3dm",preparation.identity(),e.GetMessageString()?e.GetMessageString():"Native prepared CAD failure");}
    catch(const std::exception& e){throw PreparedCadFailure(7,"CommitImport3dm",preparation.identity(),e.what());}
    catch(...){throw PreparedCadFailure(7,"CommitImport3dm",preparation.identity(),"Unknown native preparation failure");}
}
void setPreparedCadError(const PreparedCadFailure& error) noexcept {try{
    PyErr_Clear();auto exception=own(PyObject_CallFunction(PyExc_RuntimeError,"s",error.what()));
    const auto raw=QJsonDocument(error.result).toJson(QJsonDocument::Compact);
    auto value=own(PyUnicode_DecodeUTF8(raw.constData(),raw.size(),"strict"));
    if(PyObject_SetAttrString(exception.get(),"geometry_result",value.get())<0)return;
    PyErr_SetObject(PyExc_RuntimeError,exception.get());
    }catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}
    catch(...){PyErr_SetString(PyExc_RuntimeError,"KernelFailure CommitImport3dm: cannot encode native diagnostic");}
}
}
