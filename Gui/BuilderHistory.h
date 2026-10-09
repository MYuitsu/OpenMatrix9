// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/DocumentObject.h>
#include <App/PropertyGeo.h>
#include <App/PropertyLinks.h>
#include <App/PropertyStandard.h>
#include <Mod/Part/App/PartFeature.h>
#include <fastsignals/signal.h>
#include <vector>
namespace OpenMatrix9Gui {
class BuilderRecord final : public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::BuilderRecord);
public:
    BuilderRecord(); ~BuilderRecord() override;
    App::PropertyLinkGlobal SourceGem;
    App::PropertyLinkListHidden PlacementSources;
    App::PropertyString OM9FeatureId, Parameters, GemShapeTag;
    App::PropertyStringList OutputNames;
    App::PropertyVector InitialDimensions;
    App::PropertyPlacement InitialFrame;
    Part::PropertyPartShape Templates;
    App::PropertyBool Recorded,ScaleToGem;
    App::PropertyUUID RecordIdentity;
    const char* getViewProviderName() const override {return "";}
    short mustExecute() const override;
    App::DocumentObjectExecReturn* execute() override;
    void onSettingDocument() override;
    void unsetupObject() override;
    void onDocumentRestored() override;
    void onUndoRedoFinished() override;
    void onChanged(const App::Property*) override;
    void onBeforeChange(const App::Property*) override;
    void onLostLinkToObject(App::DocumentObject*) override;
    void detach();
    Part::TopoShape evaluate(unsigned slot) const;
    void validate() const;
private:
    bool writing=false;
    fastsignals::connection changed;
    void refreshPlacements();
};
class BuilderOutput final : public Part::Feature {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::BuilderOutput);
public:
    BuilderOutput(); ~BuilderOutput() override;
    App::PropertyLinkGlobal ParentRecord;
    App::PropertyLinkListHidden PlacementSources;
    App::PropertyInteger TemplateIndex;
    App::PropertyBool Recorded;
    App::PropertyUUID OriginRecordIdentity;
    short mustExecute() const override;
    App::DocumentObjectExecReturn* execute() override;
    void onBeforeChange(const App::Property*) override;
    void onChanged(const App::Property*) override;
    void onUndoRedoFinished() override;
    void detach();
    void onSettingDocument() override;
    void onDocumentRestored() override;
    void unsetupObject() override;
    void initialize(BuilderRecord*,unsigned);
    void invalidate();
private:
    bool writing=false;
    void refreshPlacements();
    void setWorldShape(const Part::TopoShape&);
    fastsignals::connection changed,beforeChanged;
};
void initializeBuilderHistoryTypes();
bool builderHistoryRecorded(App::DocumentObject*);
std::vector<App::DocumentObject*> builderHistoryParents(App::DocumentObject*);
bool detachBuilderHistory(App::DocumentObject*);
bool isBuilderStorageObject(const App::DocumentObject*);
}
void AddBuilderHistoryMethods(PyObject*);
