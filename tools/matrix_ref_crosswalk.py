"""Crosswalk: M00-M02 L2/L3 records -> verifiable reference anchors.

Two reference families are routed here, and neither replaces runtime evidence:

1. ``RCORE-xx`` - the Rhino 5 core capability routing table published in
   ``ref/rhino5/CORE_REFERENCE_INDEX.json`` (user-approved public documentation
   exception). Chapters, Guide PDF ranges, official Help URLs, capability IDs and
   fixture IDs come from that index; the index itself does not execute or pass
   fixtures.
2. ``MX-*`` - identifiers recovered from the private Matrix9 reference exports
   under ``ref/matrix`` (dnSpy .NET export + VB Decompiler Lite export + PE
   resource inspection + Ghidra listing). Only paths, module/class names and
   identifier names are recorded; no reference source body is copied.

Verification states used for Matrix9 sources:
- ``CONTENT_READ_IDENTIFIERS``: file content was read to extract identifiers.
- ``IDENTIFIER_INDEXED``: module name / declaration list extracted from the
  exported file by ``tools/matrix_ref_sources.py``.
- ``PATH_INDEXED_SYMBOL_CANDIDATE``: file is present in the export manifest and
  project tree; class/type symbol is a candidate to re-verify locally.
- ``RESOURCE_MANIFEST_READ``: metadata (hashes, resource entries) read from the
  inspection manifest, not from the binary itself.

A third family, ``MX_MANUAL``, points at the authored reference sheets shipped
inside ``OM9_Milestones`` (``reference/text/*.md``) that transcribe one Matrix
manual section each: command path, option list and stated defaults. Those sheets
are in-repository and are the right place to resolve Matrix command/option
questions; they are still documentation, not runtime evidence.
"""

# Generated registries: authored manual sheets (all milestones) and the M03 L3
# anchor rows. Both are produced by tools/gen_manual_sheets.py and
# tools/gen_m03_anchors.py so a missing/renamed sheet fails the validator.
try:  # pragma: no cover - import guard for partial checkouts
    from matrix_manual_sheets import MANUAL_SHEETS
    from matrix_m03_anchors import M03_ANCHORS
except ImportError:  # pragma: no cover
    MANUAL_SHEETS = {}
    M03_ANCHORS = {}


SCHEMA_VERSION = 1
CROSSWALK_DATE = "2026-10-15"
REFERENCE_POLICY_ID = "om9.reference_supplement.matrix9_rhino5.v1"
PROVENANCE_KIND = "reference_anchor"
REQUIREMENTS_ONLY = "requirements_only_not_runtime_evidence"

# --------------------------------------------------------------------------
# RCORE chapters (ref/rhino5/CORE_REFERENCE_INDEX.json)
# --------------------------------------------------------------------------
RCORE = {
    "RCORE-01": {
        "title": "Kiểu đối tượng và ranh giới kernel",
        "guide_pdf": "25-31 / in 17-23",
        "capabilities": ["RCORE-01.C01", "RCORE-01.C02", "RCORE-01.C03", "RCORE-01.C04",
                          "RCORE-01.C05", "RCORE-01.C06", "RCORE-01.C07", "RCORE-01.C08",
                          "RCORE-01.C09", "RCORE-01.C10", "RCORE-01.C11", "RCORE-01.C12"],
        "fixtures": ["RCORE-01.T01", "RCORE-01.T02", "RCORE-01.T03", "RCORE-01.T04"],
        "help": [("ExtrudeCrv", "https://docs.mcneel.com/rhino/5/help/en-us/commands/extrudecrv.htm"),
                 ("Mesh", "https://docs.mcneel.com/rhino/5/help/en-us/commands/mesh.htm")],
        "gap": "Không suy kernel intersection/Boolean/refit hoặc bảo toàn parameterization từ guide/openNURBS, render mesh hay tên BRep.",
    },
    "RCORE-02": {
        "title": "Units, tolerance và độ chính xác",
        "guide_pdf": "Không gán Guide thành đặc tả đầy đủ của nhóm này",
        "capabilities": ["RCORE-02.C01", "RCORE-02.C02", "RCORE-02.C03", "RCORE-02.C04",
                          "RCORE-02.C05", "RCORE-02.C06"],
        "fixtures": ["RCORE-02.T01", "RCORE-02.T02", "RCORE-02.T03", "RCORE-02.T04", "RCORE-02.T13"],
        "help": [("Units properties", "https://docs.mcneel.com/rhino/5/help/en-us/documentproperties/units.htm")],
        "gap": "Guide/layout tutorial không định nghĩa shared tolerance context; default Help, template, setting hiện tại và ngưỡng solver OM9 phải ghi riêng.",
    },
    "RCORE-03": {
        "title": "Tọa độ, parser điểm và CPlane",
        "guide_pdf": "39-43 / in 31-35; 49-51 / in 41-43",
        "capabilities": ["RCORE-03.C01", "RCORE-03.C02", "RCORE-03.C03", "RCORE-03.C04",
                          "RCORE-03.C05", "RCORE-03.C06", "RCORE-03.C07"],
        "fixtures": ["RCORE-03.T01", "RCORE-03.T02", "RCORE-03.T03", "RCORE-03.T04", "RCORE-03.T08"],
        "help": [("CPlane", "https://docs.mcneel.com/rhino/5/help/en-us/commands/cplane.htm"),
                 ("Cursor constraints / coordinate entry",
                  "https://docs.mcneel.com/rhino/5/help/en-us/user_interface/cursor_constraints.htm")],
        "gap": "Guide xác nhận x,y và x,y,z theo CPlane; không dùng native record Circle cũ để kết luận binary hiện tại dùng world XYZ.",
    },
    "RCORE-04": {
        "title": "Command engine và vòng đời tương tác",
        "guide_pdf": "11-23 / in 3-15",
        "capabilities": ["RCORE-04.C01", "RCORE-04.C02", "RCORE-04.C03", "RCORE-04.C04",
                          "RCORE-04.C05", "RCORE-04.C06", "RCORE-04.C07", "RCORE-04.C08"],
        "fixtures": ["RCORE-04.T01", "RCORE-04.T02", "RCORE-04.T04", "RCORE-04.T05", "RCORE-04.T08"],
        "help": [("Undo / Redo", "https://docs.mcneel.com/rhino/5/help/en-us/commands/undo.htm"),
                 ("Aliases", "https://docs.mcneel.com/rhino/5/help/en-us/options/aliases.htm"),
                 ("Rhino scripting / command macros",
                  "https://docs.mcneel.com/rhino/5/help/en-us/information/rhinoscripting.htm")],
        "gap": "Command transcript, successful repeat candidate, point/session Undo, document Undo và associative History là các loại state riêng.",
    },
    "RCORE-05": {
        "title": "Pick, snapping và constraints",
        "guide_pdf": "45-51 / in 37-43",
        "capabilities": ["RCORE-05.C01", "RCORE-05.C02", "RCORE-05.C03", "RCORE-05.C04",
                          "RCORE-05.C05", "RCORE-05.C06", "RCORE-05.C07", "RCORE-05.C08",
                          "RCORE-05.C09", "RCORE-05.C10", "RCORE-05.C11", "RCORE-05.C12"],
        "fixtures": ["RCORE-05.T01", "RCORE-05.T02", "RCORE-05.T11", "RCORE-05.T12", "RCORE-05.T14"],
        "help": [("Object snaps", "https://docs.mcneel.com/rhino/5/help/en-us/user_interface/object_snaps.htm"),
                 ("Ortho / OrthoAngle / SetOrtho", "https://docs.mcneel.com/rhino/5/help/en-us/commands/ortho.htm"),
                 ("Cursor constraints / Elevator",
                  "https://docs.mcneel.com/rhino/5/help/en-us/user_interface/cursor_constraints.htm")],
        "gap": "Lifecycle one-shot/suspend trong checkpoint không chứng minh geometric snapping, ranking hay precedence Project/Ortho/SmartTrack/Elevator/SnapToLocked.",
    },
    "RCORE-06": {
        "title": "Selection, layer, group, block và Worksession",
        "guide_pdf": "33-38 / in 25-30; 97-98 / in 89-90",
        "capabilities": ["RCORE-06.C01", "RCORE-06.C02", "RCORE-06.C03", "RCORE-06.C04",
                          "RCORE-06.C05", "RCORE-06.C06", "RCORE-06.C07", "RCORE-06.C08",
                          "RCORE-06.C09", "RCORE-06.C10"],
        "fixtures": ["RCORE-06.T01", "RCORE-06.T03", "RCORE-06.T04", "RCORE-06.T06",
                     "RCORE-06.T08", "RCORE-06.T10"],
        "help": [("Selection commands / sub-object selection",
                  "https://docs.mcneel.com/rhino/5/help/en-us/commands/selection_commands.htm"),
                 ("Layer", "https://docs.mcneel.com/rhino/5/help/en-us/commands/layer.htm"),
                 ("Block", "https://docs.mcneel.com/rhino/5/help/en-us/commands/block.htm")],
        "gap": "Group/App::Link/import flatten không chứng minh block editor hay Worksession; chương 06 vẫn not_run trong CAPABILITY_LOOKUP.",
    },
    "RCORE-07": {
        "title": "NURBS editing và modeling",
        "guide_pdf": "53-67 / in 45-59; 69-71 / in 61-63; 73-91 / in 65-83",
        "capabilities": ["RCORE-07.C01", "RCORE-07.C02", "RCORE-07.C03", "RCORE-07.C04",
                          "RCORE-07.C05", "RCORE-07.C06", "RCORE-07.C07", "RCORE-07.C08",
                          "RCORE-07.C09", "RCORE-07.C10", "RCORE-07.C11", "RCORE-07.C12"],
        "fixtures": ["RCORE-07.T01", "RCORE-07.T03", "RCORE-07.T05", "RCORE-07.T07",
                     "RCORE-07.T08", "RCORE-07.T10"],
        "help": [("PointsOn / PointsOff", "https://docs.mcneel.com/rhino/5/help/en-us/commands/pointson.htm"),
                 ("Rebuild", "https://docs.mcneel.com/rhino/5/help/en-us/commands/rebuild.htm"),
                 ("FitCrv", "https://docs.mcneel.com/rhino/5/help/en-us/commands/fitcrv.htm")],
        "gap": "Tutorial success, valid basis hay coarse sampled deviation không chứng minh shared solver, G0/G1/G2, knot/domain semantics hoặc full NURBS editor.",
    },
    "RCORE-08": {
        "title": "Analysis, diagnostic và repair",
        "guide_pdf": "93-96 / in 85-88",
        "capabilities": ["RCORE-08.C01", "RCORE-08.C02", "RCORE-08.C03", "RCORE-08.C04",
                          "RCORE-08.C05", "RCORE-08.C06", "RCORE-08.C07", "RCORE-08.C08"],
        "fixtures": ["RCORE-08.T01", "RCORE-08.T04", "RCORE-08.T06", "RCORE-08.T07",
                     "RCORE-08.T11", "RCORE-08.T12"],
        "help": [("CurvatureGraph", "https://docs.mcneel.com/rhino/5/help/en-us/commands/curvaturegraph.htm"),
                 ("CurvatureAnalysis", "https://docs.mcneel.com/rhino/5/help/en-us/commands/curvatureanalysis.htm"),
                 ("Zebra", "https://docs.mcneel.com/rhino/5/help/en-us/commands/zebra.htm")],
        "gap": "Zebra/EMap/shaded không tự chứng nhận G2, closedness hoặc archive integrity; validity/BOP coverage và skipped/not_run phải giữ rõ.",
    },
    "RCORE-09": {
        "title": "Associative History và dependency identity",
        "guide_pdf": "98 / in 90 (chỉ ghi chú dimension)",
        "capabilities": ["RCORE-09.C01", "RCORE-09.C02", "RCORE-09.C03", "RCORE-09.C04",
                          "RCORE-09.C05", "RCORE-09.C06", "RCORE-09.C07", "RCORE-09.C08"],
        "fixtures": ["RCORE-09.T01", "RCORE-09.T03", "RCORE-09.T04", "RCORE-09.T05",
                     "RCORE-09.T08", "RCORE-09.T11", "RCORE-09.T12"],
        "help": [("History / HistoryUpdate / HistoryPurge",
                  "https://docs.mcneel.com/rhino/5/help/en-us/commands/history.htm"),
                 ("Loft (history-enabled example)", "https://docs.mcneel.com/rhino/5/help/en-us/commands/loft.htm")],
        "gap": "Guide không đặc tả graph/schema/migration/identity; OM9 Detach/Undo hoặc dirty-state policy là quyết định OM9.",
    },
    "RCORE-10": {
        "title": "Persistence, 3DM và clipboard",
        "guide_pdf": "Không gán Guide thành đặc tả đầy đủ của nhóm này",
        "capabilities": ["RCORE-10.C01", "RCORE-10.C02", "RCORE-10.C03", "RCORE-10.C04",
                          "RCORE-10.C05", "RCORE-10.C06", "RCORE-10.C07", "RCORE-10.C08"],
        "fixtures": ["RCORE-10.T01", "RCORE-10.T02", "RCORE-10.T03", "RCORE-10.T05",
                     "RCORE-10.T06", "RCORE-10.T08", "RCORE-10.T10", "RCORE-10.T12"],
        "help": [("Save / -Save", "https://docs.mcneel.com/rhino/5/help/en-us/commands/save.htm"),
                 ("Import", "https://docs.mcneel.com/rhino/5/help/en-us/commands/import.htm"),
                 ("Export", "https://docs.mcneel.com/rhino/5/help/en-us/commands/export.htm")],
        "gap": "Guide hoặc archive version 5/parser reread không chứng minh Rhino-readable edited output, units, structural blocks, userdata/History hay hai chiều binary clipboard.",
    },
    "RCORE-11": {
        "title": "Annotation, Make2D, Layout/Detail và print",
        "guide_pdf": "98-100 / in 90-92; 281-284 / in 273-276",
        "capabilities": ["RCORE-11.C01", "RCORE-11.C02", "RCORE-11.C03", "RCORE-11.C04",
                          "RCORE-11.C05", "RCORE-11.C06", "RCORE-11.C07"],
        "fixtures": ["RCORE-11.T01", "RCORE-11.T03", "RCORE-11.T04", "RCORE-11.T06",
                     "RCORE-11.T07", "RCORE-11.T11"],
        "help": [("Dim", "https://docs.mcneel.com/rhino/5/help/en-us/commands/dim.htm"),
                 ("Make2D", "https://docs.mcneel.com/rhino/5/help/en-us/commands/make2d.htm"),
                 ("Detail", "https://docs.mcneel.com/rhino/5/help/en-us/commands/detail.htm")],
        "gap": "Layout/Make2D đã có spec; guide overview world XY không ghi đè help Current CPlane. Ví dụ trang 11×8.5 inch và scale 1:1 không phải defaults chung.",
    },
    "RCORE-12": {
        "title": "Display, render và resource lifecycle",
        "guide_pdf": "39-43 / in 31-35; 101-104 / in 93-96",
        "capabilities": ["RCORE-12.C01", "RCORE-12.C02", "RCORE-12.C03", "RCORE-12.C04",
                          "RCORE-12.C05", "RCORE-12.C06", "RCORE-12.C07", "RCORE-12.C08"],
        "fixtures": ["RCORE-12.T01", "RCORE-12.T02", "RCORE-12.T04", "RCORE-12.T05",
                     "RCORE-12.T06", "RCORE-12.T08", "RCORE-12.T13"],
        "help": [("SetDisplayMode", "https://docs.mcneel.com/rhino/5/help/en-us/commands/setdisplaymode.htm"),
                 ("Render / render-window commands", "https://docs.mcneel.com/rhino/5/help/en-us/commands/render.htm")],
        "gap": "Viewport capture, EMap hoặc Rendered label không chứng minh production renderer; mesh/cache/resource lifecycle và plugin availability cần fixture/backend evidence riêng.",
    },
}

