# SPDX-License-Identifier: LGPL-2.1-or-later

"""OpenMatrix9 App initialization."""

import FreeCAD as App

App.addImportType("Rhino 3DM (*.3dm)", "ThreeDm")
App.addExportType("Rhino 5 3DM (*.3dm)", "ThreeDm")
