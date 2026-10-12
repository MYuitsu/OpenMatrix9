#include "ThreeDmThreadPool.h"
#include "ThreeDmArchive.h"
#include <atomic>
#include <thread>
#include <chrono>
#include <iostream>
#include <stdexcept>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool b,const char* text){if(!b)throw std::runtime_error(text);}
int main(){try{
    std::atomic<int> live=0,peak=0,done=0;
    auto result=runNativeJobs(24,7,[&](size_t i){
        auto current=++live;auto p=peak.load();while(current>p&&!peak.compare_exchange_weak(p,current)){}
        std::this_thread::sleep_for(std::chrono::milliseconds(10));--live;++done;
    });
    require(done==24&&peak>1&&peak<=7,"bounded actual concurrency");
    bool failed=false;try{runNativeJobs(24,7,[&](size_t i){++live;--live;if(i==3)throw std::runtime_error("owned task failure");});}catch(const std::runtime_error&){failed=true;}
    require(failed&&live==0,"failure propagated after join");
    auto serial=readArchive(OM9_POOL_FIXTURE);
    auto deferred=readWorkingArchiveDeferred(OM9_POOL_FIXTURE);
    runNativeJobs(deferred.pendingBreps.size(),7,[&](size_t i){finishDeferredBrep(deferred,i);});
    require(serial.items.size()==deferred.items.size(),"stable occurrence count");
    for(size_t i=0;i<serial.items.size();++i){auto& a=serial.items[i];auto& b=deferred.items[i];require(a.name==b.name&&a.sourceRootUuid==b.sourceRootUuid,"stable names and root order");
        if(auto s=std::get_if<TopoDS_Shape>(&a.geometry)){auto t=std::get_if<TopoDS_Shape>(&b.geometry);require(t&&!t->IsNull(),"completed CAD geometry");GProp_GProps x,y;BRepGProp::SurfaceProperties(*s,x);BRepGProp::SurfaceProperties(*t,y);require(std::abs(x.Mass()-y.Mass())<0.001,"serial and threaded surface area");}
    }
    std::cout<<"Native thread pool: concurrency, join-on-error, ordered CAD equivalence PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
