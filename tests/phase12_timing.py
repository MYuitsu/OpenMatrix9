"""Python 2/3 host test instrumentation. Product commands are invoked once, unchanged.

Stage durations are inclusive: nested native and host stages must not be summed.
The monotonic clock measures elapsed wall time; UTC correlates distinct processes.
"""
import contextlib,datetime,json,os,threading,time

def monotonic():
    if hasattr(time,'perf_counter'):return time.perf_counter()
    from System.Diagnostics import Stopwatch
    return float(Stopwatch.GetTimestamp())/Stopwatch.Frequency

def utc_now():
    if hasattr(datetime,'timezone'):return datetime.datetime.now(datetime.timezone.utc).isoformat()
    return datetime.datetime.utcnow().isoformat()+'Z'

class Timing(object):
    def __init__(self,report,fixture,direction,operation,directory=None,clock=None):
        self.report=report;self.fixture=fixture;self.direction=direction;self.operation=operation
        self.directory=directory or os.environ.get('OM9_PHASE12_TIMING_DIRECTORY');self.clock=clock or monotonic
        self.lock=threading.Lock()
    def call(self,stage,function,*args,**kwargs):
        started=self.clock();utc=utc_now();ok=False;error=None
        try:
            value=function(*args,**kwargs);ok=value is not False;return value
        except BaseException as exception:
            error=str(exception);raise
        finally:
            event=dict(fixture=self.fixture,direction=self.direction,operation=self.operation,stage=stage,seconds=self.clock()-started,started_utc=utc,finished_utc=utc_now(),pid=os.getpid(),ok=ok,inclusive=True)
            if error is not None:event['error']=error
            with self.lock:
                self.report.setdefault('timings',[]).append(event)
                if self.directory:
                    try:
                        if not os.path.isdir(self.directory):os.makedirs(self.directory)
                        with open(os.path.join(self.directory,'timing-'+str(os.getpid())+'.jsonl'),'ab') as stream:
                            stream.write((json.dumps(event,ensure_ascii=True)+'\n').encode('ascii'))
                    except Exception as exception:self.report.setdefault('timing_log_errors',[]).append(str(exception))
    @contextlib.contextmanager
    def wrappers(self,bindings):
        originals=[]
        try:
            for owner,name,stage in bindings:
                original=getattr(owner,name);originals.append((owner,name,original))
                # Each wrapper needs its own closure, including calls on worker threads.
                def make_wrapper(function,label):
                    def invoke(*args,**kwargs):return self.call(label,function,*args,**kwargs)
                    return invoke
                setattr(owner,name,make_wrapper(original,stage))
            yield self
        finally:
            for owner,name,original in reversed(originals):setattr(owner,name,original)

def clipboard_bindings(clip,geometry,native):
    return [(geometry,'_stage_current_geometry','gui_snapshot'),(geometry,'_write_geometry_atomic','atomic_archive_write'),
            (native,'write3dm','native_encode_validate'),(native,'validateDocument','document_guard'),
            (native,'prepareModeling3dm','native_prepare_import'),(native,'commit3dm','host_commit'),
            (clip,'publish_payload','clipboard_publish'),(clip,'capture_payload','clipboard_capture')]

def read_new_events(directory,offsets):
    events=[]
    if not os.path.isdir(directory):return events
    for name in sorted(os.listdir(directory)):
        if not name.startswith('timing-') or not name.endswith('.jsonl'):continue
        path=os.path.join(directory,name);offset=offsets.get(path,0)
        with open(path,'rb') as stream:stream.seek(offset);data=stream.read()
        # IronPython2 returns str from rb; CPython3 returns bytes. JSONL is
        # ASCII (Unicode is escaped), so decoded length still equals byte offset.
        if not isinstance(data,type(u'')):data=data.decode('ascii')
        complete=data.rfind(u'\n')+1
        if complete:
            for line in data[:complete].splitlines():
                if line:events.append(json.loads(line))
            offsets[path]=offset+complete
    return events

def format_event(event):
    return u'[timing] {fixture} | {direction} | {operation}/{stage}: {seconds:.3f}s | {status} | inclusive'.format(status='PASS' if event['ok'] else 'FAIL',**event)
