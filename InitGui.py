# SPDX-License-Identifier: LGPL-2.1-or-later

import FreeCADGui as Gui


class OpenMatrix9Workbench(Gui.Workbench):
    MenuText = "OpenMatrix9"
    ToolTip = "OpenMatrix9 Jewelry CAD"

    def Initialize(self):
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


Gui.addWorkbench(OpenMatrix9Workbench())
