// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/Annotation.h>
#include <App/PropertyGeo.h>
#include <Inventor/SbVec3f.h>
#include <Gui/ViewProviderAnnotation.h>
class SoMatrixTransform;
namespace OpenMatrix9Gui {
class ExplodedText final:public App::Annotation {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::ExplodedText);
public:
    ExplodedText();
    App::PropertyMatrix TextFrame;
    const char* getViewProviderName() const override{return "OpenMatrix9Gui::ViewProviderExplodedText";}
};
class ViewProviderExplodedText final:public Gui::ViewProviderAnnotation {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::ViewProviderExplodedText);
public:
    void attach(App::DocumentObject*) override;
    void updateData(const App::Property*) override;
private:SoMatrixTransform* frame=nullptr;
};
}
