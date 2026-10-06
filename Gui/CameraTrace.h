#pragma once
#include <QFile>
#include <QTextStream>
#ifdef _WIN32
#include <windows.h>
#include <dbghelp.h>
#endif
namespace OpenMatrix9Gui {
inline void traceCameraChange(int slot,float angle) {
    const auto path=qgetenv("OM9_TRACE_CAMERA");if(path.isEmpty())return;
    QFile file(QString::fromLocal8Bit(path));if(!file.open(QIODevice::WriteOnly|QIODevice::Append))return;QTextStream output(&file);output<<"slot "<<slot<<" angle "<<angle<<'\n';
#ifdef _WIN32
    static auto library=LoadLibraryExW(L"dbghelp.dll",nullptr,LOAD_LIBRARY_SEARCH_SYSTEM32);
    if(!library)return;
    static auto initialize=reinterpret_cast<decltype(&SymInitialize)>(GetProcAddress(library,"SymInitialize"));
    static auto fromAddress=reinterpret_cast<decltype(&SymFromAddr)>(GetProcAddress(library,"SymFromAddr"));
    static bool ready=initialize&&initialize(GetCurrentProcess(),nullptr,TRUE);
    if(!ready||!fromAddress)return;
    void* frames[32];const auto count=CaptureStackBackTrace(0,32,frames,nullptr);
    for(USHORT i=0;i<count;++i){alignas(SYMBOL_INFO) char bytes[sizeof(SYMBOL_INFO)+1024]={};auto* symbol=reinterpret_cast<SYMBOL_INFO*>(bytes);symbol->SizeOfStruct=sizeof(SYMBOL_INFO);symbol->MaxNameLen=1023;DWORD64 displacement=0;
        if(fromAddress(GetCurrentProcess(),reinterpret_cast<DWORD64>(frames[i]),&displacement,symbol))output<<symbol->Name<<'\n';
    }
#endif
}
}
