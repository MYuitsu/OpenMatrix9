import ctypes
import importlib.util
from pathlib import Path
import types
import unittest

spec=importlib.util.spec_from_file_location('clipboard_text_support',Path(__file__).resolve().parents[2]/'tests/clipboard_text_support.py')
reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)

class ClipboardTextRetry(unittest.TestCase):
    def make(self,locks=0,handle=True):
        data='previous clipboard'.encode('utf-16-le')+b'\0\0'
        self.buffer=ctypes.create_string_buffer(data)
        self.calls=0;self.closes=0
        def open_clipboard(_):
            self.calls+=1
            return self.calls>locks
        def close():self.closes+=1
        user=types.SimpleNamespace(OpenClipboard=open_clipboard,CloseClipboard=close,GetClipboardData=lambda _:1 if handle else 0)
        kernel=types.SimpleNamespace(GlobalSize=lambda _:len(data),GlobalLock=lambda _:ctypes.addressof(self.buffer),GlobalUnlock=lambda _:None)
        return types.SimpleNamespace(_win32=lambda:(None,user,kernel,None))
    def test_transient_lock_retries_and_reads_exact_unicode(self):
        self.assertEqual(reader.windows_text(self.make(locks=2)),'previous clipboard')
        self.assertEqual((self.calls,self.closes),(3,1))
    def test_permanent_lock_remains_failure_and_bounded(self):
        with self.assertRaisesRegex(RuntimeError,'clipboard busy'):
            reader.windows_text(self.make(locks=100))
        self.assertLessEqual(self.calls,5)
        self.assertEqual(self.closes,0)
    def test_missing_text_never_retries_or_passes(self):
        with self.assertRaisesRegex(RuntimeError,'CF_UNICODETEXT unavailable'):
            reader.windows_text(self.make(handle=False))
        self.assertEqual((self.calls,self.closes),(1,1))

if __name__=='__main__':unittest.main()
