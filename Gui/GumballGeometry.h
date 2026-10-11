// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "CoreObjects.h"
#include "CoreCoordinates.h"
#include "GumballRust.h"
#include <memory>
#include <functional>
#include <vector>
#include <QJsonObject>
struct _object;
namespace App {class Document;}
namespace Gui {class SelectionObject;}
namespace OpenMatrix9Gui {
struct GumballNativeData;
struct GumballReadDependency {ObjectSnapshot snapshot;QJsonObject attributes;};
struct GumballInputs {
    CoordinateContext coordinates;
    std::vector<ObjectSnapshot> roots, witnesses;
    std::vector<GumballReadDependency> definitions;
    std::vector<Om9GumballSource> sources;
    std::shared_ptr<const GumballNativeData> native;
};
struct GumballStagedBatch {std::shared_ptr<GumballNativeData> native;QJsonObject facts;std::string report;std::function<void()> beforeCommit;};
GumballInputs captureGumballInputs(App::Document&,const std::vector<Gui::SelectionObject>&,const CoordinateContext&);
void verifyGumballInputs(App::Document&,const GumballInputs&);
GumballStagedBatch stageGumballBatch(const GumballInputs&,const Om9GumballSnapshot&);
void publishGumballBatch(App::Document&,const GumballInputs&,const Om9GumballSnapshot&,GumballStagedBatch&&);
std::vector<std::vector<Base::Vector3d>> gumballPreviewLines(const GumballStagedBatch&);
QJsonObject gumballInputFacts(const GumballInputs&);
QJsonObject gumballObjectFrameFacts(const GumballInputs&);
void addGumballGeometryMethods(_object*);
}
