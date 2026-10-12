// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
#include <string>

namespace OpenMatrix9Gui {
// Read-only GUI-thread companion. Python receives copied Unicode JSON values;
// no native handle/pointer, document property or transaction is created.
void addGeometryContractMethods(PyObject* module);
// Borrowed native Part wrapper, GUI thread/GIL only. Reuses the Rust-backed
// input tree before a consumer copies or evaluates the selected geometry.
void validateNativePartShape(PyObject* shape, const std::string& operation,
                             const std::string& identity);
void validateCurrentCurveCapability(unsigned representation,unsigned stage,
    const std::string& operation,const std::string& identity);
void validateCurrentSolidCapability(unsigned representation,unsigned stage,
    const std::string& operation,const std::string& identity);
}
