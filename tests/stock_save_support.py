"""Isolate verified stock Windows ZipWriter ambient-errno errors in fixtures."""
import os


def prepare_stock_fcstd_save(report):
    import FreeCAD as App
    if os.name != 'nt' or App.Version()[:3] != ['1', '1', '4']:
        return
    import ctypes
    crt = ctypes.CDLL('ucrtbase')
    crt._errno.restype = ctypes.POINTER(ctypes.c_int)
    report.setdefault('stock_save_fixture', []).append({'ambient_errno': crt._errno()[0]})
    # Stock Writer checks errno without a stream failure. Start this write
    # clean; genuine errors from saveAs remain observable and propagate.
    crt._errno()[0] = 0
