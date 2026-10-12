// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "LayerRustAbi.h"
// Borrowed schema1 native role/relationship facts. Role: geometry1, transparent
// group2, opaque final aggregate3, verified definition4, placed instance5,
// verified metadata6, unsupported7. Native adapters must prove each role.
struct Om9LayerCollectionNodeView {
    Om9LayerByteView id,label,native_type;
    const Om9LayerByteView* children;
    std::size_t child_count;
    std::uint32_t role,reserved;
};
static_assert(sizeof(Om9LayerCollectionNodeView)==72);
extern "C" {
std::uint32_t om9_layer_collection_check_size(std::size_t);
std::uint32_t om9_layer_collection_prepare(const Om9LayerCollectionNodeView*,std::size_t,std::uint32_t,const Om9LayerByteView*,std::size_t,std::uint64_t*);
std::uint32_t om9_layer_collection_json(std::uint64_t,unsigned char*,std::size_t,std::size_t*);
std::uint32_t om9_layer_collection_free(std::uint64_t);
}
