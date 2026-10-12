// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <array>
#include <QString>
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
struct GumballInputs;
struct GumballFrameObservations {std::array<double,12> frame{};QString unresolved;};
GumballFrameObservations captureGumballFrame(const GumballInputs&,Gui::View3DInventor*,unsigned alignment);
}
