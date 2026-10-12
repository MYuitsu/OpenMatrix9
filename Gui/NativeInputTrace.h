// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
// Fixture-only entry instrumentation. Normal libraries compile every hook out;
// there is no observer, counter or test state in production native classes.
#ifdef OM9_NATIVE_INPUT_TRACE
namespace OpenMatrix9Gui::ThreeDm::InputTrace {void entered(const char*);}
#define OM9_NATIVE_INPUT_ENTRY(name) ::OpenMatrix9Gui::ThreeDm::InputTrace::entered(name)
#else
#define OM9_NATIVE_INPUT_ENTRY(name) ((void)0)
#endif
