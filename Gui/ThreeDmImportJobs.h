// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstdint>

// Rust owns worker-count policy. Zero rejects configuration; a positive result
// fits int32_t. Value-only ABI: no native object, buffer, or lifetime is retained.
extern "C" std::uint32_t om9_3dm_import_worker_count(
    std::int32_t logicalCpus, std::uint64_t independentTasks,
    std::uint64_t sourceBytes, bool overridePresent, std::int64_t requestedWorkers);