# --------------------------------------------------------------------------
# Matrix9 recovered-source anchors (ref/matrix)
# --------------------------------------------------------------------------
MX = {
    # -- application shell / panels -----------------------------------------
    "mx-frm-master": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMaster.frm", unit="frmMaster",
        role="application shell",
        detail="Main Matrix window shell: owns vbalCommandBar, LEAD control, XceedZip and the "
               "profile/builder/library timers; title caption observed as 'Matrix - Untitled.3dm'.",
        observed=["frmMaster", "Form_Load", "Form_Resize", "Form_Unload", "tmrBuilderLoader_Timer",
                  "tmrLibraryBrowserOpenFile_Timer", "tmpCSharpProfileEditor_Timer", "cmdbar_UnknownEvent_A"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-cpanel": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmCPanel.frm", unit="frmCPanel",
        role="dockable sidebar panel host",
        detail="Generic Matrix sidebar panel host with caption 'Panel'; AIFCmp1.asxGenericContainer "
               "plus Matrix.ctrTitleBar and a PictureBox - the container reused by the seven sidebar "
               "sections.",
        observed=["frmCPanel", "Form_Load", "Form_Unload", "frmType", "Matrix.ctrTitleBar",
                  "AIFCmp1.asxGenericContainer"],
        state="CONTENT_READ_IDENTIFIERS"),
    "mx-cmdbar": dict(
        project="MatrixCmdBar (VB6 OCX)", path="MatrixCmdBar/vbalCommandBar.ctl", unit="vbalCommandBar",
        role="command bar control",
        detail="Docked/floatable command bar control (153 declarations) with supporting classes "
               "cButton, cButtons, cCommandBar, cCommandBars, cCommandBarImageList, cMenuPopupStack, "
               "cMenuDropShadow and cAlphaDIBSection.",
        observed=["vbalCommandBar", "cButton", "cButtons", "cCommandBar", "cCommandBars",
                  "cCommandBarImageList", "cMenuPopupStack", "cMenuDropShadow", "cAlphaDIBSection"],
        state="IDENTIFIER_INDEXED"),
    "mx-btn-csharp": dict(
        project="Matrix (managed plugin)", path="Matrix/WindowsHolder.cs", unit="WindowsHolder",
        role="native/VB6 <-> managed bridge",
        detail="Managed holder for the native Matrix windows; paired with VBBuilder.cs, "
               "WindowsMessaging.cs and WindowsMessageNo.cs which carry the Win32 message numbers "
               "used to drive the VB6 sidebar.",
        observed=["WindowsHolder", "VBBuilder", "WindowsMessaging", "WindowsMessageNo"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-com-interface": dict(
        project="Matrix (managed plugin)", path="Matrix/Interfaces/COMInterface.cs", unit="COMInterface",
        role="COM bridge to native interface",
        detail="Exported COM interface surface used by the native/VB6 layer, together with "
               "MatrixCoreInterface.cs and MatrixCoreUserDataInterface.cs.",
        observed=["COMInterface", "MatrixCoreInterface", "MatrixCoreUserDataInterface"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-clsminterface": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/clsMInterface.cls", unit="clsMInterface",
        role="VB6-side object/document facade",
        detail="VB6 class that fronts document operations for the sidebar; declarations include "
               "Import, Export, AllObjects, SelectedObjects, SelectObject(s), UnselectAllObjects, "
               "SelLast, CurrentLayer, CopyObject(s), HideObject, ShowObject, SetObjectName, "
               "objectlayer, ShowObjects.",
        observed=["clsMInterface", "Import", "Export", "AllObjects", "SelectedObjects",
                  "SelectAllObjects", "UnselectAllObjects", "CurrentLayer", "HideObject",
                  "ShowObject", "SetObjectName", "objectlayer"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-messages": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMessages.frm", unit="frmMessages",
        role="message surface",
        detail="Matrix message window used for command/status feedback to the user.",
        observed=["frmMessages"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-debuglog": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmDebugLog.frm", unit="frmDebugLog",
        role="diagnostic surface",
        detail="Diagnostic log window referenced when a Matrix action reports a nonfatal failure.",
        observed=["frmDebugLog"],
        state="IDENTIFIER_INDEXED"),
    # -- command engine -----------------------------------------------------
    "mx-mod-windows-messaging": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modWindowsMessaging.bas", unit="modWindowsMessaging",
        role="command dispatch via Win32 messaging",
        detail="Window-message dispatch module (procedures recovered as Proc_66_*) used between the "
               "Matrix shell, command bar and the Rhino host; pairs with WindowsMessageNo / "
               "WindowsMessaging in the managed export.",
        observed=["modWindowsMessaging", "Proc_66_0_1691290", "Proc_66_1_1692D10"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-special-commands": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modSpecialCommands.bas", unit="modSpecialCommands",
        role="command implementations",
        detail="Special command module (33 declarations) holding Matrix-specific command entry points.",
        observed=["modSpecialCommands"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-menu-controls": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modMenuControls.bas", unit="modMenuControls",
        role="command availability and menu state",
        detail="Menu/control state module that enables or greys icon slots for the current selection "
               "and feature readiness.",
        observed=["modMenuControls"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-global": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modGlobal.bas", unit="modGlobal",
        role="shared globals/options",
        detail="Shared constants and defaults for the VB6 interface; the declarations are large and "
               "must be re-read in the private archive before any default is treated as Matrix9 truth.",
        observed=["modGlobal"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-menu-dialog": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuDialog.frm", unit="frmMenuDialog",
        role="modal Matrix dialog host",
        detail="Dialog host used by Matrix menu actions that require an option form.",
        observed=["frmMenuDialog"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-dialog-options": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmDialogOptions.frm", unit="frmDialogOptions",
        role="option defaults",
        detail="Matrix options dialog; observed captions include 'Matrix Options', 'Density', "
               "'Maximum Angle', 'Maximum Aspect Ratio', 'Maximum Edge Length', "
               "'Minimum Edge Length', 'Minimum Initial Grid Quads', 'STL Options', "
               "'Vray Builder Options', 'Adjust Color'. Captions are UI evidence, not defaults.",
        observed=["frmDialogOptions", "ctrGo", "ctrLabel", "ctrNumberInput", "ctrOnOff",
                  "ctrSlider1Wide"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-vb-core": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modVBCoreReplacement.bas", unit="modVBCoreReplacement",
        role="host/API glue",
        detail="Replacement shim for VB runtime/host calls (33 declarations) - the layer that must "
               "not be ported as behaviour definition.",
        observed=["modVBCoreReplacement"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-rhino-controls": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modRhinoControls.bas", unit="modRhinoControls",
        role="Rhino host control calls",
        detail="Large module (44 recovered declarations) that drives Rhino host objects from the "
               "Matrix UI; recovered bodies are disassembly stubs (Proc_1_*), so it routes to the "
               "Rhino 5 core contract rather than defining it.",
        observed=["modRhinoControls", "Proc_1_0_14DEC60"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-file-system": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modFileSystem.bas", unit="modFileSystem",
        role="file/project I-O",
        detail="File-system and project path module (52 declarations) behind Matrix file actions.",
        observed=["modFileSystem"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-menu-icon-history": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuIconHistory.frm", unit="frmMenuIconHistory",
        role="recent-command icon history",
        detail="Sidebar section 'Icon History' with cmdUndoSmall/cmdRedoSmall/CmdIconUndo/CmdIconRedo "
               "and a right-click slot per icon.",
        observed=["frmMenuIconHistory", "cmdUndoSmall", "cmdRedoSmall", "CmdIconUndo", "CmdIconRedo",
                  "CmdIcon_UnknownEvent_A"],
        state="IDENTIFIER_INDEXED"),
    "mx-cmd-iconhistory": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/IconHistoryCommand.cs",
        unit="IconHistoryCommand", role="icon history command",
        detail="Managed command entry point for the Icon History sidebar.",
        observed=["IconHistoryCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- Rhino object/document semantics -----------------------------------
    "mx-matrixdata": dict(
        project="Matrix (managed plugin)", path="Matrix/MatrixData/MatrixPoint.cs",
        unit="MatrixData family", role="typed Rhino object data wrappers",
        detail="Typed Matrix data wrappers: MatrixPoint, MatrixCurve, MatrixSurface, MatrixProfile, "
               "MatrixBoolean, MatrixDouble(Length/Angle/PointOnCrv), MatrixGemInstance(List), "
               "MatrixGemBlock(List), MatrixList - i.e. the Matrix9 view of Rhino-like object types.",
        observed=["MatrixPoint", "MatrixCurve", "MatrixSurface", "MatrixProfile", "MatrixBoolean",
                  "MatrixDouble", "MatrixDoubleLength", "MatrixDoubleAngle",
                  "MatrixDoublePointOnCrv", "MatrixGemInstance", "MatrixGemInstanceList",
                  "MatrixGemBlock", "MatrixGemBlockList", "MatrixList"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-matrixdata-base": dict(
        project="Matrix (managed plugin)", path="Matrix/MatrixData/MatrixDataBase.cs",
        unit="MatrixDataBase", role="data envelope",
        detail="Base envelope for Matrix data objects including usage flags (EDataUsage).",
        observed=["MatrixDataBase", "EDataUsage"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-userdata": dict(
        project="Matrix (managed plugin)", path="Matrix/UserData/GemvisionUserDataBase.cs",
        unit="UserData family", role="jewelry semantic userdata",
        detail="Rhino userdata carriers for jewelry semantics: GemvisionUserDataBase, "
               "PricingUserData, DesignReportUserData, VikingUserData.",
        observed=["GemvisionUserDataBase", "PricingUserData", "DesignReportUserData",
                  "VikingUserData"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-matrix-userdata": dict(
        project="Matrix (managed plugin)", path="Matrix/Interfaces/MatrixUserData.cs",
        unit="MatrixUserData", role="userdata interface",
        detail="Managed interface exposing Matrix userdata on Rhino objects.",
        observed=["MatrixUserData"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-gemmap": dict(
        project="Matrix (managed plugin)", path="Matrix/PIM/GemMap.cs", unit="GemMap",
        role="gem map model",
        detail="Gem map model with the PIMData GemMap geometry helpers (Clipper/IntPoint/gXForm).",
        observed=["GemMap", "GemMapPIM", "gemCrv", "gXForm", "Clipper"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-gemblock": dict(
        project="Matrix (managed plugin)", path="Matrix/PIM/GemBlock.cs", unit="GemBlock",
        role="gem block model",
        detail="Gem block model (71 KB export) that backs gem instances and their placement.",
        observed=["GemBlock"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- layers / objects ---------------------------------------------------
    "mx-frm-menu-layers": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuLayers.frm", unit="frmMenuLayers",
        role="layers sidebar palette",
        detail="Layers palette (39 declarations): PicShow/PicHide/PicShowSmall/PicHideSmall "
               "visibility toggles, txtRename rename field, lblLayerName/lblArrow row affordances, "
               "ctrAdvancedOption advanced block, Matrix ToolBar caption and a Lights block.",
        observed=["frmMenuLayers", "PicShow", "PicHide", "PicShowSmall", "PicHideSmall", "txtRename",
                  "txtRename_KeyPress", "lblLayerName_MouseDown", "lblArrow_MouseDown",
                  "ctrAdvancedOption", "Timer2_Timer"],
        state="IDENTIFIER_INDEXED"),
    "mx-cmd-layers": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/LayersCommand.cs", unit="LayersCommand",
        role="layers command entry",
        detail="Managed command that opens the layers palette.",
        observed=["LayersCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-frm-menu-info-settings": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuInfoSettings.frm",
        unit="frmMenuInfoSettings", role="object info / settings sidebar",
        detail="Sidebar section with caption 'Menu Info History' and a tmrUpdate_Timer that "
               "refreshes object info for the current selection.",
        observed=["frmMenuInfoSettings", "tmrUpdate_Timer", "CmdIconTop", "CmdIconHistory",
                  "CmdIconBottom", "frmType"],
        state="IDENTIFIER_INDEXED"),
    "mx-info-settings-cmd": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/InfoAndSettingsCommand.cs",
        unit="InfoAndSettingsCommand", role="info/settings command",
        detail="Managed command for the Info/Settings sidebar section.",
        observed=["InfoAndSettingsCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-object-actions": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/ObjectActionsEdit.cs",
        unit="ObjectActionsEdit", role="object property editor",
        detail="Object property/action editor with its selection companion ObjectActionSelection.cs "
               "and ObjectSelectionWatcher.cs (managed observation of the Rhino selection).",
        observed=["ObjectActionsEdit", "ObjectActionSelection", "ObjectSelectionWatcher",
                  "ObjectActionsEditCommand", "ObjectActionsViewCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-seldup": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/SelDupCommand.cs", unit="SelDupCommand",
        role="duplicate selection command",
        detail="Managed SelDup command used by the sidebar super-select family.",
        observed=["SelDupCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- file / project / job bags -----------------------------------------
    "mx-frm-menu-projects": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuProjects.frm", unit="frmMenuProjects",
        role="projects sidebar + job bag shelf",
        detail="Projects sidebar (104 declarations) with caption 'Project Manager': project list "
               "control ctrProjectList, per-row ProjectImage/ProjectOut/JobBagImage/ssPJOpenClose "
               "handlers, picNew and CommonDialog for file selection.",
        observed=["frmMenuProjects", "ctrProjectList", "ProjectImage", "ProjectOut", "JobBagImage",
                  "ssPJOpenClose", "picNew", "ctrAdvancedOption", "MSComDlg.CommonDialog"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-project-manager": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmProjectManager.frm", unit="frmProjectManager",
        role="project manager window",
        detail="Project Manager window (46 declarations) that hosts project rows, notes and job "
               "bag operations.",
        observed=["frmProjectManager"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-project-manager": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modProjectManager.bas", unit="modProjectManager",
        role="project manager backend",
        detail="Project Manager backend module (34 recovered declarations, bodies as Proc_174_* "
               "stubs) that drives project and job bag persistence.",
        observed=["modProjectManager", "Proc_174_0_19BD130"],
        state="IDENTIFIER_INDEXED"),
    "mx-wpf-project-manager": dict(
        project="MatrixWPFControls", path="MatrixWPFControls/View/ProjectManagerWindow.xaml",
        unit="ProjectManagerWindow", role="WPF project manager UI",
        detail="WPF project manager window with ProjectManagerViewModel, JobBag/Project entity "
               "models and JobBagType/JobBagVisibility/Status enums.",
        observed=["ProjectManagerWindow", "ProjectManagerViewModel", "Project", "JobBag",
                  "JobBagType", "JobBagVisibility", "Status", "ProjectSerializable"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-project-dialogs": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuProjectDialogNew.frm",
        unit="project dialogs", role="project create/rename/load dialogs",
        detail="Small project dialogs: frmMenuProjectDialogNew, frmMenuProjectDialogRename, "
               "frmMenuProjectLoad, frmMenuProjectDialog1/2 and frmMenuProjectDialog1.",
        observed=["frmMenuProjectDialogNew", "frmMenuProjectDialogRename", "frmMenuProjectLoad",
                  "frmMenuProjectDialog1", "frmMenuProjectDialog2"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-dialog-save": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmDialogSave.frm", unit="frmDialogSave",
        role="save dialog",
        detail="Matrix save dialog (10 declarations) used by File Save/Save As flows.",
        observed=["frmDialogSave"],
        state="IDENTIFIER_INDEXED"),
    "mx-open-project-manager": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/OpenProjectManagerCommand.cs",
        unit="OpenProjectManagerCommand", role="project manager command",
        detail="Managed command that opens the project manager (paired with "
               "ProjectManagerCommand.cs).",
        observed=["OpenProjectManagerCommand", "ProjectManagerCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-frm-dialog-savegem": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmDialogSaveGem.frm", unit="frmDialogSaveGem",
        role="save-to-library dialog",
        detail="Save gem/part dialog used when capturing a design into a library slot.",
        observed=["frmDialogSaveGem"],
        state="IDENTIFIER_INDEXED"),
    # -- viewport / display -------------------------------------------------
    "mx-frm-cmenu-display": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmCMenuDisplay.frm", unit="frmCMenuDisplay",
        role="display sidebar section",
        detail="Sidebar section with caption 'Display'; UpdateHistory plus CmdIcon handlers, and "
               "the AIFCmp1/Matrix.ctrTitleBar shell used by every sidebar section.",
        observed=["frmCMenuDisplay", "UpdateHistory", "CmdIcon", "AICFmp1.asxGenericContainer"],
        state="IDENTIFIER_INDEXED"),
    "mx-display-net": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/Display/DisplayCommand.cs",
        unit="DisplayCommand", role="display command and forms",
        detail="Managed display family: DisplayCommand, ShadeModeForm, ShadeModeDisplayConduit, "
               "ShadeModeEventWatcher, MeshDensityForm, MeshDensityEventWatcher.",
        observed=["DisplayCommand", "ShadeModeForm", "ShadeModeDisplayConduit",
                  "ShadeModeEventWatcher", "MeshDensityForm", "MeshDensityEventWatcher"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-frm-dialog-shade-select": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmDialogShadeSelect.frm",
        unit="frmDialogShadeSelect", role="shade mode chooser",
        detail="Shade Select dialog: image-based mode chooser (imgMeshMode) with LEAD control; "
               "caption observed as 'Shade Select'.",
        observed=["frmDialogShadeSelect", "imgMeshMode", "ShowMe", "LEADLib.LEAD"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-dialog-vga": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmDialogVGADisplay.frm",
        unit="frmDialogVGADisplay", role="graphics adapter diagnostic",
        detail="VGA/display diagnostics dialog (7 declarations) - the Matrix9 surface for graphics "
               "adapter problems, relevant to device-loss and unsupported-adapter policy.",
        observed=["frmDialogVGADisplay"],
        state="IDENTIFIER_INDEXED"),
    "mx-config": dict(
        project="MatrixCommon", path="MatrixCommon/Configuration.cs", unit="Configuration",
        role="configuration and display defaults",
        detail="Shared configuration object (with MatrixOptions.cs) that carries Matrix user "
               "settings; colour/control resources live in the WPF resource dictionaries "
               "Colorresources.xaml, Controlresources.xaml, Mainresources.xaml.",
        observed=["Configuration", "MatrixOptions", "Colorresources", "Controlresources",
                  "Mainresources"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-wpf-resources": dict(
        project="MatrixWPFControls", path="MatrixWPFControls/Resources/Colorresources.xaml",
        unit="WPF resource dictionaries", role="colour and control resources",
        detail="WPF resource dictionaries (Colorresources, Controlresources, Mainresources, "
               "Combobox, Scrollbar, Datepicker, Imageresources) that describe the managed "
               "Matrix UI theme.",
        observed=["Colorresources", "Controlresources", "Mainresources", "Imageresources"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-layer-render": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/LayerRender/LayerRender.cs",
        unit="LayerRender", role="layer render/preset",
        detail="Layer render form and command (LayerRenderForm, gvLayerRenderCommand) plus "
               "batch variant frmCBatchLayerRender in the VB6 export.",
        observed=["LayerRender", "LayerRenderForm", "gvLayerRenderCommand", "frmCBatchLayerRender"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-gvpanel": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/GVPanel/GVPanelCommand.cs",
        unit="GVPanelCommand", role="viewport panel command",
        detail="Managed command for the gvMatrixCore viewport panel (native gvMatrixCore.rhp has "
               "DIALOG resources 28000-28069 recorded in native-inspection).",
        observed=["GVPanelCommand", "gvMatrixCore"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-imageeditor": dict(
        project="MatrixControls", path="MatrixControls/MImageEditor.cs", unit="MImageEditor",
        role="image reference editor",
        detail="Managed image editor/picker controls (MImageEditor, MImagePicker, MImage) used for "
               "reference-image insertion and adjustment.",
        observed=["MImageEditor", "MImagePicker", "MImage"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-layout-tools": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/LayoutTool/LayoutToolsForm.cs",
        unit="LayoutToolsForm", role="print/layout and image placement tools",
        detail="Layout/print tool family: LayoutToolsForm, LayoutTemplates, LayoutViews, "
               "LayoutUtils, XML3DMInfo, XML3DMStone; paired with "
               "frmDialogPrintLayoutOptions.frm in the VB6 export.",
        observed=["LayoutToolsForm", "LayoutTemplates", "LayoutViews", "LayoutUtils", "XML3DMInfo",
                  "frmDialogPrintLayoutOptions"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- picking / selection / handles --------------------------------------
    "mx-ctrl-inbox": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/Controls/CtrInBox.cs",
        unit="CtrInBox family", role="object/point input control",
        detail="Matrix input controls: CtrInBox, CtrInBoxImage, CtrInBoxProfile, MenuList, "
               "MenuSlider, Slider - the managed counterparts of the VB6 ctrAdvancedOption, "
               "ctrNumberInput, ctrSelectMenu and ctrSlider* controls.",
        observed=["CtrInBox", "CtrInBoxImage", "CtrInBoxProfile", "MenuList", "MenuSlider",
                  "Slider", "ctrAdvancedOption", "ctrNumberInput", "ctrSelectMenu",
                  "ctrSlider1Graphics", "ctrSlider2Wide"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-mouse-callback": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/MouseCallBackHandler.cs",
        unit="MouseCallBackHandler", role="pointer event routing",
        detail="Managed mouse callback handler that routes pointer events into Matrix commands and "
               "selection.",
        observed=["MouseCallBackHandler"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-msr": dict(
        project="Matrix (managed plugin)", path="Matrix/SmartTargets/MSR", unit="MSR",
        role="smart target / MSR handles",
        detail="Smart-target (MSR) family: BlendPoint, PointFixer, MyGetPoint, ColorList and the "
               "gvSmartTarget* commands/family under Matrix/SmartTargets.",
        observed=["BlendPoint", "PointFixer", "MyGetPoint", "ColorList", "gvSmartTargetCommand",
                  "gvSmartBlendCommand", "gvSelSmartTarget"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-tube-handles": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/TubeFromCurves/MatrixHandle/MatrixHandle.cs",
        unit="MatrixHandle family", role="viewport handle implementation",
        detail="Matrix handle implementation for curve-driven tools: MatrixHandle, "
               "MatrixHandleAngle, MatrixHandleLength, MatrixHandleLenPer, "
               "MatrixHandlePointOnCrv, MatrixHandleNormalizedPointOnCrv, GetHandle, "
               "GetHandleExecConduit, HandleValue, HandleValueSlider.",
        observed=["MatrixHandle", "MatrixHandleAngle", "MatrixHandleLength", "MatrixHandleLenPer",
                  "MatrixHandlePointOnCrv", "MatrixHandleNormalizedPointOnCrv", "GetHandle",
                  "GetHandleExecConduit", "HandleValue", "HandleValueSlider"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-matrixcontrols": dict(
        project="MatrixControls", path="MatrixControls/MButton.cs", unit="M* control family",
        role="authored UI control family",
        detail="Matrix WinForms control family that authored the sidebar look: MButton, MListBox, "
               "MCheckBox, MComboBoxLarge/Small, MControl, MGroupBox, MLabel, MNumberBox, "
               "MScrollBar, MSliderSimple/Advanced/ColorGradient, MTabBar, MTabControl, MTextBox, "
               "MTextBoxNumeric, MCollapsePanel, MBorder, MColorPicker(Sliders), DropObject.",
        observed=["MButton", "MListBox", "MCheckBox", "MComboBoxLarge", "MComboBoxSmall", "MControl",
                  "MGroupBox", "MLabel", "MNumberBox", "MScrollBar", "MSliderSimple",
                  "MSliderAdvanced", "MSliderColorGradient", "MTabBar", "MTabControl", "MTextBox",
                  "MTextBoxNumeric", "MCollapsePanel", "MBorder", "MColorPicker"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- snaps / ortho / smarttrack ----------------------------------------
    "mx-frm-menu-snaps": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuSnaps.frm", unit="frmMenuSnaps",
        role="snaps sidebar palette",
        detail="Snaps sidebar palette (19 declarations): imgosnapbuttons / imgosnapbuttonsinline "
               "image-button strips, imgGridButton, imgchksnaps, PicSnap, picGridSnap, "
               "tmrSnaps_Timer and the named procedures SetInitialSnaps and UpdateSnaps.",
        observed=["frmMenuSnaps", "imgosnapbuttons", "imgosnapbuttonsinline", "imgGridButton",
                  "imgchksnaps", "PicSnap", "picGridSnap", "tmrSnaps_Timer", "SetInitialSnaps",
                  "UpdateSnaps"],
        state="IDENTIFIER_INDEXED"),
    "mx-snapscmd": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/SnapsCommand.cs", unit="SnapsCommand",
        role="snaps command entry",
        detail="Managed command that opens the Matrix snaps palette (sidebar section 'Snaps').",
        observed=["SnapsCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-mousewheel": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/CHookMouseWheel.cls", unit="CHookMouseWheel",
        role="mouse-wheel hook",
        detail="VB6 mouse-wheel hook class (15 declarations) plus MHookXP.bas/IHookXP.cls - the "
               "wheel-routing layer for the Matrix shell.",
        observed=["CHookMouseWheel", "MHookXP", "IHookXP"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-rightclick": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modRightClick.bas", unit="modRightClick",
        role="right-click / context action routing",
        detail="Right-click routing module that dispatches the Matrix context action (object menu "
               "or disambiguation) for the current object and mode.",
        observed=["modRightClick"],
        state="IDENTIFIER_INDEXED"),
    "mx-mod-topmenu": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modTopMenu.bas", unit="modTopMenu",
        role="main menu construction",
        detail="Top/main menu construction module (9 recovered declarations as Proc_199_* stubs); "
               "pairs with frmMenuTools/frmMenuEditor for the icon category grid.",
        observed=["modTopMenu", "Proc_199_0_1A41500"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-menu-tools": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuTools.frm", unit="frmMenuTools",
        role="main menu category grid",
        detail="Icon Menu sidebar section: icon rows with CmdIcon handlers, SSMenuHeader and "
               "SSMenuReset, MSComctlLib.ImageList icon source, tmrCheckKey_Timer.",
        observed=["frmMenuTools", "CmdIcon", "SSMenuHeader", "SSMenuReset", "tmrCheckKey_Timer",
                  "MSComctlLib.ImageList"],
        state="IDENTIFIER_INDEXED"),
    "mx-frm-menu-editor": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuEditor.frm", unit="frmMenuEditor",
        role="menu layout editor",
        detail="Menu editor (21 declarations) used to rearrange/customise the Matrix icon categories "
               "and layout.",
        observed=["frmMenuEditor"],
        state="IDENTIFIER_INDEXED"),
    "mx-main-command": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/MainCommand.cs", unit="MainCommand",
        role="main menu command entry",
        detail="Managed command that opens the Matrix main menu / sidebar (with "
               "ObjectActionsImportMenusCommand for custom menu import).",
        observed=["MainCommand", "ObjectActionsImportMenusCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- F6 context menu ----------------------------------------------------
    "mx-object-context-menu": dict(
        project="MatrixWPFControls", path="MatrixWPFControls/Model/ObjectContextMenu.cs",
        unit="ObjectContextMenu", role="F6/object context menu model",
        detail="Object context menu model: ObjectContextMenu, ObjectContextMenuGroup, "
               "ObjectContextMenuItem, ObjectContextMenuItemType, ObjectContextMenuMode, ObjectType, "
               "with views ObjectContextMenuView/Window and editor "
               "ObjectContextMenuEditorView/Window.",
        observed=["ObjectContextMenu", "ObjectContextMenuGroup", "ObjectContextMenuItem",
                  "ObjectContextMenuItemType", "ObjectContextMenuMode", "ObjectType",
                  "ObjectContextMenuView", "ObjectContextMenuWindow",
                  "ObjectContextMenuEditorView", "ObjectContextMenuEditorWindow"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-config-static": dict(
        project="MatrixCommon", path="MatrixCommon/MatrixOptions.cs", unit="MatrixOptions",
        role="user options",
        detail="User-option model behind Matrix custom menus and preferences.",
        observed=["MatrixOptions"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-mod-builder-loader": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modBuilderLoader.bas", unit="modBuilderLoader",
        role="builder panel loading",
        detail="Builder loader module that mounts a builder form into the Matrix panel slot; "
               "launched from frmMaster.tmrBuilderLoader_Timer.",
        observed=["modBuilderLoader", "tmrBuilderLoader_Timer"],
        state="IDENTIFIER_INDEXED"),
    "mx-builder-base": dict(
        project="Matrix (managed plugin)", path="Matrix/MatrixBuilder/MatrixBuilderBase.cs",
        unit="MatrixBuilderBase", role="builder host contract",
        detail="Builder host contract with MatrixBuilderForm, MatrixBuilderCore, "
               "MatrixBuilderCoreDisplayConduit and MatrixGetPoint, plus the concrete builder "
               "families under MatrixBuilder/.",
        observed=["MatrixBuilderBase", "MatrixBuilderForm", "MatrixBuilderCore",
                  "MatrixBuilderCoreDisplayConduit", "MatrixGetPoint", "EBezelPointType",
                  "EUpdateType"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-builder-forms": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmBuilderPattern.frm",
        unit="frmBuilder* family", role="legacy builder panels",
        detail="Legacy builder panels: frmBuilderPattern, frmBuilderMatrixArt (89 declarations), "
               "frmBuilderCustomRail, frmBuilderQuadFlip, frmBuilderObjectChecker, "
               "frmBuilderMeshReducer, frmBuilderRingsizes, frmBuilderCenterLine.",
        observed=["frmBuilderPattern", "frmBuilderMatrixArt", "frmBuilderCustomRail",
                  "frmBuilderQuadFlip", "frmBuilderObjectChecker", "frmBuilderMeshReducer",
                  "frmBuilderRingsizes", "frmBuilderCenterLine"],
        state="IDENTIFIER_INDEXED"),
    "mx-commandline-native": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modVBCoreReplacement.bas",
        unit="modVBCoreReplacement / modLanguage", role="command-line text and prompts",
        detail="VB6 shim that carries command-line text handling and host-call replacement, "
               "together with modLanguage.bas for localized prompt text.",
        observed=["modVBCoreReplacement", "modLanguage"],
        state="IDENTIFIER_INDEXED"),
    "mx-edittext": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/EditText/EditTextForm.cs",
        unit="EditTextForm", role="numeric/text input prompt",
        detail="Managed text/number entry form used by Matrix prompts (EditTextCommand, "
               "EditTextForm).",
        observed=["EditTextCommand", "EditTextForm"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- M04: builder family, ring tools, history, metal weight -------------
    "mx-matrixbuilder": dict(
        project="Matrix (managed plugin)", path="Matrix/MatrixBuilder", unit="MatrixBuilder/* family",
        role="managed builder family",
        detail="Managed builder family under Matrix/MatrixBuilder: CSHalo (HaloBuilder, StylePicker), "
               "CylinderBuilder, FourRailProfile (FourRailProfileBuilder/Command), GemCountOnCurve, "
               "GemListOnCurve, GemOnCurveAdvanced, GemOnCurveMulti, ImageTrace, Panelling "
               "(PanelBuilder), Rope (RopeBuilder), TextureCreator (TextureBuilder), TSplineBezel, "
               "TSplineSignet, TSplineTubeFromCurves, TSplineUnderBezel, UnderGallery - each with a "
               "matching *Command entry point.",
        observed=["HaloBuilder", "StylePicker", "CylinderBuilder", "FourRailProfileBuilder",
                  "GemCountOnCurveBuilder", "GemListOnCurveBuilder", "GemOnCurveAdvancedBuilder",
                  "GemOnCurveMultiBuilder", "ImageTraceBuilder", "PanelBuilder", "RopeBuilder",
                  "TextureBuilder", "TSplineBezelBuilder", "TSplineSignetBuilder",
                  "TSplineTubeFromCurvesBuilder", "TSplineUnderBezelBuilder", "UnderGalleryBuilder"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-builder-ring": dict(
        project="Matrix (managed plugin)", path="Matrix/MatrixBuilder/FourRailProfile/FourRailProfileBuilder.cs",
        unit="FourRailProfileBuilder", role="ring/rail builder implementation",
        detail="Managed ring/rail builder family that constructs profiles between rails and sweeps "
               "them; pairs with the VB6 ring/rail dialogs and modICRail.bas.",
        observed=["FourRailProfileBuilder", "FourRailProfileCommand", "modICRail"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-builder-ring-forms": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmDialogRingRail.frm", unit="frmDialogRingRail",
        role="legacy ring rail dialogs",
        detail="Ring rail/profile dialogs: frmDialogRingRail (caption 'Set Ring Rail', uses "
               "Matrix.ctrRingSize), frmBuilderCustomRail (captions 'Ring Size Reference', "
               "'Ring Width Reference'), frmICBuilderOutsideRingRail ('Outside Ring Rail'), "
               "frmBuilderQuadFlip, frmBuilderRingsizes ('Millimeters').",
        observed=["frmDialogRingRail", "frmBuilderCustomRail", "frmICBuilderOutsideRingRail",
                  "frmBuilderQuadFlip", "frmBuilderRingsizes", "Matrix.ctrRingSize"],
        state="IDENTIFIER_INDEXED"),
    "mx-ring-resizer": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderRingResizer.frm",
        unit="frmICBuilderRingResizer", role="ring resizer builder + mesh settings",
        detail="Ring resizer builder (18 declarations; captions 'Original Ring Size', "
               "'Primary Radius', 'Left Angle', 'Right Angle', 'File Name') with "
               "frmDialogRingResizerMeshSettings ('Ring Resizer Mesh Settings': Minimum Edge Length, "
               "Maximum Edge Length, Maximum Dist. edge to srf, Maximum Aspect Ratio, Maximum Angle, "
               "Minimum Initial Grid Quads), frmDialogRingSizePick ('Select Ring Rail Region Type') "
               "and ctrRingSize/ctrRingSizeList controls.",
        observed=["frmICBuilderRingResizer", "frmDialogRingResizerMeshSettings",
                  "frmDialogRingSizePick", "ctrRingSize", "ctrRingSizeList"],
        state="IDENTIFIER_INDEXED"),
    "mx-builder-profile": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderProfilePlacer.frm",
        unit="frmICBuilderProfilePlacer", role="profile placer / profile editor",
        detail="Profile placer (12 declarations) that places and edits a cross-section profile "
               "along a rail, using Matrix.ctrInBox / ctrInBoxImage / ctrMenu / ctrSliderMenu.",
        observed=["frmICBuilderProfilePlacer", "ctrInBox", "ctrInBoxImage", "ctrMenu",
                  "ctrSliderMenu"],
        state="IDENTIFIER_INDEXED"),
    "mx-builder-ring-family": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderEternity.frm",
        unit="frmICBuilder* family", role="ring builder families",
        detail="Ring builder families: frmICBuilderEternity (repeated stone band, uses "
               "Matrix.ctrGemSelectSmall), frmICBuilderSignet, frmBuilderAwardRingBuilder, "
               "frmBuilderRaisedBand (with modICRaisedBand.bas), frmBuilderJumpRing "
               "(with modJumpRing.bas), frmICBuilderHead, frmICBuilderMetalPiece.",
        observed=["frmICBuilderEternity", "frmICBuilderSignet", "frmBuilderAwardRingBuilder",
                  "frmBuilderRaisedBand", "modICRaisedBand", "frmBuilderJumpRing", "modJumpRing",
                  "frmICBuilderHead", "frmICBuilderMetalPiece", "ctrGemSelectSmall"],
        state="IDENTIFIER_INDEXED"),
    "mx-metal-weight": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/MetalWeight/MetalWeights.cs",
        unit="MetalWeights family", role="metal weight/volume calculation",
        detail="Metal weight family: MetalWeights, MetalWeightsCommand, MetalObject, "
               "MetalMaterialObjectList and MetalWeightsMaterialView, with the VB6 surface "
               "frmCMetalWeights (caption 'Metal Weights') on the sidebar container.",
        observed=["MetalWeights", "MetalWeightsCommand", "MetalObject", "MetalMaterialObjectList",
                  "MetalWeightsMaterialView", "frmCMetalWeights"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-matrix-style": dict(
        project="Matrix (managed plugin)", path="Matrix/MatrixData/MatrixStyle.cs",
        unit="MatrixStyle", role="builder style/preset model",
        detail="Style/preset model with the style library browser "
               "(StylesLibraryBrowserCore/ControlPanel/Editor), the builder StylePicker and the "
               "stylesheet plumbing (IStyleSheet, CSharpStyleSheetCallback, RegisterStyleSheet).",
        observed=["MatrixStyle", "StylePicker", "StylesLibraryBrowserCore",
                  "StylesLibraryBrowserControlPanel", "StylesLibraryBrowserEditor", "IStyleSheet",
                  "CSharpStyleSheetCallback", "RegisterStyleSheet"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-history-commands": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/gvJoinHistory.cs",
        unit="gvJoinHistory", role="history command implementations",
        detail="Managed history commands: gvJoinHistory, GVPurge, IconHistoryCommand "
               "(VB6 counterparts frmMenuIconHistory with CmdIconUndo/CmdIconRedo and "
               "cmdUndoSmall/cmdRedoSmall).",
        observed=["gvJoinHistory", "GVPurge", "IconHistoryCommand", "frmMenuIconHistory",
                  "cmdUndoSmall", "cmdRedoSmall", "CmdIconUndo", "CmdIconRedo"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-history-ui": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuIconHistory.frm",
        unit="frmMenuIconHistory / frmMenuInfoSettings", role="history/undo UI surface",
        detail="History and undo/redo UI surfaces: Icon History sidebar with per-icon right-click "
               "slots, and Menu Info History with a refresh timer for the current selection.",
        observed=["frmMenuIconHistory", "frmMenuInfoSettings", "UpdateHistory", "tmrUpdate_Timer"],
        state="IDENTIFIER_INDEXED"),
    "mx-library-browser": dict(
        project="Matrix (managed plugin)",
        path="Matrix/UserInterface/LibraryBrowser/ILibraryBrowserCore.cs",
        unit="LibraryBrowser family", role="library/style browser core",
        detail="Library browser split into core and control panel: ILibraryBrowserCore, "
               "ILibraryBrowserControlPanel, LibraryBrowserCache/CacheStore, LibraryBrowserItem, "
               "LibraryBrowserProfile and typed sub-browsers (Styles, Model, UserLibraryBrowser, "
               "Bermark, StullerFindings).",
        observed=["ILibraryBrowserCore", "ILibraryBrowserControlPanel", "LibraryBrowserCache",
                  "LibraryBrowserCacheStore", "LibraryBrowserItem", "LibraryBrowserProfile",
                  "StylesLibraryBrowserCore", "StylesLibraryBrowserControlPanel",
                  "UserLibraryBrowserCore", "ModelLibraryBrowserCore"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- M03: geometry utilities, transform, 3dm I-O -------------------------
    "mx-util-geometry": dict(
        project="Matrix (managed plugin)", path="Matrix/Utilities/UtilGeometry.cs",
        unit="UtilGeometry", role="geometry/validation utility layer",
        detail="Shared geometry helpers: UtilGeometry (surface/curve evaluation, validity and "
               "measurement helpers), UtilMath, UtilGem, UtilGemOnCurve, UtilDisplay.",
        observed=["UtilGeometry", "UtilMath", "UtilGem", "UtilGemOnCurve", "UtilDisplay"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-object-helper": dict(
        project="Matrix (managed plugin)", path="Matrix/Utilities/Helpers/DocumentObjectHelper.cs",
        unit="DocumentObjectHelper / MatrixObjectHelper", role="object transform and grouping",
        detail="Object-level helpers used by the transform and utility commands: "
               "DocumentObjectHelper, MatrixObjectHelper, GroupHelper, ViewportPreviewImageHelper.",
        observed=["DocumentObjectHelper", "MatrixObjectHelper", "GroupHelper",
                  "ViewportPreviewImageHelper"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-brep-edge-curves": dict(
        project="Matrix (managed plugin)", path="Matrix/Utilities/BrepEdgeCurves.cs",
        unit="BrepEdgeCurves", role="B-Rep edge extraction",
        detail="Extracts curves from B-Rep edges - the managed counterpart of duplicate edge / "
               "duplicate border / edge curves commands.",
        observed=["BrepEdgeCurves"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-bend-util": dict(
        project="Matrix (managed plugin)", path="Matrix/Utilities/Bend.cs", unit="Bend",
        role="deformation utility",
        detail="Bend deformation helper used by shape-deforming commands; pairs with "
               "modICCurveTransform.bas and frmICBuilderCurveTransform.frm in the VB6 export.",
        observed=["Bend", "modICCurveTransform", "frmICBuilderCurveTransform"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-export-import": dict(
        project="Matrix (managed plugin)", path="Matrix/Utilities/Helpers/ExportHelper.cs",
        unit="ExportHelper", role="3dm/format export-import helper",
        detail="Export helper plus the file/project entry points (modFileSystem 52 declarations, "
               "clsMInterface Import/Export, frmDialogSave, frmMenuProjects) - the Matrix9 path a "
               "3DM import/export task must be compared against.",
        observed=["ExportHelper", "modFileSystem", "frmDialogSave", "Import", "Export"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-toolbar-helper": dict(
        project="Matrix (managed plugin)", path="Matrix/Utilities/RhinoToolbarHelper.cs",
        unit="RhinoToolbarHelper", role="toolbar/menu availability",
        detail="Toolbar helper behind the Matrix top-11 and transform toolbar buttons; "
               "availability is driven with modMenuControls.bas on the VB6 side.",
        observed=["RhinoToolbarHelper", "modMenuControls"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-sel-dup": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/SelDupCommand.cs",
        unit="SelDupCommand", role="duplicate/utility selection command",
        detail="Managed SelDup plus the mesh/health command family (MeshRepairCommand, "
               "ObjectActionsViewCommand) used by the utilities menu and object-health reporting.",
        observed=["SelDupCommand", "MeshRepairCommand", "ObjectActionsViewCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-mesh-helper": dict(
        project="Matrix (managed plugin)", path="Matrix/Utilities/Helpers/NetfabbHelper.cs",
        unit="NetfabbHelper", role="mesh repair/health backend",
        detail="Mesh repair/health backend (NetfabbHelper, NetFabError, MeshRepairCommand) that "
               "backs mesh-from-NURBS, mesh reduce and health reporting; the VB6 surfaces are "
               "frmBuilderMeshReducer and frmBuilderObjectChecker.",
        observed=["NetfabbHelper", "NetFabError", "MeshRepairCommand", "frmBuilderMeshReducer",
                  "frmBuilderObjectChecker"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-measure": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmMenuInfoSettings.frm",
        unit="frmMenuInfoSettings", role="measurement/object info surface",
        detail="Object-info surface with a refresh timer, paired with the Matrix measurement "
               "sheets (distance, length, angle, radius) that document the measure commands.",
        observed=["frmMenuInfoSettings", "tmrUpdate_Timer"],
        state="IDENTIFIER_INDEXED"),
    # -- M05: gem assets, layout and reports --------------------------------
    "mx-gem-util": dict(
        project="Matrix (managed plugin)", path="Matrix/Utilities/GemDefinition.cs",
        unit="GemDefinition / GemInstance / GemBlock", role="gem asset definition and instance model",
        detail="Managed gem asset layer: GemDefinition, GemInstance, GemBlock plus UtilGem, "
               "UtilGemOnCurve and the GemOnCurve/* curve helpers - the Matrix9 model a typed gem "
               "asset/instance contract must be compared against.",
        observed=["GemDefinition", "GemInstance", "GemBlock", "UtilGem", "UtilGemOnCurve",
                  "GemOnCurveCurve", "GemOnCurvePrimaryCurve", "GemOnCurveSecondaryCurve"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-gem-data": dict(
        project="Matrix (managed plugin)", path="Matrix/CS/CSData/CSGemData.cs",
        unit="CSGemData / CSGemStore", role="gem size, cut data and store",
        detail="Gem data and storage: CSGemData, CSChannelGemData(Array), GemSize, CSGemStore, "
               "CSGemInstanceDefinition(Helper), GemOffsetCurve, CSSortGems - i.e. size/cut lookup, "
               "instance definitions, sorting and offset curves.",
        observed=["CSGemData", "CSChannelGemData", "CSChannelGemDataArray", "GemSize", "CSGemStore",
                  "CSGemInstanceDefinition", "CSGemInstanceDefinitionHelper", "GemOffsetCurve",
                  "CSSortGems"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-gem-instance": dict(
        project="Matrix (managed plugin)", path="Matrix/MatrixData/MatrixGemInstance.cs",
        unit="MatrixGemInstance(List) / MatrixGemBlock(List)", role="typed gem instance records",
        detail="Typed gem records carried in document data: MatrixGemInstance, MatrixGemInstanceList, "
               "MatrixGemBlock, MatrixGemBlockList.",
        observed=["MatrixGemInstance", "MatrixGemInstanceList", "MatrixGemBlock", "MatrixGemBlockList"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-gem-on-curve-builders": dict(
        project="Matrix (managed plugin)",
        path="Matrix/MatrixBuilder/GemCountOnCurve/GemCountOnCurveBuilder.cs",
        unit="Gem*OnCurve builders", role="gem-on-curve builder family",
        detail="Managed builders for gem placement along curves: GemCountOnCurve, GemListOnCurve, "
               "GemOnCurveAdvanced, GemOnCurveMulti - each with a matching *Command entry point.",
        observed=["GemCountOnCurveBuilder", "GemCountOnCurveCommand", "GemListOnCurveBuilder",
                  "GemListOnCurveCommand", "GemOnCurveAdvancedBuilder", "GemOnCurveAdvancedCommand",
                  "GemOnCurveMultiBuilder", "GemOnCurveMultiCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-gem-map": dict(
        project="Matrix (managed plugin)", path="Matrix/UserInterface/GemMap.cs", unit="GemMap",
        role="gem map overlay and tolerance",
        detail="Gem map: UserInterface/GemMap.cs, PIM/GemMap.cs, Commands/GemMapCommand.cs, "
               "Commands/gvSetGemMapTolerance.cs and the PIMData GemMap geometry helpers "
               "(GemMapPIM, gemCrv, gXForm, Clipper).",
        observed=["GemMap", "GemMapCommand", "gvSetGemMapTolerance", "GemMapPIM", "gemCrv",
                  "gXForm", "Clipper"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-gem-declared-builders": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderEternity.frm",
        unit="frmICBuilder* gem forms", role="legacy gem builder panels",
        detail="Declared legacy gem builder panels: Baguette, BeadOnSurface, Bezel, BrightCutChannel, "
               "Channel, CustomGem, Emerald, Eternity, GemControl, GemFollow, GemLoader, GemOnCrv, "
               "GemOnSurface, GemReporter, HaloBuilder, Pave, PaveAzure, PaveProng, ClusterBuilder, "
               "with modIC*/modPave* modules carrying the procedures.",
        observed=["frmICBuilderBaguette", "frmICBuilderEmerald", "frmICBuilderCustomGem",
                  "frmICBuilderGemControl", "frmICBuilderGemFollow", "frmICBuilderGemLoader",
                  "frmICBuilderGemOnCrv", "frmICBuilderGemOnSurface", "frmICBuilderGemReporter",
                  "frmCHaloBuilder", "frmICBuilderPave", "frmICBuilderPaveAzure",
                  "frmICBuilderPaveProng", "frmBuilderClusterBuilderOld"],
        state="IDENTIFIER_INDEXED"),
    "mx-gem-reporter": dict(
        project="Matrix (managed plugin)", path="Matrix/Commands/FlipReversedGemsCommand.cs",
        unit="FlipReversedGemsCommand / SelDupCommand", role="gem selection and report commands",
        detail="Gem-oriented commands: FlipReversedGemsCommand, SelDupCommand and the VB6 "
               "frmICBuilderGemReporter surface used for gem counts and weight reporting.",
        observed=["FlipReversedGemsCommand", "SelDupCommand", "frmICBuilderGemReporter"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-pimdata-gem": dict(
        project="PIMData", path="PIMData/GemMap/GemMapPIM.cs", unit="GemMapPIM",
        role="gem map/stone numeric helpers",
        detail="PIMData numeric gem helpers: GemMapPIM, gemCrv, g3DPoint, gXForm, IntPoint/IntRect "
               "with Clipper-based polygon clipping, plus the Stuller stone-location records "
               "(PIMStoneLocations, PIMGemInfo).",
        observed=["GemMapPIM", "gemCrv", "g3DPoint", "gXForm", "IntPoint", "IntRect", "Clipper",
                  "PIMStoneLocations"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-stone-grouping": dict(
        project="MatrixWPFControls", path="MatrixWPFControls/Model/StoneGrouping.cs",
        unit="StoneGrouping / GemInfo", role="stone grouping and gem info records",
        detail="Managed grouping records for stones and gems: StoneGrouping, GemInfo, "
               "ProjectGemInfo, MetalGrouping, StockPartGrouping.",
        observed=["StoneGrouping", "GemInfo", "ProjectGemInfo", "MetalGrouping",
                  "StockPartGrouping"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    # -- M06: head, bezel, prong and bead settings --------------------------
    "mx-head-bezel-builders": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderHead.frm",
        unit="frmICBuilderHead / frmICBuilderBezel", role="legacy head and bezel builder panels",
        detail="Legacy head/bezel builder panels: frmICBuilderHead, frmICBuilderBezel, "
               "frmICBuilderBezelCutter, plus the TSpline variants frmCBezelTSplines and "
               "frmCUnderBezelTSplines and the module modICRaisedBand-style procedures.",
        observed=["frmICBuilderHead", "frmICBuilderBezel", "frmICBuilderBezelCutter",
                  "frmCBezelTSplines", "frmCUnderBezelTSplines"],
        state="IDENTIFIER_INDEXED"),
    "mx-bezel-managed": dict(
        project="Matrix (managed plugin)", path="Matrix/CS/CSBlocks/CSBezel.cs",
        unit="CSBezel / TSplineBezel / UnderGallery", role="managed bezel and gallery builders",
        detail="Managed bezel family: CSBezel, EBezelPointType, TSBezelOffsetPoints, "
               "TSplineBezelBuilder/Command, TSplineUnderBezelBuilder/Command, "
               "UnderGalleryBuilder and GVUnderGalleryCommand.",
        observed=["CSBezel", "EBezelPointType", "TSBezelOffsetPoints", "TSplineBezelBuilder",
                  "TSplineBezelCommand", "TSplineUnderBezelBuilder", "TSplineUnderBezelCommand",
                  "UnderGalleryBuilder", "GVUnderGalleryCommand"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-prong-builders": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderProngEditor.frm",
        unit="frmICBuilderProng* panels", role="legacy prong builder and editor panels",
        detail="Legacy prong panels: frmICBuilderProngEditor, frmICBuilderProngAdder, "
               "frmICBuilderProngOnSurface, frmICBuilderPaveProng, frmICBuilderMicroProngCutter, "
               "frmICBuilderChannel, frmICBuilderBrightCutChannel, frmICBuilderMetalPiece, "
               "with modPaveProng.bas and modIC* modules carrying the procedures.",
        observed=["frmICBuilderProngEditor", "frmICBuilderProngAdder", "frmICBuilderProngOnSurface",
                  "frmICBuilderPaveProng", "frmICBuilderMicroProngCutter", "frmICBuilderChannel",
                  "frmICBuilderBrightCutChannel", "frmICBuilderMetalPiece", "modPaveProng"],
        state="IDENTIFIER_INDEXED"),
    "mx-prong-placer": dict(
        project="Matrix (managed plugin)", path="Matrix/CS/CSUtilProngPlacer.cs",
        unit="CSUtilProngPlacer / CSProngCache / CSProngPlacerGC", role="managed prong placement rules",
        detail="Managed prong placement: CSUtilProngPlacer, CSProngCache, CSProngPlacerGC and "
               "the channel data records CSChannelGemData(Array) plus CSClusterMetalChannel, "
               "and the parametric input contract CSInputChannel.",
        observed=["CSUtilProngPlacer", "CSProngCache", "CSProngPlacerGC", "CSChannelGemData",
                  "CSChannelGemDataArray", "CSClusterMetalChannel", "CSInputChannel"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-bead-milgrain": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderBeadOnSurface.frm",
        unit="frmICBuilderBeadOnSurface", role="bead and milgrain panels",
        detail="Bead/milgrain surface: frmICBuilderBeadOnSurface plus the bead-on-curve and "
               "channel-border entries documented in the M06 sheets.",
        observed=["frmICBuilderBeadOnSurface"],
        state="IDENTIFIER_INDEXED"),
    # -- M07: cutters and manufacturing validation --------------------------
    "mx-cutter-forms": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderCutter.frm",
        unit="frmICBuilderCutter* panels", role="legacy cutter builder panels",
        detail="Legacy cutter panels: frmICBuilderCutter, frmICBuilderCutterLibrary, "
               "frmICBuilderBezelCutter, frmICBuilderMicroProngCutter, frmICBuilderBrightCutter, "
               "frmICBuilderBrightCutChannel, frmBuilderBooleanBuilder.",
        observed=["frmICBuilderCutter", "frmICBuilderCutterLibrary", "frmICBuilderBezelCutter",
                  "frmICBuilderMicroProngCutter", "frmICBuilderBrightCutter",
                  "frmICBuilderBrightCutChannel", "frmBuilderBooleanBuilder"],
        state="IDENTIFIER_INDEXED"),
    "mx-cutter-library": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/modCutterLibrary.bas",
        unit="modCutterLibrary / modGemCutterLibrary / modICFileCutters",
        role="cutter profile library",
        detail="Cutter profile library modules: modCutterLibrary (242 KB), modGemCutterLibrary and "
               "modICFileCutters (save/load of cutter profiles), plus modIC.bas as the shared "
               "cutter builder entry point.",
        observed=["modCutterLibrary", "modGemCutterLibrary", "modICFileCutters", "modIC"],
        state="IDENTIFIER_INDEXED"),
    "mx-boolean-builder": dict(
        project="Matrix (managed plugin)", path="Matrix/MatrixData/MatrixBoolean.cs",
        unit="MatrixBoolean / frmBuilderBooleanBuilder", role="boolean workflow data and panel",
        detail="Boolean workflow: the managed MatrixBoolean data wrapper and the VB6 "
               "frmBuilderBooleanBuilder panel that drives interactive union/difference/"
               "intersection on metal.",
        observed=["MatrixBoolean", "frmBuilderBooleanBuilder"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),
    "mx-cut-to-finger-rail": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmICBuilderRingResizer.frm",
        unit="ring resizer / finger rail tools", role="inside-finger and rail cutting",
        detail="Cut-to-finger-rail family: frmICBuilderRingResizer with "
               "frmDialogRingResizerMeshSettings and frmDialogRingSizePick, plus the managed "
               "FingerHoleArcSupport / FingerHoleSprue support commands.",
        observed=["frmICBuilderRingResizer", "frmDialogRingResizerMeshSettings",
                  "frmDialogRingSizePick", "FingerHoleArcSupport", "FingerHoleSprue"],
        state="IDENTIFIER_INDEXED"),
    "mx-quad-plane-cutters": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/frmBuilderQuadFlip.frm",
        unit="frmBuilderQuadFlip / frmBuilderPlaneCutter", role="plane, cube and quadrant cutters",
        detail="Plane/cube/quadrant cutters: frmBuilderPlaneCutter, frmBuilderQuadFlip, the "
               "cutting-plane entry documented in the M07 sheets and the M03 cutting-plane sheet.",
        observed=["frmBuilderPlaneCutter", "frmBuilderQuadFlip"],
        state="IDENTIFIER_INDEXED"),
    # -- reference export provenance ---------------------------------------
    "mx-export-manifest": dict(
        project="reference export", path="decompiled-ui/export-manifest.json",
        unit="dnSpy export manifest", role="export provenance",
        detail="Records the six exported managed assemblies with size and SHA256 (MatrixControls, "
               "MatrixWPFControls, MatrixCommon, PIMData, MatrixTranslation, Matrix.rhp) and the "
               "dnSpy exit codes; generated sources were not rebuilt.",
        observed=["MatrixControls", "MatrixWPFControls", "MatrixCommon", "PIMData",
                  "MatrixTranslation", "Matrix.rhp"],
        state="RESOURCE_MANIFEST_READ"),
    "mx-vb6-manifest": dict(
        project="reference export", path="vb6-lite/source-manifest.json",
        unit="VB Decompiler Lite manifest", role="export provenance",
        detail="Records Matrix90.exe and MatrixCmdBar.ocx sizes and SHA256 plus the limitation that "
               "form layouts/resources were recovered while native procedure bodies are primarily "
               "disassembly; the export was not compiled.",
        observed=["Matrix90.exe", "MatrixCmdBar.ocx"],
        state="RESOURCE_MANIFEST_READ"),
    "mx-native-inspection": dict(
        project="reference export", path="decompiled-ui/native-inspection",
        unit="PE resource extraction", role="native resource/trademark inventory",
        detail="Per-binary resource manifests and extracted PE resources for Matrix90.exe, "
               "MatrixCmdBar.ocx and gvMatrixCore.rhp (BITMAP/ICON/STRING/DIALOG/CUSTOM entries plus "
               "form-module name hints) - the artifact inventory relevant to asset/trademark and "
               "UI-reference limits.",
        observed=["Matrix90", "MatrixCmdBar", "gvMatrixCore"],
        state="RESOURCE_MANIFEST_READ"),
    "mx-ghidra": dict(
        project="reference export", path="ghidra/Matrix90.exe.c", unit="Ghidra listing",
        role="native behaviour listing",
        detail="Ghidra C listing of Matrix90.exe (about 14.9 MB) kept as a private reference; it is "
               "disassembly-level material and cannot by itself define Matrix9 behaviour.",
        observed=["Matrix90.exe"],
        state="PATH_INDEXED_SYMBOL_CANDIDATE"),    "mx-tool-help": dict(
        project="reference export", path="vb6-lite/tool-help/html", unit="recovered Help topics",
        role="in-product help topics",
        detail="Recovered HTML help topics and images shipped with Matrix9 (with #IDXHDR, #TOPICS, "
               "#STRINGS index files) - topic identity only, contents must be re-read bounded.",
        observed=["tool-help"],
        state="IDENTIFIER_INDEXED"),
    "mx-mkey": dict(
        project="Matrix90 (VB6 EXE)", path="Matrix90/MKey.cls", unit="MKey",
        role="licence/entitlement gate",
        detail="VB6 key class (113 KB) with SentinelKey/SentinelKeysLicense in the managed export - "
               "the entitlement path that OM9 must not reproduce.",
        observed=["MKey", "SentinelKey", "SentinelKeysLicense", "CheckKeyValid"],
        state="IDENTIFIER_INDEXED"),
}

# --------------------------------------------------------------------------
# Rhino 5 geometry / C-ABI anchors.
#
# These are the two ref/rhino5 sources a geometry capability actually needs and
# that the first milestone pass did not cite at all:
#   * the Rhino_DotNet binding layer (RMA.OpenNURBS / RMA.Rhino), which is a real
#     code tree - 898 of 2122 files carry more compute than forwarding;
#   * the Ghidra listings of the native openNURBS / rhcommon_c libraries, plus the
#     RhinoCommon P/Invoke table that names 1166 distinct ON_* entry points.
#
# `observed` records method NAMES that were read out of the file; it is an
# identifier inventory, not a behaviour claim. `state` follows the same
# vocabulary as MX: IDENTIFIER_INDEXED unless something was measured.
# --------------------------------------------------------------------------
RHINO5 = {
    # ---- Rhino_DotNet: openNURBS type surface with real bodies ----
    "rh5-on-brep": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnBrep.cs",
        unit="source file",
        role="trimmed B-rep container: trims, loops, faces, edges, vertices",
        detail="3774 dòng, 151 method. Bề mặt B-rep cấp thấp: AddSurface/AddTrimCurve/AddEdgeCurve, "
               "DeleteFace/DeleteSurface, CullUnused*, CloseTrimGap, SolidOrientation, CreateMesh.",
        observed=["AddSurface", "AddTrimCurve", "AddEdgeCurve", "CloseTrimGap", "DeleteFace",
                  "DeleteSurface", "CullUnusedFaces", "CullUnusedTrims", "CloseTrimGap",
                  "SolidOrientation", "CreateMesh", "AreaMassProperties", "VolumeMassProperties"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-brep-parts": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnBrepFace.cs (+ OnBrepEdge/Trim/Loop/Vertex)",
        unit="source file",
        role="B-rep topology parts: face, edge, trim, loop, vertex",
        detail="OnBrepFace 673 dòng, OnBrepEdge 584, OnBrepTrim 823, OnBrepLoop 548, OnBrepVertex 472. "
               "Đây là bộ phận topology mà một phép trim/join phải cập nhật.",
        observed=["OnBrepFace", "OnBrepEdge", "OnBrepTrim", "OnBrepLoop", "OnBrepVertex"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-curve": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnCurve.cs",
        unit="source file",
        role="curve cơ sở: trim, split, join, domain, closest point",
        detail="2064 dòng. Cặp với OnNurbsCurve cho phần curve cấp thấp.",
        observed=["OnCurve", "ON_JoinCurves", "ON_SortCurves", "ON_TrimCurve",
                  "ON_ForceMatchCurveEnds", "ON_BrepTrimmedPlane"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-nurbs-curve": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnNurbsCurve.cs",
        unit="source file",
        role="NURBS curve: knot vector, control points, degree, evaluation",
        detail="1260 dòng, 59 method. Có tên method cho knot/degree/eval - dùng để ràng buộc "
               "cách dựng và đánh giá NURBS, không phải để chép công thức.",
        observed=["CreateClampedUniformNurbs", "CreatePeriodicUniformNurbs", "IncreaseDegree",
                  "InsertKnot", "Knot", "KnotCount", "KnotMultiplicity",
                  "MakeClampedUniformKnotVector", "MakePeriodicUniformKnotVector", "Transform"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-surface": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnSurface.cs",
        unit="source file",
        role="surface cơ sở: domain, iso, trim, closest point",
        detail="1771 dòng. Cặp với OnNurbsSurface.",
        observed=["OnSurface"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-nurbs-surface": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnNurbsSurface.cs",
        unit="source file",
        role="NURBS surface: knot vector 2 chiều, ruled/cone dựng sẵn, seam",
        detail="1208 dòng, 58 method.",
        observed=["CreateRuledSurface", "CreateConeSurface", "ChangeSurfaceSeam",
                  "IncreaseDegree", "InsertKnot", "Knot", "KnotMultiplicity",
                  "MakeClampedUniformKnotVector", "Transform"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-mesh": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnMesh.cs",
        unit="source file",
        role="mesh: face/vertex/normal, predicate hình học, khối lượng",
        detail="2520 dòng, 91 method. Có predicate tam giác mà lớp hình học gem dùng lại.",
        observed=["ComputeFaceNormal", "ComputeFaceNormals", "ComputeVertexNormals",
                  "ConvertQuadsToTriangles", "ConvertTrianglesToQuads", "CullDegenerateFaces",
                  "AreaCentroid", "VolumeCentroid", "AreaMassProperties", "VolumeMassProperties",
                  "EvaluateMeshGeometry", "GetCurvatureStats"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-mass-properties": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnMassProperties.cs",
        unit="source file",
        role="khối lượng/diện tích/thể tích và moment quán tính",
        detail="1759 dòng, 27 method. Nguồn cho tính khối lượng kim loại và trọng lượng đá.",
        observed=["Area", "Volume", "Centroid", "CentroidCoordIntertiaMatrix",
                  "CentroidCoordMomentsOfInertia", "CentroidCoordPrincipalMoments",
                  "CentroidCoordRadiiOfGyration", "CentroidCoordSecondMoments"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-xform": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnXform.cs",
        unit="source file",
        role="transform 4x4 và các phép dựng transform",
        detail="1340 dòng. Nguồn cho đặt đá theo khung (frame) và biến đổi đối xứng gương.",
        observed=["ReadXform", "WriteXform", "Transform", "ON_GetCageXform", "UnitScale"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-geometry-math": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnPlane.cs (+ On3dPoint/On3dVector/OnInterval)",
        unit="source file",
        role="kiểu toán cơ sở: plane, point, vector, interval",
        detail="OnPlane 921 dòng, On3dPoint 967, On3dVector 992, OnInterval 703.",
        observed=["OnPlane", "On3dPoint", "On3dVector", "OnInterval"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-binary-archive": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnBinaryArchive.cs",
        unit="source file",
        role="archive nhị phân 3dm: chunk read/write, version, bảng đối tượng",
        detail="4512 dòng, 204 method. Bề mặt đọc/ghi chunk của định dạng 3dm - nguồn trực tiếp "
               "cho L3 .3dm import.",
        observed=["BeginRead3dmChunk", "BeginRead3dmBitmapTable", "BeginRead3dmGroupTable",
                  "BeginRead3dmDimStyleTable", "Archive3dmVersion", "ArchiveOpenNURBSVersion",
                  "ArchiveStartOffset", "ReadXform", "WriteXform",
                  "EnableSave3dmRenderMeshes", "Save3dmAnalysisMeshes"],
        state="IDENTIFIER_INDEXED"),
    "rh5-on-util": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.OpenNURBS)",
        path="Rhino_DotNet/RMA/OpenNURBS/OnUtil.cs",
        unit="source file",
        role="tiện ích hình học: dựng brep cơ sở, nội suy NURBS, predicate giao",
        detail="3800 dòng, 209 method.",
        observed=["ON_BrepBox", "ON_BrepCylinder", "ON_BrepCone", "ON_BrepExtrude",
                  "ON_BrepFromMesh", "ON_BrepRevSurface", "ON_BrepTrimmedPlane",
                  "ON_IntersectLineLine", "ON_LineTriangleIntersect", "ON_MeshMeshIntersect",
                  "ON_EvaluateNurbsBasis", "ON_EvaluateBezier", "ON_ConvertNurbSpanToBezier",
                  "ON_EvCurvature", "ON_EvPrincipalCurvatures", "ON_ClosestPointToTriangle",
                  "ON_ClosestPointToTriangleFast", "ON_JoinCurves", "ON_TrimCurve",
                  "ON_WriteOneObjectArchive"],
        state="IDENTIFIER_INDEXED"),
    "rh5-rh-util": dict(
        root="ref/rhino5/dnspy/Rhino_DotNet",
        project="Rhino 5 Rhino_DotNet (RMA.Rhino)",
        path="Rhino_DotNet/RMA/Rhino/RhUtil.cs",
        unit="source file",
        role="thư viện tiện ích hình học lớn nhất trong ref/: boolean vùng phẳng 2D, boolean brep/mesh/curve, "
             "heal brep, giao surface/brep, chia curve",
        detail="9482 dòng, 309 method (201 tên phân biệt đọc được). Đây là chỗ gần nhất với thao tác "
               "mà tập lệnh gem cần (outline, profile, pave, giao mặt) nhưng vẫn là tiện ích tổng quát "
               "của Rhino, KHÔNG chứa thuật toán domain gem/prong/pave.",
        observed=["RhPlanarRegionUnion", "RhPlanarRegionDifference", "RhPlanarRegionIntersection",
                  "RhPlanarRegionBoolean", "RhinoBooleanUnion", "RhinoBooleanDifference",
                  "RhinoBooleanIntersection", "RhinoCurveBooleanUnion", "RhinoMeshBooleanIntersection",
                  "RhinoBrepCapPlanarHoles", "RhinoBrepClosestPoint", "RhinoBrepSplit",
                  "RhinoBrepGet2dSection", "RhinoBrepGet2dProjection", "RhinoJoinBreps",
                  "RhinoJoinBrepNakedEdges", "RhinoIntersectBreps", "RhinoIntersectSurfaces",
                  "RhinoCurveBrepIntersect", "RhinoCurveFaceIntersect", "RhinoDivideCurve",
                  "RhinoDivideCurveEquidistant", "RhinoConvertCurveToPolyline",
                  "RhinoCreateSurfaceFromCorners", "RhinoCutUpSurface", "RhinoFitSurface",
                  "RhinoExtendSurface", "RhinoGetBrepFaceIsoCurves", "RhinoGetAnalysisMeshes",
                  "RhinoReadFile", "RhinoWriteFile"],
        state="IDENTIFIER_INDEXED"),
    # ---- RhinoCommon: managed API shape + the P/Invoke table ----
    "rh5-common-brep": dict(
        root="ref/rhino5/dnspy/RhinoCommon",
        project="Rhino 5 RhinoCommon",
        path="RhinoCommon/Geometry/Brep.cs",
        unit="source file",
        role="bề mặt B-rep managed mà OM9 phải khớp tên/hình dạng",
        detail="2305 dòng. Dùng để đối chiếu API, không phải để lấy thuật toán.",
        observed=["Brep"],
        state="IDENTIFIER_INDEXED"),
    "rh5-common-nurbs-curve": dict(
        root="ref/rhino5/dnspy/RhinoCommon",
        project="Rhino 5 RhinoCommon",
        path="RhinoCommon/Geometry/NurbsCurve.cs",
        unit="source file",
        role="NURBS curve managed: dựng từ line/arc, knot, control point",
        detail="607 dòng.",
        observed=["CreateFromLine", "CreateFromArc"],
        state="IDENTIFIER_INDEXED"),
    "rh5-common-file3dm": dict(
        root="ref/rhino5/dnspy/RhinoCommon",
        project="Rhino 5 RhinoCommon",
        path="RhinoCommon/FileIO/File3dm.cs",
        unit="source file",
        role="hợp đồng đọc/ghi file 3dm cấp managed",
        detail="1188 dòng, 18 method.",
        observed=["Read", "ReadWithLog", "Write", "WriteWithLog", "ReadArchiveVersion",
                  "ReadApplicationData", "ReadNotes", "ReadPreviewImage", "ReadRevisionHistory"],
        state="IDENTIFIER_INDEXED"),
    "rh5-common-units": dict(
        root="ref/rhino5/dnspy/RhinoCommon",
        project="Rhino 5 RhinoCommon",
        path="RhinoCommon/UnitSystem.cs",
        unit="source file",
        role="enum hệ đơn vị - nguồn cho ánh xạ đơn vị/dung sai .3dm",
        detail="115 dòng.",
        observed=["UnitSystem"],
        state="IDENTIFIER_INDEXED"),
    "rh5-common-unsafe-native": dict(
        root="ref/rhino5/dnspy/RhinoCommon",
        project="Rhino 5 RhinoCommon",
        path="RhinoCommon/UnsafeNativeMethods.cs",
        unit="source file",
        role="BẢNG P/INVOKE: 3332 khai báo DllImport, 1166 tên ON_* phân biệt, "
             "tên method managed chính là symbol native (không có EntryPoint=)",
        detail="14868 dòng / 792889 byte. Đây là C ABI mà lớp FFI openNURBS của OM9 phải khớp. "
               "Đếm theo họ: ON_Brep 129, ON_Mesh 103, ONX_Model 90, ON_Viewport 71, "
               "ON_BinaryArchive 62, ON_Curve 41, ON_Surface 41, ON_3dmObjectAttributes 26, "
               "ON_NurbsCurve 23, ON_NurbsSurface 19, ON_Layer 16, ON_InstanceDefinition 6. "
               "ON_HistoryRecord = 0 và ON_Transform = 0 trong bảng này.",
        observed=["ON_NurbsCurve_Create", "ON_NurbsCurve_CreateClampedUniformNurbs",
                  "ON_NurbsSurface_CreateRuledSurface", "ON_NurbsSurface_GetCV",
                  "ON_Brep_AddTrimCurve", "ON_Brep_AddEdgeCurve", "ON_Brep_Area",
                  "ON_Mesh_AddFace", "ON_Mesh_BrepToMeshSimple", "ONX_Model_Audit",
                  "ONX_Model_3dmSettingsPointer", "ON_3dmObjectAttributes_GetSetColor",
                  "ON_Layer_GetLayerName", "ON_InstanceDefinition_GetObjectIds",
                  "ON_BinaryArchive_Read3dmChunkVersion", "ON_2dPointArray_CopyValues"],
        state="COUNTED_AND_INDEXED"),
    # ---- Ghidra listings of the native libraries ----
    "rh5-ghidra-opennurbs": dict(
        root="ref/rhino5/ghidra",
        project="Rhino 5 ghidra export",
        path="openNURBSx64.dll.c",
        unit="decompiler listing",
        role="C ABI native của openNURBS: tên hàm + chữ ký kiểu tham số",
        detail="600651 dòng / 19.6 MB, 730 tên ON_* phân biệt, 6081 định nghĩa method C++ đã demangle. "
               "CẢNH BÁO: struct là placeholder (175 marker 'PlaceHolder Class Structure', 49633 lần "
               "dùng undefinedN) nên KHÔNG thu hồi được layout member / offset.",
        observed=["ON_Brep_Area", "ON_CRC32", "ON_DebugWriteObject",
                  "ON_BrepVertex::CopyON_BrepVertex", "RhinoMergeBrepCoplanarFaces"],
        state="COUNTED_AND_INDEXED"),
    "rh5-ghidra-rhcommon": dict(
        root="ref/rhino5/ghidra",
        project="Rhino 5 ghidra export",
        path="rhcommon_c.dll.c (+ rhcommon_c.dll.h)",
        unit="decompiler listing",
        role="tầng C API mà RhinoCommon gọi xuống: nhiều tên ON_* nhất",
        detail="123145 dòng / 3.4 MB, 1367 tên ON_*; header 9160 dòng với 1277 tên và 775 typedef "
               "nhưng 554 struct là placeholder. Đây là tầng FFI thật, không phải openNURBS thuần.",
        observed=["ON_* family (1367 names indexed)"],
        state="COUNTED_AND_INDEXED"),
}

# --------------------------------------------------------------------------
# L3 -> anchors
#   "rcore": [chapter ids]
#   "mx": {source id: role within this L3}
#   "focus": one line stating what this L3 must verify against the anchors
# --------------------------------------------------------------------------
L3_ANCHORS = {
    # ================= M00 =================
    "OM9-M00-L3-01-01": dict(
        rcore=["RCORE-10"],
        mx={"mx-export-manifest": "provenance format to mirror for the QymCAD pin record",
            "mx-vb6-manifest": "second provenance example (two binaries, hash + limitation note)"},
        focus="Ghi provenance nguồn QymCAD theo cùng chuẩn manifest đã dùng cho Matrix9 (owner, revision, sha256, công cụ, giới hạn đã biết)."),
    "OM9-M00-L3-01-02": dict(
        mx={"mx-export-manifest": "workspace/project inventory precedent",
            "mx-native-inspection": "per-binary resource inventory precedent"},
        focus="Đối chiếu bản đồ workspace 19 crate với cách manifest Matrix9 liệt kê từng assembly và resource."),
    "OM9-M00-L3-01-03": dict(
        mx={"mx-com-interface": "REUSE/ADAPT/ADD precedent: managed interface over native code",
            "mx-clsminterface": "second precedent: VB6 facade kept behind the managed layer",
            "mx-mod-vb-core": "example of glue that must be re-authored, not ported"},
        focus="Dùng quan hệ .NET-Matrix / COMInterface / clsMInterface / modVBCoreReplacement làm ví dụ cho quyết định REUSE/ADAPT/ADD và cho phần glue phải viết lại."),
    "OM9-M00-L3-02-01": dict(
        rcore=["RCORE-10"],
        mx={"mx-mod-rhino-controls": "native host-call layer that OM9 replaces with an OCCT FFI bridge"},
        focus="Chốt ABI/ownership cho bridge OCCT; ghi rõ lớp host-call kiểu modRhinoControls không được tái tạo."),
    "OM9-M00-L3-02-02": dict(
        rcore=["RCORE-07"],
        mx={"mx-mod-rhino-controls": "host-call lifetime boundary precedent"},
        focus="Chứng minh lifetime/threading/error của FFI; không suy NURBS semantics từ lớp host-call."),
    "OM9-M00-L3-02-03": dict(
        rcore=["RCORE-01", "RCORE-02", "RCORE-07"],
        mx={"mx-matrixdata": "typed numeric/geometry wrappers whose units semantics must be re-proved",
            "mx-matrixdata-base": "EDataUsage-style usage typing precedent"},
        focus="Fixture số học phải ghi đơn vị/tolerance rõ; typed wrapper kiểu MatrixData không chứng minh hình học đúng."),
    "OM9-M00-L3-03-01": dict(
        rcore=["RCORE-12", "RCORE-04"],
        mx={"mx-frm-master": "source shell composition (command bar + panel + viewport host)",
            "mx-frm-cpanel": "dockable panel container reused by all sidebar sections",
            "mx-cmdbar": "command bar control family",
            "mx-matrixcontrols": "authored control family that defines the Matrix look",
            "mx-wpf-resources": "managed theme resources",
            "mx-frm-cmenu-display": "example sidebar section shell"},
        focus="Ghi rõ kiến trúc UI nguồn (.NET Matrix + VB6 Matrix90 + MatrixCmdBar OCCT-free UI shell) và giới hạn: chỉ dùng để tái tạo hành vi, không copy control/resource."),
    "OM9-M00-L3-03-02": dict(
        rcore=["RCORE-12"],
        mx={"mx-display-net": "display/shade mode pipeline precedent",
            "mx-frm-dialog-vga": "graphics-adapter diagnostic surface precedent",
            "mx-frm-dialog-shade-select": "shade mode chooser precedent"},
        focus="Đo FPS/first-frame/memory và device-loss; đối chiếu cách Matrix9 báo lỗi adapter nhưng không lấy số liệu từ nó."),
    "OM9-M00-L3-03-03": dict(
        rcore=["RCORE-05", "RCORE-06"],
        mx={"mx-mouse-callback": "pointer routing precedent",
            "mx-tube-handles": "Matrix handle implementation precedent",
            "mx-ctrl-inbox": "in-box input controls for pick/preview"},
        focus="Chứng minh hit test + transactional selection; handle/pick contract lấy từ RCORE-05/06, không suy từ tên class Matrix."),
    "OM9-M00-L3-04-01": dict(
        rcore=["RCORE-10"],
        mx={"mx-mod-file-system": "file/project I-O surface precedent",
            "mx-frm-dialog-save": "save dialog with format choices"},
        focus="Chứng minh khoảng trống 3DM/openNURBS; ghi rõ Matrix9 dùng host Rhino cho 3DM, không phải bằng chứng OM9 có codec."),
    "OM9-M00-L3-04-02": dict(
        rcore=["RCORE-01", "RCORE-02", "RCORE-10"],
        mx={"mx-matrixdata": "typed object wrappers to map onto OM9 typed entities"},
        focus="Proof NURBS/trimmed BRep/units phải đo bằng fixture; wrapper kiểu MatrixData chỉ là hợp đồng kiểu."),
    "OM9-M00-L3-04-03": dict(
        rcore=["RCORE-10"],
        mx={"mx-frm-dialog-save": "failure/partial-write surface precedent",
            "mx-frm-debuglog": "diagnostic reporting precedent"},
        focus="Thiết kế báo cáo lỗi/partial import; đối chiếu cách Matrix9 hiện lỗi nhưng không dùng làm bằng chứng runtime."),
    "OM9-M00-L3-05-01": dict(
        rcore=["RCORE-09"],
        mx={"mx-mod-project-manager": "persistence/identity backend precedent"},
        focus="Kiến trúc naming/rebind lấy từ RCORE-09 + QymCAD kernel, không suy từ tên hàm Matrix."),
    "OM9-M00-L3-05-02": dict(
        rcore=["RCORE-09", "RCORE-04"],
        mx={"mx-mod-project-manager": "regeneration/persistence backend precedent",
            "mx-frm-project-manager": "UI surface for stored project state"},
        focus="Stress regeneration/persistence phải có log thật; UI Matrix chỉ cho biết bề mặt thao tác."),
    "OM9-M00-L3-06-01": dict(
        mx={"mx-export-manifest": "assembly hash inventory used for the reference inventory",
            "mx-vb6-manifest": "second hash inventory",
            "mx-native-inspection": "resource inventory",
            "mx-mkey": "entitlement/licence path that must not be reproduced"},
        focus="Quyết định license phải tách rõ nguồn public (QymCAD/OCCT/Rhino5 guide) và nguồn private (Matrix9 export, Ghidra, manual) - bằng chứng: manifest hash hiện có."),
    "OM9-M00-L3-06-02": dict(
        mx={"mx-native-inspection": "BITMAP/ICON/CUSTOM resource inventory for trademark review",
            "mx-export-manifest": "assembly inventory for notices/SBOM scope",
            "mx-tool-help": "in-product help topics whose text is copyrighted"},
        focus="Phân loại asset/thương hiệu trước khi phát hành; ref/matrix là nguồn private, không được đưa vào repo public."),
    "OM9-M00-L3-07-01": dict(
        rcore=["RCORE-02"],
        mx={"mx-frm-dialog-vga": "adapter/diagnostic surface to reproduce on Windows baseline"},
        focus="Baseline Windows phải ghi toolchain/OCCT thật; không lấy cấu hình từ Matrix9 đã cài."),
    "OM9-M00-L3-07-02": dict(
        rcore=["RCORE-02"],
        focus="Linux/OCCT packaged baseline chỉ được ghi khi có môi trường thật; không suy từ tài liệu Rhino 5."),
    "OM9-M00-L3-07-03": dict(
        mx={"mx-export-manifest": "artifact/hash capture precedent for gate handoff"},
        focus="Gate handoff phải có artifact hash + lệnh thật; manifest Matrix9 là ví dụ định dạng, không phải evidence."),

    # ================= M01: L2-01-01 shell/docking =================
    "OM9-L3-01-01-001": dict(
        rcore=["RCORE-12", "RCORE-04"],
        mx={"mx-frm-master": "shell bootstrap: command bar + host timers + window caption",
            "mx-cmdbar": "command bar docked control",
            "mx-mod-windows-messaging": "window-message dispatch between shell and host"},
        focus="Cửa sổ chính phải có command bar + host hook tương đương frmMaster/vbalCommandBar; caption 'Matrix - Untitled.3dm' là bằng chứng bề mặt, không phải contract."),
    "OM9-L3-01-01-002": dict(
        rcore=["RCORE-12"],
        mx={"mx-frm-cpanel": "dockable panel container",
            "mx-frm-master": "shell that hosts docked sections",
            "mx-cmdbar": "docked command bar behaviour"},
        focus="Vùng dock trái/giữa/phải phải chứa được các section dạng frmCPanel; ghi rõ container không tự sở hữu business logic."),
    "OM9-L3-01-01-003": dict(
        mx={"mx-frm-cpanel": "panel container with title bar affordances",
            "mx-cmdbar": "float/dock behaviour of the command bar"},
        focus="Reorder/collapse/float của panel phải giữ identity section; không tái tạo cơ chế float của vbalCommandBar."),
    "OM9-L3-01-01-004": dict(
        mx={"mx-frm-master": "original window composition to compare the default layout against",
            "mx-frm-cpanel": "default panel placement"},
        focus="Default layout phải là quyết định OM9 có thể tái lập; thứ tự section lấy từ danh sách form sidebar đã đọc."),
    "OM9-L3-01-01-005": dict(
        mx={"mx-config": "persisted user configuration precedent",
            "mx-frm-master": "window geometry/state owner"},
        focus="Persistence phải tách vị trí panel khỏi document state; config Matrix9 chỉ là tiền lệ định dạng."),
    "OM9-L3-01-01-006": dict(
        rcore=["RCORE-10"],
        mx={"mx-frm-master": "document window caption and title state",
            "mx-frm-menu-icon-history": "undo/redo state surfaced per window"},
        focus="Title/current document/unsaved marker phải theo document thật; caption Matrix9 là ví dụ, không phải định dạng bắt buộc."),
    "OM9-L3-01-01-007": dict(
        rcore=["RCORE-04"],
        mx={"mx-frm-messages": "Matrix message surface",
            "mx-frm-debuglog": "diagnostic log surface",
            "mx-mod-menu-controls": "availability state shown to the user"},
        focus="Lỗi không nghiêm trọng và trạng thái lệnh phải hiện rõ; đối chiếu frmMessages/frmDebugLog nhưng không copy nội dung."),
    "OM9-L3-01-01-008": dict(
        mx={"mx-mod-builder-loader": "builder mounting into a panel slot",
            "mx-builder-base": "builder host contract",
            "mx-frm-master": "host timer that loads the builder panel"},
        focus="Slot builder nằm dưới Project Manager; liên kết tới hợp đồng builder, không nhân đôi builder implementation."),

    # ================= M01: L2-01-02 command engine =================
    "OM9-L3-01-02-001": dict(
        rcore=["RCORE-04"],
        mx={"mx-mod-special-commands": "Matrix command entry points/names",
            "mx-mod-menu-controls": "menu/alias availability",
            "mx-main-command": "managed main-menu command"},
        focus="Registry lệnh/alias phải giữ tên lệnh Matrix first; danh sách tên chỉ được lấy từ identifier đã index."),
    "OM9-L3-01-02-002": dict(
        rcore=["RCORE-04"],
        mx={"mx-mod-special-commands": "command descriptor surface",
            "mx-frm-dialog-options": "option dialog vocabulary (captions only)"},
        focus="Descriptor phải typed và có prompt/option; caption dialog chỉ dùng làm từ vựng, không suy default."),
    "OM9-L3-01-02-003": dict(
        rcore=["RCORE-04", "RCORE-06"],
        mx={"mx-object-actions": "selection context used by the sidebar",
            "mx-clsminterface": "document/selection facade (SelectedObjects/CurrentLayer)"},
        focus="Context (selection/active viewport/document) phải typed; clsMInterface cho biết bề mặt thao tác VB6 cũ."),
    "OM9-L3-01-02-004": dict(
        rcore=["RCORE-04"],
        mx={"mx-edittext": "point/value prompt surface",
            "mx-mod-windows-messaging": "host message loop controlling Enter/Escape phases"},
        focus="State machine phải xử lý Enter/Escape theo phase; đối chiếu RCORE-04 phase model, không suy từ VB6."),
    "OM9-L3-01-02-005": dict(
        rcore=["RCORE-04"],
        mx={"mx-frm-dialog-options": "option token vocabulary",
            "mx-ctrl-inbox": "in-box option/input controls"},
        focus="Parser flag/số phải có lỗi typed; option token lấy từ tên lệnh/option thật, không bịa."),
    "OM9-L3-01-02-006": dict(
        rcore=["RCORE-04", "RCORE-09"],
        mx={"mx-frm-menu-icon-history": "undo/redo journal surface"},
        focus="Một thao tác = một transaction; journal phải reversible và không phụ thuộc UI state."),
    "OM9-L3-01-02-007": dict(
        rcore=["RCORE-04", "RCORE-09"],
        mx={"mx-frm-menu-icon-history": "cmdUndoSmall/cmdRedoSmall/CmdIconUndo/CmdIconRedo surface"},
        focus="Undo/Redo stack phải có test đối xứng; Icon History chỉ là bề mặt người dùng."),
    "OM9-L3-01-02-008": dict(
        rcore=["RCORE-04"],
        mx={"mx-frm-menu-icon-history": "cancel/rollback visible state",
            "mx-frm-messages": "cancel reporting surface"},
        focus="Cancel phải rollback preview và không để lại object rác; ghi rõ khác biệt Undo document."),
    "OM9-L3-01-02-009": dict(
        rcore=["RCORE-04"],
        mx={"mx-mod-vb-core": "command repeat/space behaviour precedent",
            "mx-mod-special-commands": "repeatable command surface"},
        focus="Repeat phải dựa trên command history; whitelist repeat không đóng toàn command engine."),
    "OM9-L3-01-02-010": dict(
        rcore=["RCORE-04"],
        mx={"mx-frm-messages": "progress/warning/completion surface",
            "mx-object-actions": "selection observer API precedent"},
        focus="Event/observer API phải typed và không tạo vòng lặp cập nhật; UI chỉ subscribe."),
    "OM9-L3-01-02-011": dict(
        rcore=["RCORE-04", "RCORE-06"],
        mx={"mx-mod-menu-controls": "availability/greyed state logic",
            "mx-main-command": "menu command gating"},
        focus="Action không khả dụng phải hiện disabled, không im lặng; precondition phải test được."),

    # ================= M01: L2-01-03 document/entities =================
    "OM9-L3-01-03-001": dict(
        rcore=["RCORE-01", "RCORE-09"],
        mx={"mx-matrixdata-base": "identity/usage envelope precedent",
            "mx-mod-project-manager": "persisted identity precedent"},
        focus="EntityId/FeatureId phải ổn định qua save/load và rebuild; không dùng index UI làm ID."),
    "OM9-L3-01-03-002": dict(
        rcore=["RCORE-01"],
        mx={"mx-matrixdata": "typed object envelope per Rhino-like type"},
        focus="Envelope entity phải typed (Point/Curve/Surface/BRep/Mesh/Instance) và không trộn display data."),
    "OM9-L3-01-03-003": dict(
        rcore=["RCORE-01", "RCORE-12"],
        mx={"mx-matrixdata": "geometry + attribute carrier",
            "mx-config": "display attribute source"},
        focus="Handle + attribute phải tách geometry truth khỏi display/metadata."),
    "OM9-L3-01-03-004": dict(
        rcore=["RCORE-06"],
        mx={"mx-frm-menu-layers": "layer record fields observed (name, show/hide, rename, lock, colour rows)",
            "mx-cmd-layers": "layers command"},
        focus="Layer record phải giữ ID/label/colour/visibility/lock; palette quan sát chỉ xác nhận bề mặt."),
    "OM9-L3-01-03-005": dict(
        rcore=["RCORE-06"],
        mx={"mx-clsminterface": "objectlayer/CurrentLayer assignment facade",
            "mx-frm-menu-layers": "assignment UI surface"},
        focus="Gán object vào layer phải qua một owner; không tự tạo layer ngầm khi assign."),
    "OM9-L3-01-03-006": dict(
        rcore=["RCORE-06", "RCORE-05"],
        mx={"mx-frm-menu-layers": "hide/lock controls (PicShow/PicHide, lock rows)",
            "mx-layer-render": "layer render presets"},
        focus="Hide/lock là input cho selection và display; locked-visible vẫn snap theo policy RCORE-05/06."),
    "OM9-L3-01-03-007": dict(
        rcore=["RCORE-06"],
        mx={"mx-object-actions": "object action/selection UI precedent"},
        focus="Group membership không merge geometric identity; ghi rõ quan hệ với block."),
    "OM9-L3-01-03-008": dict(
        rcore=["RCORE-06", "RCORE-10"],
        mx={"mx-ghidra": "native listing only if a block path must be checked (disassembly, not spec)"},
        focus="Definition/instance/nesting phải theo RCORE-06/10 và fixture 3DM; không suy từ Matrix block UI."),
    "OM9-L3-01-03-009": dict(
        rcore=["RCORE-01"],
        mx={"mx-userdata": "jewelry userdata carriers",
            "mx-matrix-userdata": "userdata interface",
            "mx-gemblock": "gem block/instance model",
            "mx-gemmap": "gem map model"},
        focus="Metadata slot cho gem/rail/head/cutter phải typed và persist; không copy userdata của Matrix9."),
    "OM9-L3-01-03-010": dict(
        rcore=["RCORE-06"],
        mx={"mx-object-actions": "selection observation precedent",
            "mx-clsminterface": "SelectedObjects/SelectObject surface"},
        focus="Selection contract phải typed + stable ID; document event payload không chứa handle tạm."),
    "OM9-L3-01-03-011": dict(
        rcore=["RCORE-04", "RCORE-09"],
        mx={"mx-object-actions": "object change notification precedent"},
        focus="Notification phải observable và không churn; kiểm cả trường hợp no-op."),
    "OM9-L3-01-03-012": dict(
        rcore=["RCORE-10", "RCORE-09"],
        mx={"mx-mod-project-manager": "project/jobbag snapshot backend precedent",
            "mx-wpf-project-manager": "job bag snapshot UI"},
        focus="Snapshot whole-scene vs selection phải khác nhau rõ; job bag chỉ là ví dụ bề mặt."),
    "OM9-L3-01-03-013": dict(
        rcore=["RCORE-10"],
        mx={"mx-mod-file-system": "import/source path handling precedent"},
        focus="Ghi provenance nguồn .3dm (tên, ID, phần bỏ qua); không biến tên Matrix thành ID ổn định."),

    # ================= M01: L2-01-05 OCCT bridge =================
    "OM9-L3-01-05-001": dict(rcore=["RCORE-01"], mx={"mx-mod-rhino-controls": "host-call lifetime boundary to replace with OCCT FFI ownership"},
                             focus="RAII/ownership của handle OCCT phải có test tạo-hủy và leak."),
    "OM9-L3-01-05-002": dict(rcore=["RCORE-01", "RCORE-08"],
                             mx={"mx-matrixdata-base": "typed envelope with usage flags (EDataUsage) as adjacency"},
                             focus="Shape type/validity/null phải trả lỗi typed; đối chiếu RCORE-08 cho diagnostic."),
    "OM9-L3-01-05-003": dict(rcore=["RCORE-07"], mx={"mx-matrixdata": "curve wrapper precedent (MatrixCurve)"},
                             focus="Evaluation curve phải kèm derivative/tolerance và fixture knot/weight."),
    "OM9-L3-01-05-004": dict(rcore=["RCORE-07"], mx={"mx-matrixdata": "surface wrapper precedent (MatrixSurface)"},
                             focus="Evaluation surface phải ghi UV domain/seam/normal; periodic surface cần fixture riêng."),
    "OM9-L3-01-05-005": dict(rcore=["RCORE-01", "RCORE-06"],
                             mx={"mx-clsminterface": "object/selection facade that must not own topology"},
                             focus="Face/wire/edge topology phải truy cập được không qua chỉ số UI; sub-object selection dùng ID."),
    "OM9-L3-01-05-006": dict(rcore=["RCORE-07"], mx={"mx-matrixdata": "Point wrapper used by Matrix transforms"},
                             focus="Transform BRep phải giữ validity và không mutate shape dùng chung."),
    "OM9-L3-01-05-007": dict(rcore=["RCORE-07", "RCORE-05"], mx={"mx-gemmap": "2D clipping/intersection helper (Clipper) as adjacency only"},
                             focus="Intersection phải phân biệt true/apparent và ghi tolerance (RCORE-05.T04)."),
    "OM9-L3-01-05-008": dict(rcore=["RCORE-12", "RCORE-01"],
                             mx={"mx-display-net": "display-mesh density precedent",
                                 "mx-layout-tools": "mesh/3DM XML info precedent (XML3DMInfo)"},
                             focus="Tessellation là display cache; BRep vẫn là source of truth."),
    "OM9-L3-01-05-009": dict(rcore=["RCORE-08"], mx={"mx-frm-debuglog": "diagnostic reporting precedent"},
                             focus="Error contract phải nêu failed shape/tolerance/warning; không báo pass khi shape rỗng."),
    "OM9-L3-01-05-010": dict(rcore=["RCORE-04"],
                             mx={"mx-mod-rhino-controls": "host-call boundary to avoid reproducing",
                                 "mx-mod-windows-messaging": "message/thread handoff precedent"},
                             focus="FFI isolation/cancellation phải test được; không gọi OCCT từ UI thread tùy tiện."),
    "OM9-L3-01-05-011": dict(rcore=["RCORE-01", "RCORE-02"],
                             mx={"mx-matrixdata": "typed reference geometry wrappers for fixture comparison"},
                             focus="Fixture surface/closed-solid phải chốt đơn vị và tolerance trước khi đo."),

    # ================= M01: L2-01-06 GPU =================
    "OM9-L3-01-06-001": dict(
        rcore=["RCORE-12"],
        mx={"mx-frm-dialog-vga": "adapter failure diagnostic surface",
            "mx-display-net": "display pipeline owner"},
        focus="Init backend + frame loop phải có fallback chẩn đoán; đối chiếu cách Matrix9 báo lỗi adapter."),
    "OM9-L3-01-06-002": dict(rcore=["RCORE-12"],
                             mx={"mx-display-net": "scene display resource owner",
                                 "mx-layer-render": "layer render/preset resource precedent"},
                             focus="GPU resource phải bind theo scene object/version và giải phóng khi object xoá."),
    "OM9-L3-01-06-003": dict(rcore=["RCORE-12", "RCORE-07"],
                             mx={"mx-display-net": "MeshDensityForm/MeshDensityEventWatcher density control"},
                             focus="Cache tessellation theo object/version; density change không đổi BRep."),
    "OM9-L3-01-06-004": dict(rcore=["RCORE-07", "RCORE-12"],
                             mx={"mx-display-net": "wireframe/isocurve pass predecessor",
                                 "mx-layer-render": "layer visibility/render preset interaction"},
                             focus="Wireframe/isocurve pass phải phân biệt edge thật và isocurve hiển thị."),
    "OM9-L3-01-06-005": dict(rcore=["RCORE-12"],
                             mx={"mx-display-net": "ShadeModeDisplayConduit precedent",
                                 "mx-frm-dialog-shade-select": "shade mode selector UI"},
                             focus="Shaded pass không được mutate model hoặc document dirty."),
    "OM9-L3-01-06-006": dict(rcore=["RCORE-12", "RCORE-11"],
                             mx={"mx-config": "grid/axis display configuration",
                                 "mx-layout-tools": "layout/print overlay precedent"},
                             focus="Depth/grid/axis overlay là display-only; grid finite theo viewport."),
    "OM9-L3-01-06-007": dict(rcore=["RCORE-12", "RCORE-06"],
                             mx={"mx-object-actions": "selection/hover visual precedent",
                                 "mx-seldup": "duplicate-selection highlight case"},
                             focus="Highlight pass đọc selection state, không sở hữu selection."),
    "OM9-L3-01-06-008": dict(rcore=["RCORE-12"],
                             mx={"mx-display-net": "invalidation watchers (ShadeModeEventWatcher)",
                                 "mx-frm-cmenu-display": "display section refresh precedent (UpdateHistory)"},
                             focus="Invalidate/redraw chỉ khi dirty; đo số frame redraw không cần thiết."),
    "OM9-L3-01-06-009": dict(rcore=["RCORE-12"],
                             mx={"mx-frm-dialog-vga": "device/problem diagnostic precedent"},
                             focus="Device loss phải có đường phục hồi hoặc chẩn đoán rõ, không thoát im lặng."),

    # ================= M01: L2-01-08 units =================
    "OM9-L3-01-08-001": dict(rcore=["RCORE-02"], mx={"mx-matrixdata": "length wrapper precedent (MatrixDoubleLength)"},
                             focus="Đơn vị nội bộ mm và chuyển đổi hiển thị phải test ở biên; không lấy default từ Matrix9."),
    "OM9-L3-01-08-002": dict(rcore=["RCORE-02"], mx={"mx-matrixdata": "angle wrapper precedent (MatrixDoubleAngle)"},
                             focus="Degree input/radian nội bộ phải round-trip trong tolerance."),
    "OM9-L3-01-08-003": dict(rcore=["RCORE-02", "RCORE-08"],
                             mx={"mx-frm-dialog-options": "mesh/tolerance option captions (evidence of UI vocabulary only)"},
                             focus="Absolute/relative tolerance phải là policy tài liệu hoá; giá trị solver ghi riêng khỏi Help default."),
    "OM9-L3-01-08-004": dict(rcore=["RCORE-02"], mx={"mx-matrixdata": "typed numeric wrapper precedent"},
                             focus="Typed parameter (length/angle/count/ratio) không được nhập nhằng khi serialize."),
    "OM9-L3-01-08-005": dict(rcore=["RCORE-02", "RCORE-04"],
                             mx={"mx-frm-dialog-options": "min/max/increment UI vocabulary",
                                 "mx-ctrl-inbox": "numeric input control precedent"},
                             focus="Range validation phải trả lỗi typed; increment hiển thị không phải ràng buộc dữ liệu."),
    "OM9-L3-01-08-006": dict(rcore=["RCORE-02", "RCORE-10"],
                             mx={"mx-matrixdata": "dimensional wrapper that must round-trip"},
                             focus="Serialize phải giữ dimensional semantics; test round-trip đơn vị."),
    "OM9-L3-01-08-007": dict(rcore=["RCORE-03"],
                             mx={"mx-ctrl-inbox": "coordinate input control precedent",
                                 "mx-matrixdata": "MatrixPoint local/world carrier"},
                             focus="Point local/world phải phân biệt rõ; parser theo RCORE-03 (CPlane-aware)."),

    # ================= M01: L2-01-12 project codec =================
    "OM9-L3-01-12-001": dict(rcore=["RCORE-10"], mx={"mx-mod-file-system": "project container/IO precedent",
                                                     "mx-frm-dialog-save": "save dialog surface"},
                             focus="Container versioned + manifest; không sao chép định dạng .mpj/FCStd."),
    "OM9-L3-01-12-002": dict(rcore=["RCORE-10", "RCORE-06"], mx={"mx-clsminterface": "state surface to encode"},
                             focus="Encode entity/layer/visibility/locked phải round-trip; test bằng fixture roundtrip."),
    "OM9-L3-01-12-003": dict(rcore=["RCORE-10", "RCORE-01"], mx={"mx-gemblock": "gem instance record precedent"},
                             focus="BRep/mesh/instance record phải có schema tường minh; không lưu display mesh thay BRep."),
    "OM9-L3-01-12-004": dict(rcore=["RCORE-09", "RCORE-10"], focus="Giữ chỗ feature association schema nhưng không tuyên bố history đã chạy."),
    "OM9-L3-01-12-005": dict(rcore=["RCORE-10"], mx={"mx-frm-dialog-save": "write/failure surface"},
                             focus="Temp-write + validate + commit; test lỗi ghi giữa đường."),
    "OM9-L3-01-12-006": dict(rcore=["RCORE-10"], focus="Kiểm format/version trước khi mutate scene; sai version phải từ chối rõ."),
    "OM9-L3-01-12-007": dict(rcore=["RCORE-10", "RCORE-09"], focus="Reload phải tái tạo ID/quan hệ typed; so sánh trước-sau."),
    "OM9-L3-01-12-008": dict(rcore=["RCORE-10", "RCORE-12"],
                             mx={"mx-frm-dialog-options": "STL Options caption evidence"},
                             focus="STL qua tessellator chung; kiểm watertight/winding theo đơn vị mm."),
    "OM9-L3-01-12-009": dict(rcore=["RCORE-10", "RCORE-02"], focus="Normals + mm unit policy cho STL phải đo, không suy từ caption 'STL Options'."),
    "OM9-L3-01-12-010": dict(rcore=["RCORE-10"], focus="Fixture roundtrip phải so layer/ID/instance bằng giá trị, không chỉ mở được file."),

    # ================= M01: L2-02-01 Rhino object semantics =================
    "OM9-L3-02-01-001": dict(rcore=["RCORE-01"], mx={"mx-matrixdata": "Point/Polyline wrapper precedent"},
                             focus="Semantics Point/Polyline/Polycurve phải typed; không gộp polycurve thành line."),
    "OM9-L3-02-01-002": dict(rcore=["RCORE-01", "RCORE-07"],
                             mx={"mx-matrixdata": "curve wrapper precedent"},
                             focus="Degree/knot/weight phải đọc-ghi được; test locus sau edit knot."),
    "OM9-L3-02-01-003": dict(rcore=["RCORE-01", "RCORE-07"],
                             mx={"mx-matrixdata": "curve wrapper carrying closedness/planarity flags"},
                             focus="Open/closed/planar phải là thuộc tính đo được, không suy từ tên lệnh."),
    "OM9-L3-02-01-004": dict(rcore=["RCORE-01"], mx={"mx-matrixdata": "surface wrapper carrying UV/trim data"},
                             focus="UV domain/seam/periodic phải có fixture riêng; ghi rõ chưa chứng minh nếu thiếu."),
    "OM9-L3-02-01-005": dict(rcore=["RCORE-01"], mx={"mx-matrixdata": "surface/trim wrapper precedent"},
                             focus="Trimmed vs underlying surface phải giữ cả hai; không thay mặt nền bằng trim."),
    "OM9-L3-02-01-006": dict(rcore=["RCORE-01", "RCORE-12"], mx={"mx-display-net": "isocurve visibility toggle precedent"},
                             focus="Edge thật vs isocurve hiển thị phải phân biệt trong model, không chỉ trong display."),
    "OM9-L3-02-01-007": dict(rcore=["RCORE-01", "RCORE-08"], focus="Polysurface closed = solid phải kiểm validity/closedness bằng OCCT."),
    "OM9-L3-02-01-008": dict(rcore=["RCORE-01"], focus="Lightweight extrusion vs full BRep: ghi rõ ranh giới chuyển đổi và mất mát."),
    "OM9-L3-02-01-009": dict(rcore=["RCORE-01", "RCORE-08"], focus="Mesh không được ngầm hiểu là NURBS chính xác; lỗi phải tường minh."),
    "OM9-L3-02-01-010": dict(rcore=["RCORE-06"], mx={"mx-gemblock": "instance model precedent"},
                             focus="Definition identity vs instance identity phải tách; explode không đổi definition."),

    # ================= M01: L2-03-01 viewport layout =================
    "OM9-L3-03-01-001": dict(
        rcore=["RCORE-12"],
        mx={"mx-frm-master": "one host window with four-view composition",
            "mx-gvpanel": "viewport panel command"},
        focus="Preset 4 view (Looking Down/Perspective/Side/Through Finger) phải có camera test; không suy từ ảnh chụp."),
    "OM9-L3-03-01-002": dict(rcore=["RCORE-12"], focus="Camera preset phải xác định axis/perspective default và test được."),
    "OM9-L3-03-01-003": dict(rcore=["RCORE-12"], mx={"mx-frm-master": "active window/title state precedent"},
                             focus="Active viewport indicator phải theo view identity, không gắn cứng một view."),
    "OM9-L3-03-01-004": dict(rcore=["RCORE-03"], mx={"mx-frm-dialog-options": "origin/datum option vocabulary"},
                             focus="F4/0,0,0 affordance là display + chèn điểm, không tạo geometry thật."),
    "OM9-L3-03-01-005": dict(rcore=["RCORE-12", "RCORE-02"], mx={"mx-config": "grid/axis theme configuration"},
                             focus="Grid mm/axis theme theo viewport; grid finite, không suy spacing default."),
    "OM9-L3-03-01-006": dict(rcore=["RCORE-12"], focus="Maximize/restore phải giữ document state và không đổi mode view khác."),
    "OM9-L3-03-01-007": dict(rcore=["RCORE-12"], focus="Tab switch khi maximized phải map đúng view id; reject stale view."),

    # ================= M01: L2-03-07 sidebar/main menu =================
    "OM9-L3-03-07-001": dict(
        rcore=["RCORE-12"],
        mx={"mx-frm-menu-icon-history": "section 1: Icon History",
            "mx-frm-menu-tools": "section: Main Menu (Icon Menu)",
            "mx-frm-cmenu-display": "section: Display",
            "mx-frm-menu-snaps": "section: Snaps",
            "mx-frm-menu-info-settings": "section: Menu Info History",
            "mx-frm-menu-layers": "section: Layers",
            "mx-frm-menu-projects": "section: Project Manager"},
        focus="Stack/order 7 section phải khớp danh sách form sidebar thật; mỗi section giữ đúng một owner."),
    "OM9-L3-03-07-002": dict(
        rcore=["RCORE-04"],
        mx={"mx-frm-menu-tools": "icon category grid host",
            "mx-frm-menu-editor": "category/layout editor",
            "mx-mod-topmenu": "menu construction module",
            "mx-main-command": "managed main menu command"},
        focus="Category tabs lấy từ tên category quan sát được; không thêm/bớt category từ ảnh."),
    "OM9-L3-03-07-003": dict(
        rcore=["RCORE-04"],
        mx={"mx-frm-menu-icon-history": "right-click slot per icon",
            "mx-frm-menu-tools": "CmdIcon handlers + ImageList icons"},
        focus="Tooltip/right-click slot phải có handler hoặc disabled rõ; glyph không đọc được thì không gán chức năng."),
    "OM9-L3-03-07-004": dict(rcore=["RCORE-12"], mx={"mx-frm-menu-tools": "scrolling icon rows/ImageList"},
                             focus="Scrolling/icon grid phải giữ identity slot khi scroll."),
    "OM9-L3-03-07-005": dict(rcore=["RCORE-04"],
                             mx={"mx-frm-menu-icon-history": "recent command surface + undo/redo small buttons",
                                 "mx-cmd-iconhistory": "managed Icon History command"},
                             focus="Recent-command list phải theo lịch sử thật, không trùng lệnh failed/cancel."),
    "OM9-L3-03-07-006": dict(rcore=["RCORE-12"], mx={"mx-cmdbar": "float/dock command bar precedent",
                                                     "mx-frm-cpanel": "panel container"},
                             focus="Collapse/reorder/float phải tích hợp dock manager, không tạo hệ dock thứ hai."),
    "OM9-L3-03-07-007": dict(rcore=["RCORE-12"], mx={"mx-frm-menu-editor": "layout reset/editor surface"},
                             focus="Reset default layout phải tái lập được và có test trước/sau."),
    "OM9-L3-03-07-008": dict(rcore=["RCORE-04"], mx={"mx-mod-menu-controls": "greyed/disabled availability logic"},
                             focus="Lệnh chưa làm phải hiện disabled-visible; không ẩn và không giả PASS."),

    # ================= M01: L2-03-10 command line =================
    "OM9-L3-03-10-001": dict(rcore=["RCORE-04"], mx={"mx-commandline-native": "VB6 command-line text handling"},
                             focus="Nhập lệnh + Enter phải đi cùng một đường với menu/mouse command."),
    "OM9-L3-03-10-002": dict(rcore=["RCORE-04"], mx={"mx-edittext": "prompt input form", "mx-frm-messages": "prompt/message surface"},
                             focus="Prompt phải phản ánh phase thật của state machine; không hard-code prompt."),
    "OM9-L3-03-10-003": dict(rcore=["RCORE-04"], mx={"mx-frm-dialog-options": "option vocabulary",
                                                      "mx-ctrl-inbox": "in-box option controls"},
                             focus="Option token phải clickable và map đúng tên option đã khai báo."),
    "OM9-L3-03-10-004": dict(rcore=["RCORE-03", "RCORE-02"],
                             mx={"mx-matrixdata": "typed numeric wrapper"},
                             focus="Feedback số/đơn vị phải typed; test nhập distance/angle/coordinate."),
    "OM9-L3-03-10-005": dict(rcore=["RCORE-04"], mx={"mx-frm-messages": "message/history surface"},
                             focus="History panel phải phân biệt event hợp lệ với callback trùng."),
    "OM9-L3-03-10-006": dict(rcore=["RCORE-04"], mx={"mx-mod-vb-core": "Escape/repeat behaviour precedent"},
                             focus="Escape abort theo phase; Enter/Space repeat chỉ khi idle và có whitelist."),

    # ================= M01: L2-03-14 layers/object info =================
    "OM9-L3-03-14-001": dict(rcore=["RCORE-06"],
                             mx={"mx-frm-menu-layers": "palette with categories + advanced block",
                                 "mx-cmd-layers": "layers command"},
                             focus="Category palette (Metals/Gems/User/Heads/Finger/Cutting/Creation) phải có danh sách layer thật, không bịa."),
    "OM9-L3-03-14-002": dict(rcore=["RCORE-06"], mx={"mx-clsminterface": "CurrentLayer/assignment facade",
                                                      "mx-frm-menu-layers": "current-layer UI"},
                             focus="Current layer + assign phải là một đường duy nhất qua document owner."),
    "OM9-L3-03-14-003": dict(rcore=["RCORE-06"], mx={"mx-frm-menu-layers": "PicShow/PicHide/lock controls"},
                             focus="Visible/locked toggle phải đổi state document, không chỉ đổi màu UI."),
    "OM9-L3-03-14-004": dict(rcore=["RCORE-06"], mx={"mx-clsminterface": "SelectAllObjects/SelectObjects surface"},
                             focus="Select-by-layer phải theo ID layer và tôn trọng hidden/locked policy."),
    "OM9-L3-03-14-005": dict(rcore=["RCORE-01", "RCORE-08"],
                             mx={"mx-frm-menu-info-settings": "object info refresh timer",
                                 "mx-info-settings-cmd": "info/settings command"},
                             focus="Object Info (type/validity/dimensions) phải đọc từ model; validity theo RCORE-08."),
    "OM9-L3-03-14-006": dict(rcore=["RCORE-06", "RCORE-08"],
                             mx={"mx-seldup": "SelDup managed command",
                                 "mx-object-actions": "object action family"},
                             focus="Super-select (bad/open/duplicate/gem) phải là truy vấn typed trên document."),
    "OM9-L3-03-14-007": dict(rcore=["RCORE-06"], mx={"mx-object-actions": "property editor surface"},
                             focus="Editor chỉ phơi bày attribute của document owner, không giữ bản sao state."),
    "OM9-L3-03-14-008": dict(rcore=["RCORE-10", "RCORE-06"], mx={"mx-frm-menu-layers": "palette state to re-verify after reopen"},
                             focus="Reopen document rồi đối chiếu palette state; đây là visual check, không đóng logic."),

    # ================= M01: L2-03-15 file/project/jobbag =================
    "OM9-L3-03-15-001": dict(
        rcore=["RCORE-10"],
        mx={"mx-frm-dialog-save": "save dialog", "mx-clsminterface": "Import/Export facade",
            "mx-mod-file-system": "file path handling", "mx-frm-master": "title/untitled state"},
        focus="File New/Open/Save dùng codec native; đối chiếu bề mặt dialog Matrix9 nhưng không copy định dạng."),
    "OM9-L3-03-15-002": dict(rcore=["RCORE-10"],
                             mx={"mx-clsminterface": "Import/Export entry surface",
                                 "mx-mod-file-system": "import/export path handling"},
                             focus="Một importer .3dm và một exporter STL dùng chung; không nhân bản trong menu/adapter."),
    "OM9-L3-03-15-003": dict(rcore=["RCORE-10"],
                             mx={"mx-frm-menu-projects": "project row list (ctrProjectList) + create/rename",
                                 "mx-wpf-project-manager": "managed project entity model",
                                 "mx-project-dialogs": "create/rename/load dialogs"},
                             focus="Project row CRUD phải có identity ổn định; không dùng index/offset làm ID."),
    "OM9-L3-03-15-004": dict(rcore=["RCORE-10"],
                             mx={"mx-frm-menu-projects": "job bag shelf rows/thumbnails",
                                 "mx-wpf-project-manager": "JobBag entity + JobBagType/Visibility"},
                             focus="Slot grid + thumbnail phải theo identity job bag; capacity không bị khoá ở 3/10."),
    "OM9-L3-03-15-005": dict(rcore=["RCORE-10"], mx={"mx-frm-dialog-savegem": "capture-to-slot dialog precedent"},
                             focus="Capture selection vào slot phải ghi rõ phần nào được lưu và phần nào bỏ qua."),
    "OM9-L3-03-15-006": dict(rcore=["RCORE-10"], mx={"mx-frm-project-manager": "restore/insert surface"},
                             focus="Restore/insert phải tạo object mới với ID mới, không tái dùng handle cũ."),
    "OM9-L3-03-15-007": dict(rcore=["RCORE-10", "RCORE-09"], mx={"mx-mod-project-manager": "master snapshot backend"},
                             focus="Master snapshot phải phân biệt với snapshot toàn document và có Save/Discard/Cancel."),
    "OM9-L3-03-15-008": dict(rcore=["RCORE-10"], mx={"mx-frm-project-manager": "project notes/metadata surface"},
                             focus="Note/metadata phải persist và không ảnh hưởng geometry identity."),
    "OM9-L3-03-15-009": dict(rcore=["RCORE-12"], mx={"mx-frm-cpanel": "panel container",
                                                      "mx-frm-menu-projects": "Projects section placement"},
                             focus="Projects nằm trên Builder panel trong dock manager; không tạo container thứ hai."),

    # ================= M02: L2-01-07 picking/selection/handles =================
    "OM9-L3-01-07-001": dict(
        rcore=["RCORE-05", "RCORE-06"],
        mx={"mx-mouse-callback": "pointer event routing",
            "mx-clsminterface": "pick/select facade used by the sidebar"},
        focus="Ray/hit test phải trả candidate typed + khoảng cách; aperture theo logical pixel (RCORE-05)."),
    "OM9-L3-01-07-002": dict(rcore=["RCORE-06"], mx={"mx-clsminterface": "SelectObject(s)/UnSelectObject(s) facade",
                                                      "mx-object-actions": "object action selection model"},
                             focus="Selection set theo persistent ID; add/remove/toggle phải test được."),
    "OM9-L3-01-07-003": dict(rcore=["RCORE-06", "RCORE-01"],
                             mx={"mx-ctrl-inbox": "sub-object input control precedent"},
                             focus="Sub-object pick phải có identity ổn định cho face/edge/vertex; không dùng index hiển thị."),
    "OM9-L3-01-07-004": dict(rcore=["RCORE-06"], focus="Window/crossing phải theo RCORE-06, xử lý object cắt biên."),
    "OM9-L3-01-07-005": dict(rcore=["RCORE-06", "RCORE-05"],
                             mx={"mx-mod-rightclick": "context/ambiguity routing precedent"},
                             focus="Disambiguation popup phải liệt kê candidate typed; không tự chọn ngầm."),
    "OM9-L3-01-07-006": dict(rcore=["RCORE-12", "RCORE-06"],
                             mx={"mx-display-net": "display conduit for highlight"},
                             focus="Hover/highlight là display-only; test không đổi document dirty."),
    "OM9-L3-01-07-007": dict(rcore=["RCORE-05", "RCORE-07"],
                             mx={"mx-tube-handles": "Matrix handle implementation precedent",
                                 "mx-msr": "smart-target handle precedent"},
                             focus="Handle placement/drag feedback tách khỏi phép biến đổi; không copy cơ chế handle Matrix."),
    "OM9-L3-01-07-008": dict(rcore=["RCORE-04", "RCORE-06"],
                             mx={"mx-frm-menu-icon-history": "undo entry per drag observed"},
                             focus="Drag preview + commit/cancel = một bước undo; test cancel không để lại geometry."),

    # ================= M02: L2-02-02 CPlane/coordinate =================
    "OM9-L3-02-02-001": dict(rcore=["RCORE-03", "RCORE-12"],
                             mx={"mx-frm-master": "per-view host precedent"},
                             focus="CPlane per viewport vs world phải rõ trong model; test đổi view giữa chừng."),
    "OM9-L3-02-02-002": dict(rcore=["RCORE-03", "RCORE-12"], focus="Preset World/Top/Front/Right phải map đúng trục của từng view."),
    "OM9-L3-02-02-003": dict(rcore=["RCORE-03"], focus="Đổi origin/orientation CPlane phải không mutate geometry đã tạo."),
    "OM9-L3-02-02-004": dict(rcore=["RCORE-03"],
                             mx={"mx-ctrl-inbox": "coordinate input control", "mx-edittext": "text/number prompt"},
                             focus="Parser world/local/relative theo RCORE-03; test w/r/wr với previous point reset."),
    "OM9-L3-02-02-005": dict(rcore=["RCORE-03", "RCORE-04"], mx={"mx-frm-dialog-options": "origin shortcut vocabulary"},
                             focus="F4 chèn world origin vào prompt hiện hành, không tự tạo object."),
    "OM9-L3-02-02-006": dict(rcore=["RCORE-05"], mx={"mx-frm-menu-snaps": "Ortho/Planar/Project toggles in snaps palette"},
                             focus="Ortho/Planar/Project phải theo policy RCORE-05 và ghi setting của lần chạy."),
    "OM9-L3-02-02-007": dict(rcore=["RCORE-03", "RCORE-12"], focus="Đổi viewport giữa chừng phải cho cùng điểm world; ghi frame/CPlane."),

    # ================= M02: L2-02-03 osnaps/smarttrack =================
    "OM9-L3-02-03-001": dict(rcore=["RCORE-05"], mx={"mx-frm-menu-snaps": "Osnap button strip (imgosnapbuttons)"},
                             focus="End/Mid/Cen/Quad phải trả candidate + marker; test nhiều candidate cùng aperture."),
    "OM9-L3-02-03-002": dict(rcore=["RCORE-05"], mx={"mx-frm-menu-snaps": "osnap inline strip"},
                             focus="Near/Point/Int phải phân biệt true/apparent intersection (RCORE-05.T04)."),
    "OM9-L3-02-03-003": dict(rcore=["RCORE-05"], focus="Tan/Perp chỉ nhận điểm trên geometry hợp lệ; impossible case phải từ chối."),
    "OM9-L3-02-03-004": dict(rcore=["RCORE-05"], mx={"mx-mod-rhino-controls": "host surface snap call path (disassembly stub)"},
                             focus="Surface nearest point phải kiểm trim hole (RCORE-05.T03), không nhận điểm trên mặt nền."),
    "OM9-L3-02-03-005": dict(rcore=["RCORE-05"], mx={"mx-frm-menu-snaps": "SetInitialSnaps/UpdateSnaps persistent vs one-shot state"},
                             focus="Precedence/one-shot theo RCORE-05.T07/T11/T12; ghi rõ phần chưa có evidence."),
    "OM9-L3-02-03-006": dict(rcore=["RCORE-05"], focus="SmartTrack inference phải transient: cancel không để lại object/undo entry."),
    "OM9-L3-02-03-007": dict(rcore=["RCORE-05", "RCORE-12"], mx={"mx-frm-menu-snaps": "snap marker/tooltip feedback"},
                             focus="Marker/tooltip phải khớp accepted point; log marker và điểm đo."),

    # ================= M02: L2-03-02 navigation =================
    "OM9-L3-03-02-001": dict(rcore=["RCORE-12"], mx={"mx-mousewheel": "wheel/routing hook precedent"},
                             focus="Orbit perspective qua drag; camera per view, không đổi CPlane/geometry."),
    "OM9-L3-03-02-002": dict(rcore=["RCORE-12"], focus="Pan orthographic giữ scale; đo camera trước/sau."),
    "OM9-L3-03-02-003": dict(rcore=["RCORE-12"], focus="Wheel zoom quanh con trỏ; test DPR và logical pixel."),
    "OM9-L3-03-02-004": dict(rcore=["RCORE-12"], focus="Shift+right drag pan perspective không orbit; kiểm camera target."),
    "OM9-L3-03-02-005": dict(rcore=["RCORE-12", "RCORE-06"], focus="Zoom extents/selected phải tính theo bbox thật; selection rỗng có nhánh riêng."),
    "OM9-L3-03-02-006": dict(rcore=["RCORE-12", "RCORE-03"], mx={"mx-frm-dialog-options": "view reset vocabulary"},
                             focus="Reset/home phải là hành vi có thể tái lập; không suy từ ảnh chụp."),
    "OM9-L3-03-02-007": dict(rcore=["RCORE-12"], mx={"mx-frm-master": "title bar/window host"},
                             focus="Double-click title/tab maximize phải giữ phần còn lại của layout."),

    # ================= M02: L2-03-03 selection/gumball =================
    "OM9-L3-03-03-001": dict(rcore=["RCORE-06"],
                             mx={"mx-clsminterface": "Select/Unselect facade",
                                 "mx-mod-menu-controls": "selection-driven availability"},
                             focus="Shift/Ctrl add/remove phải theo selection set typed; test toggle hai lần."),
    "OM9-L3-03-03-002": dict(rcore=["RCORE-06"], focus="Rectangle trái/phải phải phân biệt contained vs crossing và vẽ đúng."),
    "OM9-L3-03-03-003": dict(rcore=["RCORE-06"],
                             mx={"mx-object-context-menu": "ObjectType/context menu model for disambiguation",
                                 "mx-mod-rightclick": "right-click routing"},
                             focus="Popup chọn theo object type/layer swatch; không tự chọn candidate đầu tiên."),
    "OM9-L3-03-03-004": dict(rcore=["RCORE-07", "RCORE-06"],
                             mx={"mx-tube-handles": "MatrixHandle axis implementation precedent",
                                 "mx-msr": "MSR handle precedent"},
                             focus="Axis drag phải áp transform một lần và commit một undo entry."),
    "OM9-L3-03-03-005": dict(rcore=["RCORE-03", "RCORE-07"], focus="Alignment world/CPlane/object + relocate handle origin phải test được."),
    "OM9-L3-03-03-006": dict(rcore=["RCORE-07", "RCORE-04"],
                             mx={"mx-edittext": "numeric prompt for exact input"},
                             focus="Numeric edit phải cho kết quả bằng transform chính xác; log value + sai số."),

    # ================= M02: L2-03-04 snaps palette =================
    "OM9-L3-03-04-001": dict(
        rcore=["RCORE-05"],
        mx={"mx-frm-menu-snaps": "always-visible Object Snap + Grid palette (imgosnapbuttons, imgGridButton, imgchksnaps)",
            "mx-snapscmd": "managed snaps command"},
        focus="Palette luôn hiện với Object Snap + Grid; trạng thái đọc từ document/session, không từ ảnh."),
    "OM9-L3-03-04-002": dict(rcore=["RCORE-05"], mx={"mx-frm-menu-snaps": "PicSnap/picGridSnap toggles"},
                             focus="Ortho/Project/Planar toggle phải đổi policy snapping và ghi setting lần chạy."),
    "OM9-L3-03-04-003": dict(rcore=["RCORE-05"], mx={"mx-frm-menu-snaps": "SetInitialSnaps vs UpdateSnaps (persistent/momentary)"},
                             focus="Indicator phải khớp trạng thái persistent/momentary; test one-shot tiêu thụ đúng một lần."),
    "OM9-L3-03-04-004": dict(rcore=["RCORE-05"], mx={"mx-frm-menu-snaps": "tmrSnaps_Timer + UpdateSnaps feedback loop"},
                             focus="SmartTrack toggle + cue phải transient; không thêm object/undo entry."),
    "OM9-L3-03-04-005": dict(rcore=["RCORE-05"], mx={"mx-mod-vb-core": "shortcut handling precedent"},
                             focus="Shortcut đổi snap mode phải không đổi setting persistent ngoài ý muốn."),

    # ================= M02: L2-03-05 display/shading =================
    "OM9-L3-03-05-001": dict(
        rcore=["RCORE-12"],
        mx={"mx-display-net": "ShadeModeForm/ShadeModeDisplayConduit",
            "mx-frm-dialog-shade-select": "Shade Select chooser",
            "mx-frm-cmenu-display": "Display sidebar section"},
        focus="Wireframe/Shaded per viewport; đổi mode không mutate model/dirty/undo journal."),
    "OM9-L3-03-05-002": dict(rcore=["RCORE-12", "RCORE-01"],
                             mx={"mx-gemblock": "gem display model", "mx-gemmap": "gem map colour cues"},
                             focus="Gem View là display cue cho modeling, không phải offline render; test không đổi material thật."),
    "OM9-L3-03-05-003": dict(rcore=["RCORE-12"], mx={"mx-display-net": "curve/isocurve visibility toggles"},
                             focus="Preset ẩn/hiện helper chỉ đổi display state, không xoá object."),
    "OM9-L3-03-05-004": dict(rcore=["RCORE-12", "RCORE-08"], mx={"mx-display-net": "backface/surface display cues"},
                             focus="Backface cue là display; không tự chứng nhận surface hợp lệ (RCORE-08)."),
    "OM9-L3-03-05-005": dict(rcore=["RCORE-12", "RCORE-06"], focus="Selected highlight override phải đọc selection state và không ghi vào model."),
    "OM9-L3-03-05-006": dict(rcore=["RCORE-12"], mx={"mx-config": "persistent display configuration"},
                             focus="Preset display lưu theo user/document; ghi rõ scope và không đổi geometry."),

    # ================= M02: L2-03-06 picture frame =================
    "OM9-L3-03-06-001": dict(rcore=["RCORE-12", "RCORE-10"],
                             mx={"mx-imageeditor": "image picker/editor controls",
                                 "mx-layout-tools": "image placement/layout precedent"},
                             focus="Import ảnh và chèn quad reference; ghi rõ định dạng và đường dẫn asset."),
    "OM9-L3-03-06-002": dict(rcore=["RCORE-12"], mx={"mx-imageeditor": "image adjust surface"},
                             focus="Move/scale ảnh theo viewport; test không đổi geometry khác."),
    "OM9-L3-03-06-003": dict(rcore=["RCORE-12", "RCORE-06"], focus="Opacity/lock/select của ảnh phải tách khỏi layer lock của geometry."),
    "OM9-L3-03-06-004": dict(rcore=["RCORE-02", "RCORE-03"], mx={"mx-imageeditor": "calibration by measured distance surface"},
                             focus="Calibration phải ghi đơn vị mm và sai số; không lấy scale từ ảnh mẫu."),
    "OM9-L3-03-06-005": dict(rcore=["RCORE-10", "RCORE-12"],
                             mx={"mx-mod-file-system": "asset path handling precedent"},
                             focus="Asset reference phải survive save/reopen; test missing/relink như RCORE-12."),

    # ================= M02: L2-03-09 F6 context =================
    "OM9-L3-03-09-001": dict(
        rcore=["RCORE-06", "RCORE-04"],
        mx={"mx-object-context-menu": "ObjectType/ObjectContextMenu model",
            "mx-mod-rightclick": "context routing",
            "mx-object-actions": "object action families"},
        focus="Dispatch F6 theo class đối tượng chọn; danh sách action phải đến từ registry, không hard-code."),
    "OM9-L3-03-09-002": dict(rcore=["RCORE-04"],
                             mx={"mx-object-context-menu": "general mode defaults",
                                 "mx-frm-menu-dialog": "dialog host for menu actions"},
                             focus="General mode khi selection rỗng phải có danh sách ổn định và test được."),
    "OM9-L3-03-09-003": dict(rcore=["RCORE-04", "RCORE-07"], mx={"mx-object-context-menu": "curve/surface mode grouping"},
                             focus="Curve/surface mode chỉ hiện action khả dụng; action thiếu phải disabled."),
    "OM9-L3-03-09-004": dict(rcore=["RCORE-04"], mx={"mx-gemmap": "gem map/report actions",
                                                      "mx-object-context-menu": "gem/report grouping"},
                             focus="Gem/report mode phải theo report data thật, không hiện action giả."),
    "OM9-L3-03-09-005": dict(rcore=["RCORE-04"],
                             mx={"mx-object-context-menu": "editor view for user menus",
                                 "mx-config-static": "user options model",
                                 "mx-frm-menu-editor": "legacy menu editor precedent"},
                             focus="Custom user mode lưu cấu hình action, không copy implementation; test import/export menu."),
    "OM9-L3-03-09-006": dict(rcore=["RCORE-04"],
                             mx={"mx-mod-builder-loader": "builder prefill/mount precedent",
                                 "mx-builder-base": "builder input contract"},
                             focus="Prefill builder input phải qua contract builder; không truyền handle UI."),
    "OM9-L3-03-09-007": dict(rcore=["RCORE-12"], mx={"mx-object-context-menu": "floating window/position behaviour"},
                             focus="Pin/position menu phải giữ được qua lần mở sau; không đổi selection."),
    "OM9-L3-03-09-008": dict(rcore=["RCORE-04"], mx={"mx-mod-menu-controls": "disabled availability logic"},
                             focus="Action chưa làm phải hiện disabled-visible và không tạo PASS giả."),

    # ================= M04: L2-01-04 associative rebuild =================
    "OM9-L3-01-04-001": dict(
        rcore=["RCORE-09", "RCORE-01"],
        mx={"mx-history-commands": "matrix history command surface adjacent to the feature graph",
            "mx-manual-rhino-history": "Rhino History contract that the feature record must respect",
            "mx-manual-history": "Matrix history section: record/update/clear semantics",
            "mx-manual-builders-vs-commands": "what counts as a builder feature vs. a plain command"},
        focus="Feature node + typed IO phải theo RCORE-09/01; phân biệt builder feature với command thường theo sheet builders-vs-commands."),
    "OM9-L3-01-04-002": dict(
        rcore=["RCORE-09", "RCORE-04"],
        mx={"mx-manual-history-record": "Matrix history-record command and its stated options",
            "mx-manual-history": "record/update/clear policy context",
            "mx-manual-builders-vs-commands": "command provenance for produced geometry"},
        focus="Provenance (command + user parameters) phải ghi được cho từng kết quả; đối chiếu History Record nhưng không suy default."),
    "OM9-L3-01-04-003": dict(
        rcore=["RCORE-09"],
        mx={"mx-manual-history": "associative update behaviour in Matrix",
            "mx-manual-rhino-history": "Rhino HistoryUpdate contract",
            "mx-history-ui": "update toggle/status surface"},
        focus="Dirty propagation + topological order phải test bằng DAG thật; không suy từ UI toggle."),
    "OM9-L3-01-04-004": dict(
        rcore=["RCORE-09", "RCORE-06"],
        mx={"mx-manual-history": "reference binding under recompute",
            "mx-manual-ring-rail": "rail/profile attachment case used by ring builders",
            "mx-builder-ring-forms": "rail/profile dialogs that create the references"},
        focus="Semantic reference (rail/profile/gem attachment) phải survive recompute; fixture rail→profile→sweep."),
    "OM9-L3-01-04-005": dict(
        rcore=["RCORE-09"],
        mx={"mx-manual-history-update": "Matrix associative update on/off behaviour",
            "mx-history-commands": "history command implementations"},
        focus="Incremental recompute: chỉ tính lại phần bị ảnh hưởng; đo số node recompute, không chỉ kết quả cuối."),
    "OM9-L3-01-04-006": dict(
        rcore=["RCORE-09", "RCORE-04"],
        mx={"mx-builder-base": "builder preview lifecycle contract",
            "mx-manual-builders-vs-commands": "builder vs command commit model"},
        focus="Preview graph phải bị huỷ mà không ảnh hưởng design đã lưu; test Escape không để lại node/undo entry."),
    "OM9-L3-01-04-007": dict(
        rcore=["RCORE-09", "RCORE-04"],
        mx={"mx-manual-history": "recompute failure reporting policy",
            "mx-frm-debuglog": "diagnostic surface for failed features"},
        focus="Giữ kết quả hợp lệ cuối và báo feature lỗi; không ghi PASS khi recompute fail im lặng."),
    "OM9-L3-01-04-008": dict(
        rcore=["RCORE-09", "RCORE-06"],
        mx={"mx-manual-history": "delete/clear history entry behaviour",
            "mx-manual-matrix-clear-history": "Matrix Clear Object History command"},
        focus="Chính sách khi parent bị xoá/đổi input topology; đối chiếu Clear Object History nhưng policy OM9 ghi riêng."),
    "OM9-L3-01-04-009": dict(
        rcore=["RCORE-09", "RCORE-01", "RCORE-08"],
        mx={"mx-manual-history": "reference regression cases",
            "mx-manual-join-history": "history-enabled edit case"},
        focus="Fixture rebuild ring/gem/settings phải so geometry + ID bằng giá trị; không chỉ so số đối tượng."),

    # ================= M04: L2-03-11 builder UI =================
    "OM9-L3-03-11-001": dict(
        rcore=["RCORE-04", "RCORE-01"],
        mx={"mx-ctrl-inbox": "in-box typed input controls (CtrInBox/CtrInBoxImage)",
            "mx-matrixcontrols": "MTextBox/MTextBoxNumeric/MNumberBox input controls",
            "mx-manual-builders": "builder input-slot behaviour"},
        focus="Input box phải validate đúng slot type (curve/gem/surface); sai type phải bị từ chối typed."),
    "OM9-L3-03-11-002": dict(
        rcore=["RCORE-04"],
        mx={"mx-matrixcontrols": "MTabBar/MTabControl mode tabs",
            "mx-builder-forms": "legacy builder panels with mode tabs",
            "mx-manual-builders": "builder mode/context behaviour"},
        focus="Mode tab đổi context typed; không giữ param của mode khác mà không báo."),
    "OM9-L3-03-11-003": dict(
        rcore=["RCORE-02", "RCORE-04"],
        mx={"mx-matrixcontrols": "MSliderSimple/MSliderAdvanced/MNumberBox controls",
            "mx-ctrl-inbox": "MenuSlider/Slider parameter controls",
            "mx-manual-builders": "parameter control and units behaviour"},
        focus="Numeric/slider/step/đơn vị phải round-trip với model; test đơn vị mm và bước nhảy."),
    "OM9-L3-03-11-004": dict(
        rcore=["RCORE-04", "RCORE-12"],
        mx={"mx-builder-base": "MatrixBuilderCore live preview + display conduit",
            "mx-manual-builders": "preview before commit behaviour"},
        focus="Preview throttle không commit; Escape trả state gốc; đo cả history/undo journal."),
    "OM9-L3-03-11-005": dict(
        rcore=["RCORE-04", "RCORE-05"],
        mx={"mx-tube-handles": "MatrixHandle dimension/rotation handle implementation",
            "mx-builder-base": "MatrixGetPoint handle plumbing",
            "mx-manual-builders": "viewport handle behaviour in builders"},
        focus="Handle phải bind vào param (không giữ bản sao); drag = một bước undo."),
    "OM9-L3-03-11-006": dict(
        rcore=["RCORE-04"],
        mx={"mx-builder-base": "one parameter source across panel and viewport",
            "mx-matrixcontrols": "panel control change notification",
            "mx-manual-builders": "panel/handle sync behaviour"},
        focus="Một nguồn sự thật cho param; test sửa panel → handle và ngược lại không lệch giá trị."),
    "OM9-L3-03-11-007": dict(
        rcore=["RCORE-01", "RCORE-07"],
        mx={"mx-builder-profile": "profile placer/editor window",
            "mx-manual-profile-placer": "profile placer command, options and defaults",
            "mx-manual-profile": "profile selection/edit behaviour"},
        focus="Profile selector preview + entry sang profile edit; đối chiếu option thật trong sheet profile-placer."),
    "OM9-L3-03-11-008": dict(
        rcore=["RCORE-04"],
        mx={"mx-builder-base": "builder accept/cancel/reset lifecycle",
            "mx-manual-builders": "Enter/Escape semantics for builders"},
        focus="Accept/Cancel/Reset/Undo phải đối xứng; test từng nhánh và trạng thái document sau đó."),
    "OM9-L3-03-11-009": dict(
        rcore=["RCORE-04", "RCORE-06"],
        mx={"mx-object-context-menu": "F6/object context menu model",
            "mx-mod-builder-loader": "builder prefill/mount path",
            "mx-manual-f6-menu": "F6 menu behaviour when a builder is active"},
        focus="Prefill từ F6/typed command phải qua builder input contract; không truyền handle UI."),

    # ================= M04: L2-03-12 builder styles =================
    "OM9-L3-03-12-001": dict(
        rcore=["RCORE-10", "RCORE-02"],
        mx={"mx-matrix-style": "MatrixStyle preset model",
            "mx-manual-styles": "what a saved style contains",
            "mx-manual-save-styles": "save-style workflow and defaults"},
        focus="Snapshot schema typed + versioned; test round-trip theo đơn vị và mode."),
    "OM9-L3-03-12-002": dict(
        rcore=["RCORE-12", "RCORE-10"],
        mx={"mx-matrix-style": "StylesLibraryBrowser (browse/thumbnail)",
            "mx-manual-styles": "style browser behaviour",
            "mx-library-browser": "library browser control panel/core separation"},
        focus="Browser/thumbnail chỉ đọc catalog; không tự apply khi chưa xác nhận."),
    "OM9-L3-03-12-003": dict(
        rcore=["RCORE-04", "RCORE-09"],
        mx={"mx-matrix-style": "style apply path into active builder",
            "mx-manual-load-styles": "load-style workflow and options"},
        focus="Load → preview → apply phải giữ một transaction; không tạo node lịch sử thừa."),
    "OM9-L3-03-12-004": dict(
        rcore=["RCORE-10"],
        mx={"mx-matrix-style": "user preset save path",
            "mx-manual-save-styles": "save-style naming/overwrite behaviour"},
        focus="Lưu preset người dùng không sửa default; test ghi đè và tên trùng."),
    "OM9-L3-03-12-005": dict(
        rcore=["RCORE-12", "RCORE-06"],
        mx={"mx-matrix-style": "style mode/shape filtering",
            "mx-manual-styles": "style compatibility rules",
            "mx-manual-match-attributes": "attribute matching filter case"},
        focus="Lọc style theo hình học/gem/setting phải dựa trên thuộc tính thật, không theo tên."),
    "OM9-L3-03-12-006": dict(
        rcore=["RCORE-06", "RCORE-01"],
        mx={"mx-matrix-style": "attribute reuse from selected entity",
            "mx-manual-match-attributes": "Match Attributes command and its options",
            "mx-object-actions": "selected-object attribute surface"},
        focus="Match attributes chỉ copy thuộc tính tương thích; ghi rõ thuộc tính nào bị bỏ qua."),

    # ================= M04: L2-03-13 matrix history UI =================
    "OM9-L3-03-13-001": dict(
        rcore=["RCORE-09", "RCORE-04"],
        mx={"mx-manual-history-record": "Matrix History Record command (record on/off)",
            "mx-manual-history": "record/update/clear family",
            "mx-history-ui": "history state surface"},
        focus="Record toggle phải đổi policy record thật; test chuỗi lệnh có/không ghi lịch sử."),
    "OM9-L3-03-13-002": dict(
        rcore=["RCORE-09"],
        mx={"mx-manual-history-update": "Matrix History Update command",
            "mx-history-ui": "associative update display state"},
        focus="Update toggle bật/tắt recompute hiển thị; không được làm mất liên kết đã ghi."),
    "OM9-L3-03-13-003": dict(
        rcore=["RCORE-09", "RCORE-08"],
        mx={"mx-manual-history": "broken/missing input reporting",
            "mx-frm-messages": "warning surface",
            "mx-manual-rhino-history": "Rhino broken-history reporting contract"},
        focus="Báo link hỏng/đổi input kèm danh sách cụ thể; không im lặng bỏ qua."),
    "OM9-L3-03-13-004": dict(
        rcore=["RCORE-09"],
        mx={"mx-manual-matrix-clear-history": "Matrix Clear Object History command",
            "mx-history-commands": "history command implementations"},
        focus="Clear history cho một object không ảnh hưởng association của object khác; test chọn lẫn."),
    "OM9-L3-03-13-005": dict(
        rcore=["RCORE-09", "RCORE-04"],
        mx={"mx-builder-base": "reopen builder with stored parameters",
            "mx-manual-history": "edit-by-history behaviour",
            "mx-mod-builder-loader": "builder mount/prefill path"},
        focus="Mở lại builder từ history phải nạp đúng param đã lưu; test param thiếu/đổi schema."),
    "OM9-L3-03-13-006": dict(
        rcore=["RCORE-04"],
        mx={"mx-manual-command-history": "command history panel behaviour",
            "mx-frm-messages": "status/transcript surface",
            "mx-history-ui": "history status surface"},
        focus="Status khi rebuild phải phân biệt event hợp lệ với callback trùng; không dedupe toàn bộ transcript."),

    # ================= M04: L2-04-02 ring rail/profile =================
    "OM9-L3-04-02-001": dict(
        rcore=["RCORE-02", "RCORE-01"],
        mx={"mx-builder-ring-forms": "frmBuilderRingsizes / ctrRingSize size reference UI",
            "mx-manual-ring-rail": "Ring Rail command, size standard and options",
            "mx-ring-resizer": "ring size pick/list controls"},
        focus="Chuẩn size ngón + đơn vị phải ghi rõ; rail tham chiếu là dữ liệu, không phải geometry đã chốt."),
    "OM9-L3-04-02-002": dict(
        rcore=["RCORE-07"],
        mx={"mx-builder-ring-forms": "frmICBuilderOutsideRingRail (Outside Ring Rail)",
            "mx-manual-outside-ring-rail": "Outside Ring Rail command and offset behaviour"},
        focus="Offset rail ngoài phải test với rail gốc và tolerance; ghi rõ chiều offset."),
    "OM9-L3-04-02-003": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-builder-profile": "profile placer control set",
            "mx-manual-profile-placer": "Profile Placer command, options and defaults",
            "mx-manual-profile": "profile edit behaviour"},
        focus="Place/edit profile dọc rail phải giữ tham số hoá; đối chiếu option thật của Profile Placer."),
    "OM9-L3-04-02-004": dict(
        rcore=["RCORE-07"],
        mx={"mx-builder-ring-forms": "frmBuilderCustomRail captions 'Ring Size/Width Reference'",
            "mx-manual-custom-rail": "Custom Rail and advanced rail options",
            "mx-manual-custom-rail-advanced": "advanced custom rail case"},
        focus="Rail tuỳ chọn/biến thể cục bộ phải tái lập được; ghi rõ option nào ảnh hưởng hình học."),
    "OM9-L3-04-02-005": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-matrixbuilder": "FourRailProfileBuilder/Command implementation",
            "mx-manual-four-rail-profile": "Four Rail Profile command and options"},
        focus="Dựng profile giữa bốn rail phải theo hợp đồng RCORE-07; test rail thiếu/không đồng phẳng."),
    "OM9-L3-04-02-006": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-builder-profile": "profile merge/endcap editing surface",
            "mx-manual-profile-merge": "profile merge behaviour",
            "mx-manual-profile-end-cap": "profile end cap behaviour"},
        focus="Edit/merge/endcap phải giữ validity; test seam và hướng cap."),
    "OM9-L3-04-02-007": dict(
        rcore=["RCORE-07", "RCORE-09", "RCORE-02"],
        mx={"mx-ring-resizer": "Ring Resizer command, options, defaults and mesh settings",
            "mx-manual-ring-resizer": "Ring Resizer section: Symmetrical ON, Add Rings ON, Save to File, STL Options"},
        focus="Ring Resizer tạo mesh output từ rail hợp lệ; ghi rõ option/default thật (Symmetrical ON, Add Rings ON) và đơn vị size."),
    "OM9-L3-04-02-008": dict(
        rcore=["RCORE-07", "RCORE-09"],
        mx={"mx-matrixbuilder": "sweep builder re-evaluation",
            "mx-manual-history": "associative re-evaluation after edit",
            "mx-builder-ring": "profile sweep builder family"},
        focus="Sweep phải re-evaluate sau khi sửa profile/ring size; so geometry + history ID trước/sau."),
    "OM9-L3-04-02-009": dict(
        rcore=["RCORE-04", "RCORE-05"],
        mx={"mx-tube-handles": "MatrixHandle dimension handles",
            "mx-builder-base": "builder handle plumbing",
            "mx-manual-builders": "viewport handle behaviour"},
        focus="Handle width/height/position phải đồng bộ param và không tạo object thừa khi cancel."),

    # ================= M04: L2-04-03 ring builder family =================
    "OM9-L3-04-03-001": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-builder-ring-family": "frmICBuilderEternity + ctrGemSelectSmall",
            "mx-manual-eternity-band-builder": "Eternity Band Builder command, options and defaults",
            "mx-builder-base": "builder host contract"},
        focus="Bố trí đá lặp + tham số band phải tái lập; ghi rõ default thật từ sheet eternity-band-builder."),
    "OM9-L3-04-03-002": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-builder-ring-family": "frmICBuilderSignet (VB6)",
            "mx-matrixbuilder": "TSplineSignetBuilder/Command (managed)",
            "mx-manual-signet-builder": "Signet Builder command, options and defaults"},
        focus="Top face/shoulder/shank profile phải test riêng từng nhóm tham số; đối chiếu option thật của Signet Builder."),
    "OM9-L3-04-03-003": dict(
        rcore=["RCORE-07"],
        mx={"mx-builder-ring-family": "frmBuilderRaisedBand + modICRaisedBand",
            "mx-manual-raised-band-builder": "Raised Band Builder command, options and defaults"},
        focus="Band nâng + rail profile phải có fixture riêng; ghi rõ default và mối quan hệ với profile placer."),
    "OM9-L3-04-03-004": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-builder-ring-family": "frmBuilderAwardRingBuilder (ctrInBox/ctrSliderMenu)",
            "mx-manual-award-ring-builder": "Award Ring Builder command, options and defaults"},
        focus="Cấu trúc top/shoulder kiểu signet phải tái lập; không nhân đôi thuật toán của Signet nếu khác contract."),
    "OM9-L3-04-03-005": dict(
        rcore=["RCORE-10"],
        mx={"mx-matrix-style": "builder style presets",
            "mx-manual-styles": "style behaviour for ring builders",
            "mx-manual-save-styles": "save/load style workflow"},
        focus="Preset shape + tham số phải persist và không đè default; test round-trip theo builder mode."),
    "OM9-L3-04-03-006": dict(
        rcore=["RCORE-09", "RCORE-07"],
        mx={"mx-manual-history": "history preservation on ring edit/resize",
            "mx-builder-ring-family": "builder family that owns the inputs",
            "mx-manual-ring-resizer": "resize path that is explicitly not history-related"},
        focus="Giữ input khi resize/edit; lưu ý Ring Resizer được tài liệu ghi là KHÔNG history-related - không gán nhầm."),
    "OM9-L3-04-03-007": dict(
        rcore=["RCORE-01", "RCORE-08", "RCORE-02"],
        mx={"mx-ring-resizer": "mesh settings that gate ring output quality",
            "mx-manual-ring-resizer": "valid rail/closed-object precondition for mesh output",
            "mx-frm-dialog-options": "mesh option vocabulary"},
        focus="Gate closed solid + wall thickness tối thiểu; đối chiếu precondition thật trong sheet ring-resizer."),

    # ================= M04: L2-04-19 metal weight =================
    "OM9-L3-04-19-001": dict(
        rcore=["RCORE-01", "RCORE-02", "RCORE-08"],
        mx={"mx-metal-weight": "MetalWeights volume path",
            "mx-manual-metal-weights": "metal weight section volume/units discussion"},
        focus="Volume mm³ từ closed B-Rep; test open/shell phải bị từ chối chứ không trả số ước lượng."),
    "OM9-L3-04-19-002": dict(
        rcore=["RCORE-02"],
        mx={"mx-metal-weight": "MetalWeights material list/density source",
            "mx-manual-metal-weights": "metal weight section material handling",
            "mx-manual-metal-weights-object-materials": "Object Materials note (applied VRay material only)"},
        focus="Registry density + đơn vị (g/cm³) có provenance; giá trị test-only phải gắn nhãn, không trộn với catalog thật."),
    "OM9-L3-04-19-003": dict(
        rcore=["RCORE-02"],
        mx={"mx-metal-weight": "MetalObject mass calculation",
            "mx-manual-metal-weights": "estimated mass computation and reporting"},
        focus="Mass = volume × density với hệ số mm³→cm³ = 1/1000; test sai số theo tolerance đã khai báo."),
    "OM9-L3-04-19-004": dict(
        rcore=["RCORE-02", "RCORE-06"],
        mx={"mx-metal-weight": "MetalMaterialObjectList/MetalWeightsMaterialView per-material breakdown",
            "mx-manual-metal-weights": "multi-material reporting behaviour",
            "mx-userdata": "userdata carriers for material assignment"},
        focus="Tổng theo từng loại kim loại + theo selection; test object nhiều material và object chưa gán."),
    "OM9-L3-04-19-005": dict(
        rcore=["RCORE-08", "RCORE-02"],
        mx={"mx-metal-weight": "MetalWeights error/open-geometry policy surface",
            "mx-manual-metal-weights": "validity requirement stated for weight results",
            "mx-frm-messages": "uncertainty/warning reporting"},
        focus="Từ chối/đánh dấu không chắc chắn cho open B-Rep; không trả gram như thể đã xác nhận."),
    # ================= M05: gems and layout =================
    "OM9-L3-04-09-001": dict(
        rcore=["RCORE-01", "RCORE-02"],
        mx={"mx-gem-util": "typed gem definition contrast",
            "mx-gem-data": "gem size/cut data and store",
            "mx-manual-gem-loader": "Gem Loader command and dimension fields",
            "mx-manual-placing-gems": "gem placement overview and stated defaults"},
        focus="Schema cut/shape/diameter/depth phải typed, đơn vị mm, validate biên; không suy default từ sheet."),
    "OM9-L3-04-09-002": dict(
        rcore=["RCORE-01", "RCORE-10"],
        mx={"mx-gem-data": "gem instance definition/store",
            "mx-gem-util": "GemDefinition/GemBlock asset layer",
            "mx-manual-gem-loader": "reference asset expectations in the loader"},
        focus="Asset tham chiếu phải có identity/version; render mesh không thay B-Rep chuẩn."),
    "OM9-L3-04-09-003": dict(
        rcore=["RCORE-01", "RCORE-04"],
        mx={"mx-gem-declared-builders": "frmICBuilderGemLoader legacy panel",
            "mx-manual-gem-loader": "Gem Loader: shape/cut/size picker and options",
            "mx-manual-placing-gems": "gem creation workflow"},
        focus="Tạo một gem theo shape/size đã chọn; picker không tạo solver thứ hai và sai input phải bị từ chối."),
    "OM9-L3-04-09-004": dict(
        rcore=["RCORE-01", "RCORE-07"],
        mx={"mx-gem-declared-builders": "frmICBuilderCustomGem panel",
            "mx-manual-custom-gem-builder": "Custom Gem Builder: profile-based gem workflow",
            "mx-gem-data": "gem size data used by the custom shape"},
        focus="Gem từ profile người dùng phải giữ profile gốc và cho phép sửa lại; không hard-code tỷ lệ."),
    "OM9-L3-04-09-005": dict(
        rcore=["RCORE-01"],
        mx={"mx-gem-declared-builders": "frmICBuilderEmerald / frmICBuilderBaguette",
            "mx-manual-emerald-builder": "Emerald Builder dimensions and options",
            "mx-manual-baguette-builder": "Baguette Builder dimensions and options"},
        focus="Emerald/baguette phải sửa được chiều theo đúng ràng buộc cut; sheet cho biết option, không chứng minh hình học."),
    "OM9-L3-04-09-006": dict(
        rcore=["RCORE-07", "RCORE-06"],
        mx={"mx-gem-instance": "MatrixGemInstance records",
            "mx-gem-util": "GemInstance transform model",
            "mx-manual-placing-gems": "instance placement behaviour"},
        focus="Instance transform thay vì nhân bản mesh; test transform + ID ổn định khi copy."),
    "OM9-L3-04-09-007": dict(
        rcore=["RCORE-06", "RCORE-10"],
        mx={"mx-gem-declared-builders": "gem defaults inside the legacy panels",
            "mx-mod-menu-controls": "layer/default routing surface",
            "mx-manual-placing-gems": "gem layer/default statements"},
        focus="Default gem/layer/attribute phải cấu hình được và persist; không trộn default của builder với setting người dùng."),
    "OM9-L3-04-09-008": dict(
        rcore=["RCORE-01", "RCORE-07"],
        mx={"mx-gem-data": "gem geometry/size access used downstream",
            "mx-gem-util": "facet/girdle reference helpers",
            "mx-manual-emerald-builder": "gem dimension semantics referenced by cutters"},
        focus="API facet/outline/girdle là hợp đồng cho M06/M07; phải typed và ổn định, không phơi chi tiết nội bộ."),

    "OM9-L3-04-10-001": dict(
        rcore=["RCORE-07", "RCORE-05"],
        mx={"mx-gem-on-curve-builders": "gem-on-curve builder family",
            "mx-manual-gem-on-curve": "Gem On Curve command, spacing/size options",
            "mx-manual-placing-gems": "placement overview"},
        focus="Đặt đá đều trên curve: đo khoảng cách thực và frame orientation; sheet cho đường lệnh/option."),
    "OM9-L3-04-10-002": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-on-curve-builders": "GemOnCurveAdvancedBuilder",
            "mx-manual-gem-on-curve-advanced": "Gem On Curve Advanced: size/spacing/orientation rules",
            "mx-gem-data": "gem size lookup for variable sizes"},
        focus="Size/spacing/orientation thay đổi dọc curve phải tái lập được; đo từng frame, không chỉ tổng số đá."),
    "OM9-L3-04-10-003": dict(
        rcore=["RCORE-07", "RCORE-02"],
        mx={"mx-gem-on-curve-builders": "GemCountOnCurveBuilder",
            "mx-manual-gem-count-on-curve": "Gem Count On Curve: length/spacing/count options"},
        focus="Đếm số đá vừa length+spacing phải khớp công thức và làm tròn đã ghi; test biên length/spacing."),
    "OM9-L3-04-10-004": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-on-curve-builders": "GemListOnCurveBuilder",
            "mx-manual-gem-list-on-curve": "Gem List On Curve: heterogeneous list behaviour",
            "mx-gem-data": "per-gem size/cut data"},
        focus="Danh sách đá khác shape/size phải tiêu thụ đúng thứ tự và hết list phải báo rõ."),
    "OM9-L3-04-10-005": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-on-curve-builders": "GemOnCurveMultiBuilder",
            "mx-manual-gem-on-curve-multi": "Gem On Curve Multi: multiple gem lines",
            "mx-manual-gem-on-curve-advanced": "multi-line option context"},
        focus="Nhiều dòng đá trên nhiều path: mỗi dòng giữ owner/param riêng, không nhân bản solver."),
    "OM9-L3-04-10-006": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-gem-util": "GemOnCurve rail/guide helpers",
            "mx-manual-gem-profile-curve": "Gem Profile Curve command",
            "mx-manual-gem-guides": "Gem Guides command and options"},
        focus="Rail/guide curve và gem reference curve phải là geometry thật có thể kiểm; sheet chỉ mô tả đường lệnh."),
    "OM9-L3-04-10-007": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-gem-declared-builders": "frmICBuilderGemOnSurface panel",
            "mx-manual-gem-on-surface": "Gem On Surface: UV/normal placement",
            "mx-gem-data": "gem frame data"},
        focus="Frame theo UV+normal phải đo được; trim/seam ảnh hưởng placement phải xử lý tường minh."),
    "OM9-L3-04-10-008": dict(
        rcore=["RCORE-09", "RCORE-07"],
        mx={"mx-gem-declared-builders": "frmICBuilderGemFollow panel",
            "mx-manual-gem-follow": "Gem Follow behaviour when the base changes",
            "mx-manual-history": "associative update contract"},
        focus="Đá cập nhật khi curve/surface nền đổi: test ID và vị trí trước/sau, không chỉ đếm lại."),
    "OM9-L3-04-10-009": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "legacy gem flow entry",
            "mx-manual-gem-flow": "Gem Flow: base-to-target mapping options",
            "mx-object-helper": "transform helper used by flow"},
        focus="Flow base→target phải giữ hình dạng instance; test rigid vs deformed và sai số."),
    "OM9-L3-04-10-010": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "legacy gem splop entry",
            "mx-manual-gem-splop": "Gem Splop: surface placement with sphere reference",
            "mx-manual-pav-sphere": "sphere-reference placement case"},
        focus="Splop dùng sphere reference: đo orientation và vị trí tiếp xúc; không suy từ tên lệnh."),
    "OM9-L3-04-10-011": dict(
        rcore=["RCORE-09", "RCORE-06"],
        mx={"mx-gem-instance": "gem instance identity records",
            "mx-gem-map": "gem map association",
            "mx-manual-gem-update": "Gem Update: refresh from stored attributes"},
        focus="Liên kết gem↔path/surface phải ổn định và có policy khi nguồn bị xoá; test broken link."),

    "OM9-L3-04-11-001": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "frmCHaloBuilder panel",
            "mx-manual-halo-builder": "Halo Builder options and placement rules",
            "mx-gem-data": "melee gem size lookup for the halo"},
        focus="Halo quanh đá chính: đường kính/number/offset phải đo; sheet cho option chứ không phải oracle."),
    "OM9-L3-04-11-002": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "frmBuilderClusterBuilderOld panel",
            "mx-manual-cluster-builder": "Cluster Builder layout options",
            "mx-stone-grouping": "cluster/stone grouping records"},
        focus="Cluster quanh tâm: phân bố không chồng lấn phải kiểm bằng đo, không chỉ bằng ảnh."),
    "OM9-L3-04-11-003": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "frmICBuilderChannel / frmICBuilderBaguette",
            "mx-manual-baguette-channel": "Baguette Channel command and options",
            "mx-gem-data": "baguette dimension data"},
        focus="Baguette trong channel/ring segment: ràng buộc bề rộng channel phải validate."),
    "OM9-L3-04-11-004": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "frmICBuilderTaperBagBetweenTwoCurves",
            "mx-manual-tapered-baguette-between-two-curves": "Tapered Baguette Between Two Curves",
            "mx-manual-baguette-builder": "tapered baguette dimension options"},
        focus="Baguette taper giữa hai guide: đo taper ratio và orientation từng viên."),
    "OM9-L3-04-11-005": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "frmICBuilderTaperBagChannel",
            "mx-manual-gems-between-2-curves": "Gems Between 2 Curves command",
            "mx-manual-baguette-channel": "channel context"},
        focus="Layout trong kênh giới hạn bởi hai curve: xử lý curve không song song và thiếu chỗ."),
    "OM9-L3-04-11-006": dict(
        rcore=["RCORE-07", "RCORE-03"],
        mx={"mx-gem-declared-builders": "frmICBuilderOrientToGem panel",
            "mx-manual-orient-to-gem": "Orient To Gem command and options",
            "mx-object-helper": "orientation transform helper"},
        focus="Orient theo frame của đá: phải kiểm trục và dấu; test với gem đã xoay."),
    "OM9-L3-04-11-007": dict(
        rcore=["RCORE-09", "RCORE-04"],
        mx={"mx-gem-declared-builders": "halo/cluster builder panels",
            "mx-builder-base": "builder host contract for parameter edits",
            "mx-manual-halo-builder": "builder reopen/edit behaviour"},
        focus="Sửa tham số sau khi tạo phải qua builder/history, không tạo đá trùng; test undo một bước."),

    "OM9-L3-04-12-001": dict(
        rcore=["RCORE-07", "RCORE-12"],
        mx={"mx-gem-declared-builders": "frmICBuilderPave panel",
            "mx-manual-auto-pav-builder": "Auto Pave Builder options and defaults",
            "mx-gem-data": "pave stone size data"},
        focus="Auto pave sinh candidate trên vùng surface: mật độ/biên phải đo; candidate không phải commit."),
    "OM9-L3-04-12-002": dict(
        rcore=["RCORE-04", "RCORE-07"],
        mx={"mx-gem-declared-builders": "frmICBuilderPave / PaveProng panels",
            "mx-manual-pav-builder": "Pav Builder manual placement options",
            "mx-manual-auto-pav-builder": "region/density control context"},
        focus="Pave thủ công: người dùng kiểm soát vị trí/khoảng; test thêm/xoá một viên không phá layout khác."),
    "OM9-L3-04-12-003": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "frmICBuilderPaveAzure panel",
            "mx-manual-pav-azure-builder": "Pav Azure Builder command and options"},
        focus="Azure phải chuẩn bị context đặt đá đúng như tài liệu; không tự sinh geometry ngoài phạm vi."),
    "OM9-L3-04-12-004": dict(
        rcore=["RCORE-07", "RCORE-06"],
        mx={"mx-gem-declared-builders": "frmICBuilderPaveProng panel",
            "mx-manual-pav-prong-builder": "Pav Prong Builder command and options",
            "mx-manual-baguette-channel": "channel/prong context"},
        focus="Prong marker chia sẻ giữa các viên: đếm và vị trí phải khớp layout; prong vật lý thuộc M06."),
    "OM9-L3-04-12-005": dict(
        rcore=["RCORE-07"],
        mx={"mx-gem-declared-builders": "legacy pave sphere entry",
            "mx-manual-pav-sphere": "Pav Sphere: spherical layout options",
            "mx-manual-gem-splop": "sphere-reference placement case"},
        focus="Layout trên cầu: đo phân bố và không chồng lấn; kiểm cả cực của sphere."),
    "OM9-L3-04-12-006": dict(
        rcore=["RCORE-07", "RCORE-08"],
        mx={"mx-gem-declared-builders": "frmBuilderGemSprings panel",
            "mx-manual-gem-springs": "Gem Springs relaxation behaviour",
            "mx-gem-data": "gem size used by the relaxation"},
        focus="Relaxation/repel phải hội tụ và không đổi số đá; ghi ngưỡng dừng và sai số còn lại."),
    "OM9-L3-04-12-007": dict(
        rcore=["RCORE-04", "RCORE-12"],
        mx={"mx-gem-declared-builders": "pave region controls in the legacy panels",
            "mx-manual-auto-pav-builder": "density/border options",
            "mx-manual-pav-builder": "region control behaviour"},
        focus="Density/border/gap sửa tương tác: preview phải ephemeral, Accept một transaction."),
    "OM9-L3-04-12-008": dict(
        rcore=["RCORE-09", "RCORE-06"],
        mx={"mx-gem-instance": "gem ID preservation records",
            "mx-gem-map": "gem map refresh",
            "mx-manual-gem-update": "Gem Update behaviour"},
        focus="Sửa surface phải giữ gem ID/vị trí theo policy; test ID trước/sau và broken-link reporting."),

    "OM9-L3-04-13-001": dict(
        rcore=["RCORE-04", "RCORE-05"],
        mx={"mx-gem-declared-builders": "gem positioner handle surface",
            "mx-manual-gem-positioner-msr": "Gem Positioner (MSR) handles and modes",
            "mx-msr": "MSR handle implementation precedent"},
        focus="Move/scale/rotate bằng handle: mỗi drag = một bước undo; đo transform thực thi."),
    "OM9-L3-04-13-002": dict(
        rcore=["RCORE-09", "RCORE-01"],
        mx={"mx-gem-declared-builders": "frmICBuilderGemControl panel",
            "mx-manual-gem-control": "Gem Control attributes and size edits",
            "mx-gem-data": "gem dimension data"},
        focus="Sửa attribute/kích thước phải cập nhật geometry theo policy; test sai số và giới hạn size."),
    "OM9-L3-04-13-003": dict(
        rcore=["RCORE-06"],
        mx={"mx-gem-declared-builders": "match-attribute entry",
            "mx-manual-match-attributes": "Match Attributes command and options",
            "mx-matrix-style": "attribute reuse model"},
        focus="Copy cut/shape/size giữa các viên: chỉ copy thuộc tính tương thích, ghi rõ phần bỏ qua."),
    "OM9-L3-04-13-004": dict(
        rcore=["RCORE-10"],
        mx={"mx-gem-declared-builders": "gem style save entry",
            "mx-manual-save-styles": "Save Styles command and options",
            "mx-matrix-style": "style preset store"},
        focus="Lưu gem style vào preset chia sẻ: không sửa default, test ghi đè/tên trùng."),
    "OM9-L3-04-13-005": dict(
        rcore=["RCORE-10"],
        mx={"mx-gem-declared-builders": "gem style load entry",
            "mx-manual-load-styles": "Load Styles command and options",
            "mx-matrix-style": "style preset store"},
        focus="Nạp gem style: áp đúng thuộc tính và báo khi style không tương thích."),
    "OM9-L3-04-13-006": dict(
        rcore=["RCORE-09", "RCORE-01"],
        mx={"mx-gem-instance": "instance records refreshed by update",
            "mx-manual-gem-update": "Gem Update: regenerate from stored attributes",
            "mx-gem-data": "stored attribute lookup"},
        focus="Regenerate gem từ attribute đã lưu: geometry đổi, ID giữ; test attribute thiếu/đổi schema."),
    "OM9-L3-04-13-007": dict(
        rcore=["RCORE-12"],
        mx={"mx-gem-map": "GemMap overlay",
            "mx-manual-gem-map": "Gem Map: numbers/positions/sizes overlay",
            "mx-gem-declared-builders": "legacy gem map surface"},
        focus="Overlay chỉ hiển thị vị trí/số/size: không mutate model và không tạo object."),
    "OM9-L3-04-13-008": dict(
        rcore=["RCORE-02", "RCORE-08"],
        mx={"mx-gem-declared-builders": "frmICBuilderGemReporter panel",
            "mx-manual-gem-reporter": "Gem Reporter counts and weight reporting",
            "mx-metal-weight": "mass calculation reuse"},
        focus="Đếm và cân nặng theo shape/size: ghi đơn vị và tolerance; thiếu dữ liệu phải báo chứ không đoán."),
    "OM9-L3-04-13-009": dict(
        rcore=["RCORE-09", "RCORE-06"],
        mx={"mx-gem-instance": "stable gem IDs",
            "mx-gem-map": "ID association in the map",
            "mx-manual-gem-control": "attribute edit behaviour"},
        focus="Giữ ID/liên kết khi sửa attribute: test ID và history link trước/sau; conflict phải tường minh."),
}
# M03 rows are generated (tools/gen_m03_anchors.py) from the authored sheet
# crosswalk; they are merged last so a hand edit here can still override one.
for _l3_id, _row in M03_ANCHORS.items():
    L3_ANCHORS.setdefault(_l3_id, _row)

# L3-level default for M00 rows that carry no topic-specific anchor.
M00_FALLBACK = {}

# Short role text used when a Matrix source needs a default role.
DEFAULT_MX_ROLE = "Matrix9 reference surface for this capability"

# --------------------------------------------------------------------------
# MX_MANUAL: authored reference sheets shipped inside OM9_Milestones
# (reference/text/*.md). Each sheet transcribes one Matrix manual section with
# its command path, option list and stated defaults. Anchors are resolved to a
# concrete file by sheet id through tools/matrix_ref_sources.py, so a missing
# sheet fails the crosswalk validator instead of silently dropping the anchor.
# --------------------------------------------------------------------------
MANUAL = {
    "mx-manual-rhino-history": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="43-44",
        sheet="m8book1__p43-44__023-rhino-history",
        section="Rhino History", topic="Rhino History engine contract as documented for Matrix users"),
    "mx-manual-history": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="48-53",
        sheet="m8book1__p48-53__031-history",
        section="History", topic="Matrix History family: record, update, clear and reporting"),
    "mx-manual-history-record": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="44",
        sheet="m8book1__p44-44__025-matrix-history-record",
        section="Matrix History Record", topic="History Record command and its options"),
    "mx-manual-history-update": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="44",
        sheet="m8book1__p44-44__024-matrix-history-update",
        section="Matrix History Update", topic="History Update command and its options"),
    "mx-manual-matrix-clear-history": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="44",
        sheet="m8book1__p44-44__026-matrix-clear-object-history",
        section="Matrix Clear Object History", topic="Clear Object History scope per object"),
    "mx-manual-join-history": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="149",
        sheet="m8book2__p149-149__154-join-history",
        section="Join History", topic="history-enabled edit case"),
    "mx-manual-command-history": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="36",
        sheet="m8book1__p36-36__013-command-history",
        section="Command History", topic="command transcript panel behaviour"),
    "mx-manual-builders-vs-commands": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="27-29",
        sheet="m8book1__p27-29__006-builders-vs-commands",
        section="Builders vs Commands", topic="what a Matrix builder is versus a plain command"),
    "mx-manual-builders": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="114-115",
        sheet="m8book2__p114-115__128-builders",
        section="Builders", topic="builder panel behaviour and entry points"),
    "mx-manual-f6-menu": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="19-21",
        sheet="m8book1__p19-21__004-the-f6-menu",
        section="The F6 Menu", topic="F6 context menu behaviour and builder prefill"),
    "mx-manual-styles": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="30-31",
        sheet="m8book1__p30-31__007-styles",
        section="Styles", topic="style/preset concept and compatibility"),
    "mx-manual-save-styles": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="235-236",
        sheet="m8book2__p235-236__210-save-styles",
        section="Save Styles", topic="save-style workflow, naming and overwrite"),
    "mx-manual-load-styles": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="236",
        sheet="m8book2__p236-236__211-load-styles",
        section="Load Styles", topic="load/apply style workflow"),
    "mx-manual-match-attributes": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="235",
        sheet="m8book2__p235-235__209-match-attributes",
        section="Match Attributes", topic="attribute matching command and options"),
    "mx-manual-ring-rail": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="134",
        sheet="m8book2__p134-134__140-ring-rail",
        section="Ring Rail", topic="Ring Rail command, size standard and options"),
    "mx-manual-ring-rail-tools": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="132-134",
        sheet="m8book2__p132-134__139-tools-menu",
        section="Tools menu", topic="Tools menu placement of ring/rail/resizer commands"),
    "mx-manual-ring-rail-book1": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="87-88",
        sheet="m8book1__p87-88__073-ring-rail",
        section="Ring Rail", topic="Ring Rail section in book 1"),
    "mx-manual-outside-ring-rail": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="139-140",
        sheet="m8book2__p139-140__142-outside-ring-rail",
        section="Outside Ring Rail", topic="Outside Ring Rail command and offset behaviour"),
    "mx-manual-custom-rail": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="140-141",
        sheet="m8book2__p140-141__143-custom-rail",
        section="Custom Rail", topic="Custom Rail options"),
    "mx-manual-custom-rail-advanced": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="163-164",
        sheet="m8book2__p163-164__173-custom-rail-advanced",
        section="Custom Rail Advanced", topic="advanced custom rail options"),
    "mx-manual-four-rail-profile": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="141",
        sheet="m8book2__p141-141__144-four-rail-profile",
        section="Four Rail Profile", topic="Four Rail Profile command and options"),
    "mx-manual-profile-placer": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="135-139",
        sheet="m8book2__p135-139__141-profile-placer",
        section="Profile Placer", topic="Profile Placer command, options and defaults"),
    "mx-manual-profile": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="160-161",
        sheet="m8book2__p160-161__170-profile",
        section="Profile", topic="profile selection and editing"),
    "mx-manual-profile-merge": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="162",
        sheet="m8book2__p162-162__171-profile-merge",
        section="Profile Merge", topic="profile merge behaviour"),
    "mx-manual-profile-end-cap": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="162-163",
        sheet="m8book2__p162-163__172-profile-end-cap",
        section="Profile End Cap", topic="profile end cap behaviour"),
    "mx-manual-ring-resizer": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="143-144",
        sheet="m8book2__p143-144__146-ring-resizer",
        section="Ring Resizer", topic="Ring Resizer command, outputs, options and stated defaults"),
    "mx-manual-eternity-band-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="116-117",
        sheet="m8book2__p116-117__129-eternity-band-builder",
        section="Eternity Band Builder", topic="Eternity Band Builder command, options and defaults"),
    "mx-manual-signet-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="117-120",
        sheet="m8book2__p117-120__130-signet-builder",
        section="Signet Builder", topic="Signet Builder command, options and defaults"),
    "mx-manual-raised-band-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="121-123",
        sheet="m8book2__p121-123__132-raised-band-builder",
        section="Raised Band Builder", topic="Raised Band Builder command, options and defaults"),
    "mx-manual-award-ring-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="123-127",
        sheet="m8book2__p123-127__133-award-ring-builder",
        section="Award Ring Builder", topic="Award Ring Builder command, options and defaults"),
    "mx-manual-metal-weights": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="142-143",
        sheet="m8book2__p142-143__145-metal-weights",
        section="Metal Weights", topic="Metal Weights command, material selection and reporting"),
    "mx-manual-object-materials": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="143",
        sheet="m8book2__p143-144__146-ring-resizer",
        section="Use Object Materials", topic="Object Materials note: weight reflects the applied material only"),

    # -- M05 gem and layout sheets -----------------------------------------
    "mx-manual-placing-gems": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="174-175",
        sheet="m8book2__p174-175__placing-gems",
        section="Placing Gems", topic="gem placement overview and stated defaults"),
    "mx-manual-gem-loader": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="176-177",
        sheet="m8book2__p176-177__gem-loader",
        section="Gem Loader", topic="Gem Loader shape/cut/size picker and options"),
    "mx-manual-gem-on-curve": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="177-181",
        sheet="m8book2__p177-181__gem-on-curve",
        section="Gem On Curve", topic="Gem On Curve command, spacing and sizing options"),
    "mx-manual-gem-on-curve-advanced": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="181-187",
        sheet="m8book2__p181-187__gem-on-curve-advanced",
        section="Gem On Curve Advanced", topic="variable size/spacing/orientation along a curve"),
    "mx-manual-gem-count-on-curve": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="187-191",
        sheet="m8book2__p187-191__gem-count-on-curve",
        section="Gem Count On Curve", topic="count stones fitting length and spacing"),
    "mx-manual-gem-list-on-curve": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="191-194",
        sheet="m8book2__p191-194__gem-list-on-curve",
        section="Gem List On Curve", topic="heterogeneous gem list along a curve"),
    "mx-manual-gem-on-curve-multi": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="195-197",
        sheet="m8book2__p195-197__gem-on-curve-multi",
        section="Gem On Curve Multi", topic="multiple gem lines on selected paths"),
    "mx-manual-gem-profile-curve": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="197-200",
        sheet="m8book2__p197-200__gem-profile-curve",
        section="Gem Profile Curve", topic="profile/rail curve generation for gem placement"),
    "mx-manual-gem-guides": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="200-202",
        sheet="m8book2__p200-202__gem-guides",
        section="Gem Guides", topic="gem reference guides and options"),
    "mx-manual-orient-to-gem": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="202",
        sheet="m8book2__p202-202__orient-to-gem",
        section="Orient To Gem", topic="orient companion objects to a gemstone frame"),
    "mx-manual-halo-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="203-207",
        sheet="m8book2__p203-207__halo-builder",
        section="Halo Builder", topic="Halo Builder options and placement rules"),
    "mx-manual-gem-on-surface": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="208-209",
        sheet="m8book2__p208-209__gem-on-surface",
        section="Gem On Surface", topic="UV/normal gem placement on a surface"),
    "mx-manual-custom-gem-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="209-212",
        sheet="m8book2__p209-212__the-custom-gem-builder",
        section="The Custom Gem Builder", topic="profile-based custom gem geometry"),
    "mx-manual-emerald-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="212-214",
        sheet="m8book2__p212-214__emerald-builder",
        section="Emerald Builder", topic="Emerald dimension editing and options"),
    "mx-manual-baguette-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="214-217",
        sheet="m8book2__p214-217__baguette-builder",
        section="Baguette Builder", topic="Baguette dimension editing and options"),
    "mx-manual-baguette-channel": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="217-218",
        sheet="m8book2__p217-218__baguette-channel",
        section="Baguette Channel", topic="baguette line inside a channel"),
    "mx-manual-tapered-baguette-between-two-curves": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="218-219",
        sheet="m8book2__p218-219__tapered-baguette-between-two-curves",
        section="Tapered Baguette Between Two Curves", topic="tapered baguette distribution between guides"),
    "mx-manual-gems-between-2-curves": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="219-220",
        sheet="m8book2__p219-220__gems-between-2-curves",
        section="Gems Between 2 Curves", topic="gem layout bounded by two curves"),
    "mx-manual-cluster-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="220-224",
        sheet="m8book2__p220-224__cluster-builder",
        section="Cluster Builder", topic="Cluster Builder layout options"),
    "mx-manual-gem-reporter": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="224",
        sheet="m8book2__p224-224__gem-reporter",
        section="Gem Reporter", topic="gem counts and weight reporting"),
    "mx-manual-gem-map": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="224-225",
        sheet="m8book2__p224-225__gem-map",
        section="Gem Map", topic="gem map overlay: numbers, positions, sizes"),
    "mx-manual-auto-pav-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="225-230",
        sheet="m8book2__p225-230__auto-pav-builder",
        section="Auto Pav Builder", topic="automatic pave stone generation options"),
    "mx-manual-pav-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="230-231",
        sheet="m8book2__p230-231__pav-builder",
        section="Pav Builder", topic="manual pave placement options"),
    "mx-manual-pav-azure-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="231",
        sheet="m8book2__p231-231__pav-azure-builder",
        section="Pav Azure Builder", topic="azure placement preparation"),
    "mx-manual-pav-prong-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="231-232",
        sheet="m8book2__p231-232__pav-prong-builder",
        section="Pav Prong Builder", topic="shared prong marker distribution"),
    "mx-manual-gem-springs": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="233",
        sheet="m8book2__p233-233__gem-springs",
        section="Gem Springs", topic="spring/relaxation adjustment of gem layouts"),
    "mx-manual-gem-follow": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="233-234",
        sheet="m8book2__p233-234__gem-follow",
        section="Gem Follow", topic="gems follow a changed underlying curve/surface"),
    "mx-manual-gem-control": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="234-235",
        sheet="m8book2__p234-235__gem-control",
        section="Gem Control", topic="gem attribute and size editing"),
    "mx-manual-gem-flow": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="236-238",
        sheet="m8book2__p236-238__gem-flow",
        section="Gem Flow", topic="flowing gem instances from base to target"),
    "mx-manual-gem-splop": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="238",
        sheet="m8book2__p238-238__gem-splop",
        section="Gem Splop", topic="sphere-reference surface placement"),
    "mx-manual-pav-sphere": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="238-240",
        sheet="m8book2__p238-240__pav-sphere",
        section="Pav Sphere", topic="pave layout on a spherical object"),
    "mx-manual-gem-update": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="240",
        sheet="m8book2__p240-240__gem-update",
        section="Gem Update", topic="refresh gem geometry from stored attributes"),
    "mx-manual-gem-positioner-msr": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="240-241",
        sheet="m8book2__p240-241__gem-positioner-msr",
        section="Gem Positioner MSR", topic="move/scale/rotate gems with handles"),

    # -- M06 head / bezel / prong / bead sheets ----------------------------
    "mx-manual-setting-menu": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="242",
        sheet="m8book2__p242-242__setting-menu",
        section="Setting menu", topic="Setting menu entry points"),
    "mx-manual-head-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="243-249",
        sheet="m8book2__p243-249__head-builder",
        section="Head Builder", topic="Head Builder modes, basket/prong rail and options"),
    "mx-manual-bezel-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="249-252",
        sheet="m8book2__p249-252__bezel-builder",
        section="Bezel Builder", topic="Bezel Builder full/partial modes and options"),
    "mx-manual-scallop-bezel": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="255",
        sheet="m8book2__p255-255__scallop-bezel",
        section="Scallop Bezel", topic="scallop bezel builder options"),
    "mx-manual-pull-object-to-rail": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="255-257",
        sheet="m8book2__p255-257__pull-object-to-rail",
        section="Pull Object to Rail", topic="pull object onto rail setting"),
    "mx-manual-prong-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="257-260",
        sheet="m8book2__p257-260__prong-builder",
        section="Prong Builder", topic="Prong Builder gem line and spacing options"),
    "mx-manual-prong-editor": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="260-261",
        sheet="m8book2__p260-261__prong-editor",
        section="Prong Editor", topic="Prong Editor selected-prong editing"),
    "mx-manual-prong-on-surface": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="261-263",
        sheet="m8book2__p261-263__prong-on-surface",
        section="Prong On Surface", topic="Prong On Surface placement options"),
    "mx-manual-bead-on-surface": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="263-264",
        sheet="m8book2__p263-264__bead-on-surface",
        section="Bead On Surface", topic="Bead On Surface options"),
    "mx-manual-metal-from-gems": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="264-268",
        sheet="m8book2__p264-268__metal-from-gems",
        section="Metal from Gems", topic="Metal Piece from Gems workflow and options"),
    "mx-manual-bead-on-curve": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="268-270",
        sheet="m8book2__p268-270__bead-on-curve",
        section="Bead On Curve", topic="Bead On Curve / milgrain options"),
    "mx-manual-channel-border-curve": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="270-271",
        sheet="m8book2__p270-271__channel-border-curve",
        section="Channel Border Curve", topic="channel border curve generation"),

    # -- M07 cutter and manufacturing sheets -------------------------------
    "mx-manual-creating-cutters": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="272-274",
        sheet="m8book2__p272-274__creating-cutters",
        section="Creating Cutters", topic="cutter concept, creation and reuse"),
    "mx-manual-gem-cutter": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="274-279",
        sheet="m8book2__p274-279__gem-cutter",
        section="Gem Cutter", topic="Gem Cutter options for one or many stones"),
    "mx-manual-gem-cutter-library": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="279-280",
        sheet="m8book2__p279-280__gem-cutter-library",
        section="Gem Cutter Library", topic="cutter library profiles and saving"),
    "mx-manual-channel-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="280-282",
        sheet="m8book2__p280-282__channel-builder",
        section="Channel Builder", topic="channel cutter through a line of gems"),
    "mx-manual-micro-prong-cutter": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="282-283",
        sheet="m8book2__p282-283__micro-prong-cutter",
        section="Micro Prong Cutter", topic="micro-prong relief along a stone line"),
    "mx-manual-bright-cut-channel-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="283-285",
        sheet="m8book2__p283-285__bright-cut-channel-builder",
        section="Bright Cut Channel Builder", topic="bright-cut channel for a gem row"),
    "mx-manual-bright-cutter": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="285-286",
        sheet="m8book2__p285-286__bright-cutter",
        section="Bright Cutter", topic="bright cutter around a single gem"),
    "mx-manual-bezel-cutter-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="286-288",
        sheet="m8book2__p286-288__bezel-cutter-builder",
        section="Bezel Cutter Builder", topic="bezel cutter builder options"),
    "mx-manual-plane-cube-cutters": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="288",
        sheet="m8book2__p288-288__plane-cube-cutters",
        section="Plane and Cube Cutters", topic="standard plane/cube cutter geometry"),
    "mx-manual-cut-to-finger-rail": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="288-290",
        sheet="m8book2__p288-290__cut-to-finger-rail",
        section="Cut to Finger Rail", topic="subtract the forbidden inside-finger volume"),
    "mx-manual-quad-flip-routines": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="290",
        sheet="m8book2__p290-290__quad-flip-routines",
        section="Quad Flip Routines", topic="mirror a quadrant to ring symmetry"),
    "mx-manual-boolean-builder": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="290-292",
        sheet="m8book2__p290-292__boolean-builder",
        section="Boolean Builder", topic="interactive boolean workflow on jewelry form"),
    "mx-manual-boolean-builder-m03": dict(
        manual="Matrix 8 Instruction Manual Book 2", pages="171-173",
        sheet="m8book2__p171-173__boolean-builder",
        section="Boolean Builder (early section)", topic="boolean builder as documented earlier"),
    "mx-manual-cutting-plane": dict(
        manual="Matrix 8 Instruction Manual Book 1", pages="217",
        sheet="m8book1__p217-217__cutting-plane",
        section="Cutting Plane", topic="cutting plane entry"),
}

