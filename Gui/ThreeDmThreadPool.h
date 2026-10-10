#pragma once
#include <functional>
#include <cstddef>
#include <mutex>
namespace OpenMatrix9Gui::ThreeDm {
unsigned availableCpuThreads();
unsigned memoryWorkerSlots();
unsigned runNativeJobs(size_t count,unsigned workers,const std::function<void(size_t)>&);
std::recursive_mutex& sdkArchiveMutex();
}
