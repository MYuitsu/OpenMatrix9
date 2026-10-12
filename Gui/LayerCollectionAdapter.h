// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <string>
#include <vector>
#include <cstdint>
namespace App {class Document;}
namespace OpenMatrix9Gui {
// GUI-thread native facts only. Safe Rust owns traversal, scope and duplication.
// Scope1 Selected (empty allowed for palette), scope2 whole Session.
std::vector<std::string> collectLayerTransferObjects(App::Document&,const std::vector<std::string>& selected,std::uint32_t scope);
}