# Aliases so a task can name the same sheet by the concept it is asked about.
MANUAL["mx-manual-metal-weights-object-materials"] = MANUAL["mx-manual-object-materials"]
# Authored sheets for every milestone command, generated from the sheet index.
MANUAL.update(MANUAL_SHEETS)

# --------------------------------------------------------------------------
# M06 head / bezel / prong / bead settings
# --------------------------------------------------------------------------
M06_ANCHORS = {
    "OM9-L3-04-14-001": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-head-bezel-builders": "frmICBuilderHead basket mode",
            "mx-manual-head-builder": "Head Builder: basket mode options and defaults",
            "mx-manual-setting-menu": "Setting menu entry point"},
        focus="Basket head: seat/prong rail phải đo được và validate B-Rep; sheet cho option chứ không phải oracle."),
    "OM9-L3-04-14-002": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-head-bezel-builders": "frmICBuilderHead half-bezel mode",
            "mx-manual-head-builder": "Head Builder half-bezel mode",
            "mx-bezel-managed": "managed bezel builder contrast"},
        focus="Half bezel trong Head Builder: phân biệt với Bezel Builder đầy đủ; kiểm seat/girdle bằng đo."),
    "OM9-L3-04-14-003": dict(
        rcore=["RCORE-07", "RCORE-04"],
        mx={"mx-head-bezel-builders": "prong rail level editing in the head panels",
            "mx-manual-head-builder": "prong rail level edit options",
            "mx-builder-base": "builder parameter edit contract"},
        focus="Sửa mức prong rail phải qua builder parameter, một bước undo; test giá trị biên."),
    "OM9-L3-04-14-004": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-bezel-managed": "CSBezel/TSplineBezelBuilder",
            "mx-head-bezel-builders": "frmICBuilderBezel panel",
            "mx-manual-bezel-builder": "Bezel Builder full mode options"},
        focus="Bezel đầy đủ: đo profile, seat và độ dày thành; không chấp nhận B-Rep hở khi claim solid."),
    "OM9-L3-04-14-005": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-bezel-managed": "TSplineUnderBezelBuilder / UnderGalleryBuilder",
            "mx-head-bezel-builders": "frmCUnderBezelTSplines panel",
            "mx-manual-bezel-builder": "under-bezel and gallery options"},
        focus="Under bezel + gallery: kiểm khoảng hở và gallery geometry; ghi rõ phần nào là display."),
    "OM9-L3-04-14-006": dict(
        rcore=["RCORE-07"],
        mx={"mx-head-bezel-builders": "scallop bezel entry",
            "mx-manual-scallop-bezel": "Scallop Bezel builder and options",
            "mx-bezel-managed": "bezel builder family"},
        focus="Scallop bezel: số/độ sâu scallop phải đo; không suy từ tên lệnh."),
    "OM9-L3-04-14-007": dict(
        rcore=["RCORE-10", "RCORE-12"],
        mx={"mx-head-bezel-builders": "profile browser inside the head/bezel panels",
            "mx-manual-head-builder": "profile selection behaviour",
            "mx-library-browser": "library browser core/panel split"},
        focus="Profile browser: chỉ đọc catalog profile, apply phải có xác nhận; thumbnail không phải bằng chứng."),
    "OM9-L3-04-14-008": dict(
        rcore=["RCORE-01", "RCORE-07"],
        mx={"mx-bezel-managed": "girdle/seat attachment geometry",
            "mx-manual-head-builder": "girdle/seat attachment statements",
            "mx-gem-data": "gem dimension data used by the seat"},
        focus="Gắn seat vào girdle phải dùng dữ liệu gem thật (từ M05) và đo tiếp xúc; không hard-code tỷ lệ."),
    "OM9-L3-04-14-009": dict(
        rcore=["RCORE-08", "RCORE-09"],
        mx={"mx-bezel-managed": "bezel validity/update path",
            "mx-manual-head-builder": "rebuild/update behaviour",
            "mx-manual-history": "associative update contract"},
        focus="Validity + update: kiểm closedness/thickness và test update sau khi sửa tham số; lỗi phải tường minh."),

    "OM9-L3-04-15-001": dict(
        rcore=["RCORE-07"],
        mx={"mx-prong-builders": "frmICBuilderProngEditor / gem line",
            "mx-prong-placer": "CSUtilProngPlacer placement rules",
            "mx-manual-prong-builder": "Prong Builder gem line and spacing options"},
        focus="Prong theo gem line: đếm và khoảng cách phải khớp layout; test line thiếu/đổi hướng."),
    "OM9-L3-04-15-002": dict(
        rcore=["RCORE-04", "RCORE-06"],
        mx={"mx-prong-builders": "frmICBuilderProngEditor selected-prong editing",
            "mx-manual-prong-editor": "Prong Editor behaviour and options",
            "mx-prong-placer": "prong placement/cache"},
        focus="Sửa prong đã chọn: chỉ đổi prong đó, giữ ID; test chọn nhiều prong."),
    "OM9-L3-04-15-003": dict(
        rcore=["RCORE-07", "RCORE-05"],
        mx={"mx-prong-builders": "frmICBuilderProngOnSurface panel",
            "mx-manual-prong-on-surface": "Prong On Surface placement options",
            "mx-prong-placer": "surface placement rules"},
        focus="Prong on surface: frame theo UV/normal phải đo; xử lý trim hole và thiếu chỗ."),
    "OM9-L3-04-15-004": dict(
        rcore=["RCORE-01", "RCORE-07"],
        mx={"mx-prong-builders": "frmICBuilderMetalPiece panel",
            "mx-manual-metal-from-gems": "Metal from Gems workflow and options",
            "mx-util-geometry": "geometry/validity helpers"},
        focus="Metal piece từ gems: dựng khối hợp lệ và đo thể tích; không dựa vào ảnh minh hoạ."),
    "OM9-L3-04-15-005": dict(
        rcore=["RCORE-07", "RCORE-06"],
        mx={"mx-prong-builders": "frmICBuilderChannel / BrightCutChannel",
            "mx-manual-channel-border-curve": "Channel Border Curve command and options",
            "mx-prong-placer": "channel data records"},
        focus="Channel border curve: hai biên phải là geometry thật và khép kín theo policy; test biên hở."),
    "OM9-L3-04-15-006": dict(
        rcore=["RCORE-07"],
        mx={"mx-bead-milgrain": "bead on curve entry",
            "mx-manual-bead-on-curve": "Bead On Curve / milgrain options",
            "mx-gem-data": "gem spacing data used by the bead line"},
        focus="Bead on curve/milgrain: kích thước và bước bead phải đo; test curve quá ngắn cho số bead yêu cầu."),
    "OM9-L3-04-15-007": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-bead-milgrain": "frmICBuilderBeadOnSurface panel",
            "mx-manual-bead-on-surface": "Bead On Surface options",
            "mx-util-geometry": "surface evaluation helper"},
        focus="Bead on surface: tiếp xúc surface phải đo; không nhận bead nằm trong trim hole."),
    "OM9-L3-04-15-008": dict(
        rcore=["RCORE-07", "RCORE-09"],
        mx={"mx-prong-builders": "pull-to-rail entry",
            "mx-manual-pull-object-to-rail": "Pull Object to Rail command and options",
            "mx-object-helper": "transform helper used by the pull"},
        focus="Pull object lên rail setting: đo biến dạng và giữ liên kết; test undo/cancel."),
    "OM9-L3-04-15-009": dict(
        rcore=["RCORE-07", "RCORE-02"],
        mx={"mx-prong-placer": "prong spacing rules",
            "mx-manual-prong-builder": "row end modes and spacing options",
            "mx-prong-builders": "legacy prong spacing UI"},
        focus="Row end mode + prong spacing: số prong và khoảng cuối hàng phải khớp policy; test biên."),
    "OM9-L3-04-15-010": dict(
        rcore=["RCORE-09", "RCORE-02", "RCORE-08"],
        mx={"mx-prong-placer": "associative edit and mass data",
            "mx-manual-history": "associative update contract",
            "mx-metal-weight": "mass calculation reuse from M04"},
        focus="Sửa associative + khối lượng: giữ ID/liên kết và ghi đơn vị; open B-Rep không được trả số như đã xác nhận."),
}

