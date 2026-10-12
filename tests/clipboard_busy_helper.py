"""Owned independent clipboard lock holder for the runtime busy-path test."""
import ctypes,sys,time
from pathlib import Path
out=Path(sys.argv[1]);user=ctypes.WinDLL('user32');kernel=ctypes.WinDLL('kernel32')
user.CreateWindowExW.argtypes=[ctypes.c_ulong,ctypes.c_wchar_p,ctypes.c_wchar_p,ctypes.c_ulong]+[ctypes.c_int]*4+[ctypes.c_void_p]*4
user.CreateWindowExW.restype=ctypes.c_void_p;user.OpenClipboard.argtypes=[ctypes.c_void_p];user.DestroyWindow.argtypes=[ctypes.c_void_p]
kernel.GetModuleHandleW.restype=ctypes.c_void_p
window=user.CreateWindowExW(0,'STATIC','OM9 owned clipboard lock fixture',0x80000000,0,0,1,1,None,None,kernel.GetModuleHandleW(None),None)
if not window or not user.OpenClipboard(window):raise SystemExit('Cannot acquire owned clipboard window lock')
try:
    (out/'clipboard-lock-ready').write_text('locked')
    start=time.monotonic()
    while not (out/'clipboard-lock-release').exists() and time.monotonic()-start<10:time.sleep(.02)
finally:user.CloseClipboard();user.DestroyWindow(window)
