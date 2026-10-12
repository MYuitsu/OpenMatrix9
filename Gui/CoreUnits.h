// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <cstdint>
#include <string>
struct _object;
namespace OpenMatrix9Gui {
class UnitSettings final : public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::UnitSettings);
public:
    UnitSettings();
    App::PropertyString Context;
    const char* getViewProviderName() const override { return ""; }
};
// Empty means legacy command-specific mm input, with no invented tolerance.
std::string unitContextSnapshot(const App::Document&);
double unitInputScale(const App::Document&);
void addUnitContextProvenance(App::DocumentObject&, const std::string&);
void initializeUnitTypes();
void addUnitMethods(_object*);
}
