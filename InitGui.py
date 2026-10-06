# SPDX-License-Identifier: LGPL-2.1-or-later

import os

import FreeCADGui as Gui


class OpenMatrix9Workbench(Gui.Workbench):
    MenuText = "OpenMatrix9"
    ToolTip = "OpenMatrix9 Jewelry CAD"

    def Initialize(self):
        # Khi user chọn OpenMatrix9 thì load native GUI module.
        import OpenMatrix9Gui

    def GetClassName(self):
        # Link Python shell tới native Workbench.
        return "OpenMatrix9Gui::Workbench"


# FreeCAD executes InitGui.py; __file__ can belong to its loader instead.
OpenMatrix9Workbench.Icon = os.path.join(
    os.path.dirname(OpenMatrix9Workbench.Initialize.__code__.co_filename),
    "Resources", "branding", "workbench.png",
)

Gui.addWorkbench(OpenMatrix9Workbench())
