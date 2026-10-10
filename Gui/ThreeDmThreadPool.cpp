#include "ThreeDmThreadPool.h"
#include <QThread>
#include <thread>
#include <vector>
#include <atomic>
#include <algorithm>
#include <bit>
#include <exception>
#ifdef _WIN32
#include <Windows.h>
#endif
namespace OpenMatrix9Gui::ThreeDm {
std::recursive_mutex& sdkArchiveMutex(){static std::recursive_mutex mutex;return mutex;}
unsigned availableCpuThreads(){
    auto count=QThread::idealThreadCount();unsigned n=count>0?count:1;
#ifdef _WIN32
    DWORD_PTR process=0,system=0;
    if(GetProcessAffinityMask(GetCurrentProcess(),&process,&system)&&process)n=std::min(n,static_cast<unsigned>(std::popcount(process)));
    else n=1;
#endif
    return n;
}
unsigned memoryWorkerSlots(){
#ifdef _WIN32
    MEMORYSTATUSEX memory{};memory.dwLength=sizeof(memory);
    if(GlobalMemoryStatusEx(&memory))return static_cast<unsigned>(std::clamp<ULONGLONG>(memory.ullAvailPhys/(256ULL*1024*1024),1,1024));
#endif
    return 1;
}
unsigned runNativeJobs(size_t count,unsigned workers,const std::function<void(size_t)>& job){
    if(!count)return 0;
    workers=static_cast<unsigned>(std::min<size_t>(std::max(1u,workers),count));
    if(workers==1){for(size_t i=0;i<count;++i)job(i);return 1;}
    std::atomic<size_t> next{0};std::atomic<unsigned> live{0},peak{0};std::atomic<bool> stop{false};
    std::exception_ptr failure;std::mutex failureMutex;
    {
        // RAII joins even if starting a later worker throws.
        std::vector<std::jthread> threads;threads.reserve(workers);
        for(unsigned w=0;w<workers;++w)threads.emplace_back([&]{
#ifdef _WIN32
            SetThreadPriority(GetCurrentThread(),THREAD_PRIORITY_BELOW_NORMAL);
#endif
            while(!stop.load()){
                const auto i=next.fetch_add(1);if(i>=count)break;
                auto current=live.fetch_add(1)+1,p=peak.load();while(current>p&&!peak.compare_exchange_weak(p,current)){}
                try{job(i);}catch(...){std::lock_guard lock(failureMutex);if(!failure)failure=std::current_exception();stop=true;}
                --live;
            }
        });
    }
    if(failure)std::rethrow_exception(failure);
    return peak;
}
}
