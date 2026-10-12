// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Base/Placement.h>
#include <array>
#include <functional>
#include <memory>
#include <set>
#include <string>
#include <vector>
namespace App {class Document;class DocumentObject;class Property;enum class LinkScope;}
namespace OpenMatrix9Gui {
struct GumballScaleLinkEdit {
    struct OriginalTarget {
        App::DocumentObject* object=nullptr;
        App::Document* document=nullptr;
        std::string documentName,name;
        long id=0;
    };
    App::DocumentObject* owner=nullptr;
    std::string key;
    std::shared_ptr<App::Property> before;
    std::vector<std::string> subelements;
    // Copy() of native Global link properties loses its subtype and scope.
    // Capture the live descriptor separately from the copied value.
    std::string propertyType;
    App::LinkScope scope{};
    short propertyFlags=0;
    unsigned long protection=0;
    // Stable original identities remain applicable to untouched entries after
    // a batch replaces other targets in the same multi-link property.
    std::vector<OriginalTarget> originalTargets;
};
// Exact live descriptor and stored link-value comparison, independent of the
// native PropertyLinkBase::isSame implementation and Copy() subtype loss.
bool sameGumballIncomingValue(const App::Property&,const GumballScaleLinkEdit&);
struct GumballAssociativeScalePlan {
    App::DocumentObject* source=nullptr;
    // Input and result frames are local to the source's owning App::Part.
    Base::Placement inputFrame,resultFrame;
    std::array<double,3> factors{};
    std::string sourcePlacementExpression;
    // A native Compound gives Link shapes an identity root Location while
    // retaining their complete occurrence transform in the child geometry.
    bool normalizeOccurrence=false;
    // Legacy ShapeBinder.Support reads only native Part::Feature owners.
    bool requiresShapeOwner=false;
    std::vector<App::DocumentObject*> groups;
    std::vector<GumballScaleLinkEdit> incoming;
    bool copy=false;
};
struct GumballAssociativeScaleResult {
    struct Identity {
        App::DocumentObject* object=nullptr;
        std::string name,type;
        long id=0;
    };
    App::Document* document=nullptr;
    std::string documentName,placementExpression;
    std::vector<Identity> identities;
    App::DocumentObject* source=nullptr;
    // Native Global Link bridges the original occurrence into the helper CS;
    // Compound.Links and Scale.Base keep their persisted native Local scopes.
    App::DocumentObject* sourceBridge=nullptr;
    App::DocumentObject* normalizer=nullptr;
    App::DocumentObject* input=nullptr;
    App::DocumentObject* scale=nullptr;
    App::DocumentObject* output=nullptr;
    App::DocumentObject* shapeOwner=nullptr;
    std::vector<App::DocumentObject*> created;
    std::vector<GumballScaleLinkEdit> rewritten;
    std::vector<App::DocumentObject*> groups;
};
// Pure capture/admission; throws with an explicit unsupported-case reason.
// affine is the world-space operation, frame is origin+XYZ axes (12 doubles).
GumballAssociativeScalePlan planGumballAssociativeScale(
    App::DocumentObject&,const double* affine,const double* frame,bool copy,
    const std::set<std::string>& retainedDefinitions={});
// Caller owns one open transaction, mutation guard, recompute/shape validation,
// layer assignment, original visibility, selection output mapping and commit.
// Alt leaves source/dependents intact and adds a new associative native result.
GumballAssociativeScaleResult createGumballAssociativeScale(
    App::Document&,const GumballAssociativeScalePlan&,
    const std::function<void(App::DocumentObject*,const char*,bool begin)>& writeObserver={});
// Read-only durable-input verification. Call after recompute and again after
// callbacks immediately before commit; shape equality alone can miss a late
// input edit whose geometry has not been recomputed yet.
void verifyGumballAssociativeScale(
    const GumballAssociativeScalePlan&,const GumballAssociativeScaleResult&);
}
