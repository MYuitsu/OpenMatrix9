#include "SnapQueryIndex.h"
#include "SnapObjectInfo.h"
#include "RustBridge.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Mod/Part/App/TopoShape.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/ViewProvider.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoCamera.h>
#include <QElapsedTimer>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QApplication>
#include <QThread>
#include <fastsignals/signal.h>
#include <unordered_set>
#include <algorithm>
#include <cmath>
#include <limits>
namespace {
struct Index {
    std::uint64_t handle=om9_phase2_snap_create(),generation=1;
    std::vector<fastsignals::scoped_connection> connections;
    Index(){OpenMatrix9Gui::phase2Require(handle!=0);auto reset=[this]{if(generation<std::numeric_limits<std::uint64_t>::max())++generation;om9_phase2_snap_invalidate(handle);};auto& app=App::GetApplication();
        connections.emplace_back(app.signalChangedObject.connect([reset](const App::DocumentObject&,const App::Property&){reset();}));
        connections.emplace_back(app.signalNewObject.connect([reset](const App::DocumentObject&){reset();}));
        connections.emplace_back(app.signalDeletedObject.connect([reset](const App::DocumentObject&){reset();}));
        connections.emplace_back(app.signalDeleteDocument.connect([reset](const App::Document&){reset();}));
        connections.emplace_back(app.signalUndoDocument.connect([reset](const App::Document&){reset();}));
        connections.emplace_back(app.signalRedoDocument.connect([reset](const App::Document&){reset();}));
        connections.emplace_back(Gui::Application::Instance->signalChangedObject.connect([reset](const Gui::ViewProvider&,const App::Property&){reset();}));
        connections.emplace_back(Gui::Application::Instance->signalDeletedObject.connect([reset](const Gui::ViewProvider&){reset();}));
    }
};
Index& cache(){static auto* index=new Index;return *index;}
bool visible(Gui::Document* gui,const App::DocumentObject* object){auto* provider=gui->getViewProvider(object);if(!provider||!provider->isVisible())return false;std::unordered_set<const App::DocumentObject*> parents;for(auto* parent=App::GeoFeatureGroupExtension::getGroupOfObject(object);parent;parent=App::GeoFeatureGroupExtension::getGroupOfObject(parent)){if(!parents.insert(parent).second)return false;auto* p=gui->getViewProvider(parent);if(!p||!p->isVisible())return false;}return true;}
struct BuildGuard {std::uint64_t handle;bool done=false;~BuildGuard(){if(!done)om9_phase2_snap_abort(handle);}};
struct QueryGuard {std::uint64_t handle;~QueryGuard(){om9_phase2_query_drop(handle);}};
}
namespace OpenMatrix9Gui {
SnapQueryResult querySnapCandidates(Gui::View3DInventor* view,const SnapQuery& request){
    SnapQueryResult result;QElapsedTimer timer;timer.start();auto finish=[&]{result.elapsed_us=timer.nsecsElapsed()/1000;return result;};
    try {
        if(QThread::currentThread()!=qApp->thread())throw std::runtime_error("Native CAD snap requires the GUI thread");
        auto* gui=Gui::Application::Instance->activeDocument();auto* doc=App::GetApplication().getActiveDocument();if(!view||!doc||!gui||gui->isAboutToClose())throw std::runtime_error("No active CAD view");
        auto* viewer=view->getViewer();auto* camera=viewer->getSoRenderManager()->getCamera();result.viewport_width=viewer->viewport()->width();result.viewport_height=viewer->viewport()->height();if(!camera||result.viewport_width<=0||result.viewport_height<=0)throw std::runtime_error("No native camera/viewport");
        const auto volume=camera->getViewVolume(float(result.viewport_width)/result.viewport_height);const auto matrix=volume.getMatrix();auto& index=cache();
        Om9ViewKey key{std::uint64_t(reinterpret_cast<std::uintptr_t>(doc)),std::uint64_t(reinterpret_cast<std::uintptr_t>(view)),index.generation,unsigned(result.viewport_width),unsigned(result.viewport_height),{}};for(int i=0;i<4;++i)for(int j=0;j<4;++j)key.camera[i*4+j]=matrix[i][j];
        if(!om9_phase2_snap_matches(index.handle,&key)){
            phase2Require(om9_phase2_snap_begin(index.handle,&key),index.handle);BuildGuard guard{index.handle};result.index_rebuilt=true;
            for(auto* object:doc->getObjects()){
                ++result.index_objects;if(!visible(gui,object))continue;const auto info=classifySnapObject(object);if(!om9_modeling_snap_allowed(info.kind,info.native_cad,info.preview,2)&&!om9_modeling_snap_allowed(info.kind,info.native_cad,info.preview,4)&&!om9_modeling_snap_allowed(info.kind,info.native_cad,info.preview,8))continue;
                const auto bounds=Part::TopoShape(info.shape).getBoundBox();if(!bounds.IsValid())continue;double x0=1e300,y0=1e300,x1=-1e300,y1=-1e300;bool clipped=false;unsigned before=0,after=0;
                for(unsigned corner=0;corner<8;++corner){Base::Vector3d world;info.global_transform.multVec(Base::Vector3d(corner&1?bounds.MaxX:bounds.MinX,corner&2?bounds.MaxY:bounds.MinY,corner&4?bounds.MaxZ:bounds.MinZ),world);SbVec3f p;volume.projectToScreen(SbVec3f(float(world.x),float(world.y),float(world.z)),p);if(!std::isfinite(p[0])||!std::isfinite(p[1])||!std::isfinite(p[2])){clipped=true;continue;}before+=p[2]<0;after+=p[2]>1;clipped=clipped||p[2]<0||p[2]>1;const double x=double(p[0])*result.viewport_width,y=(1.-double(p[1]))*result.viewport_height;x0=std::min(x0,x);x1=std::max(x1,x);y0=std::min(y0,y);y1=std::max(y1,y);}
                if(before==8||after==8)continue;if(clipped){x0=0;y0=0;x1=result.viewport_width;y1=result.viewport_height;}
                const Om9SnapRow row{std::uint64_t(object->getID())+1,{x0,y0,x1,y1}};phase2Require(om9_phase2_snap_add(index.handle,&row),index.handle);
            }
            phase2Require(om9_phase2_snap_finish(index.handle),index.handle);guard.done=true;
        }
        const double cursor[2]={double(request.cursor_px.x()),double(request.cursor_px.y())};const Om9SnapLimits limits{request.max_objects,request.max_candidates_per_object,request.max_candidates_total};const auto h=om9_phase2_query_create(index.handle,cursor,request.radius_px,request.modes,&limits);phase2Require(h!=0);QueryGuard guard{h};
        std::uint64_t objects[64]={};const auto count=om9_phase2_query_objects(h,objects,64);if(count>64)throw std::runtime_error("Rust object output exceeds native buffer");
        bool incomplete=false;
        for(std::size_t i=0;i<count&&!incomplete;++i){if(objects[i]==0||objects[i]-1>std::uint64_t(std::numeric_limits<long>::max()))throw std::runtime_error("Invalid native object token");auto* object=doc->getObjectByID(long(objects[i]-1));if(!object)throw std::runtime_error("Native snap object disappeared");
            for(unsigned mode:{2u,4u,8u}){if(!(request.modes&mode))continue;if(om9_phase2_query_cached(h,objects[i],mode))continue;const auto budget=om9_phase2_query_budget(h,objects[i],mode);const auto candidates=boundedSnapCandidates(object,mode,budget);std::vector<Om9SnapPoint> copied;copied.reserve(candidates.points.size());for(const auto& p:candidates.points){SbVec3f screen;volume.projectToScreen(SbVec3f(float(p.x),float(p.y),float(p.z)),screen);copied.push_back({{p.x,p.y,p.z},{double(screen[0])*result.viewport_width,(1.-double(screen[1]))*result.viewport_height,double(screen[2])}});}phase2Require(om9_phase2_query_consume(h,objects[i],mode,copied.data(),copied.size(),candidates.complete?1:0,candidates.visited_topology),h);if(!candidates.complete){incomplete=true;break;}}
        }
        Om9SnapResult r{};phase2Require(om9_phase2_query_finish(h,&r),h);result.complete=r.complete!=0;result.picked=r.picked!=0;result.point=Base::Vector3d(r.point[0],r.point[1],r.point[2]);result.visited_objects=r.visited_objects;result.visited_topology=r.visited_topology;result.generated_candidates=r.generated;result.max_per_object=r.max_per_object;if(!result.complete)result.reason="Snap topology/candidate budget exhausted";
        const auto doubles=om9_phase2_query_points(h,nullptr,0);if(doubles>8192*3||doubles%3)throw std::runtime_error("Invalid Rust candidate output capacity");std::vector<double> points(doubles);om9_phase2_query_points(h,points.data(),points.size());for(std::size_t i=0;i<points.size();i+=3)result.candidates.emplace_back(points[i],points[i+1],points[i+2]);return finish();
    }catch(const Standard_Failure& e){result.complete=false;result.picked=false;result.reason=e.GetMessageString()?e.GetMessageString():"Native CAD snap failed";return finish();}
    catch(const std::exception& e){result.complete=false;result.picked=false;result.reason=e.what();return finish();}
}
}
namespace {
PyObject* query(PyObject*,PyObject* args){int x,y;unsigned modes=2,objects=64,per=2048,total=8192;if(!PyArg_ParseTuple(args,"ii|IIII",&x,&y,&modes,&objects,&per,&total))return nullptr;auto* gui=Gui::Application::Instance->activeDocument();auto* view=gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;try{OpenMatrix9Gui::SnapQuery request;request.cursor_px={x,y};request.modes=modes;request.max_objects=objects;request.max_candidates_per_object=per;request.max_candidates_total=total;const auto r=OpenMatrix9Gui::querySnapCandidates(view,request);QJsonArray candidates;for(const auto& p:r.candidates)candidates.append(QJsonArray{p.x,p.y,p.z});QJsonObject data{{"candidates",candidates},{"complete",r.complete},{"picked",r.picked},{"point",QJsonArray{r.point.x,r.point.y,r.point.z}},{"visited_objects",int(r.visited_objects)},{"visited_topology",int(r.visited_topology)},{"max_per_object",int(r.max_per_object)},{"read_mesh_vertices",0},{"read_cloud_points",0},{"generated_candidates",int(r.generated_candidates)},{"index_rebuilt",r.index_rebuilt},{"index_objects",int(r.index_objects)},{"viewport_width",r.viewport_width},{"viewport_height",r.viewport_height},{"elapsed_us",double(r.elapsed_us)},{"reason",QString::fromStdString(r.reason)},{"state_owner","rust"}};const auto bytes=QJsonDocument(data).toJson(QJsonDocument::Compact);return PyUnicode_FromStringAndSize(bytes.constData(),bytes.size());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}}
}
void AddSnapQueryMethods(PyObject* module){static PyMethodDef methods[]={{"querySnap3dm",query,METH_VARARGS,"Rust-owned bounded index/cache/query with native CAD projection/extraction."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
