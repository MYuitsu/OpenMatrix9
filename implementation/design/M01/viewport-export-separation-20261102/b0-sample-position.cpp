// SPDX-License-Identifier: LGPL-2.1-or-later
// Diagnostic for the agent-owned synthetic B0 test process only.
#include <windows.h>
#include <tlhelp32.h>
#include <dbghelp.h>
#include <iostream>
#include <vector>
#include <cstdlib>

int main(int argc, char** argv) {
    if (argc != 2) return 1;
    DWORD pid = std::strtoul(argv[1], nullptr, 10);
    HANDLE process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid);
    if (!process) { std::cerr << "OpenProcess failed " << GetLastError() << '\n'; return 2; }
    SymSetOptions(SYMOPT_UNDNAME | SYMOPT_DEFERRED_LOADS);
    const bool symbols = SymInitialize(process, "L:\\om9-target\\debug\\deps", TRUE) != FALSE;
    for (int sample=0; sample<8; ++sample) {
        HANDLE snapshot=CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD,0);
        if (snapshot==INVALID_HANDLE_VALUE) break;
        THREADENTRY32 entry{}; entry.dwSize=sizeof(entry);
        if (Thread32First(snapshot,&entry)) do {
            if (entry.th32OwnerProcessID != pid) continue;
            HANDLE thread=OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION, FALSE, entry.th32ThreadID);
            if (!thread) continue;
            CONTEXT context{}; context.ContextFlags=CONTEXT_CONTROL;
            DWORD prior=SuspendThread(thread);
            bool captured = prior != DWORD(-1) && GetThreadContext(thread,&context);
            if (prior != DWORD(-1)) ResumeThread(thread);
            CloseHandle(thread);
            if (!captured) continue;
            char buffer[sizeof(SYMBOL_INFO)+MAX_SYM_NAME]{};
            auto* info=reinterpret_cast<SYMBOL_INFO*>(buffer);
            info->SizeOfStruct=sizeof(SYMBOL_INFO); info->MaxNameLen=MAX_SYM_NAME;
            DWORD64 displacement=0;
            if (symbols && SymFromAddr(process,context.Rip,&displacement,info))
                std::cout << "thread=" << entry.th32ThreadID << " function=" << info->Name << "\n";
            else std::cout << "thread=" << entry.th32ThreadID << " ip=0x" << std::hex << context.Rip << std::dec << "\n";
        } while (Thread32Next(snapshot,&entry));
        CloseHandle(snapshot);
        Sleep(80);
    }
    if (symbols) SymCleanup(process);
    CloseHandle(process);
}
