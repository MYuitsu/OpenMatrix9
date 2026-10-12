"""OpenMatrix9-authored vector symbols; no Matrix bitmap data is embedded."""

BLUE='#2678c9'; GOLD='#f5b400'
SHAPES={
    'file':'<path d="M10 6h9l5 5v15H10z" fill="white"/><path d="M19 6v6h5M13 17h8M13 21h6"/>',
    'new':'<path d="M7 5h12l5 5v17H7z" fill="white"/><path d="M18 5v6h6"/><path d="M21 18v9M17 22h9" stroke="#f5b400" stroke-width="3"/>',
    'open':'<path d="M5 10h9l3 3h10v13H5z" fill="#f5b400"/><path d="M7 15h21l-4 11H5z" fill="#49a8e8"/>',
    'save':'<path d="M6 5h19l2 3v19H6z" fill="#328bda"/><path d="M11 5h11v8H11zM11 19h12v8H11z" fill="white"/><path d="M19 7v4" stroke="#f5b400"/>',
    'import':'<path d="M5 7h18v20H5z" fill="white"/><path d="M9 11h10v11H9z" fill="#6dc8f2"/><path d="M28 14v9H15m5-5-5 5 5 5" stroke="#f5b400" stroke-width="3"/>',
    'export':'<path d="M5 7h18v20H5z" fill="white"/><path d="M11 20h16V10m-5 5 5-5 4 5" stroke="#f5b400" stroke-width="3"/>',
    'duplicate':'<rect x="5" y="6" width="16" height="17" rx="2" fill="#93d7f7"/><rect x="11" y="12" width="16" height="17" rx="2" fill="white"/>',
    'undo':'<path d="M9 11h9a9 9 0 0 1 0 18" stroke-width="4"/><path d="m11 5-7 6 7 6" stroke="#f5b400" stroke-width="4"/>',
    'redo':'<path d="M23 11h-9a9 9 0 0 0 0 18" stroke-width="4"/><path d="m21 5 7 6-7 6" stroke="#f5b400" stroke-width="4"/>',
    'curve':'<path d="M5 25C5 5 27 27 27 7" stroke-width="3"/><circle cx="5" cy="25" r="3" fill="#f5b400"/><circle cx="27" cy="7" r="3" fill="#f5b400"/>',
    'surface':'<path d="m5 11 20-6 3 18-20 6z" fill="#85d2f6"/><path d="m12 9 3 18m5-20 3 17M6 17l20-6M7 23l20-6"/>',
    'solid':'<path d="m16 5 11 6v14l-11 5L5 24V11z" fill="#82ccf1"/><path d="m5 11 11 6 11-6M16 17v13"/><path d="m16 5 11 6-11 6-11-6z" fill="#d1effc"/>',
    'transform':'<path d="M16 5v22M5 16h22" stroke-width="3"/><path d="m12 9 4-4 4 4m-8 14 4 4 4-4M9 12l-4 4 4 4m14-8 4 4-4 4" stroke="#f5b400" stroke-width="3"/>',
    'rotate':'<path d="M9 9a10 10 0 1 1-2 15" stroke-width="3"/><path d="m8 4 1 7 7-1" stroke="#f5b400" stroke-width="3"/>',
    'gem':'<path d="m9 7 14 0 6 8-13 14L3 15z" fill="#59bdf0"/><path d="m3 15 26 0M9 7l7 22 7-22M9 7l-2 8m16-8 2 8"/><path d="m26 3 1 5 4 1-4 1-1 4-1-4-4-1 4-1z" fill="#f5b400" stroke="none"/>',
    'tools':'<path d="m9 5 5 5-4 4-5-5a7 7 0 0 0 9 9l9 10 5-5-10-9a7 7 0 0 0-9-9z" fill="#69bef0"/><circle cx="24" cy="24" r="1.5" fill="#f5b400"/>',
    'settings':'<path d="m13 5 6 0 1 4 4 2 4-1 2 6-4 2-1 4 1 4-6 3-3-3-4-1-4 1-3-6 3-3 1-4-1-4z" fill="#7ac8ef"/><circle cx="17" cy="17" r="5" fill="white"/>',
    'cutters':'<path d="M7 7h18v18H7z" fill="#80cff4"/><path d="m5 26 22-22" stroke="#f5b400" stroke-width="4"/><path d="M9 26h17"/>',
    'render':'<path d="m8 5 12 4-4 14-12-4z" fill="#82d3f7"/><path d="m18 14 11-3v13l-15-3z" fill="#f9d554"/><path d="M10 22v7M5 29h13"/>',
    'measure':'<path d="m5 23 19-19 5 5-19 19z" fill="#f7d04b"/><path d="m11 17 3 3m1-7 3 3m1-7 3 3"/>',
    'points':'<path d="M6 24 11 8 26 21"/><g fill="#f5b400"><rect x="3" y="21" width="6" height="6"/><rect x="8" y="5" width="6" height="6"/><rect x="23" y="18" width="6" height="6"/></g>',
    'info':'<rect x="6" y="5" width="21" height="24" rx="3" fill="white"/><circle cx="16" cy="10" r="1" fill="#2678c9"/><path d="M16 15v9m-3 0h6" stroke-width="3"/>',
    'shade':'<circle cx="16" cy="16" r="11" fill="url(#sphere)"/><path d="M9 10a8 8 0 0 1 9-3" stroke="white" stroke-width="2"/>',
    'grid':'<rect x="5" y="5" width="23" height="23" fill="#9bdaf8"/><path d="M11 5v23m6-23v23m6-23v23M5 11h23M5 17h23M5 23h23" stroke="white"/>',
    'snap':'<path d="M5 22h22M16 5v22"/><circle cx="16" cy="16" r="4" fill="#f5b400"/>',
    'lock':'<rect x="8" y="14" width="17" height="14" rx="2" fill="#f5b400"/><path d="M11 14V9a5 5 0 0 1 10 0v5" stroke-width="3"/><path d="M16 20v4"/>',
    'eye':'<path d="M3 16q13-17 26 0-13 17-26 0" fill="white"/><circle cx="16" cy="16" r="5" fill="#4bb4eb"/>',
    'arrow':'<path d="m12 8 9 8-9 8z" fill="#2678c9"/>',
    'plus':'<path d="M16 7v18M7 16h18" stroke="#f5b400" stroke-width="4"/>',
    'delete':'<path d="m9 9 14 14M23 9 9 23" stroke="#e66352" stroke-width="4"/>',
    'up':'<path d="M16 26V7m-7 8 7-8 7 8" stroke="#f5b400" stroke-width="3"/>',
}
EXACT={
    'ViewIsometric':'solid',
    'FileNew':'new','FileOpen':'open','FileSave':'save','FileSaveAs':'save','FileSaveSmall':'save','FileSaveSmallAs':'save',
    'FileImport':'import','FileExportSelected':'export','Undo':'undo','Redo':'redo',
    'TopIconDuplicate':'duplicate','TopIconMove':'transform','TopIconRotate':'rotate','TopIconEditPointsOn':'points',
    'RhinoOptions':'settings','ObjectProperties':'info','GVObjectInfo':'info','AllObjectInfo':'info','CommandHistory':'info','ProjectNotes':'file',
    'GridON':'grid','PreviewCutterON':'cutters','PreviewShadeON':'shade','ShadeSelectedOnlyON':'shade','GVGemView_1':'gem','GVSurfaceView_1':'surface',
    'LayerArrow':'arrow','LayerLock':'lock','LayerVisibility':'eye','LayerHide':'eye','LayerShow':'eye',
    'ProjectAdd':'plus','ProjectOut':'arrow','ProjectIn':'up','ProjectSave':'save','ProjectDelete':'delete','ProjectManager':'open',
    'GridSnapON':'grid','OrthoSnapON':'snap','PlanarSnapON':'surface','ProjectSnapON':'snap','SnapBetween':'snap',
    'SnapOnSurface':'surface','SnapOnPolysurface':'solid',
}

