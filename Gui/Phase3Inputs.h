// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
#include "CurveGeometry.h"
#include "Phase3RustBridge.h"
#include <memory>
#include <string>
#include <stdexcept>
#include <array>
#include <utility>
#include <vector>
namespace App {class Document;}
namespace OpenMatrix9Gui {
struct Phase3SnapshotState;
using Phase3Snapshot=std::shared_ptr<Phase3SnapshotState>;
Phase3Snapshot capturePhase3Object(App::Document&,const std::string&,const std::string& sub={});
void verifyPhase3Object(App::Document&,const Phase3Snapshot&);
Om9Phase3Facts phase3ShapeFacts(PyObject*,bool protectedInput=false);
void phase3Require(std::uint32_t);
void phase3Validate(unsigned,const std::vector<Om9Phase3Facts>&,bool complete=true);
bool phase3SurfaceJoinSelection();
CurvePyRef publishedSurfaceSplineShape();
std::vector<std::array<double,3>> sampleCurve(PyObject*,std::size_t);
}
