// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <windows.h>
// Legacy Win32 distance modifiers collide with CAD variables named near/far.
#undef near
#undef far
// On flat Windows x64 FAR originally expands to the empty far modifier.
// Keep that meaning after removing far, including for host zlib declarations.
#if defined(_WIN64)
#undef FAR
#define FAR
#endif