def symbol_for_key(key):
    if key in EXACT:return EXACT[key],True
    if key.startswith(('ToolsObjectSnap','RhinoSmartTrack','InfoSettingsSmartTargets')):return 'snap',False
    if key.startswith(('Dial_','DisplayMode','RhinoHistory','GVHistory')):return 'shade',False
    for prefix,symbol in [('Curve','curve'),('Surface','surface'),('Solid','solid'),('Clayoo','surface'),('Emboss','surface'),
                          ('Transform','transform'),('TopIcon','transform'),('Builder','tools'),('Gems','gem'),('Settings','settings'),
                          ('Cutters','cutters'),('Render','render'),('Dimension','measure'),('View','solid'),('File','file'),('Info','info')]:
        if key.startswith(prefix):return symbol,False
    return 'tools',False

def svg(symbol):
    return ('<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">'
            '<defs><linearGradient id="card" x2="0" y2="1"><stop stop-color="#ffffff"/><stop offset="1" stop-color="#e5f3fc"/></linearGradient>'
            '<radialGradient id="sphere" cx=".3" cy=".25"><stop stop-color="#dcf9ff"/><stop offset=".45" stop-color="#55b7ec"/><stop offset="1" stop-color="#235391"/></radialGradient></defs>'
            '<rect x=".7" y=".7" width="30.6" height="30.6" rx="6" fill="url(#card)" stroke="#c5dce9"/>'
            '<g stroke="#2678c9" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none">'+SHAPES[symbol]+'</g></svg>\n')
