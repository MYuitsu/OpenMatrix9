// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SolidReferences.h"
#include "CircleHistoryAdapter.h"
#include <App/PropertyGeo.h>
#include <App/PropertyLinks.h>
#include <App/PropertyStandard.h>
#include <App/PropertyUnits.h>
#include <Mod/Part/App/PartFeature.h>
#include <fastsignals/signal.h>
namespace OpenMatrix9Gui {
class CircleHistory final:public Part::Feature {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::CircleHistory);
public:
    CircleHistory();
    ~CircleHistory() override;
    App::PropertyLinkSubGlobal SourceCurve;
    App::PropertyLinkListHidden PlacementSources;
    App::PropertyFloat PathFraction;
    App::PropertyLength Radius;
    App::PropertyBool Recorded;
    // Empty Recipe preserves the original v1 analytic AroundCurve record.
    App::PropertyFloatList Recipe;
    App::PropertyVectorList Points;
    App::PropertyLinkSubListGlobal Sources;
    App::PropertyIntegerList SourceRoles;
    App::PropertyFloatList SourceParameters;
    App::PropertyFloat FitDeviation,ApproxDeviation;
    short mustExecute()const override;
    App::DocumentObjectExecReturn* execute() override;
    void initialize(PyObject*,const SolidCurveReference&,double,double);
    void initializeRecipe(PyObject*,const std::vector<double>&,const std::vector<SolidPoint>&,const std::vector<CircleHistorySource>&,const std::optional<SolidCurveReference>&,double);
    void detach();
    void onLostLinkToObject(App::DocumentObject*) override;
protected:
    void onBeforeChange(const App::Property*) override;
    void onChanged(const App::Property*) override;
    void onSettingDocument() override;
    void onDocumentRestored() override;
    void unsetupObject() override;
private:
    bool changing=false;
    fastsignals::connection changed,undone,redone;
    void disconnect();
    void refreshPlacements();
    void clearShape();
};
CircleHistory* createCircleHistory(App::Document&,PyObject*,const SolidCurveReference&,double,double);
}
