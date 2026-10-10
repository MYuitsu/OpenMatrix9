// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstddef>
#include <cstdint>
// Borrowed, initialized C views valid for one call. Rust copies request strings.
struct Om9Phase3Facts {std::uint32_t kind;std::uint8_t valid,closed,protectedInput,reserved;std::uint32_t components;};
struct Om9Phase3Snapshot {const std::uint8_t* identity;std::size_t identityLength;const std::uint8_t* signature;std::size_t signatureLength;};
extern "C" {
std::uint32_t om9_phase3_result_count(std::size_t);
std::uint32_t om9_phase3_capability(std::uint32_t,const Om9Phase3Facts*,std::size_t,bool);
std::uint64_t om9_phase3_request_begin(const std::uint8_t*,std::size_t,const Om9Phase3Snapshot*,std::size_t);
std::uint32_t om9_phase3_request_validate(std::uint64_t,const std::uint8_t*,std::size_t,const Om9Phase3Snapshot*,std::size_t,bool);
void om9_phase3_request_cancel(std::uint64_t);
std::size_t om9_phase3_message(std::uint32_t,std::uint8_t*,std::size_t);
std::uint32_t om9_phase3_surface_options(std::uint32_t,std::uint32_t,bool,std::uint32_t,std::uint32_t,double,std::size_t);
bool om9_phase3_join_route(std::uint32_t);
double om9_phase3_kernel_tolerance();
}
