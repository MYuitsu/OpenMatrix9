"""Independent binary payload/corruption fixtures for the plugin FCStd codec."""
import copy
import hashlib
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
import ThreeDmStorage as storage

class Object:
    def __init__(self, name='Payload'):
        self.Name=name
        self.Document=type('Document',(),{'Name':'Test'})()
        self.PropertiesList=[]
        self.types={}
    def addProperty(self, kind, name, *args):
        self.PropertiesList.append(name);self.types[name]=kind
    def getTypeIdOfProperty(self,name):return self.types[name]
    def setEditorMode(self,*args):pass
    def setPropertyStatus(self,*args):pass

class StorageTests(unittest.TestCase):
    def test_repeated_edits_keep_one_live_cache_and_cleanup(self):
        a=Object('BoundedCache');storage.bind(a,'OM9HatchLoopFile',b'original',100)
        original_path=a.OM9HatchLoopFile
        for i in range(20):
            storage.write(a,'OM9HatchLoopFile',str(i).encode(),100)
            self.assertEqual(a.OM9HatchLoopFile,original_path)
        storage._Observer().slotDeletedObject(a)
        self.assertFalse(Path(original_path).exists())
    def test_binary_multichunk_recovery_and_copy_isolation(self):
        raw=bytes(range(256))*1100+b'\x00\xfftail'
        a=Object();storage.bind(a,'OM9HatchLoopFile',raw,len(raw))
        b=copy.deepcopy(a);b.Name='Copy';b.Proxy.onDocumentRestored(b)
        self.assertNotEqual(a.OM9HatchLoopFile,b.OM9HatchLoopFile)
        self.assertEqual(Path(a.OM9HatchLoopFile).read_bytes(),raw)
        storage.write(b,'OM9HatchLoopFile',b'edited',len(raw))
        self.assertEqual(storage.read(a,'OM9HatchLoopFile',len(raw)),raw)
        self.assertEqual(storage.read(b,'OM9HatchLoopFile',len(raw)),b'edited')
        self.assertNotIn('App::PropertyFileIncluded',a.types.values())
    def test_cache_deleted_rebuilds_from_payload(self):
        a=Object('Cache');storage.bind(a,'OM9HatchLoopFile',b'persisted bytes',100)
        os.unlink(a.OM9HatchLoopFile)
        a.Proxy.onDocumentRestored(a)
        self.assertEqual(Path(a.OM9HatchLoopFile).read_bytes(),b'persisted bytes')
    def test_corrupt_chunks_never_fall_back_to_valid_cache(self):
        a=Object('Corrupt');storage.bind(a,'OM9HatchLoopFile',b'payload',100)
        a.OM9HatchLoopFileChunks=['not base64!']
        with self.assertRaises(RuntimeError):storage.path(a,'OM9HatchLoopFile',100)
        self.assertEqual(Path(a.OM9HatchLoopFile).read_bytes(),b'payload')
    def test_digest_mismatch_and_oversize_rejected(self):
        a=Object('Limits');storage.bind(a,'OM9HatchLoopFile',b'payload',100)
        with self.assertRaises(RuntimeError):storage.read(a,'OM9HatchLoopFile',3)
        a.OM9HatchLoopFileDigest=hashlib.sha256(b'other').hexdigest()
        with self.assertRaises(RuntimeError):storage.read(a,'OM9HatchLoopFile',100)
    def test_failed_write_keeps_all_properties(self):
        a=Object('Reject');storage.bind(a,'OM9HatchLoopFile',b'original',100)
        before=copy.deepcopy(a.__dict__)
        with self.assertRaises(RuntimeError):storage.write(a,'OM9HatchLoopFile',b'x'*101,100)
        self.assertEqual(a.OM9HatchLoopFileChunks,before['OM9HatchLoopFileChunks'])
        self.assertEqual(a.OM9HatchLoopFileDigest,before['OM9HatchLoopFileDigest'])
        self.assertEqual(a.OM9HatchLoopFile,before['OM9HatchLoopFile'])
    def test_corrupted_materialized_file_is_rejected(self):
        a=Object('Tamper');storage.bind(a,'OM9HatchLoopFile',b'original',100)
        Path(a.OM9HatchLoopFile).write_bytes(b'changed')
        with self.assertRaises(RuntimeError):storage.path(a,'OM9HatchLoopFile',100)
    def test_legacy_file_can_be_read_without_modification(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'legacy';p.write_bytes(b'legacy')
            a=Object('Legacy');a.addProperty('App::PropertyFileIncluded','OM9HatchLoopFile');a.OM9HatchLoopFile=str(p)
            self.assertEqual(storage.read(a,'OM9HatchLoopFile',100),b'legacy')
            self.assertEqual(a.types['OM9HatchLoopFile'],'App::PropertyFileIncluded')
            self.assertEqual(p.read_bytes(),b'legacy')

if __name__=='__main__':unittest.main()
