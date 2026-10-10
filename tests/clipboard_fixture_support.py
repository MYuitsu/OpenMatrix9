"""Test-only Qt/Win32 fixture preparation; never retries product commands."""
import ctypes
import time
from PySide6 import QtCore, QtWidgets


def _pump(milliseconds=10):
    loop = QtCore.QEventLoop()
    QtCore.QTimer.singleShot(milliseconds, loop.quit)
    loop.exec()


def capture_fixture(clip, measurements, timeout=0.2):
    deadline = time.perf_counter() + timeout
    while True:
        try:
            return clip.capture_payload()
        except RuntimeError as error:
            if 'clipboard is busy' not in str(error).lower() or time.perf_counter() >= deadline:
                raise
            measurements['observer_busy_reads'] = measurements.get('observer_busy_reads', 0) + 1
            _pump()


def wait_fixture(clip, expected, measurements, timeout=2.0, quiet=0.1):
    """Await readable, unchanged fixture; foreign data is an error, never republished."""
    _, user, _, _ = clip._win32()
    started = time.perf_counter()
    deadline = started + timeout
    stable_since = None
    stable_sequence = None
    while time.perf_counter() < deadline:
        sequence = user.GetClipboardSequenceNumber()
        try:
            actual = clip.capture_payload()
        except RuntimeError as error:
            if 'clipboard is busy' not in str(error).lower():
                raise
            measurements['fixture_busy_reads'] = measurements.get('fixture_busy_reads', 0) + 1
            stable_since = None
        else:
            if actual != expected:
                raise RuntimeError('Clipboard fixture changed; refusing to overwrite another owner')
            now = time.perf_counter()
            if stable_since is None or sequence != stable_sequence:
                stable_since, stable_sequence = now, sequence
            elif now - stable_since >= quiet:
                measurements.setdefault('fixture_ready', []).append({
                    'seconds': now - started, 'sequence': sequence, 'bytes': len(expected)
                })
                return
        _pump()
    raise RuntimeError('Clipboard fixture did not become stable within %.1f seconds' % timeout)


def publish_fixture(clip, payload, measurements):
    # Deliberate test setup only. Product publication/rollback is covered by
    # menu Copy and the separate failure gate, without replacing its functions.
    clip.validate_payload(payload)
    mime = QtCore.QMimeData()
    mime.setData(clip.MIME, QtCore.QByteArray(payload))
    QtWidgets.QApplication.clipboard().setMimeData(mime)
    _, user, _, _ = clip._win32()
    sequence = user.GetClipboardSequenceNumber()
    flush = ctypes.WinDLL('ole32').OleFlushClipboard
    flush.restype = ctypes.c_long
    deadline = time.perf_counter() + 0.2
    while True:
        if user.GetClipboardSequenceNumber() != sequence:
            raise RuntimeError('Clipboard fixture changed before flush; refusing to touch another owner')
        result = flush()
        if result == 0:
            break
        # Clipboard observers may briefly hold OpenClipboard after publication.
        # Retry only this test setup operation while the exact publication is
        # still current. Product Copy/Paste is invoked once and never retried.
        if (result & 0xffffffff) != 0x800401d0 or time.perf_counter() >= deadline:
            raise RuntimeError('Cannot materialize clipboard test fixture: %s' % result)
        measurements['fixture_flush_busy'] = measurements.get('fixture_flush_busy', 0) + 1
        _pump()
    wait_fixture(clip, payload, measurements)
