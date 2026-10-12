// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/DocumentObject.h>
#include <App/PropertyLinks.h>
#include <App/PropertyStandard.h>
#include <Mod/Part/App/PartFeature.h>
#include <fastsignals/signal.h>
#include <string>
#include <vector>
namespace OpenMatrix9Gui {
class HistorySettings final : public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::HistorySettings);
public:
    HistorySettings();
    App::PropertyBool Record, Update, Lock, BrokenHistoryWarning;
    const char* getViewProviderName() const override {return "";}
    void onChanged(const App::Property*) override;
    void onUndoRedoFinished() override;
};
class HistoryJoin final : public Part::Feature {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::HistoryJoin);
public:
    HistoryJoin();
    ~HistoryJoin() override;
    App::PropertyLinkList Parents;
    App::PropertyLinkListHidden PlacementSources;
    App::PropertyFloat Tolerance;
    App::PropertyBool Recorded, HistoryDirty;
    App::PropertyString OM9FeatureId;
    short mustExecute() const override;
    App::DocumentObjectExecReturn* execute() override;
    void onBeforeChange(const App::Property*) override;
    void onChanged(const App::Property*) override;
    void detach();
    void setInitialShape(PyObject*);
    void onSettingDocument() override;
    void onDocumentRestored() override;
    void onUndoRedoFinished() override;
    void unsetupObject() override;
private:
    bool writing=false;
    fastsignals::connection changedConnection;
    void refreshPlacements();
};
void initializeHistoryTypes();
HistorySettings* historySettings(App::Document&, bool create=true);
bool hasRecordedHistory(App::DocumentObject*);
bool hasHistoryDependents(App::DocumentObject*);
bool historyEditLocked(App::DocumentObject*);
bool historyRecordingEnabled(App::Document&);
bool historyUpdatesEnabled(App::Document&);
void warnBrokenHistory(App::DocumentObject*);
void detachObjectHistory(App::DocumentObject*);
// Call inside the same transaction that replaces/consumes ordinary Edit inputs.
// Incoming and affected outgoing records detach; existing geometry stays intact.
void breakHistoryForEdit(App::Document&,const std::vector<std::string>&);
HistoryJoin* createHistoryJoin(App::Document&,const std::vector<std::string>&,double tolerance=1e-7);
}
