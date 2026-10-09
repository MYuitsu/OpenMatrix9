// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SolidReferences.h"
#include <QString>
namespace App {class DocumentObject;}
namespace OpenMatrix9Gui {
// Native source roles: 0 point, -1 direction endpoint, 1 FitPoints source,
// 2 tangent curve, 3 FitPoints prefix retained after Undo, 4 picked Radius
// endpoint anchored at a prior point. Offsets/counts refer
// to the immutable recorded point snapshot.
struct CircleHistorySource {
    std::string name,sub,signature;
    long role=0;
    std::size_t index=0,count=1;
    std::vector<SolidPoint> snapshot;
    App::DocumentObject* identity=nullptr;
};
std::vector<SolidPoint> circleHistorySourcePoints(App::Document&,const std::string&,const std::string&);
std::optional<unsigned> circleHistoryNativeInput(App::Document&,const QString&);
void clearCircleHistoryReferences();
void pruneCircleHistoryReferences(unsigned effect);
void verifyCircleHistoryReferences(App::Document&);
App::DocumentObject* createCircleHistoryForCommand(App::Document&,PyObject*,const std::map<std::size_t,SolidCurveReference>&,const std::optional<SolidCurveReference>&);
// Construct a private native shape from owned replay data; never publishes into
// the shared preview spline state.
CurvePyRef circleHistoryShape(const double* replay);
}
