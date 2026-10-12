// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/GeoFeature.h>
#include <App/PropertyUnits.h>
#include <Gui/ViewProviderGeometryObject.h>
#include <Base/Vector3D.h>
#include <vector>
#include "GumballParametric.h"
class SoSeparator;
struct _object;
namespace OpenMatrix9Gui {
// Physical pose belongs to native Placement. Local -Z is camera/emission
// direction; +Y is up. Symbols are display geometry, never a replacement Shape.
class GumballCamera final:public App::GeoFeature {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::GumballCamera);
public:
    GumballCamera();
    App::PropertyLength FocalDistance;
    App::PropertyAngle HeightAngle;
    App::PropertyLength Height;
    App::PropertyBool Perspective;
    App::PropertyFloat AspectRatio;
    const char* getViewProviderName()const override{return "OpenMatrix9Gui::ViewProviderGumballFrameObject";}
};
class GumballLight final:public App::GeoFeature {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::GumballLight);
public:
    GumballLight();
    App::PropertyLength SizeU,SizeV,Reach;
    App::PropertyFloat Power;
    const char* getViewProviderName()const override{return "OpenMatrix9Gui::ViewProviderGumballFrameObject";}
};
class ViewProviderGumballFrameObject final:public Gui::ViewProviderGeometryObject {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::ViewProviderGumballFrameObject);
public:
    void attach(App::DocumentObject*)override;
    void updateData(const App::Property*)override;
    std::vector<std::string> getDisplayModes()const override{return {"Wireframe"};}
    const char* getDefaultDisplayMode()const override{return "Wireframe";}
    void setDisplayMode(const char*)override;
private:SoSeparator* symbol=nullptr;
};
struct GumballFramePayload {std::vector<std::vector<Base::Vector3d>> lines;Base::Vector3d origin;};
bool isGumballFrameObject(const App::DocumentObject&);
GumballFramePayload gumballFramePayload(const App::DocumentObject&);
std::optional<GumballParametricScale> planGumballFrameScale(App::DocumentObject&,const double*,const double*);
void initializeGumballFrameTypes();
void addGumballFrameObjectMethods(_object*);
std::string createGumballFrameObject(const std::string& kind);
}
