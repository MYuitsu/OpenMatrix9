import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch
root=Path(__file__).parents[2]
spec=importlib.util.spec_from_file_location('clipboard',root/'ThreeDmClipboard.py')
clipboard=importlib.util.module_from_spec(spec)
if spec.loader and (root/'ThreeDmClipboard.py').exists():spec.loader.exec_module(clipboard)

class ClipboardPayloadTests(unittest.TestCase):
    def test_native_v5_archive_header(self):
        self.assertTrue(hasattr(clipboard,'validate_payload'),'Rhino clipboard decoder is missing')
        valid=b'3D Geometry File Format       50'+b'\x01'+b'\x00'*100
        self.assertEqual(clipboard.validate_payload(valid),valid)

    def test_malformed_and_text_are_refused(self):
        self.assertTrue(hasattr(clipboard,'validate_payload'),'Rhino clipboard decoder is missing')
        for payload in [b'',b'H:/a.3dm',b'3D Geometry File Format       50',b'3D Geometry File Format       80'+b'\x00'*100]:
            with self.subTest(payload=payload),self.assertRaises(RuntimeError):clipboard.validate_payload(payload)

    def test_size_checked_before_copying_buffer(self):
        self.assertTrue(hasattr(clipboard,'check_payload_size'),'clipboard allocation limit is missing')
        for size in [0,31,clipboard.MAX_PAYLOAD+1]:
            with self.assertRaises(RuntimeError):clipboard.check_payload_size(size)

    def test_publication_verification_retries_transient_busy_only(self):
        with patch.object(clipboard,'capture_payload',side_effect=[RuntimeError('Windows clipboard is busy'),b'archive']),patch.object(clipboard.time,'sleep'):
            self.assertEqual(clipboard._capture_published(),b'archive')
        with patch.object(clipboard,'capture_payload',side_effect=RuntimeError('Clipboard has no Rhino 5 geometry')) as capture,patch.object(clipboard.time,'sleep'):
            with self.assertRaisesRegex(RuntimeError,'no Rhino'):clipboard._capture_published()
            self.assertEqual(capture.call_count,1)
        with patch.object(clipboard,'capture_payload',side_effect=RuntimeError('Windows clipboard is busy')) as capture,patch.object(clipboard.time,'sleep'):
            with self.assertRaisesRegex(RuntimeError,'busy'):clipboard._capture_published()
            self.assertEqual(capture.call_count,5)

if __name__=='__main__':unittest.main()
