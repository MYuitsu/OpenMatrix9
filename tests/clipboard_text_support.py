"""Independent CF_UNICODETEXT readback for controlled Windows test strings."""
import ctypes
import time


def windows_text(clip):
    _,user,kernel,_=clip._win32()
    # Clipboard viewers can hold the just-restored OLE data briefly. Only retry
    # acquiring the lock (20 ms total); missing/invalid text remains a failure.
    for attempt in range(5):
        if user.OpenClipboard(None):break
        if attempt==4:raise RuntimeError('Windows clipboard busy during text verification')
        time.sleep(.005)
    try:
        handle=user.GetClipboardData(13)
        if not handle:raise RuntimeError('CF_UNICODETEXT unavailable')
        size=kernel.GlobalSize(handle)
        if size<2 or size>1024*1024 or size%2:raise RuntimeError('Invalid CF_UNICODETEXT test buffer')
        pointer=kernel.GlobalLock(handle)
        if not pointer:raise RuntimeError('Cannot lock CF_UNICODETEXT')
        try:raw=ctypes.string_at(pointer,size)
        finally:kernel.GlobalUnlock(handle)
        # CF_UNICODETEXT is a NUL-terminated UTF-16 string; GlobalSize may pad.
        return raw.decode('utf-16-le').split('\x00',1)[0]
    finally:user.CloseClipboard()