for _l3_id, _row in M06_ANCHORS.items():
    L3_ANCHORS.setdefault(_l3_id, _row)

# --------------------------------------------------------------------------
# M07 cutters and manufacturing validation
# --------------------------------------------------------------------------
M07_ANCHORS = {
    "OM9-L3-01-12-008": dict(
        rcore=["RCORE-10", "RCORE-01", "RCORE-12"],
        mx={"mx-export-import": "STL/native export path",
            "mx-mesh-helper": "mesh generation/repair backend",
            "mx-gem-data": "gem/instance records that must survive export",
            "mx-manual-mesh-from-nurbs": "mesh-from-NURBS settings used for STL output"},
        focus="STL watertight qua tessellator chung: kiểm closedness/winding/đơn vị mm trước khi ghi file; lỗi hình học phải báo."),
    "OM9-L3-04-16-001": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-cutter-forms": "frmICBuilderCutter azure mode",
            "mx-cutter-library": "cutter profile and entry module",
            "mx-manual-gem-cutter": "Gem Cutter options for a single stone",
            "mx-manual-creating-cutters": "cutter creation concept"},
        focus="Azure cutter dưới một viên đá: đo độ sâu/khoảng hở và validity sau boolean; sheet cho option."),
    "OM9-L3-04-16-002": dict(
        rcore=["RCORE-07", "RCORE-06"],
        mx={"mx-cutter-forms": "multi-stone cutter allocation",
            "mx-manual-gem-cutter": "Gem Cutter multiple-stones behaviour",
            "mx-gem-instance": "gem instance group that the cutter targets"},
        focus="Phân bổ cutter cho nhóm đá: mỗi viên giữ cutter riêng hoặc dùng chung theo policy đã ghi."),
    "OM9-L3-04-16-003": dict(
        rcore=["RCORE-10", "RCORE-06"],
        mx={"mx-cutter-library": "modCutterLibrary / modGemCutterLibrary / modICFileCutters",
            "mx-manual-gem-cutter-library": "Gem Cutter Library: pick, resize, save profiles",
            "mx-library-browser": "library browser core/panel split"},
        focus="Chọn và resize preset profile: catalog phải versioned; ghi rõ profile nào bị sửa và không đè default."),
    "OM9-L3-04-16-004": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-cutter-forms": "frmICBuilderBezelCutter",
            "mx-manual-bezel-cutter-builder": "Bezel Cutter Builder options",
            "mx-bezel-managed": "bezel geometry from M06"},
        focus="Bezel cutter cắt seat quanh bezel+gem: phải dùng dữ liệu bezel/gem thật và đo kết quả boolean."),
    "OM9-L3-04-16-005": dict(
        rcore=["RCORE-02", "RCORE-07"],
        mx={"mx-manual-gem-cutter": "pavilion clearance and depth options",
            "mx-cutter-forms": "legacy depth controls",
            "mx-gem-data": "gem depth data"},
        focus="Pavilion clearance/Z-offset/lower depth: ghi đơn vị và test biên âm/0; không hard-code tolerance."),
    "OM9-L3-04-16-006": dict(
        rcore=["RCORE-01", "RCORE-08"],
        mx={"mx-boolean-builder": "MatrixBoolean + boolean panel",
            "mx-manual-boolean-builder": "interactive boolean workflow",
            "mx-util-geometry": "validity helpers"},
        focus="Boolean trừ cutter khỏi metal: preview phải ephemeral, kết quả phải valid; test thất bại không để lại shape rác."),
    "OM9-L3-04-16-007": dict(
        rcore=["RCORE-04", "RCORE-09"],
        mx={"mx-boolean-builder": "preserve/remove option in the boolean workflow",
            "mx-manual-gem-cutter": "cutter body keep/delete statements",
            "mx-manual-history": "document state after the operation"},
        focus="Giữ hay xoá cutter object: một transaction, undo khôi phục đúng; không để cutter mồ côi."),

    "OM9-L3-04-17-001": dict(
        rcore=["RCORE-07"],
        mx={"mx-cutter-forms": "channel builder cutter",
            "mx-prong-builders": "frmICBuilderChannel from M06",
            "mx-manual-channel-builder": "Channel Builder cutter through a gem line"},
        focus="Channel cutter qua dãy đá: profile và chiều sâu phải đo; test dãy đá cong."),
    "OM9-L3-04-17-002": dict(
        rcore=["RCORE-07"],
        mx={"mx-cutter-forms": "frmICBuilderMicroProngCutter",
            "mx-manual-micro-prong-cutter": "Micro Prong Cutter options",
            "mx-prong-placer": "prong spacing rules from M06"},
        focus="Micro-prong relief dọc dãy đá: số/độ sâu scallop phải khớp spacing; không suy từ tên lệnh."),
    "OM9-L3-04-17-003": dict(
        rcore=["RCORE-07"],
        mx={"mx-cutter-forms": "frmICBuilderBrightCutChannel",
            "mx-manual-bright-cut-channel-builder": "Bright Cut Channel Builder options",
            "mx-manual-channel-builder": "channel context"},
        focus="Bright-cut channel cho hàng đá: đo góc cắt và độ sâu; kiểm kết quả sau boolean."),
    "OM9-L3-04-17-004": dict(
        rcore=["RCORE-07"],
        mx={"mx-cutter-forms": "frmICBuilderBrightCutter",
            "mx-manual-bright-cutter": "Bright Cutter around a single gem",
            "mx-gem-data": "gem outline used by the bright cut"},
        focus="Bright cutter quanh một viên: dùng outline gem thật; test gem xoay/không đối xứng."),
    "OM9-L3-04-17-005": dict(
        rcore=["RCORE-07", "RCORE-02"],
        mx={"mx-manual-channel-builder": "profile, end mirror and spacing options",
            "mx-cutter-forms": "legacy channel profile controls",
            "mx-cutter-library": "stored channel profiles"},
        focus="Profile + end mirror + spacing + local depth: mỗi tham số phải test riêng; ghi rõ mirror ảnh hưởng gì."),
    "OM9-L3-04-17-006": dict(
        rcore=["RCORE-01", "RCORE-04"],
        mx={"mx-boolean-builder": "transactional boolean on jewelry B-Rep",
            "mx-manual-boolean-builder": "multi-stone boolean workflow",
            "mx-util-geometry": "post-boolean validity"},
        focus="Boolean nhiều đá trong một transaction: hoặc thành công toàn bộ hoặc rollback; không để nửa vời."),

    "OM9-L3-04-18-001": dict(
        rcore=["RCORE-07", "RCORE-01"],
        mx={"mx-cut-to-finger-rail": "ring resizer / finger rail tools",
            "mx-manual-cut-to-finger-rail": "Cut to Finger Rail behaviour and options",
            "mx-ring-resizer": "ring size/rail controls from M04"},
        focus="Trừ vùng cấm trong lòng ngón: đo thể tích bị cắt và giữ rail hợp lệ; test size biên."),
    "OM9-L3-04-18-002": dict(
        rcore=["RCORE-01", "RCORE-07"],
        mx={"mx-quad-plane-cutters": "frmBuilderPlaneCutter / plane+cube cutters",
            "mx-manual-plane-cube-cutters": "Plane and Cube Cutters options",
            "mx-manual-cutting-plane": "Cutting Plane entry"},
        focus="Plane/cube cutter chuẩn: kích thước và hướng phải đo; không suy từ tên lệnh."),
    "OM9-L3-04-18-003": dict(
        rcore=["RCORE-07", "RCORE-06"],
        mx={"mx-quad-plane-cutters": "frmBuilderQuadFlip",
            "mx-manual-quad-flip-routines": "Quad Flip routines",
            "mx-manual-history": "symmetry preservation contract"},
        focus="Mirror một quadrant sang pattern đối xứng: kiểm ID và tính đối xứng bằng đo, không bằng mắt."),
    "OM9-L3-04-18-004": dict(
        rcore=["RCORE-01", "RCORE-04"],
        mx={"mx-boolean-builder": "union/difference/intersection workflow",
            "mx-manual-boolean-builder": "Boolean Builder operations",
            "mx-util-geometry": "validity helpers"},
        focus="Union/difference/intersection tương tác: mỗi phép phải validate và có nhánh lỗi tường minh."),
    "OM9-L3-04-18-005": dict(
        rcore=["RCORE-09", "RCORE-10"],
        mx={"mx-manual-history": "history/symmetry state in the project",
            "mx-manual-quad-flip-routines": "symmetry routine that must be replayed",
            "mx-manual-boolean-builder": "boolean order that must persist"},
        focus="Giữ symmetry và thứ tự boolean trong project state: test save/reopen và replay đúng thứ tự."),
    "OM9-L3-04-18-006": dict(
        rcore=["RCORE-01", "RCORE-08"],
        mx={"mx-export-import": "STL export gate",
            "mx-mesh-helper": "mesh/validity backend",
            "mx-manual-creating-cutters": "completed-ring validation expectation"},
        focus="Kiểm watertight trước khi xuất STL: ring hở/naked edge phải chặn export và báo rõ, không xuất file im lặng."),
}

