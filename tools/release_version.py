"""Canonical OM9 release names used by packaging and Windows installation.

Legacy numeric triplets remain valid. New builds append a calendar timestamp
in yyyyMMddhhmm form. This is release tooling, independent of CAD model logic.
"""
from datetime import datetime
import re

VERSION_PATTERN = r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9]{12}))?'


def parse_version(value):
    if not isinstance(value, str):
        raise ValueError('OM9 version must be text')
    match = re.fullmatch(VERSION_PATTERN, value)
    if match is None:
        raise ValueError('Expected aa.bb.cc or aa.bb.cc-yyyyMMddhhmm')
    stamp = match[4]
    if stamp is not None:
        datetime.strptime(stamp, '%Y%m%d%H%M')
    return tuple(int(match[i]) for i in (1, 2, 3)) + (int(stamp or '0'),)
