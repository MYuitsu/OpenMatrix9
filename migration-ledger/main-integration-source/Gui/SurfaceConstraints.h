// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SurfaceGeometry.h"
namespace OpenMatrix9Gui {
// These bounded eligibility probes never modify the source document.
bool surfaceConstraintHasSupport(App::Document&, const SurfaceInput&) noexcept;
bool surfaceConstraintProfilesEligible(App::Document&, const std::vector<SurfaceInput>&,
                                      unsigned firstProfile) noexcept;
// Owned Part.Shape. The caller holds the GIL; unsupported or unsatisfied
// constraints throw before any document object is created.
PyObject* constrainedSurface(App::Document&, const std::vector<SurfaceInput>&,
                             const SurfaceOptions&);
}
