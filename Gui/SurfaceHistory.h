// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SurfaceGeometry.h"
#include <App/PropertyLinks.h>
#include <App/PropertyStandard.h>
#include <Mod/Part/App/PartFeature.h>
#include <fastsignals/signal.h>

namespace OpenMatrix9Gui {
// A native document feature. Its namespace also identifies the module loaded
// by FreeCAD's type system when restoring an FCStd without an active workbench.
class SurfaceHistory final : public Part::Feature {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::SurfaceHistory);
public:
    SurfaceHistory();
    ~SurfaceHistory() override;
    App::PropertyLinkSubListGlobal SourceCurves;
    // Hidden scope avoids a cycle when the result shares a source's App::Part.
    // Native document notifications schedule these placement dependencies.
    App::PropertyLinkListHidden PlacementSources;
    App::PropertyString SurfaceOptions;
    App::PropertyString OM9FeatureId;
    App::PropertyString OM9Command;
    App::PropertyBool HistoryEnabled;
    App::PropertyBool UpdateHistory;
    short mustExecute() const override;
    void initialize(PyObject*, const std::vector<SurfaceInput>&, const OpenMatrix9Gui::SurfaceOptions&);
    void onLostLinkToObject(App::DocumentObject*) override;
protected:
    App::DocumentObjectExecReturn* execute() override;
    void onBeforeChange(const App::Property*) override;
    void onChanged(const App::Property*) override;
    void onDocumentRestored() override;
    void onSettingDocument() override;
    void unsetupObject() override;
private:
    void refreshPlacements();
    void clearShape();
    void disconnect();
    bool changing=false;
    fastsignals::connection changedConnection;
    std::vector<App::DocumentObject*> previousSources;
    std::vector<std::string> previousSubs;
};
std::string serializeSurfaceHistoryOptions(const std::vector<SurfaceInput>&, const SurfaceOptions&);
// The caller owns the document transaction. Returns a fully initialized native
// feature and removes it on failure; no incomplete feature survives an exception.
SurfaceHistory* createSurfaceHistory(App::Document&, PyObject*,
                                    const std::vector<SurfaceInput>&, const SurfaceOptions&);
}
