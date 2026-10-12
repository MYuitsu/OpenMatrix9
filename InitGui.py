# SPDX-License-Identifier: LGPL-2.1-or-later

import os
import sys
import FreeCADGui as Gui


class OpenMatrix9Workbench(Gui.Workbench):
    MenuText = "OpenMatrix9"
    ToolTip = "OpenMatrix9 Jewelry CAD"

    def Initialize(self):
        import Part  # Native Surface History derives from the Part document feature.
        # Khi user chọn OpenMatrix9 thì load native GUI module.
        import OpenMatrix9Gui
        from ThreeDmArchiveState import ensure_preview_observer
        ensure_preview_observer()

    def GetClassName(self):
        # Link Python shell tới native Workbench.
        return "OpenMatrix9Gui::Workbench"

    def Activated(self):
        from ThreeDmClipboard import install_shortcuts
        install_shortcuts()

    def Deactivated(self):
        from ThreeDmClipboard import remove_shortcuts
        remove_shortcuts()


# FreeCAD's loader may own __file__; resolve the actual workbench source.
_om9_root = os.path.dirname(OpenMatrix9Workbench.Initialize.__code__.co_filename)
os.environ['OM9_PLUGIN_ROOT'] = _om9_root
_om9_bin = os.path.join(_om9_root, 'bin')
if os.path.isdir(_om9_bin):
    sys.path.insert(0, _om9_bin)
OpenMatrix9Workbench.Icon = os.path.join(
    _om9_root,
    "Resources", "branding", "workbench.png",
)
Gui.addWorkbench(OpenMatrix9Workbench())
