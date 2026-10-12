// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstdint>
#include <optional>
#include <string>
namespace OpenMatrix9Gui {
struct ToleranceValues {
    std::uint64_t contextRevision;
    std::uint32_t operation, ruleVersion, source, usedFields;
    double requestedMm, effectiveMm, numericalFloorMm;
};
static_assert(sizeof(ToleranceValues)==48);
struct ToleranceSnapshot {
    ToleranceValues values{};
    std::string context, report;
};
// Owned values only. Operation 1 = curve Join, 2 = surface Join. No mutation.
ToleranceSnapshot resolveTolerance(const std::string& context,unsigned operation,
    const std::optional<double>& overrideMm,double worldCoordinateBound);
}
