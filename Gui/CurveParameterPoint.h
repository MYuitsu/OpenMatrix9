// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/PropertyLinks.h>
#include <App/PropertyStandard.h>
#include <Mod/Part/App/PartFeature.h>
#include <fastsignals/signal.h>
namespace OpenMatrix9Gui {
class CurveParameterPoint final:public Part::Feature {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::CurveParameterPoint);
public:
    CurveParameterPoint();
    ~CurveParameterPoint() override;
    App::PropertyLinkSubGlobal Source;
    App::PropertyLink UnitSource;
    App::PropertyLinkListHidden PlacementSources;
    App::PropertyFloat SourceParameter,LengthTolerance,ParameterTolerance;
    App::PropertyBool UseRawParameter,Recorded,HistoryDirty;
    Part::PropertyPartShape ConvertedShape;
    App::PropertyString QueryReport;
    short mustExecute() const override;
    App::DocumentObjectExecReturn* execute() override;
    void onBeforeChange(const App::Property*) override;
    void onChanged(const App::Property*) override;
    void onSettingDocument() override;
    void onDocumentRestored() override;
    void onUndoRedoFinished() override;
    void onLostLinkToObject(App::DocumentObject*) override;
    void unsetupObject() override;
    void detach();
    void initialize(App::DocumentObject*,const std::string&,double,double,double,bool);
private:
    bool writing=false;
    fastsignals::connection changed,recomputed;
    void refreshFrames();
    void clearOutputs();
    void setWorldPoint(const Part::TopoShape&);
};
void initializeCurveParameterTypes();
void addCurveParameterMethods(PyObject*);
bool curveParameterRecorded(App::DocumentObject*);
std::vector<App::DocumentObject*> curveParameterParents(App::DocumentObject*);
bool detachCurveParameter(App::DocumentObject*);
}
