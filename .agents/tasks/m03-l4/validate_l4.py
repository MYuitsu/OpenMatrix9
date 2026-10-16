#!/usr/bin/env python3
"""validate_l4.py - validator for OM9 M03 L4 planning artifacts (spec-only task).

Python standard library only. It validates ADDED L4 files and writes nothing at all:
it never creates, renames, rewrites or deletes any file (--print-digests prints to stdout).
It also refuses to report success when a pre-existing L1/L2/l3/reference file differs
from the digests embedded below (EXPECTED_DIGESTS).

Usage (run from anywhere; ROOT is the M03 folder):

  python validate_l4.py ROOT                       # whole M03 tree
  python validate_l4.py ROOT L2-01-09              # one branch (name prefix, relative or absolute path)
  python validate_l4.py ROOT --l2 PATH             # same as above
  python validate_l4.py ROOT --check-examples CONVENTIONS.md   # validate the two examples in the conventions
  python validate_l4.py ROOT --print-digests       # print pre-existing-file digests (JSON) and exit
  python validate_l4.py ROOT --json                # machine readable summary on stdout

Exit code: 0 = PASS, 1 = FAIL, 2 = usage / environment error.

Rules are documented in CONVENTIONS.md (same folder). Anything this script enforces is
listed there under "Validator checks".
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Set, Tuple

# --------------------------------------------------------------------------------------
# Constants
# --------------------------------------------------------------------------------------

MAX_PATH_EXCLUSIVE = 240  # every added path must be shorter than this
CANONICAL_ROOT = r"H:\OpenMatrix9\OM9_Milestones\M03 - Curve, Surface, Solid and 3DM Exchange"
SCHEMA_VERSION = "om9.l4.v1"
MILESTONE = "M03"
MAX_SHORT_NAME = 27  # "L4-NNN - " (9 chars) + 27 = 36 chars max folder name
MAX_IMPL_LOC = 250
MAX_TOTAL_LOC = 450

ID_RE = re.compile(r"^OM9-L4-(\d\d)-(\d\d)-(\d{3})-(\d\d)$")
L3_RE = re.compile(r"^OM9-L3-(\d\d)-(\d\d)-(\d{3})$")
L2_RE = re.compile(r"^OM9-L2-(\d\d)-(\d\d)$")
FOLDER_RE = re.compile(r"^L4-(\d{3}) - ([A-Za-z0-9][A-Za-z0-9 _+-]{0,25}[A-Za-z0-9])$")
FEATURE_RE = re.compile(r"^OM9-[A-Z0-9]+-\d{3}$")
IDENT_RE = re.compile(r"^[a-z][a-z0-9_]{2,79}$")
TOL_NAME_RE = re.compile(r"^[A-Z][A-Z0-9_]+$")
ASSUMPTION_ID_RE = re.compile(r"^A[1-9]\d?$")
CARGO_CMD_RE = re.compile(
    r"^cargo test --locked --manifest-path (?:standalone|rust)/Cargo\.toml"
    r"(?: -p [a-z0-9_-]+)?(?: --features [a-z0-9_,-]+)?(?: --test [a-z0-9_]+)? ([A-Za-z0-9_:]+)$"
)
CARGO_NATIVE_RE = re.compile(
    r"^cargo test --locked --manifest-path standalone/Cargo\.toml -p [a-z0-9_-]+"
    r"(?: --features [a-z0-9_,-]+)?(?: --test [a-z0-9_]+)? [A-Za-z0-9_:]+ -- --ignored$"
)
PY_CMD_RE = re.compile(r"^python -m unittest [A-Za-z0-9_./-]+$")

OWNER_LAYERS = {"rust-domain", "occt-adapter", "opennurbs-adapter", "command-session", "ui-binding", "fixture-tool"}
LANGS = {"rust", "cpp", "python"}
KINDS = {"contract", "algorithm", "adapter", "command-session", "ui-binding", "fixture", "integration-test"}
TAGS = {"be", "febe"}
FILE_LANGS = {"rust", "cpp", "python", "toml", "json", "cmake", "markdown"}
SYMBOL_KINDS = {"fn", "method", "struct", "enum", "trait", "const", "mod", "type", "cpp-fn", "cpp-class"}
CASE_KINDS = {"positive", "negative", "edge"}
RUNS_IN = {"portable", "native-ignored"}
STATUS = {"existing", "planned"}

TOP_REQUIRED = [
    "schema_version", "level", "id", "milestone", "parent_l3_id", "parent_l2_id", "seq", "title",
    "planning_state", "tag", "owner_layer", "language", "kind", "size", "goal", "scope",
    "dependencies", "files", "api", "algorithm", "interaction", "tests", "definition_of_done",
    "references", "assumptions",
]
TOP_OPTIONAL = ["language_note", "interaction_na_reason", "notes_vi"]

BANNED = [
    (re.compile(r"\bTODO\b"), "TODO"),
    (re.compile(r"\bTBD\b"), "TBD"),
    (re.compile(r"\bTBA\b"), "TBA"),
    (re.compile(r"\bFIXME\b"), "FIXME"),
    (re.compile(r"\bXXX\b"), "XXX"),
    (re.compile(r"placeholder", re.I), "placeholder"),
    (re.compile(r"lorem ipsum", re.I), "lorem ipsum"),
    (re.compile(r"to be (determined|decided|defined)", re.I), "to be determined"),
    (re.compile(r"\?\?\?"), "???"),
    (re.compile(r"\b(todo|unimplemented)!\s*\("), "todo!/unimplemented!"),
    (re.compile(r"chưa xác định|sẽ bổ sung sau|đang cập nhật|sẽ cập nhật sau", re.I), "vi-placeholder"),
]

# Digests of every PRE-EXISTING M03 file (everything except L4 artifacts) captured when the
# conventions were authored (no L4 existed). A mismatch means an existing L1/L2/l3/reference
# file was modified, added or removed. Override only with --expected-digests FILE after an
# explicitly approved change to pre-existing files.
EXPECTED_DIGESTS: Dict[str, Any] = {
    "files_total": 2960,
    "groups": {
        "ROOT": {"files": 1, "sha256": "85dd216dd3377130f573bbd256dd26cb69de16d2560c68c8c586cfc21669693f"},
        "L2-01-09 - 01-be-occt-geometry-creation-ops": {"files": 198, "sha256": "57ea4aa2e40d287784d3cc1042719a8af9acc4ce2189356ebecdb45d705e3ae8"},
        "L2-01-10 - 02-be-occt-boolean-fillet-shell-ops": {"files": 119, "sha256": "8717032bafb265fd970c55e508be810ee90bd7f2ddae9c9b7db26268275bdb23"},
        "L2-01-11 - 03-be-occt-transforms-and-pattern-ops": {"files": 161, "sha256": "2dff184c9e42cac51c68979d6b812ad69fc9954367a7799eef93a61e65c01cd2"},
        "L2-01-13 - 14-febe-import-rhino-3dm": {"files": 57, "sha256": "452c76268b3ad57686cf41c820509bd3348fa329206814847e8d66f08aa9e275"},
        "L2-02-04 - 04-febe-freeform-curve-modeling": {"files": 419, "sha256": "8558dbb35f4409e6afb3602f35e02c7405e46359f7e41c475e117430158761f0"},
        "L2-02-05 - 05-febe-freeform-surface-modeling": {"files": 483, "sha256": "78841076a97a82fa00c19c223025e151dbd4aa4a12ce83bcea137bcb4ea4c99f"},
        "L2-02-06 - 06-febe-rhino-transform-orient-workflows": {"files": 157, "sha256": "3a3ea65f6cf6b5851fd99cf02cb0807e1bc06a9182a2aa675b23ff40b7d63c59"},
        "L2-03-08 - 07-febe-matrix-top11-and-shortcut-actions": {"files": 43, "sha256": "c6e5071b3cb2355318c1ab40d729308c4ee791554c1e9c1784ab6c27c6f1cb37"},
        "L2-03-16 - 08-febe-matrix-curve-menu-workflows": {"files": 421, "sha256": "f3a4ebd51af8e8152d322286bd88a629c05d2cbbd19946447e04568b6ce974d1"},
        "L2-03-17 - 09-febe-matrix-surface-menu-workflows": {"files": 331, "sha256": "aab95058056bdd4a67bc3032ef298424c3fff2541c073deb1402802ac6f5aa83"},
        "L2-03-18 - 10-febe-matrix-solid-menu-workflows": {"files": 153, "sha256": "fabc4c8e356f98fdb1cbf40c3a68e9594090b2c2b13db44620fda8ad28de35a2"},
        "L2-03-19 - 11-febe-matrix-utilities-and-mesh-health": {"files": 69, "sha256": "b20df62b6391ef5322d8a1638c4a2143c7b577d456cacf4620b21aa3d88b07b7"},
        "L2-03-20 - 12-febe-matrix-measure-and-model-statistics": {"files": 36, "sha256": "cb5824a6a8de4f20cf6c3c61b69f654edefd16312eceeed49435128da3b70d55"},
        "L2-04-01 - 13-febe-matrix-transform-toolbar-bridges": {"files": 312, "sha256": "004b5fd8e8aabdece88028afb6155fe988769d0a0984c405faea440b80c198c3"},
    },
}

# --------------------------------------------------------------------------------------
# Report
# --------------------------------------------------------------------------------------


class Report:
    def __init__(self) -> None:
        self.checked: Dict[str, int] = {}
        self.failed: Dict[str, int] = {}
        self.failures: List[Tuple[str, str, str]] = []
        self.warnings: List[str] = []
        self.order: List[str] = []

    def _touch(self, check: str) -> None:
        if check not in self.checked:
            self.checked[check] = 0
            self.failed[check] = 0
            self.order.append(check)

    def count(self, check: str, n: int = 1) -> None:
        self._touch(check)
        self.checked[check] += n

    def fail(self, check: str, where: str, msg: str) -> None:
        self._touch(check)
        self.failed[check] += 1
        self.failures.append((check, where, msg))

    def warn(self, msg: str) -> None:
        self.warnings.append(msg)

    def ok(self) -> bool:
        return not self.failures


# --------------------------------------------------------------------------------------
# Small helpers
# --------------------------------------------------------------------------------------


class DupKeyError(ValueError):
    pass


def _pairs_hook(pairs: List[Tuple[str, Any]]) -> Dict[str, Any]:
    out: Dict[str, Any] = {}
    for k, v in pairs:
        if k in out:
            raise DupKeyError(f"duplicate key {k!r}")
        out[k] = v
    return out


def _no_const(name: str) -> Any:
    raise ValueError(f"non-finite number literal {name}")


def load_json_file(path: Path) -> Tuple[Optional[Any], Optional[str]]:
    """Strict UTF-8 (no BOM) JSON loader. Returns (data, error)."""
    try:
        raw = path.read_bytes()
    except OSError as exc:
        return None, f"cannot read: {exc}"
    if raw.startswith(b"\xef\xbb\xbf"):
        return None, "UTF-8 BOM is not allowed"
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        return None, f"not valid UTF-8: {exc}"
    return load_json_text(text)


def load_json_text(text: str) -> Tuple[Optional[Any], Optional[str]]:
    try:
        return json.loads(text, object_pairs_hook=_pairs_hook, parse_constant=_no_const), None
    except (ValueError, DupKeyError) as exc:
        return None, f"invalid JSON: {exc}"


def walk_strings(node: Any, path: str = "$") -> Iterable[Tuple[str, str]]:
    if isinstance(node, str):
        yield path, node
    elif isinstance(node, dict):
        for k, v in node.items():
            yield from walk_strings(v, f"{path}.{k}")
    elif isinstance(node, list):
        for i, v in enumerate(node):
            yield from walk_strings(v, f"{path}[{i}]")


def is_int(v: Any) -> bool:
    return isinstance(v, int) and not isinstance(v, bool)


def nonempty_str(v: Any) -> bool:
    return isinstance(v, str) and v.strip() != ""


def short_rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def path_len(path: Path, root: Path) -> int:
    """max(actual absolute length, length under the canonical H:\\ checkout)."""
    actual = len(os.path.abspath(str(path)))
    try:
        rel = os.path.relpath(os.path.abspath(str(path)), os.path.abspath(str(root)))
    except ValueError:
        return actual
    canonical = len(CANONICAL_ROOT) + 1 + len(rel.replace("/", "\\"))
    return max(actual, canonical)


def load_glossary(conventions: Path) -> Optional[Dict[str, Tuple[float, str]]]:
    try:
        text = conventions.read_text(encoding="utf-8")
    except OSError:
        return None
    m = re.search(r"<!-- GLOSSARY:BEGIN -->(.*?)<!-- GLOSSARY:END -->", text, re.S)
    if not m:
        return None
    out: Dict[str, Tuple[float, str]] = {}
    for line in m.group(1).splitlines():
        cells = [c.strip().strip("`") for c in line.strip().strip("|").split("|")]
        if len(cells) >= 3 and TOL_NAME_RE.match(cells[0]):
            try:
                out[cells[0]] = (float(cells[1]), cells[2])
            except ValueError:
                continue
    return out or None


# --------------------------------------------------------------------------------------
# Tree model
# --------------------------------------------------------------------------------------


class L3Info:
    def __init__(self, path: Path, data: Dict[str, Any]) -> None:
        self.path = path
        spec = data.get("specification", {}) if isinstance(data, dict) else {}
        self.id: str = data.get("id", "") if isinstance(data, dict) else ""
        self.title: str = data.get("title", "") if isinstance(data, dict) else ""
        self.l2_id: str = data.get("parent_l2_id", "") if isinstance(data, dict) else ""
        self.tag: str = spec.get("tag", "") if isinstance(spec, dict) else ""
        self.l4_dirs: List[Path] = []


class Tree:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.l2_dirs: List[Path] = sorted(p for p in root.iterdir() if p.is_dir() and p.name.startswith("L2-"))
        self.l2_ids: Dict[Path, str] = {}
        self.l3: Dict[Path, L3Info] = {}
        self.l3_by_id: Dict[str, L3Info] = {}
        self.bad_json: List[Tuple[Path, str]] = []

    def load(self) -> None:
        for l2 in self.l2_dirs:
            data, err = load_json_file(l2 / "L2.json")
            if err or not isinstance(data, dict):
                self.bad_json.append((l2 / "L2.json", err or "not an object"))
                continue
            self.l2_ids[l2] = data.get("id", "")
            for l3p in sorted(p for p in l2.iterdir() if p.is_dir() and p.name.startswith("L3-")):
                d3, e3 = load_json_file(l3p / "l3.json")
                if e3 or not isinstance(d3, dict):
                    self.bad_json.append((l3p / "l3.json", e3 or "not an object"))
                    continue
                info = L3Info(l3p, d3)
                info.l4_dirs = sorted(p for p in l3p.iterdir() if p.is_dir() and p.name.startswith("L4-"))
                self.l3[l3p] = info
                self.l3_by_id[info.id] = info


# --------------------------------------------------------------------------------------
# Digest of pre-existing files
# --------------------------------------------------------------------------------------


def _sha256_file(path: str) -> Tuple[int, str]:
    h = hashlib.sha256()
    size = 0
    with open(path, "rb") as fh:
        while True:
            chunk = fh.read(1 << 20)
            if not chunk:
                break
            size += len(chunk)
            h.update(chunk)
    return size, h.hexdigest()


def _group_files(root: Path, l2: Optional[Path]) -> List[Path]:
    """Pre-existing files of one group. l2=None means the files directly under ROOT."""
    if l2 is None:
        return sorted(p for p in root.iterdir() if p.is_file())
    out: List[Path] = []
    for dirpath, dirnames, filenames in os.walk(str(l2)):
        dp = Path(dirpath)
        if dp.parent == l2 and dp.name.startswith("L3-"):
            dirnames[:] = [d for d in dirnames if not d.startswith("L4-")]
            filenames = [f for f in filenames if f != "L4_INDEX.md"]
        for f in filenames:
            out.append(dp / f)
    return out


def group_digest(root: Path, l2: Optional[Path]) -> Tuple[int, str]:
    lines: List[str] = []
    for p in _group_files(root, l2):
        rel = p.relative_to(root).as_posix()
        size, sha = _sha256_file(str(p))
        lines.append(f"{rel}\x00{size}\x00{sha}\n")
    lines.sort()
    return len(lines), hashlib.sha256("".join(lines).encode("utf-8")).hexdigest()


def compute_digests(root: Path, only: Optional[List[Optional[Path]]] = None) -> Dict[str, Any]:
    groups: Dict[str, Any] = {}
    total = 0
    l2s = sorted(p for p in root.iterdir() if p.is_dir() and p.name.startswith("L2-"))
    targets: List[Optional[Path]] = [None] + l2s if only is None else only
    for g in targets:
        name = "ROOT" if g is None else g.name
        n, d = group_digest(root, g)
        groups[name] = {"files": n, "sha256": d}
        total += n
    return {"files_total": total, "groups": groups}


# --------------------------------------------------------------------------------------
# L4 document validation
# --------------------------------------------------------------------------------------


class Ctx:
    def __init__(
        self,
        rep: Report,
        root: Path,
        repo_root: Path,
        tree: Tree,
        known_l4: Dict[str, Dict[str, Any]],
        known_l2: Optional[Set[str]],
        glossary: Optional[Dict[str, Tuple[float, str]]],
        lenient_ids: bool = False,
    ) -> None:
        self.rep = rep
        self.root = root
        self.repo_root = repo_root
        self.tree = tree
        self.known_l4 = known_l4
        self.known_l2 = known_l2
        self.glossary = glossary
        self.lenient_ids = lenient_ids
        self._text_cache: Dict[Path, str] = {}

    def dep_doc(self, l4_id: str) -> Optional[Dict[str, Any]]:
        info = self.known_l4.get(l4_id)
        if not info:
            return None
        data, err = load_json_file(info["path"] / "l4.json")
        return data if not err and isinstance(data, dict) else None

    def read_text(self, p: Path) -> str:
        if p not in self._text_cache:
            try:
                self._text_cache[p] = p.read_text(encoding="utf-8", errors="replace")
            except OSError:
                self._text_cache[p] = ""
        return self._text_cache[p]


def _req(doc: Dict[str, Any], key: str, typ: Any, where: str, ctx: Ctx, check: str = "schema") -> Any:
    if key not in doc:
        ctx.rep.fail(check, where, f"missing required field '{key}'")
        return None
    v = doc[key]
    if typ is int:
        if not is_int(v):
            ctx.rep.fail(check, where, f"field '{key}' must be an integer")
            return None
    elif typ is str:
        if not nonempty_str(v):
            ctx.rep.fail(check, where, f"field '{key}' must be a non-empty string")
            return None
    elif typ is bool:
        if not isinstance(v, bool):
            ctx.rep.fail(check, where, f"field '{key}' must be a boolean")
            return None
    elif not isinstance(v, typ):
        ctx.rep.fail(check, where, f"field '{key}' must be of type {typ.__name__}")
        return None
    return v


def _str_list(doc: Dict[str, Any], key: str, where: str, ctx: Ctx, min_len: int = 1, min_chars: int = 3) -> List[str]:
    v = _req(doc, key, list, where, ctx)
    if v is None:
        return []
    if len(v) < min_len:
        ctx.rep.fail("schema", where, f"'{key}' needs at least {min_len} item(s)")
    for i, s in enumerate(v):
        if not nonempty_str(s) or len(s.strip()) < min_chars:
            ctx.rep.fail("schema", where, f"'{key}[{i}]' must be a non-empty string")
    return [s for s in v if isinstance(s, str)]


def _exact_keys(obj: Dict[str, Any], required: List[str], optional: List[str], where: str, ctx: Ctx, label: str) -> None:
    for k in obj:
        if k not in required and k not in optional:
            ctx.rep.fail("schema", where, f"{label}: unknown field '{k}'")


def _norm_repo_path(p: str) -> Optional[str]:
    if not nonempty_str(p) or "\\" in p or p.startswith("/") or re.match(r"^[A-Za-z]:", p) or ".." in p.split("/"):
        return None
    return p


def check_doc(doc: Any, folder: Path, ctx: Ctx, label: str) -> None:
    rep = ctx.rep
    if not isinstance(doc, dict):
        rep.fail("schema", label, "top level must be a JSON object")
        return
    rep.count("schema")
    _exact_keys(doc, TOP_REQUIRED, TOP_OPTIONAL, label, ctx, "top level")

    for key, typ in (("schema_version", str), ("level", str), ("id", str), ("milestone", str),
                     ("parent_l3_id", str), ("parent_l2_id", str), ("title", str),
                     ("planning_state", str), ("tag", str), ("owner_layer", str),
                     ("language", str), ("kind", str), ("goal", str)):
        _req(doc, key, typ, label, ctx)
    seq = _req(doc, "seq", int, label, ctx)

    if doc.get("schema_version") != SCHEMA_VERSION:
        rep.fail("schema", label, f"schema_version must be '{SCHEMA_VERSION}'")
    if doc.get("level") != "L4":
        rep.fail("schema", label, "level must be 'L4'")
    if doc.get("milestone") != MILESTONE:
        rep.fail("schema", label, f"milestone must be '{MILESTONE}'")
    if doc.get("planning_state") != "SPEC_ONLY_NOT_IMPLEMENTED":
        rep.fail("schema", label, "planning_state must be 'SPEC_ONLY_NOT_IMPLEMENTED'")
    if doc.get("tag") not in TAGS:
        rep.fail("schema", label, f"tag must be one of {sorted(TAGS)}")
    if doc.get("owner_layer") not in OWNER_LAYERS:
        rep.fail("schema", label, f"owner_layer must be one of {sorted(OWNER_LAYERS)}")
    if doc.get("language") not in LANGS:
        rep.fail("schema", label, f"language must be one of {sorted(LANGS)}")
    if doc.get("kind") not in KINDS:
        rep.fail("schema", label, f"kind must be one of {sorted(KINDS)}")
    title = doc.get("title")
    if isinstance(title, str) and (len(title) > 80 or "\n" in title):
        rep.fail("schema", label, "title must be one line, <= 80 chars")
    goal = doc.get("goal")
    if isinstance(goal, str):
        if "\n" in goal or len(goal) > 400:
            rep.fail("schema", label, "goal must be a single line, <= 400 chars")
        if not goal.rstrip().endswith("."):
            rep.fail("schema", label, "goal must be exactly one sentence ending with '.'")
        elif re.search(r"[.!?](\s+)[A-Z]", goal.strip()[:-1]):
            rep.fail("schema", label, "goal must be ONE sentence (found a second sentence)")

    # ---- identity ---------------------------------------------------------------------
    rep.count("identity")
    fm = FOLDER_RE.match(folder.name)
    if not fm:
        rep.fail("identity", label, f"folder name '{folder.name}' must match 'L4-NNN - Short Name' "
                 f"(ASCII letters/digits/space/_/+/-, short name 2-{MAX_SHORT_NAME} chars)")
    elif "  " in fm.group(2):
        rep.fail("identity", label, "folder name must not contain double spaces")
    if len(folder.name) > 9 + MAX_SHORT_NAME:
        rep.fail("identity", label, f"folder name longer than {9 + MAX_SHORT_NAME} chars")
    id_ = doc.get("id", "")
    l3id = doc.get("parent_l3_id", "")
    l2id = doc.get("parent_l2_id", "")
    idm = ID_RE.match(id_) if isinstance(id_, str) else None
    l3m = L3_RE.match(l3id) if isinstance(l3id, str) else None
    if not idm:
        rep.fail("identity", label, f"id '{id_}' must match OM9-L4-<LL>-<NN>-<LLL>-<NN>")
    if not l3m:
        rep.fail("identity", label, f"parent_l3_id '{l3id}' must match OM9-L3-<LL>-<NN>-<LLL>")
    if not (isinstance(l2id, str) and L2_RE.match(l2id)):
        rep.fail("identity", label, f"parent_l2_id '{l2id}' must match OM9-L2-<LL>-<NN>")
    if idm and l3m:
        if idm.group(1, 2, 3) != l3m.group(1, 2, 3):
            rep.fail("identity", label, "id prefix must be derived from parent_l3_id (OM9-L3-X -> OM9-L4-X-NN)")
        if is_int(seq) and int(idm.group(4)) != seq:
            rep.fail("identity", label, f"id suffix {idm.group(4)} must equal seq {seq}")
    if fm and is_int(seq) and int(fm.group(1)) != seq:
        rep.fail("identity", label, f"folder number {fm.group(1)} must equal seq {seq}")
    if is_int(seq) and not (1 <= seq <= 99):
        rep.fail("identity", label, "seq must be 1..99")
    if l3m and isinstance(l2id, str) and L2_RE.match(l2id):
        if f"OM9-L2-{l3m.group(1)}-{l3m.group(2)}" != l2id:
            rep.fail("identity", label, "parent_l2_id does not match the L2 part of parent_l3_id")
    l3info = ctx.tree.l3_by_id.get(l3id) if isinstance(l3id, str) else None
    if l3info is None:
        rep.fail("identity", label, f"parent_l3_id '{l3id}' does not exist in the M03 tree")
    else:
        if l3info.path != folder.parent:
            rep.fail("identity", label, "L4 folder is not located directly inside its parent L3 folder")
        if l3info.l2_id != l2id:
            rep.fail("identity", label, f"parent_l2_id must be '{l3info.l2_id}' (from l3.json)")
        if l3info.tag and doc.get("tag") != l3info.tag:
            rep.fail("identity", label, f"tag must equal the parent L3 tag '{l3info.tag}'")

    # ---- size --------------------------------------------------------------------------
    rep.count("size")
    size = _req(doc, "size", dict, label, ctx, "size")
    if size is not None:
        impl = size.get("impl_loc")
        tl = size.get("test_loc")
        if not is_int(impl) or not is_int(tl) or impl < 1 or tl < 1:
            rep.fail("size", label, "size.impl_loc and size.test_loc must be positive integers")
        else:
            if impl > MAX_IMPL_LOC:
                rep.fail("size", label, f"impl_loc {impl} > {MAX_IMPL_LOC}: split this L4")
            if impl + tl > MAX_TOTAL_LOC:
                rep.fail("size", label, f"impl_loc+test_loc {impl + tl} > {MAX_TOTAL_LOC}: split this L4")
        _exact_keys(size, ["impl_loc", "test_loc"], [], label, ctx, "size")

    # ---- scope -------------------------------------------------------------------------
    scope = _req(doc, "scope", dict, label, ctx)
    if scope is not None:
        _exact_keys(scope, ["inputs", "outputs", "not_in_this_task"], [], label, ctx, "scope")
        for k in ("inputs", "outputs", "not_in_this_task"):
            _str_list(scope, k, label, ctx)

    # ---- dependencies ------------------------------------------------------------------
    rep.count("dag")
    deps = _req(doc, "dependencies", dict, label, ctx)
    dep_ids: List[str] = []
    if deps is not None:
        _exact_keys(deps, ["depends_on_l4", "external_contracts", "reuse_symbols"], ["no_reuse_reason"], label, ctx, "dependencies")
        raw = _req(deps, "depends_on_l4", list, label, ctx, "dag")
        for d in raw or []:
            if not isinstance(d, str) or not ID_RE.match(d):
                rep.fail("dag", label, f"depends_on_l4 entry {d!r} is not an L4 id")
                continue
            dep_ids.append(d)
            if d == id_:
                rep.fail("dag", label, "an L4 cannot depend on itself")
            if ctx.lenient_ids:
                dl3 = "OM9-L3-" + d[len("OM9-L4-"):-3]
                if dl3 not in ctx.tree.l3_by_id:
                    rep.fail("dag", label, f"dependency {d} refers to unknown L3 {dl3}")
            elif d not in ctx.known_l4:
                rep.fail("dag", label, f"dependency {d} does not exist as an M03 L4")
        if len(set(dep_ids)) != len(dep_ids):
            rep.fail("dag", label, "duplicate ids in depends_on_l4")
        ext = _req(deps, "external_contracts", list, label, ctx)
        for i, e in enumerate(ext or []):
            w = f"{label} external_contracts[{i}]"
            if not isinstance(e, dict):
                rep.fail("schema", w, "must be an object")
                continue
            _exact_keys(e, ["l2_id", "contract", "status"], [], w, ctx, "external_contracts")
            l2e = e.get("l2_id")
            if not (isinstance(l2e, str) and L2_RE.match(l2e)):
                rep.fail("schema", w, "l2_id must match OM9-L2-<LL>-<NN>")
            elif ctx.known_l2 is not None and l2e not in ctx.known_l2:
                rep.fail("links", w, f"l2_id {l2e} not found in OM9_Milestones")
            if not nonempty_str(e.get("contract")):
                rep.fail("schema", w, "contract must be a non-empty string")
            if e.get("status") not in STATUS:
                rep.fail("schema", w, "status must be 'existing' or 'planned'")
        rs = _req(deps, "reuse_symbols", list, label, ctx)
        if rs is not None and not rs and not nonempty_str(deps.get("no_reuse_reason")):
            rep.fail("schema", label, "reuse_symbols is empty: give dependencies.no_reuse_reason")
        for i, s in enumerate(rs or []):
            w = f"{label} reuse_symbols[{i}]"
            if not isinstance(s, dict):
                rep.fail("schema", w, "must be an object")
                continue
            _exact_keys(s, ["path", "symbol", "kind", "status", "from_l4", "signature", "use"], [], w, ctx, "reuse_symbols")
            for k in ("path", "symbol", "use"):
                if not nonempty_str(s.get(k)):
                    rep.fail("schema", w, f"'{k}' must be a non-empty string")
            if s.get("kind") not in SYMBOL_KINDS:
                rep.fail("schema", w, f"kind must be one of {sorted(SYMBOL_KINDS)}")
            st = s.get("status")
            if st not in STATUS:
                rep.fail("schema", w, "status must be 'existing' or 'planned'")
            p = _norm_repo_path(s.get("path", "")) if isinstance(s.get("path"), str) else None
            if p is None:
                rep.fail("schema", w, "path must be repo-relative with '/' separators and no '..'")
            elif p.startswith("QymCAD/"):
                rep.fail("links", w, "reuse_symbols must not point into QymCAD (sample-only); use references.qymcad")
            if st == "existing":
                if s.get("from_l4") is not None:
                    rep.fail("schema", w, "existing symbols must have from_l4 = null")
                if p is not None:
                    target = ctx.repo_root / p
                    if not target.is_file():
                        rep.fail("links", w, f"existing symbol file not found: {p}")
                    else:
                        name = re.split(r"::", str(s.get("symbol", "")))[-1]
                        name = re.sub(r"[^A-Za-z0-9_].*$", "", name)
                        if name and not re.search(r"\b" + re.escape(name) + r"\b", ctx.read_text(target)):
                            rep.fail("links", w, f"symbol '{s.get('symbol')}' not found in {p}")
            elif st == "planned":
                fl4 = s.get("from_l4")
                if not (isinstance(fl4, str) and ID_RE.match(fl4)):
                    rep.fail("schema", w, "planned symbols need from_l4 = producing L4 id")
                elif fl4 not in dep_ids:
                    rep.fail("dag", w, f"planned symbol comes from {fl4}, which must be listed in depends_on_l4")
                if not nonempty_str(s.get("signature")):
                    rep.fail("schema", w, "planned symbols need the exact 'signature'")

    # ---- files -------------------------------------------------------------------------
    files = _req(doc, "files", dict, label, ctx)
    declared_paths: Dict[str, str] = {}
    lang_set: Set[str] = set()
    if files is not None:
        _exact_keys(files, ["create", "modify"], [], label, ctx, "files")
        for grp, key in (("create", "purpose"), ("modify", "change")):
            arr = _req(files, grp, list, label, ctx)
            for i, f in enumerate(arr or []):
                w = f"{label} files.{grp}[{i}]"
                if not isinstance(f, dict):
                    rep.fail("schema", w, "must be an object")
                    continue
                _exact_keys(f, ["path", "language", key] + (["from_l4"] if grp == "modify" else []), [], w, ctx, "files")
                p = _norm_repo_path(f.get("path", "")) if isinstance(f.get("path"), str) else None
                if p is None:
                    rep.fail("schema", w, "path must be repo-relative with '/' separators and no '..'")
                    continue
                if p.startswith("QymCAD/") or p.startswith("OM9_Milestones/"):
                    rep.fail("links", w, "files must live in OM9 code trees; never under QymCAD/ or OM9_Milestones/")
                if f.get("language") not in FILE_LANGS:
                    rep.fail("schema", w, f"language must be one of {sorted(FILE_LANGS)}")
                else:
                    lang_set.add(f["language"])
                if not nonempty_str(f.get(key)):
                    rep.fail("schema", w, f"'{key}' must be a non-empty string")
                if p in declared_paths:
                    rep.fail("schema", w, f"path {p} declared twice")
                declared_paths[p] = grp
                if grp == "modify":
                    fl4 = f.get("from_l4")
                    if "from_l4" not in f or not (fl4 is None or (isinstance(fl4, str) and ID_RE.match(fl4))):
                        rep.fail("schema", w, "from_l4 must be null (file exists today) or the L4 id that creates the file")
                    elif not (ctx.repo_root / p).is_file():
                        # The file does not exist yet: a dependency L4 must create it.
                        if fl4 is None:
                            rep.fail("links", w, f"file to modify does not exist: {p} (set from_l4 to the L4 that creates it)")
                        elif fl4 not in dep_ids:
                            rep.fail("dag", w, f"{p} is created by {fl4}, which must be listed in depends_on_l4")
                        elif not ctx.lenient_ids:
                            dep_doc = ctx.dep_doc(fl4)
                            created = [c.get("path") for c in (dep_doc or {}).get("files", {}).get("create", []) if isinstance(c, dict)]
                            if p not in created:
                                rep.fail("links", w, f"{fl4} does not list {p} in files.create")
        if files.get("create") == [] and files.get("modify") == []:
            rep.fail("schema", label, "files.create and files.modify cannot both be empty")
    owner = doc.get("owner_layer")
    lang = doc.get("language")
    if "cpp" in lang_set and owner not in ("occt-adapter", "opennurbs-adapter"):
        rep.fail("schema", label, "C++ files are only allowed for owner_layer occt-adapter/opennurbs-adapter")
    if "python" in lang_set and owner != "fixture-tool":
        rep.fail("schema", label, "Python files are only allowed for owner_layer fixture-tool")
    if lang != "rust" or (lang_set & {"cpp", "python"}):
        if not nonempty_str(doc.get("language_note")):
            rep.fail("schema", label, "language_note (real constraint justifying non-Rust code) is required")
    if lang == "cpp" and owner not in ("occt-adapter", "opennurbs-adapter"):
        rep.fail("schema", label, "primary language cpp is only allowed for native adapters")
    if lang == "python" and owner != "fixture-tool":
        rep.fail("schema", label, "primary language python is only allowed for fixture-tool")

    # ---- api ---------------------------------------------------------------------------
    api = _req(doc, "api", dict, label, ctx)
    if api is not None:
        _exact_keys(api, ["signatures", "data_structures", "error_variants", "invariants"], [], label, ctx, "api")
        sigs = _req(api, "signatures", list, label, ctx)
        if sigs is not None and len(sigs) < 1:
            rep.fail("schema", label, "api.signatures needs at least one entry")
        for i, s in enumerate(sigs or []):
            w = f"{label} api.signatures[{i}]"
            if not isinstance(s, dict):
                rep.fail("schema", w, "must be an object")
                continue
            _exact_keys(s, ["path", "code"], [], w, ctx, "signatures")
            if not nonempty_str(s.get("path")) or s.get("path") not in declared_paths:
                rep.fail("schema", w, "path must be listed in files.create or files.modify")
            if not nonempty_str(s.get("code")) or len(s["code"].strip()) < 12:
                rep.fail("schema", w, "code must contain the exact signature text")
            elif re.search(r"\bqymcad[_-][a-z]", s["code"], re.I):
                rep.fail("links", w, "OM9 code must not depend on QymCAD crates")
        ds = _req(api, "data_structures", list, label, ctx)
        for i, d in enumerate(ds or []):
            w = f"{label} api.data_structures[{i}]"
            if not isinstance(d, dict):
                rep.fail("schema", w, "must be an object")
                continue
            _exact_keys(d, ["name", "path", "fields"], [], w, ctx, "data_structures")
            if not nonempty_str(d.get("name")) or not nonempty_str(d.get("path")):
                rep.fail("schema", w, "name and path are required")
            flds = d.get("fields")
            if not isinstance(flds, list) or not flds:
                rep.fail("schema", w, "fields must be a non-empty list")
                continue
            for j, f in enumerate(flds):
                if not isinstance(f, dict):
                    rep.fail("schema", w, f"fields[{j}] must be an object")
                    continue
                _exact_keys(f, ["name", "type", "unit", "constraint"], [], w, ctx, "fields")
                for k in ("name", "type", "unit", "constraint"):
                    if not nonempty_str(f.get(k)):
                        rep.fail("schema", w, f"fields[{j}].{k} must be a non-empty string")
        ev = _req(api, "error_variants", list, label, ctx)
        for i, e in enumerate(ev or []):
            w = f"{label} api.error_variants[{i}]"
            if not isinstance(e, dict):
                rep.fail("schema", w, "must be an object")
                continue
            _exact_keys(e, ["enum", "variant", "display", "when"], [], w, ctx, "error_variants")
            for k in ("enum", "variant", "display", "when"):
                if not nonempty_str(e.get(k)):
                    rep.fail("schema", w, f"'{k}' must be a non-empty string")
        _str_list(api, "invariants", label, ctx)

    # ---- algorithm ---------------------------------------------------------------------
    rep.count("algorithm")
    alg = _req(doc, "algorithm", dict, label, ctx, "algorithm")
    if alg is not None:
        _exact_keys(alg, ["tolerances", "parameter_ranges", "steps", "edge_cases", "failures"], [], label, ctx, "algorithm")
        tols = _req(alg, "tolerances", list, label, ctx, "algorithm")
        for i, t in enumerate(tols or []):
            w = f"{label} algorithm.tolerances[{i}]"
            if not isinstance(t, dict):
                rep.fail("algorithm", w, "must be an object")
                continue
            _exact_keys(t, ["name", "value", "unit"], ["local"], w, ctx, "tolerances")
            name = t.get("name")
            if not (isinstance(name, str) and TOL_NAME_RE.match(name)):
                rep.fail("algorithm", w, "name must be UPPER_SNAKE_CASE")
                continue
            try:
                val = float(str(t.get("value")))
            except ValueError:
                rep.fail("algorithm", w, "value must be a number written as text, e.g. \"1e-7\"")
                continue
            if not nonempty_str(t.get("unit")):
                rep.fail("algorithm", w, "unit is required (use 'none' for dimensionless)")
            if ctx.glossary is not None:
                if name in ctx.glossary:
                    gv, gu = ctx.glossary[name]
                    if not math.isclose(val, gv, rel_tol=1e-12, abs_tol=0.0) and not (val == 0 and gv == 0):
                        rep.fail("algorithm", w, f"{name} must be {gv:g} (glossary), got {t.get('value')}")
                    if t.get("unit") != gu:
                        rep.fail("algorithm", w, f"{name} unit must be '{gu}' (glossary)")
                elif t.get("local") is not True:
                    rep.fail("algorithm", w, f"{name} is not in the glossary: add it there or set \"local\": true")
        pr = _req(alg, "parameter_ranges", list, label, ctx, "algorithm")
        for i, r in enumerate(pr or []):
            w = f"{label} algorithm.parameter_ranges[{i}]"
            if not isinstance(r, dict):
                rep.fail("algorithm", w, "must be an object")
                continue
            _exact_keys(r, ["name", "range", "unit"], [], w, ctx, "parameter_ranges")
            for k in ("name", "range", "unit"):
                if not nonempty_str(r.get(k)):
                    rep.fail("algorithm", w, f"'{k}' must be a non-empty string")
        steps = _req(alg, "steps", list, label, ctx, "algorithm")
        if steps is not None:
            if len(steps) < 3:
                rep.fail("algorithm", label, "algorithm.steps needs at least 3 micro-steps")
            for i, s in enumerate(steps):
                w = f"{label} algorithm.steps[{i}]"
                if not isinstance(s, dict):
                    rep.fail("algorithm", w, "must be an object")
                    continue
                _exact_keys(s, ["n", "action", "formula", "on_failure"], [], w, ctx, "steps")
                if s.get("n") != i + 1 or not is_int(s.get("n")):
                    rep.fail("algorithm", w, f"n must be {i + 1} (steps are numbered 1..N without gaps)")
                if not nonempty_str(s.get("action")):
                    rep.fail("algorithm", w, "action must be a non-empty string")
                for k in ("formula", "on_failure"):
                    v = s.get(k)
                    if k not in s or not (v is None or nonempty_str(v)):
                        rep.fail("algorithm", w, f"'{k}' must be a string or null")
        ec = _req(alg, "edge_cases", list, label, ctx, "algorithm")
        if ec is not None and len(ec) < 2:
            rep.fail("algorithm", label, "algorithm.edge_cases needs at least 2 entries")
        for i, e in enumerate(ec or []):
            w = f"{label} algorithm.edge_cases[{i}]"
            if not isinstance(e, dict) or not nonempty_str(e.get("case")) or not nonempty_str(e.get("behavior")):
                rep.fail("algorithm", w, "needs non-empty 'case' and 'behavior'")
            elif isinstance(e, dict):
                _exact_keys(e, ["case", "behavior"], [], w, ctx, "edge_cases")
        fl = _req(alg, "failures", list, label, ctx, "algorithm")
        if fl is not None and len(fl) < 1:
            rep.fail("algorithm", label, "algorithm.failures needs at least 1 entry")
        for i, e in enumerate(fl or []):
            w = f"{label} algorithm.failures[{i}]"
            if not isinstance(e, dict):
                rep.fail("algorithm", w, "must be an object")
                continue
            _exact_keys(e, ["condition", "error", "message", "state_after"], [], w, ctx, "failures")
            for k in ("condition", "error", "message", "state_after"):
                if not nonempty_str(e.get(k)):
                    rep.fail("algorithm", w, f"'{k}' must be a non-empty string")

    # ---- interaction -------------------------------------------------------------------
    rep.count("interaction")
    if "interaction" in doc:
        inter = doc["interaction"]
        if inter is None:
            if not nonempty_str(doc.get("interaction_na_reason")):
                rep.fail("interaction", label, "interaction is null: interaction_na_reason is required")
        else:
            if "interaction_na_reason" in doc:
                rep.fail("interaction", label, "interaction_na_reason must be absent when interaction is an object")
            if not isinstance(inter, dict):
                rep.fail("interaction", label, "interaction must be null or an object")
            else:
                _check_interaction(inter, label, ctx)

    # ---- tests -------------------------------------------------------------------------
    rep.count("tests")
    tests = _req(doc, "tests", dict, label, ctx, "tests")
    if tests is not None:
        _check_tests(tests, doc, declared_paths, label, ctx)

    # ---- definition of done ------------------------------------------------------------
    dod = _str_list(doc, "definition_of_done", label, ctx, min_len=4, min_chars=10)
    if dod and not any(("cargo test" in x or "unittest" in x) for x in dod):
        rep.fail("schema", label, "definition_of_done must contain an item running the test command")

    # ---- references --------------------------------------------------------------------
    rep.count("links")
    refs = _req(doc, "references", dict, label, ctx, "links")
    if refs is not None:
        _check_refs(refs, doc, folder, label, ctx)

    # ---- assumptions -------------------------------------------------------------------
    asm = _req(doc, "assumptions", list, label, ctx)
    seen_a: Set[str] = set()
    for i, a in enumerate(asm or []):
        w = f"{label} assumptions[{i}]"
        if not isinstance(a, dict):
            rep.fail("schema", w, "must be an object")
            continue
        _exact_keys(a, ["id", "statement", "chosen"], [], w, ctx, "assumptions")
        aid = a.get("id")
        if not (isinstance(aid, str) and ASSUMPTION_ID_RE.match(aid)) or aid in seen_a:
            rep.fail("schema", w, "id must be unique and look like A1, A2, ...")
        else:
            seen_a.add(aid)
        for k in ("statement", "chosen"):
            if not nonempty_str(a.get(k)):
                rep.fail("schema", w, f"'{k}' must be a non-empty string")

    # ---- placeholders / banned words ---------------------------------------------------
    rep.count("placeholders")
    for p, s in walk_strings(doc):
        for rx, name in BANNED:
            if rx.search(s):
                rep.fail("placeholders", f"{label} {p}", f"banned placeholder text '{name}'")

    # ---- cited ids and relative links anywhere in prose --------------------------------
    for p, s in walk_strings(doc):
        for m in re.finditer(r"OM9-L4-\d\d-\d\d-\d{3}-\d\d", s):
            tok = m.group(0)
            if ctx.lenient_ids:
                if "OM9-L3-" + tok[len("OM9-L4-"):-3] not in ctx.tree.l3_by_id:
                    rep.fail("links", f"{label} {p}", f"cited {tok} belongs to an unknown L3")
            elif tok not in ctx.known_l4:
                rep.fail("links", f"{label} {p}", f"cited L4 id {tok} does not exist")
        for m in re.finditer(r"OM9-L3-\d\d-\d\d-\d{3}", s):
            if m.group(0) not in ctx.tree.l3_by_id:
                rep.fail("links", f"{label} {p}", f"cited L3 id {m.group(0)} does not exist")
        if ctx.known_l2 is not None:
            for m in re.finditer(r"OM9-L2-\d\d-\d\d", s):
                if m.group(0) not in ctx.known_l2:
                    rep.fail("links", f"{label} {p}", f"cited L2 id {m.group(0)} does not exist")
        for m in re.finditer(r"(?:\.\./)+reference/[^\s\"'`)\]]+", s):
            tgt = (folder / m.group(0)).resolve() if folder.exists() else None
            if tgt is not None and not tgt.exists():
                rep.fail("links", f"{label} {p}", f"cited link target missing: {m.group(0)}")

    # ---- path length for the artifacts this L4 adds ------------------------------------
    rep.count("pathlen")
    for fname in ("l4.json", "L4.md"):
        n = path_len(folder / fname, ctx.root)
        if n >= MAX_PATH_EXCLUSIVE:
            rep.fail("pathlen", label, f"{fname} path is {n} chars (limit < {MAX_PATH_EXCLUSIVE}); shorten the folder name")


def _check_interaction(inter: Dict[str, Any], label: str, ctx: Ctx) -> None:
    rep = ctx.rep
    w = f"{label} interaction"
    _exact_keys(inter, ["feature_id", "command", "entry_points", "prompts", "defaults", "keys", "preview", "source_note"], [], w, ctx, "interaction")
    fid = inter.get("feature_id")
    if not (isinstance(fid, str) and FEATURE_RE.match(fid)):
        rep.fail("interaction", w, "feature_id must look like OM9-SOLID-012 (see standalone/crates/om9-app/assets/features.json)")
    for k in ("command", "source_note"):
        if not nonempty_str(inter.get(k)):
            rep.fail("interaction", w, f"'{k}' must be a non-empty string")
    _str_list(inter, "entry_points", w, ctx)
    prompts = inter.get("prompts")
    if not isinstance(prompts, list) or not prompts:
        rep.fail("interaction", w, "prompts must be a non-empty list")
        prompts = []
    seen: Set[str] = set()
    for i, p in enumerate(prompts):
        pw = f"{w}.prompts[{i}]"
        if not isinstance(p, dict):
            rep.fail("interaction", pw, "must be an object")
            continue
        _exact_keys(p, ["id", "text", "accepts", "options", "on_enter", "on_esc", "on_undo"], [], pw, ctx, "prompt")
        for k in ("id", "text", "accepts", "on_enter", "on_esc", "on_undo"):
            if not nonempty_str(p.get(k)):
                rep.fail("interaction", pw, f"'{k}' must be a non-empty string")
        if p.get("id") in seen:
            rep.fail("interaction", pw, "duplicate prompt id")
        seen.add(str(p.get("id")))
        opts = p.get("options")
        if not isinstance(opts, list):
            rep.fail("interaction", pw, "options must be a list (may be empty)")
            continue
        for j, o in enumerate(opts):
            ow = f"{pw}.options[{j}]"
            if not isinstance(o, dict):
                rep.fail("interaction", ow, "must be an object")
                continue
            _exact_keys(o, ["name", "alias", "default", "enabled", "reason"], [], ow, ctx, "option")
            if not nonempty_str(o.get("name")):
                rep.fail("interaction", ow, "name is required")
            for k in ("alias", "default", "reason"):
                if k not in o or not (o[k] is None or nonempty_str(o[k])):
                    rep.fail("interaction", ow, f"'{k}' must be a string or null")
            if not isinstance(o.get("enabled"), bool):
                rep.fail("interaction", ow, "enabled must be a boolean")
            elif o["enabled"] is False and not nonempty_str(o.get("reason")):
                rep.fail("interaction", ow, "a disabled option needs a visible 'reason'")
    defaults = inter.get("defaults")
    if not isinstance(defaults, list):
        rep.fail("interaction", w, "defaults must be a list (may be empty)")
    else:
        for j, d in enumerate(defaults):
            if not (isinstance(d, dict) and nonempty_str(d.get("name")) and nonempty_str(d.get("value"))):
                rep.fail("interaction", w, f"defaults[{j}] needs name and value strings")
    keys = inter.get("keys")
    if not isinstance(keys, dict):
        rep.fail("interaction", w, "keys must be an object with enter/esc/undo")
    else:
        _exact_keys(keys, ["enter", "esc", "undo"], [], w, ctx, "keys")
        for k in ("enter", "esc", "undo"):
            if not nonempty_str(keys.get(k)):
                rep.fail("interaction", w, f"keys.{k} must be a non-empty string")
    pv = inter.get("preview")
    if "preview" not in inter or not (pv is None or nonempty_str(pv)):
        rep.fail("interaction", w, "preview must be a string or null")


def _check_tests(tests: Dict[str, Any], doc: Dict[str, Any], declared: Dict[str, str], label: str, ctx: Ctx) -> None:
    rep = ctx.rep
    w = f"{label} tests"
    _exact_keys(tests, ["file", "command", "native_command", "cases", "fixtures"], [], w, ctx, "tests")
    tf = tests.get("file")
    if not nonempty_str(tf) or _norm_repo_path(tf) is None:
        rep.fail("tests", w, "file must be a repo-relative path")
    elif tf not in declared:
        rep.fail("tests", w, f"test file {tf} must be listed in files.create or files.modify")
    cmd = tests.get("command")
    flt = None
    if not nonempty_str(cmd):
        rep.fail("tests", w, "command is required")
    else:
        m = CARGO_CMD_RE.match(cmd)
        if m:
            flt = m.group(1)
            if doc.get("language") == "python":
                rep.fail("tests", w, "python L4 must use a python -m unittest command")
        elif PY_CMD_RE.match(cmd):
            if doc.get("language") != "python":
                rep.fail("tests", w, "python test commands are only allowed when language is python")
        else:
            rep.fail("tests", w, "command must be 'cargo test --locked --manifest-path standalone/Cargo.toml -p <crate> <filter>' "
                     "(or the legacy rust/Cargo.toml form, or python -m unittest for fixture tools)")
    native = tests.get("native_command")
    if "native_command" not in tests or not (native is None or nonempty_str(native)):
        rep.fail("tests", w, "native_command must be a string or null")
    elif isinstance(native, str) and not CARGO_NATIVE_RE.match(native):
        rep.fail("tests", w, "native_command must end with '-- --ignored' and use the standalone workspace")
    fx = tests.get("fixtures")
    if not isinstance(fx, list) or not all(nonempty_str(x) for x in fx):
        rep.fail("tests", w, "fixtures must be a list of strings (may be empty)")
    cases = tests.get("cases")
    if not isinstance(cases, list):
        rep.fail("tests", w, "cases must be a list")
        return
    names: Set[str] = set()
    portable = 0
    portable_neg = 0
    any_native = False
    for i, c in enumerate(cases):
        cw = f"{w}.cases[{i}]"
        if not isinstance(c, dict):
            rep.fail("tests", cw, "must be an object")
            continue
        _exact_keys(c, ["name", "kind", "runs_in", "input", "expected"], [], cw, ctx, "case")
        nm = c.get("name")
        if not (isinstance(nm, str) and IDENT_RE.match(nm)):
            rep.fail("tests", cw, "name must be a snake_case test function name (3-80 chars)")
        elif nm in names:
            rep.fail("tests", cw, f"duplicate test name {nm}")
        else:
            names.add(nm)
            if flt and flt not in nm:
                rep.fail("tests", cw, f"test name must contain the command filter '{flt}' so one command runs it")
        if c.get("kind") not in CASE_KINDS:
            rep.fail("tests", cw, f"kind must be one of {sorted(CASE_KINDS)}")
        if c.get("runs_in") not in RUNS_IN:
            rep.fail("tests", cw, f"runs_in must be one of {sorted(RUNS_IN)}")
        for k in ("input", "expected"):
            v = c.get(k)
            if not nonempty_str(v) or len(v.strip()) < 8:
                rep.fail("tests", cw, f"'{k}' must state explicit values (non-empty text)")
        if c.get("runs_in") == "portable":
            portable += 1
            if c.get("kind") == "negative":
                portable_neg += 1
        elif c.get("runs_in") == "native-ignored":
            any_native = True
    if portable < 3:
        rep.fail("tests", w, f"needs >= 3 portable test cases (found {portable})")
    if portable_neg < 1:
        rep.fail("tests", w, "needs >= 1 portable NEGATIVE case")
    if any_native and not isinstance(native, str):
        rep.fail("tests", w, "native-ignored cases require native_command")


def _check_refs(refs: Dict[str, Any], doc: Dict[str, Any], folder: Path, label: str, ctx: Ctx) -> None:
    rep = ctx.rep
    w = f"{label} references"
    _exact_keys(refs, ["text", "images", "qymcad", "legacy_om9", "behavior_notes", "no_reference_reason"], [], w, ctx, "references")
    for k in ("text", "images", "qymcad", "legacy_om9", "behavior_notes"):
        if not isinstance(refs.get(k), list) or not all(nonempty_str(x) for x in refs.get(k, [])):
            rep.fail("links", w, f"'{k}' must be a list of non-empty strings (may be empty)")
    nrr = refs.get("no_reference_reason")
    if "no_reference_reason" not in refs or not (nrr is None or nonempty_str(nrr)):
        rep.fail("links", w, "no_reference_reason must be a string or null")
    root = ctx.root.resolve()
    for kind, sub in (("text", "text"), ("images", "images")):
        for p in refs.get(kind, []) if isinstance(refs.get(kind), list) else []:
            if not isinstance(p, str):
                continue
            if not p.startswith("../") or "\\" in p:
                rep.fail("links", w, f"{kind} link '{p}' must be relative to the L4 folder and start with ../ (L3 reference) or ../../ (L2 reference)")
                continue
            if f"/reference/{sub}/" not in p:
                rep.fail("links", w, f"{kind} link '{p}' must point into a reference/{sub}/ folder")
            tgt = (folder / p).resolve()
            try:
                tgt.relative_to(root)
            except ValueError:
                rep.fail("links", w, f"{kind} link '{p}' leaves the M03 tree")
                continue
            if not tgt.is_file():
                rep.fail("links", w, f"{kind} link target not found: {p}")
    for key, prefix in (("qymcad", "QymCAD/"),):
        for p in refs.get(key, []) if isinstance(refs.get(key), list) else []:
            if not isinstance(p, str):
                continue
            if not p.startswith(prefix) or _norm_repo_path(p) is None:
                rep.fail("links", w, f"{key} path '{p}' must start with {prefix} (repo-relative, '/' separators)")
            elif not (ctx.repo_root / p).is_file():
                rep.fail("links", w, f"{key} file not found: {p}")
    for p in refs.get("legacy_om9", []) if isinstance(refs.get("legacy_om9"), list) else []:
        if not isinstance(p, str):
            continue
        if _norm_repo_path(p) is None or p.startswith(("QymCAD/", "OM9_Milestones/")):
            rep.fail("links", w, f"legacy_om9 path '{p}' must be a repo-relative OM9 code path")
        elif not (ctx.repo_root / p).is_file():
            rep.fail("links", w, f"legacy_om9 file not found: {p}")
    has_ref = bool(refs.get("text")) or bool(refs.get("images"))
    if not has_ref and not nonempty_str(nrr):
        rep.fail("links", w, "no reference text/images: give references.no_reference_reason")
    if has_ref and nrr is not None:
        rep.fail("links", w, "no_reference_reason must be null when reference links are present")
    if doc.get("interaction") is not None and not refs.get("text") and not nonempty_str(nrr):
        rep.fail("links", w, "an interactive L4 must cite reference text (or explain why none exists)")


# --------------------------------------------------------------------------------------
# Directory level checks
# --------------------------------------------------------------------------------------


def check_dag_cycles(rep: Report, known: Dict[str, Dict[str, Any]], scope_ids: Set[str]) -> None:
    rep.count("dag")
    graph = {i: [d for d in v.get("deps", []) if d in known] for i, v in known.items()}
    WHITE, GREY, BLACK = 0, 1, 2
    color = {n: WHITE for n in graph}
    reported: Set[Tuple[str, ...]] = set()
    for start in sorted(graph):
        if color[start] != WHITE:
            continue
        stack: List[Tuple[str, int]] = [(start, 0)]
        path: List[str] = []
        while stack:
            node, idx = stack.pop()
            if idx == 0:
                color[node] = GREY
                path.append(node)
            if idx < len(graph[node]):
                stack.append((node, idx + 1))
                nxt = graph[node][idx]
                if color[nxt] == WHITE:
                    stack.append((nxt, 0))
                elif color[nxt] == GREY:
                    cyc = path[path.index(nxt):] + [nxt]
                    key = tuple(sorted(set(cyc)))
                    if key not in reported and (not scope_ids or any(c in scope_ids for c in cyc)):
                        reported.add(key)
                        rep.fail("dag", "cycle", " -> ".join(cyc))
            else:
                color[node] = BLACK
                path.pop()


def parse_index_sections(text: str) -> Dict[str, List[str]]:
    sections: Dict[str, List[str]] = {}
    cur: Optional[str] = None
    for line in text.splitlines():
        if line.startswith("## "):
            cur = line[3:].strip()
            sections[cur] = []
        elif cur is not None:
            sections[cur].append(line)
    return sections


def check_index(rep: Report, l3: L3Info, docs: Dict[str, Dict[str, Any]], root: Path) -> None:
    rep.count("index")
    p = l3.path / "L4_INDEX.md"
    where = short_rel(p, root)
    if not p.is_file():
        rep.fail("index", where, "L4_INDEX.md is missing in this L3 folder")
        return
    text = p.read_text(encoding="utf-8", errors="replace")
    first = text.splitlines()[0] if text else ""
    if not first.startswith(f"# L4 Index - {l3.id} - "):
        rep.fail("index", where, f"first line must start with '# L4 Index - {l3.id} - <L3 title>'")
    for rx, name in BANNED:
        if rx.search(text):
            rep.fail("placeholders", where, f"banned placeholder text '{name}'")
    secs = parse_index_sections(text)
    for need in ("L4 list", "Dependency order", "Recommended implementation order"):
        if need not in secs:
            rep.fail("index", where, f"missing section '## {need}'")
    ids = sorted(docs)
    table_ids: Set[str] = set()
    for row in secs.get("L4 list", []):
        if row.lstrip().startswith("|"):
            cells = [c.strip() for c in row.strip().strip("|").split("|")]
            if len(cells) >= 2 and re.fullmatch(r"OM9-L4-\d\d-\d\d-\d{3}-\d\d", cells[1]):
                table_ids.add(cells[1])  # second column only: dependency columns may cite other L3's ids
    if table_ids != set(ids):
        rep.fail("index", where, f"'L4 list' table must list exactly {len(ids)} L4 ids (missing {sorted(set(ids) - table_ids)}, extra {sorted(table_ids - set(ids))})")
    orders: Dict[str, List[str]] = {}
    for need in ("Dependency order", "Recommended implementation order"):
        items = [m.group(1) for l in secs.get(need, []) for m in [re.match(r"^\s*\d+\.\s+(OM9-L4-\d\d-\d\d-\d{3}-\d\d)", l)] if m]
        orders[need] = items
        if sorted(items) != ids:
            rep.fail("index", where, f"'{need}' must be a numbered list containing every L4 id exactly once")
    for need, items in orders.items():
        pos = {i: n for n, i in enumerate(items)}
        for i in items:
            for d in docs.get(i, {}).get("deps", []):
                if d in pos and pos[d] > pos[i]:
                    rep.fail("index", where, f"'{need}': {i} is listed before its dependency {d}")


def check_l4_md(rep: Report, folder: Path, doc: Dict[str, Any], root: Path) -> None:
    md = folder / "L4.md"
    if not md.is_file():
        return
    rep.count("l4md")
    where = short_rel(md, root)
    try:
        raw = md.read_bytes()
        text = raw.decode("utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        rep.fail("l4md", where, f"cannot read as UTF-8: {exc}")
        return
    first = text.splitlines()[0] if text else ""
    if first != f"# {doc.get('id')} - {doc.get('title')}":
        rep.fail("l4md", where, f"first line must be '# {doc.get('id')} - {doc.get('title')}'")
    for h in ("## Goal", "## Scope", "## Steps", "## Tests", "## Definition of done", "## References"):
        if not re.search(r"^" + re.escape(h) + r"\s*$", text, re.M):
            rep.fail("l4md", where, f"missing heading '{h}'")
    for c in (doc.get("tests", {}) or {}).get("cases", []) if isinstance(doc.get("tests"), dict) else []:
        if isinstance(c, dict) and isinstance(c.get("name"), str) and c["name"] not in text:
            rep.fail("l4md", where, f"test name {c['name']} from l4.json is not mentioned")
    for rx, name in BANNED:
        if rx.search(text):
            rep.fail("placeholders", where, f"banned placeholder text '{name}'")
    n = path_len(md, root)
    if n >= MAX_PATH_EXCLUSIVE:
        rep.fail("pathlen", where, f"path is {n} chars (limit < {MAX_PATH_EXCLUSIVE})")


def collect_all_l4(tree: Tree) -> Dict[str, Dict[str, Any]]:
    """Light scan of EVERY L4 in the tree (id -> {deps, path}) for global DAG resolution."""
    known: Dict[str, Dict[str, Any]] = {}
    for info in tree.l3.values():
        for d in info.l4_dirs:
            data, err = load_json_file(d / "l4.json")
            if err or not isinstance(data, dict):
                continue
            i = data.get("id")
            deps = data.get("dependencies", {}).get("depends_on_l4", []) if isinstance(data.get("dependencies"), dict) else []
            if isinstance(i, str) and i not in known:
                known[i] = {"deps": [x for x in deps if isinstance(x, str)] if isinstance(deps, list) else [], "path": d}
    return known


def global_l2_ids(root: Path, tree: Tree) -> Optional[Set[str]]:
    ids = {v for v in tree.l2_ids.values() if v}
    ms_root = root.parent
    sibs = [p for p in ms_root.iterdir() if p.is_dir() and re.match(r"^M\d\d ", p.name)] if ms_root.is_dir() else []
    if len(sibs) < 2:
        return None  # not inside OM9_Milestones: cannot verify cross-milestone L2 ids
    for m in sibs:
        for l2 in m.iterdir():
            if l2.is_dir() and l2.name.startswith("L2-") and (l2 / "L2.json").is_file():
                data, err = load_json_file(l2 / "L2.json")
                if not err and isinstance(data, dict) and isinstance(data.get("id"), str):
                    ids.add(data["id"])
    return ids


def check_integrity(rep: Report, root: Path, scope_l2: Optional[Path], expected: Dict[str, Any], tree: Tree, allow_index_updates: bool) -> None:
    groups = expected.get("groups", {})
    if not groups:
        rep.fail("integrity", "digests", "no expected digests embedded/provided: pre-existing files cannot be verified")
        return
    names: List[Optional[Path]] = [None]
    if scope_l2 is None:
        names += tree.l2_dirs
    else:
        names.append(scope_l2)
    for g in names:
        rep.count("integrity")
        gname = "ROOT" if g is None else g.name
        exp = groups.get(gname)
        if exp is None:
            rep.fail("integrity", gname, "group missing from expected digests")
            continue
        n, d = group_digest(root, g)
        if n != exp["files"]:
            rep.fail("integrity", gname, f"pre-existing file count changed: expected {exp['files']}, found {n} (file added or removed)")
        elif d != exp["sha256"]:
            rep.fail("integrity", gname, "pre-existing file content changed (a file was modified or renamed); restore it")
    # structural 'still untouched' invariants (cheap, give a precise message)
    if not allow_index_updates:
        l1, err = load_json_file(root / "L1.json")
        if err or not isinstance(l1, dict):
            rep.fail("integrity", "L1.json", err or "not an object")
        elif l1.get("l4_count") != 0:
            rep.fail("integrity", "L1.json", f"l4_count must stay 0 (found {l1.get('l4_count')}); L1.json must not be edited")
        for info in tree.l3.values():
            if scope_l2 is not None and info.path.parent != scope_l2:
                continue
            data, err = load_json_file(info.path / "l3.json")
            if err or not isinstance(data, dict):
                continue
            if data.get("children_l4") != [] or data.get("l4_status") != "NOT_PLANNED_YET":
                rep.fail("integrity", short_rel(info.path / "l3.json", root), "l3.json must stay unchanged (children_l4 == [] and l4_status == 'NOT_PLANNED_YET')")


def check_structure(rep: Report, tree: Tree, scope_l2: Optional[Path], root: Path) -> None:
    """Only L4-NNN folders (with l4.json / L4.md) and L4_INDEX.md may be added, and only inside L3 folders."""
    rep.count("structure")
    l2s = tree.l2_dirs if scope_l2 is None else [scope_l2]
    for l2 in l2s:
        for child in l2.iterdir():
            if child.name.lower().startswith("l4"):
                rep.fail("structure", short_rel(child, root), "L4 artifacts must live inside an L3 folder, not directly in the L2 folder")
        for l3p in sorted(p for p in l2.iterdir() if p.is_dir() and p.name.startswith("L3-")):
            for child in l3p.iterdir():
                if child.is_dir() and child.name.startswith("L4-"):
                    for f in child.iterdir():
                        if f.is_dir() or f.name not in ("l4.json", "L4.md"):
                            rep.fail("structure", short_rel(f, root), "only l4.json and optional L4.md may exist in an L4 folder")
                    if not (child / "l4.json").is_file():
                        rep.fail("structure", short_rel(child, root), "L4 folder has no l4.json")
                elif child.name.lower().startswith("l4") and child.name != "L4_INDEX.md":
                    rep.fail("structure", short_rel(child, root), "unexpected L4-like entry in an L3 folder (use 'L4-NNN - Name' folders and L4_INDEX.md)")


# --------------------------------------------------------------------------------------
# Examples mode
# --------------------------------------------------------------------------------------

EXAMPLE_FENCE = re.compile(r"^```json l4-example folder=\"([^\"]+)\"\s*$")


def extract_examples(md: Path) -> List[Tuple[str, str]]:
    out: List[Tuple[str, str]] = []
    cur: Optional[List[str]] = None
    folder = ""
    for line in md.read_text(encoding="utf-8").splitlines():
        if cur is None:
            m = EXAMPLE_FENCE.match(line)
            if m:
                folder = m.group(1)
                cur = []
        elif line.strip() == "```":
            out.append((folder, "\n".join(cur)))
            cur = None
        else:
            cur.append(line)
    return out


# --------------------------------------------------------------------------------------
# Main
# --------------------------------------------------------------------------------------


def resolve_l2(root: Path, arg: str) -> Optional[Path]:
    for c in (Path(arg), root / arg):
        if c.is_dir() and c.name.startswith("L2-") and c.resolve().parent == root.resolve():
            return root / c.name
    matches = [p for p in root.iterdir() if p.is_dir() and p.name.startswith("L2-") and p.name.startswith(arg)]
    return matches[0] if len(matches) == 1 else None


def print_report(rep: Report, args: argparse.Namespace, summary: Dict[str, Any]) -> None:
    out = sys.stdout
    out.write("\n=== OM9 M03 L4 validation ===\n")
    for k, v in summary.items():
        out.write(f"{k}: {v}\n")
    out.write("\nCheck                 checked  failed\n")
    for c in rep.order:
        out.write(f"  {c:<18} {rep.checked[c]:>7}  {rep.failed[c]:>6}\n")
    if rep.warnings:
        out.write("\nWARNINGS:\n")
        for w in rep.warnings:
            out.write(f"  - {w}\n")
    if rep.failures:
        out.write(f"\nFAILURES ({len(rep.failures)}):\n")
        shown = 0
        for check, where, msg in rep.failures:
            out.write(f"  [{check}] {where}: {msg}\n")
            shown += 1
            if shown >= args.max_failures:
                out.write(f"  ... {len(rep.failures) - shown} more (use --max-failures N)\n")
                break
    out.write("\nRESULT: " + ("PASS" if rep.ok() else "FAIL") + "\n")


def main(argv: Optional[List[str]] = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    ap = argparse.ArgumentParser(description="Validate OM9 M03 L4 planning artifacts (spec only).")
    ap.add_argument("root", help="M03 root folder ('M03 - Curve, Surface, Solid and 3DM Exchange')")
    ap.add_argument("l2", nargs="?", help="optional single L2 folder (path, or name prefix such as L2-01-09)")
    ap.add_argument("--l2", dest="l2_opt", help="same as the positional L2 argument")
    ap.add_argument("--repo-root", help="OpenMatrix9 repo root (default: two levels above ROOT)")
    ap.add_argument("--conventions", help="CONVENTIONS.md used for the tolerance glossary (default: next to this script)")
    ap.add_argument("--check-examples", metavar="CONVENTIONS_MD", help="validate the l4-example blocks of a conventions file")
    ap.add_argument("--expected-digests", metavar="FILE", help="JSON with pre-existing-file digests (override of the embedded ones)")
    ap.add_argument("--print-digests", action="store_true", help="print digests of the current pre-existing files and exit")
    ap.add_argument("--allow-index-updates", action="store_true", help="do not require l3.json/L1.json to stay untouched (approved changes only)")
    ap.add_argument("--no-index-check", action="store_true", help="skip L4_INDEX.md checks")
    ap.add_argument("--json", action="store_true", help="print a JSON summary instead of text")
    ap.add_argument("--max-failures", type=int, default=60)
    args = ap.parse_args(argv)

    root = Path(args.root)
    if not root.is_dir() or not (root / "L1.json").is_file():
        print(f"error: {root} is not an M03 root (no L1.json)", file=sys.stderr)
        return 2
    root = root.resolve()
    repo_root = Path(args.repo_root).resolve() if args.repo_root else root.parent.parent
    if not (repo_root / "AGENTS.md").is_file():
        print(f"error: repo root {repo_root} has no AGENTS.md; pass --repo-root", file=sys.stderr)
        return 2

    if args.print_digests:
        json.dump(compute_digests(root), sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0

    l2_arg = args.l2_opt or args.l2
    scope_l2: Optional[Path] = None
    if l2_arg:
        scope_l2 = resolve_l2(root, l2_arg)
        if scope_l2 is None:
            print(f"error: cannot resolve a single L2 folder from '{l2_arg}'", file=sys.stderr)
            return 2

    rep = Report()
    tree = Tree(root)
    tree.load()
    for p, err in tree.bad_json:
        rep.fail("schema", short_rel(p, root), err)

    conv = Path(args.conventions) if args.conventions else Path(__file__).with_name("CONVENTIONS.md")
    glossary = load_glossary(conv)
    if glossary is None:
        rep.warn(f"tolerance glossary not found in {conv}: tolerance consistency not checked")
    known_l2 = global_l2_ids(root, tree)
    if known_l2 is None:
        rep.warn("ROOT is not inside OM9_Milestones: cross-milestone L2 ids were not verified")

    # ---------------- examples mode ----------------
    if args.check_examples:
        ex = extract_examples(Path(args.check_examples))
        if len(ex) < 2:
            rep.fail("schema", args.check_examples, f"expected >= 2 l4-example blocks, found {len(ex)}")
        known: Dict[str, Dict[str, Any]] = {}
        ctx = Ctx(rep, root, repo_root, tree, known, known_l2, glossary, lenient_ids=True)
        tags: Set[str] = set()
        for folder_name, text in ex:
            doc, err = load_json_text(text)
            if err:
                rep.fail("schema", folder_name, err)
                continue
            l3id = doc.get("parent_l3_id") if isinstance(doc, dict) else None
            info = tree.l3_by_id.get(l3id) if isinstance(l3id, str) else None
            if info is None:
                rep.fail("identity", folder_name, f"example parent_l3_id {l3id} not found in the tree")
                continue
            folder = info.path / folder_name
            tags.add(str(doc.get("tag")))
            check_doc(doc, folder, ctx, f"example '{folder_name}'")
            if isinstance(doc.get("id"), str):
                known[doc["id"]] = {"deps": doc.get("dependencies", {}).get("depends_on_l4", [])}
        if "be" not in tags or "febe" not in tags:
            rep.fail("schema", "examples", "need one example for a 'be' L3 and one for a 'febe' L3")
        summary = {"mode": "examples", "examples": len(ex), "root": str(root)}
        if args.json:
            print(json.dumps({"result": "PASS" if rep.ok() else "FAIL", "failures": rep.failures, "warnings": rep.warnings, "summary": summary}, indent=2))
        else:
            print_report(rep, args, summary)
        return 0 if rep.ok() else 1

    # ---------------- tree mode ----------------
    # integrity of pre-existing files
    expected = EXPECTED_DIGESTS
    if args.expected_digests:
        data, err = load_json_file(Path(args.expected_digests))
        if err or not isinstance(data, dict):
            print(f"error: --expected-digests: {err}", file=sys.stderr)
            return 2
        expected = data
    check_integrity(rep, root, scope_l2, expected, tree, args.allow_index_updates)
    check_structure(rep, tree, scope_l2, root)

    if scope_l2 is None:
        l1, err = load_json_file(root / "L1.json")
        if isinstance(l1, dict):
            rep.count("tree")
            if l1.get("l3_count") != len(tree.l3):
                rep.fail("tree", "L1.json", f"l3_count {l1.get('l3_count')} != {len(tree.l3)} L3 folders found")
            if l1.get("module_count") != len(tree.l2_dirs):
                rep.fail("tree", "L1.json", f"module_count {l1.get('module_count')} != {len(tree.l2_dirs)} L2 folders found")

    all_l4 = collect_all_l4(tree)
    l3_in_scope = [i for i in tree.l3.values() if scope_l2 is None or i.path.parent == scope_l2]
    docs_by_l3: Dict[Path, Dict[str, Dict[str, Any]]] = {}
    ctx = Ctx(rep, root, repo_root, tree, all_l4, known_l2, glossary)
    n_l4 = 0
    scope_ids: Set[str] = set()
    seen_ids: Dict[str, Path] = {}
    for info in sorted(l3_in_scope, key=lambda i: str(i.path)):
        rep.count("coverage")
        if not info.l4_dirs:
            rep.fail("coverage", short_rel(info.path, root), "L3 folder has no L4 child folder")
            continue
        seqs: List[int] = []
        interactive = 0
        docs: Dict[str, Dict[str, Any]] = {}
        for d in info.l4_dirs:
            n_l4 += 1
            rep.count("json")
            where = short_rel(d / "l4.json", root)
            doc, err = load_json_file(d / "l4.json")
            if err:
                rep.fail("json", where, err)
                continue
            check_doc(doc, d, ctx, where)
            if not isinstance(doc, dict):
                continue
            i = doc.get("id")
            if isinstance(i, str):
                scope_ids.add(i)
                if i in seen_ids and seen_ids[i] != d:
                    rep.fail("identity", where, f"duplicate id (also in {short_rel(seen_ids[i], root)})")
                seen_ids[i] = d
                dd = doc.get("dependencies", {}).get("depends_on_l4", []) if isinstance(doc.get("dependencies"), dict) else []
                docs[i] = {"deps": [x for x in dd if isinstance(x, str)] if isinstance(dd, list) else []}
            if is_int(doc.get("seq")):
                seqs.append(doc["seq"])
            if doc.get("interaction") is not None:
                interactive += 1
            check_l4_md(rep, d, doc, root)
        if seqs and sorted(seqs) != list(range(1, len(seqs) + 1)):
            rep.fail("identity", short_rel(info.path, root), f"L4 sequence numbers must be 1..N without gaps or repeats (found {sorted(seqs)})")
        if len(info.l4_dirs) > 8:
            rep.warn(f"{short_rel(info.path, root)}: {len(info.l4_dirs)} L4 (> 8): confirm the split is intentional")
        if info.tag == "febe" and interactive == 0:
            rep.fail("interaction", short_rel(info.path, root), "febe L3 needs at least one L4 with an interaction block (prompts/options/keys)")
        docs_by_l3[info.path] = docs
        if not args.no_index_check:
            check_index(rep, info, docs, root)

    check_dag_cycles(rep, all_l4, scope_ids)

    summary = {
        "mode": "single-L2 " + scope_l2.name if scope_l2 else "full tree",
        "root": str(root),
        "L2 folders": len(tree.l2_dirs) if scope_l2 is None else 1,
        "L3 folders in scope": len(l3_in_scope),
        "L4 folders validated": n_l4,
        "L4 in whole tree": len(all_l4),
    }
    if args.json:
        print(json.dumps({"result": "PASS" if rep.ok() else "FAIL", "summary": summary,
                          "checks": {c: {"checked": rep.checked[c], "failed": rep.failed[c]} for c in rep.order},
                          "failures": rep.failures, "warnings": rep.warnings}, indent=2))
    else:
        print_report(rep, args, summary)
    return 0 if rep.ok() else 1


if __name__ == "__main__":
    sys.exit(main())