for _l3_id, _row in M07_ANCHORS.items():
    L3_ANCHORS.setdefault(_l3_id, _row)

# M04 rows are generated (tools/gen_m04_anchors.py) so every authored sheet that
# ships under an L3 or its parent L2 is referenced automatically.
try:  # pragma: no cover - import guard for partial checkouts
    from matrix_m04_anchors import M04_ANCHORS
except ImportError:  # pragma: no cover
    M04_ANCHORS = {}
for _l3_id, _row in M04_ANCHORS.items():
    L3_ANCHORS[_l3_id] = _row

# --------------------------------------------------------------------------
# Rhino 5 geometry anchors attached to the L3 rows that need them.
#
# Applied after the generated M03/M04 rows are merged, because those rows are
# produced by tools/gen_m03_anchors.py and tools/gen_m04_anchors.py. A target that
# does not exist raises here rather than silently dropping the anchor.
# --------------------------------------------------------------------------
RHINO5_L3 = {

    "OM9-L3-01-13-001": {
        "rh5-common-file3dm": "hợp đồng Read/ReadWithLog/ReadArchiveVersion để dò chữ ký và version",
        "rh5-on-binary-archive": "chunk archive: BeginRead3dmChunk/Archive3dmVersion/ArchiveStartOffset",
    },
    "OM9-L3-01-13-002": {
        "rh5-common-unsafe-native": "bảng P/Invoke 1166 tên ON_* - C ABI mà bridge Rust phải khớp",
        "rh5-ghidra-opennurbs": "chữ ký hàm native openNURBS để đối chiếu bridge",
        "rh5-ghidra-rhcommon": "tầng rhcommon_c thực sự được gọi xuống",
    },
    "OM9-L3-01-13-003": {
        "rh5-common-units": "enum UnitSystem là nguồn ánh xạ đơn vị",
        "rh5-common-file3dm": "File3dmSettings: đơn vị và dung sai của document",
    },
    "OM9-L3-01-13-004": {
        "rh5-common-unsafe-native": "ON_Layer_GetLayerName / ON_3dmObjectAttributes_GetSetColor",
        "rh5-on-binary-archive": "bảng layer/group/attribute trong chunk 3dm",
    },
    "OM9-L3-01-13-005": {
        "rh5-on-nurbs-curve": "dựng và đánh giá NURBS curve: knot, degree, control point",
        "rh5-on-curve": "curve cơ sở: domain, trim, join, closest point",
        "rh5-common-nurbs-curve": "hình dạng API managed phải khớp",
    },
    "OM9-L3-01-13-006": {
        "rh5-on-brep": "B-rep có trim: AddTrimCurve/AddEdgeCurve/CloseTrimGap",
        "rh5-on-brep-parts": "face/edge/trim/loop/vertex - topology phải cập nhật khi import",
        "rh5-on-nurbs-surface": "NURBS surface: knot 2 chiều, seam, ruled/cone",
        "rh5-on-surface": "surface cơ sở: domain, iso, trim",
    },
    "OM9-L3-01-13-007": {
        "rh5-on-mesh": "mesh: face/vertex/normal, quads<->triangles, khối lượng",
        "rh5-on-binary-archive": "EnableSave3dmRenderMeshes / Save3dmAnalysisMeshes",
        "rh5-common-unsafe-native": "ON_Mesh_BrepToMeshSimple / ON_InstanceDefinition_GetObjectIds",
    },
    "OM9-L3-01-13-008": {
        "rh5-on-binary-archive": "chunk không nhận dạng được: đọc và bỏ qua an toàn",
        "rh5-common-file3dm": "ReadWithLog: log cảnh báo đối tượng không hỗ trợ",
    },
    "OM9-L3-01-13-009": {
        "rh5-common-file3dm": "Read/ReadWithLog là ranh giới transaction khi import",
        "rh5-on-brep": "CreateMesh/DeleteFace/DeleteSurface: dựng toàn bộ rồi mới công bố",
    },
    "OM9-L3-01-13-010": {
        "rh5-common-file3dm": "tuỳ chọn đọc (FileReadOptions) là bề mặt dialog",
        "rh5-common-units": "tuỳ chọn đơn vị đích",
    },
    "OM9-L3-01-13-011": {
        "rh5-common-file3dm": "log/warning của ReadWithLog là nguồn nội dung cảnh báo",
    },
    "OM9-L3-01-13-012": {
        "rh5-common-file3dm": "round-trip Read -> Write -> Read để đối chiếu golden fixture",
        "rh5-on-binary-archive": "chunk version và openNURBS version ghi ra file",
    },
    "OM9-L3-01-10-001": {
        "rh5-rh-util": "RhinoBooleanUnion + RhinoMeshBooleanIntersection: hình dạng phép hợp",
        "rh5-on-brep": "B-rep sau boolean: trim/loop/edge phải nhất quán",
    },
    "OM9-L3-01-10-002": {
        "rh5-rh-util": "RhinoBooleanDifference: hình dạng phép trừ",
        "rh5-on-brep": "B-rep sau phép trừ",
    },
    "OM9-L3-01-10-003": {
        "rh5-rh-util": "RhinoBooleanIntersection / RhinoIntersectSurfaces / RhinoIntersectBreps",
        "rh5-on-util": "ON_MeshMeshIntersect / ON_LineTriangleIntersect predicate giao",
    },
    "OM9-L3-01-10-004": {
        "rh5-rh-util": "RhinoBrepSplit / RhinoCurveBrepIntersect / RhinoCurveFaceIntersect",
        "rh5-on-util": "ON_TrimCurve / ON_JoinCurves / ON_SortCurves",
        "rh5-on-brep": "gán trim curve vào face: AddTrimCurve",
    },
    "OM9-L3-01-10-005": {
        "rh5-rh-util": "RhinoBrepCapPlanarHoles - đậy lỗ phẳng",
        "rh5-on-brep": "SolidOrientation và CloseTrimGap sau khi cap",
    },
    "OM9-L3-01-10-007": {
        "rh5-on-brep": "B-rep sau offset/thicken",
        "rh5-on-mass-properties": "Volume kiểm tra thể tích sau khi thicken",
    },
    "OM9-L3-01-09-001": {
        "rh5-on-util": "ON_BrepBox / ON_BrepCylinder / ON_BrepCone - primitive dựng sẵn",
    },
    "OM9-L3-01-09-002": {
        "rh5-on-util": "ON_BrepExtrude / ON_BrepExtrudeFace / ON_BrepExtrudeEdge",
    },
    "OM9-L3-01-09-003": {
        "rh5-on-util": "ON_BrepRevSurface / ON_BrepCone: dựng surface tròn xoay",
        "rh5-on-nurbs-surface": "surface có seam - kiểm tra seam sau revolve",
    },
    "OM9-L3-01-09-004": {
        "rh5-on-nurbs-surface": "CreateRuledSurface cho pipe/loft",
        "rh5-on-curve": "curve dẫn hướng: domain và chia",
    },
    "OM9-L3-01-09-005": {
        "rh5-on-util": "ON_BrepTrimmedPlane / ON_BrepFromMesh",
    },
    "OM9-L3-01-09-006": {
        "rh5-on-nurbs-surface": "CreateRuledSurface là dạng loft 2 tiết diện",
        "rh5-on-brep": "ghép mặt loft thành solid",
    },
    "OM9-L3-01-09-007": {
        "rh5-on-nurbs-surface": "CreateRuledSurface giữa các rail",
        "rh5-rh-util": "RhinoCutUpSurface / RhinoExtendSurface / RhinoFitSurface",
        "rh5-on-util": "ON_ForceMatchCurveEnds cho khớp đầu rail",
    },
    "OM9-L3-01-09-008": {
        "rh5-on-curve": "curve biên: join và trim trước khi dựng mặt",
        "rh5-rh-util": "RhinoCreateSurfaceFromCorners / RhinoJoinBrepNakedEdges",
    },
    "OM9-L3-01-11-003": {
        "rh5-on-xform": "transform 4x4 và UnitScale cho scale không đồng nhất",
        "rh5-common-units": "hệ đơn vị đích khi scale",
    },
    "OM9-L3-01-11-005": {
        "rh5-rh-util": "RhinoDivideCurve / RhinoDivideCurveEquidistant - chia đều theo cung",
        "rh5-on-xform": "đặt frame dọc curve",
    },
    "OM9-L3-01-11-006": {
        "rh5-on-nurbs-surface": "đánh giá theo UV: knot/domain surface",
        "rh5-on-xform": "dựng frame từ normal của surface",
    },
    "OM9-L3-04-02-001": {
        "rh5-rh-util": "RhinoDivideCurveEquidistant cho chia đều vành khuyên",
        "rh5-on-xform": "đặt và định hướng đá dọc vành",
    },
    "OM9-L3-04-02-004": {
        "rh5-on-nurbs-curve": "profile tuỳ biến dựng bằng NURBS: knot và degree",
    },
    "OM9-L3-04-02-007": {
        "rh5-on-mass-properties": "Volume/Area để tính lại khối lượng sau khi resizer",
        "rh5-on-xform": "UnitScale khi đổi cỡ vành",
    },
    "OM9-M00-L3-04-01": {
        "rh5-common-file3dm": "hợp đồng File3dm managed: Read/Write/ReadWithLog là bề mặt năng lực cần đạt",
        "rh5-ghidra-opennurbs": "liệt kê 730 tên ON_* để đo khoảng cách năng lực so với openNURBS",
        "rh5-common-unsafe-native": "1166 symbol ON_* là phạm vi C ABI phải bao phủ",
    },
    "OM9-M00-L3-04-02": {
        "rh5-on-nurbs-curve": "NURBS curve: knot, degree, control point - bằng chứng dạng dữ liệu cần khớp",
        "rh5-on-nurbs-surface": "NURBS surface và seam",
        "rh5-on-brep": "trimmed B-rep: AddTrimCurve/AddEdgeCurve/CloseTrimGap",
        "rh5-on-brep-parts": "topology face/edge/trim/loop/vertex",
        "rh5-common-units": "enum UnitSystem cho ánh xạ đơn vị",
        "rh5-on-mass-properties": "Area/Volume để đối chiếu số liệu đo được",
    },
    "OM9-M00-L3-04-03": {
        "rh5-on-binary-archive": "chunk 3dm: BeginRead3dmChunk và version - nguồn cho kiểm tra an toàn khi import",
        "rh5-common-file3dm": "ReadWithLog là nơi phát cảnh báo đối tượng không hỗ trợ",
    },
}

for _l3_id, _map in RHINO5_L3.items():
    _row = L3_ANCHORS.get(_l3_id)
    if _row is None:
        raise KeyError(
            f"RHINO5_L3 references unknown L3 {_l3_id!r}; fix the mapping or the "
            "generated anchor module before applying.")
    _row["rhino5"] = dict(_map)

RHINO5_L3_ANCHOR_IDS = sorted({a for _m in RHINO5_L3.values() for a in _m})
_unknown_rh5 = [a for a in RHINO5_L3_ANCHOR_IDS if a not in RHINO5]
if _unknown_rh5:
    raise KeyError(f"RHINO5_L3 references unknown anchors: {_unknown_rh5}")
