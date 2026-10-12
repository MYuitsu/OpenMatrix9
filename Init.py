# SPDX-License-Identifier: LGPL-2.1-or-later

"""OpenMatrix9 App initialization."""

import FreeCAD as App
import os
import sys
from pathlib import Path
import ThreeDmStorage
from ThreeDmStorage import ensure_observer

# FreeCAD discovers user workbenches in AppData; the native extension and
# import worker live beside the plugin rather than in the protected host bin.
_om9_plugin_root = Path(ThreeDmStorage.__file__).resolve().parent
os.environ['OM9_PLUGIN_ROOT'] = str(_om9_plugin_root)
_om9_native_path = str(_om9_plugin_root / 'bin')
if _om9_native_path not in sys.path:
    sys.path.insert(0, _om9_native_path)

ensure_observer()

App.addImportType("Rhino 3DM (*.3dm)", "ThreeDm")
App.addExportType("Rhino 5 3DM (*.3dm)", "ThreeDm")
