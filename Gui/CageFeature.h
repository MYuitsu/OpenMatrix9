// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/DocumentObject.h>
#include <App/PropertyGeo.h>
#include <App/PropertyLinks.h>
#include <App/PropertyStandard.h>
#include <Mod/Part/App/PartFeature.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <fastsignals/signal.h>
#include <vector>
namespace OpenMatrix9Gui {
class CageControl final : public Part::Feature {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::CageControl);
public:
    CageControl();
    App::PropertyVectorList ControlPoints;
    App::PropertyIntegerList Counts,Degrees;
    App::PropertyFloatList UKnots,VKnots,WKnots,Weights;
    App::PropertyString OM9FeatureId,SourceMetadata;
    App::PropertyMatrix ReferenceFrame;
    short mustExecute()const override;
    App::DocumentObjectExecReturn* execute()override;
};
class CageBinding final : public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::CageBinding);
public:
    CageBinding();~CageBinding()override;
    App::PropertyLinkGlobal Control,Captive;
    App::PropertyLinkListHidden ControlPlacementSources,CaptivePlacementSources;
    App::PropertyBool Active,MeshSource;
    Part::PropertyPartShape SourceShape;
    App::PropertyVectorList SourceVertices,BoundParameters;
    App::PropertyIntegerList SourceTriangles;
    App::PropertyMatrix BoundFrame;
    App::PropertyBool AffineBind;
    App::PropertyString Region,OM9FeatureId;
    App::PropertyVector LocalMin,LocalMax;
    App::PropertyFloat Falloff;
    const char* getViewProviderName()const override{return "";}
    short mustExecute()const override;
    App::DocumentObjectExecReturn* execute()override;
    void onSettingDocument()override;
    void onUndoRedoFinished()override;
    void onDocumentRestored()override;
    void onChanged(const App::Property*)override;
    void onLostLinkToObject(App::DocumentObject*)override;
    void unsetupObject()override;
    void detach();void clearInvalidGeometry();
    bool writing=false;
private:
    void refreshPlacements();
    fastsignals::connection changed,beforeChanged;
};
void initializeCageTypes();void AddCageMethods(PyObject*);
bool cageHistoryRecorded(App::DocumentObject*);
std::vector<App::DocumentObject*> cageHistoryParents(App::DocumentObject*);
void detachCageHistory(App::DocumentObject*);
bool isCageStorageObject(App::DocumentObject*);
}
