// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "GumballRust.h"
#include <memory>
#include <optional>
#include <string>
#include <vector>
namespace App {class DocumentObject; class Expression;}
namespace OpenMatrix9Gui {
struct GumballParametricEdit {
    std::string key;
    double factor=1,before=0,value=0;
    std::string beforeExpression,expressionText;
    std::shared_ptr<App::Expression> expression;
};
struct GumballParametricScale {
    Om9GumballOwnerScale owner{};
    std::vector<GumballParametricEdit> edits;
};
// Native metadata/ExpressionEngine bridge only. Rust admits the exact affine.
// No document writes. nullopt routes the owner to an associative affine result.
// Reparse expressionText against a copied output owner before publication.
std::optional<GumballParametricScale> planGumballParametricScale(
    App::DocumentObject&,const double* affine,const double* world);
}
