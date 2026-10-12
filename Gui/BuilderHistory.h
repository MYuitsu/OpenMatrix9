// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/DocumentObject.h>
#include <App/PropertyGeo.h>
#include <App/PropertyLinks.h>
#include <App/PropertyStandard.h>
#include <Mod/Part/App/PartFeature.h>
#include <fastsignals/signal.h>
#include <vector>
#include <optional>
namespace OpenMatrix9Gui {
class BuilderRecord final : public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::BuilderRecord);
public:
    BuilderRecord(); ~BuilderRecord() override;
    App::PropertyLinkGlobal SourceGem;
    App::PropertyLinkListHidden PlacementSources;
    App::PropertyString OM9FeatureId, Parameters, GemShapeTag, TemplateAttributes;
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
    App::PropertyString OM9GeometryResult;
    short mustExecute() const override;
    App::DocumentObjectExecReturn* execute() override;
    void onBeforeChange(const App::Property*) override;
    void onChanged(const App::Property*) override;
    void onUndoRedoFinished() override;
    void detach();
    void onSettingDocument() override;
    void onDocumentRestored() override;
    void unsetupObject() override;
    void initialize(BuilderRecord*,unsigned,const Part::TopoShape& preparedLocal,const std::string& result);
    // Native-owned, synchronous initial publication only. The API scope clears
    // this slot after recompute/abort; it is never a persisted replay cache.
    void stagePreparedShape(const Part::TopoShape&,const std::string& result);
    void clearPreparedShape();
    void invalidate();
private:
    bool writing=false;
    struct Prepared {Part::TopoShape shape;std::string result;};
    std::optional<Prepared> preparedShape;
    void refreshPlacements();
    void setWorldShape();
    fastsignals::connection changed,beforeChanged;
};
void initializeBuilderHistoryTypes();
bool builderHistoryRecorded(App::DocumentObject*);
std::vector<App::DocumentObject*> builderHistoryParents(App::DocumentObject*);
bool detachBuilderHistory(App::DocumentObject*);
bool isBuilderStorageObject(const App::DocumentObject*);
}
void AddBuilderHistoryMethods(PyObject*);
