// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SolidReferences.h"
namespace OpenMatrix9Gui {
// Synchronous GUI/GIL reader of builtin native properties. Counts are admitted
// by Rust before private point storage; no unbounded Python geometry lists.
std::vector<SolidPoint> nativeFitPoints(App::Document&,const std::string& name,
    const std::string& sub,bool surfacePoles,std::size_t used,const char* operation,bool currentCurve=false,bool currentSolid=false,
    const std::function<void(unsigned)>& independentAdmission={});
}
