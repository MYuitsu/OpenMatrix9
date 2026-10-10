"""Timing must preserve commands, failures, and partial live log records."""
import importlib.util,json,tempfile,types,unittest,io
from unittest.mock import patch
from pathlib import Path
path=Path(__file__).parents[2]/'tests/phase12_timing.py'
class TimingTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(path.exists(),'Phase1/2 timing instrumentation is missing')
        spec=importlib.util.spec_from_file_location('phase12_timing',path)
        self.module=importlib.util.module_from_spec(spec);spec.loader.exec_module(self.module)
    def test_result_and_failure_preserved_without_retry(self):
        with tempfile.TemporaryDirectory(dir=path.parents[1]/'build') as out:
            report={};clock=iter([2.,2.25,3.,3.5]);rec=self.module.Timing(report,'ring','OM9 -> Rhino','Copy',out,lambda:next(clock))
            calls=[]
            def good():calls.append(1);return 'geometry'
            def bad():calls.append(2);raise RuntimeError('native failure')
            self.assertEqual(rec.call('total',good),'geometry')
            with self.assertRaisesRegex(RuntimeError,'native failure'):rec.call('total',bad)
            self.assertEqual(calls,[1,2]);self.assertEqual([e['seconds'] for e in report['timings']],[.25,.5])
            self.assertEqual([e['ok'] for e in report['timings']],[True,False])
            self.assertIn('native failure',report['timings'][-1]['error'])
    def test_wrappers_restored_after_failure_and_nested_scopes(self):
        owner=types.SimpleNamespace(run=lambda:5);original=owner.run;rec=self.module.Timing({},'mesh','Rhino -> OM9','Paste')
        with self.assertRaisesRegex(RuntimeError,'stop'):
            with rec.wrappers([(owner,'run','prepare')]):
                self.assertEqual(owner.run(),5);raise RuntimeError('stop')
        self.assertIs(owner.run,original)
        self.assertEqual(rec.report['timings'][0]['stage'],'prepare')
    def test_tail_emits_complete_lines_once_and_waits_for_partial_line(self):
        with tempfile.TemporaryDirectory(dir=path.parents[1]/'build') as out:
            p=Path(out)/'timing-123.jsonl';first={'fixture':'ring','seconds':1};second={'fixture':'mesh','seconds':2}
            p.write_bytes((json.dumps(first)+'\n'+json.dumps(second)[:8]).encode())
            offsets={};self.assertEqual(self.module.read_new_events(out,offsets),[first]);self.assertEqual(self.module.read_new_events(out,offsets),[])
            with p.open('ab') as stream:stream.write((json.dumps(second)[8:]+'\n').encode())
            self.assertEqual(self.module.read_new_events(out,offsets),[second]);self.assertEqual(self.module.read_new_events(out,offsets),[])
    def test_log_failure_does_not_mask_command_exception(self):
        with tempfile.TemporaryDirectory(dir=path.parents[1]/'build') as out:
            occupied=Path(out)/'occupied';occupied.write_text('file')
            rec=self.module.Timing({},'ring','OM9 -> Rhino','Copy',str(occupied))
            def bad():raise ValueError('original')
            with self.assertRaisesRegex(ValueError,'original'):rec.call('total',bad)
            self.assertTrue(rec.report['timing_log_errors'])
    def test_format_identifies_inclusive_stage_and_failure(self):
        row=dict(fixture='ring',direction='OM9 -> Rhino',operation='Copy',stage='native_encode_validate',seconds=12.34,ok=False,inclusive=True)
        text=self.module.format_event(row)
        for word in ['ring','OM9 -> Rhino','Copy','native_encode_validate','12.340','FAIL','inclusive']:self.assertIn(word,text)
    def test_false_command_return_logged_as_failure(self):
        rec=self.module.Timing({},'ring','Rhino -> OM9','Copy')
        self.assertIs(rec.call('total',lambda:False),False)
        self.assertFalse(rec.report['timings'][0]['ok'])
    def test_binary_open_returning_text_like_ironpython_keeps_byte_offsets(self):
        with tempfile.TemporaryDirectory(dir=path.parents[1]/'build') as out:
            p=Path(out)/'timing-123.jsonl';row={'fixture':'nhẫn','seconds':1}
            data=json.dumps(row,ensure_ascii=True)+'\r\n'+json.dumps(row)[:8]
            p.write_text(data,encoding='ascii');offsets={}
            with patch.object(self.module,'open',lambda *args:io.StringIO(data),create=True):
                self.assertEqual(self.module.read_new_events(out,offsets),[row])
                self.assertEqual(offsets[str(p)],data.index('\r\n')+2)
                self.assertEqual(self.module.read_new_events(out,offsets),[])
if __name__=='__main__':unittest.main()
