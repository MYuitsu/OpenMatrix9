"""Real Qt input/native host fixtures, never a product input proxy."""
import hashlib, json, os, re, traceback
from pathlib import Path
import FreeCAD as App
import FreeCADGui as Gui
from PySide6 import QtCore, QtGui, QtWidgets, QtTest

out=Path(os.environ['OM9_SMOKE_OUTPUT'])
out.mkdir(parents=True,exist_ok=True)
report={'checks':[]}
def settle():
    loop=QtCore.QEventLoop();QtCore.QTimer.singleShot(100,loop.quit);loop.exec()
def check(name,value):
    report['checks'].append({'name':name,'passed':bool(value)})
    if not value:raise AssertionError(name)
def field():return Gui.getMainWindow().findChild(QtWidgets.QPlainTextEdit,'OM9CommandTranscript')
def draft():
    f=field();c=f.textCursor();c.setPosition(int(f.property('om9InputStart')));c.movePosition(QtGui.QTextCursor.End,QtGui.QTextCursor.KeepAnchor);return c.selectedText()
def set_draft(text):
    f=field();f.setFocus();c=f.textCursor();c.setPosition(int(f.property('om9InputStart')));c.movePosition(QtGui.QTextCursor.End,QtGui.QTextCursor.KeepAnchor);c.insertText(text);f.setTextCursor(c);settle()
def key(k):QtTest.QTest.keyClick(field(),k);settle()
def command(text=''):set_draft(text);key(QtCore.Qt.Key_Return)
def prompt():return str(field().property('om9Prompt'))

def check_mnemonics(words,verify=check,presentations=None):
    f=field();p=prompt();start=int(f.property('om9InputStart'))-len(p)
    spans=[(s.cursor.selectionStart()-start,s.cursor.selectedText()) for s in f.extraSelections() if s.format.fontUnderline()]
    if presentations is None:presentations=report.setdefault('presentations',[])
    presentations.append({'prompt':p,'underlined':spans})
    for word,letter in words.items():
        pattern=re.compile(r'(?<![A-Za-z0-9_])'+re.escape(word)+r'(?=[= ])')
        found=pattern.search(p,max(0,p.find('( ')+2))
        offset=found.start() if found else -1
        verify('option visible '+word,offset>=0)
        own=[(pos,text) for pos,text in spans if offset<=pos<offset+len(word)]
        suffix=p[offset+len(word):].split(' ',1)[0]
        if p.startswith(word+suffix+' [unsupported]',offset):
            verify('disabled option has no working underline '+word,len(own)==0)
        else:verify('single mnemonic '+word,len(own)==1 and own[0][1].lower()==letter.lower() and len(own[0][1])==1)
def click_option(word):
    f=field();start=int(f.property('om9InputStart'))-len(prompt());offset=prompt().find(word);check('clickable word '+word,offset>=0)
    c=f.textCursor();c.setPosition(start+offset+1);QtTest.QTest.mouseClick(f.viewport(),QtCore.Qt.LeftButton,QtCore.Qt.NoModifier,f.cursorRect(c).center());settle()
def execute(body):
    try:
        import OpenMatrix9Gui as native
        report['module']=native.__file__;report['module_sha256']=hashlib.sha256(Path(native.__file__).read_bytes()).hexdigest();report['version']=App.Version()
        source=Path('H:/FreeCAD-src/Mod/OpenMatrix9')
        paths=['Gui/CommandOptions.cpp','Gui/CommandOptions.h','Gui/CommandConsole.cpp','Gui/CommandConsole.h','Gui/CurveController.cpp','Gui/SurfaceController.cpp','Gui/CoreRebuild.cpp','Gui/RustBridge.h','rust/src/command_options.rs','rust/src/command_options_ffi.rs','rust/src/command_option_schemas.rs','rust/src/curve_ffi.rs','rust/src/curve.rs']
        report['source_sha256']={p:hashlib.sha256((source/p).read_bytes()).hexdigest() for p in paths}
        Gui.activateWorkbench('OpenMatrix9Workbench');Gui.getMainWindow().show();Gui.getMainWindow().resize(1500,950);settle();body();report['ok']=True
        field().grab().save(str(out/'command-options.png'));Gui.getMainWindow().grab().save(str(out/'native-window.png'))
    except Exception:report['ok']=False;report['error']=traceback.format_exc()
    finally:
        (out/'results.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
        for name in list(App.listDocuments()):App.closeDocument(name)
        QtCore.QTimer.singleShot(250,Gui.getMainWindow().close)
