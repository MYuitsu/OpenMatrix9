// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "LayerRustAbi.h"
extern "C" {
std::uint32_t om9_layer_source_payload_prepare(const Om9LayerByteView*,std::size_t,Om9LayerByteView,std::uint64_t*);
std::uint32_t om9_layer_source_payload_bytes(std::uint64_t,unsigned char*,std::size_t,std::size_t*);
std::uint32_t om9_layer_source_payload_range(std::uint64_t,std::size_t,unsigned char*,std::size_t,std::size_t*);
std::uint32_t om9_layer_source_manifest_verify(std::uint64_t,Om9LayerByteView,Om9LayerByteView);
std::uint32_t om9_layer_source_manifest_verify_witness(std::uint64_t,Om9LayerByteView,Om9LayerByteView,Om9LayerByteView);
std::uint32_t om9_layer_source_payload_free(std::uint64_t);
}
