"""Extract the Matrix VB6 command registry as Menu -> Command -> Option layers.

The decompiled modules contain repeated blocks ending in `call 01A5B610h`.
Each block carries string arguments: a caption, an optional secondary label,
one or more Rhino scripts, a command id and a group (submenu) name. Group
headers additionally carry an "M <name>" marker and the parent menu/group.

This is derived structural analysis, not a copy of the reference sources.
"""

from __future__ import annotations

import argparse
import csv
import json
import re
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class Entry:
    kind: str  # "header" or "item"
    label: str
    alt_label: str
    group: str
    parent: str
    command_id: str
    scripts: list[str]
    source: str
    line: int


@dataclass
class Row:
    menu: str
    command: str
    option: str
    alt_label: str
    command_id: str
    script: str
    alt_script: str
    source: str
    line: int


STRING_RE = re.compile(r';\s*"([^"]*)"')
CALL = "call 01A5B610h"

MENU_ORDER = [
    "File",
    "Edit",
    "View",
    "Curve",
    "Surface",
    "Solid",
    "Mesh",
    "Dimension",
    "Transform",
    "Tools",
    "Analyze",
    "Render",
    "Rhino 5.0",
    "Blend",
    "Clayoo",
    "T-Splines",
    "Help",
    "Builders",
    "Others",
    "Top Icons",
]


def parse_file(path: Path, encoding: str) -> list[Entry]:
    text = path.read_text(encoding=encoding, errors="replace")
    blocks = text.split(CALL)
    entries: list[Entry] = []
    offset = 0
    for block in blocks[:-1]:
        all_strings = STRING_RE.findall(block)
        # Module-level preamble strings can precede the first registration block;
        # the registration arguments are the trailing strings before the call.
        strings = all_strings[-8:]
        line = text.count("\n", 0, offset + len(block)) + 1
        offset += len(block) + len(CALL)
        if not strings:
            continue
        m_index = next((i for i, s in enumerate(strings) if s.startswith("M ")), None)
        if m_index is not None and len(strings) >= m_index + 3:
            label = strings[m_index - 1] if m_index > 0 else strings[m_index][2:]
            entries.append(
                Entry(
                    kind="header",
                    label=label,
                    alt_label="",
                    group=strings[-2],
                    parent=strings[-1],
                    command_id="",
                    scripts=[],
                    source=path.name,
                    line=line,
                )
            )
        elif len(strings) >= 3:
            middle = strings[1:-2]
            command_id = strings[-2]
            if strings[0].startswith("!"):
                # Some commands are registered script-first; the following
                # non-script string is the display label.
                labels = [s for s in middle if not s.startswith("!")]
                label = labels[0] if labels else command_id
                alt_label = labels[1] if len(labels) > 1 else ""
                scripts = [strings[0]] + [s for s in middle if s.startswith("!")]
            else:
                label = strings[0]
                alt_label = next((s for s in middle if not s.startswith("!")), "")
                scripts = [s for s in middle if s.startswith("!")]
            entries.append(
                Entry(
                    kind="item",
                    label=label,
                    alt_label=alt_label,
                    group=strings[-1],
                    parent="",
                    command_id=command_id,
                    scripts=scripts,
                    source=path.name,
                    line=line,
                )
            )
    return entries


def load() -> tuple[dict[str, Entry], list[Entry]]:
    base = Path("ref/matrix/vb6-lite/Matrix90")
    entries: list[Entry] = []
    for name, encoding in [
        ("modMillC_Defaults.bas", "utf-8"),
        ("modTopMenu.bas", "cp1252"),
    ]:
        entries.extend(parse_file(base / name, encoding))

    headers: dict[str, Entry] = {}
    items: list[Entry] = []
    for entry in entries:
        if entry.kind == "header":
            current = headers.get(entry.group)
            if current is None or (not current.parent and entry.parent):
                headers[entry.group] = entry
        else:
            items.append(entry)

    merged: dict[tuple[str, str, str], Entry] = {}
    for item in items:
        key = (item.group, item.label, item.command_id)
        current = merged.get(key)
        if current is None:
            merged[key] = item
            continue
        for script in item.scripts:
            if script not in current.scripts:
                current.scripts.append(script)
        if not current.alt_label:
            current.alt_label = item.alt_label
        if item.source not in current.source:
            current.source = f"{current.source};{item.source}"
    return headers, list(merged.values())


def chain_for(group: str, headers: dict[str, Entry]) -> list[Entry]:
    chain: list[Entry] = []
    seen: set[str] = set()
    current = group
    while current and current not in seen:
        seen.add(current)
        header = headers.get(current)
        if header is None:
            break
        chain.append(header)
        parent = header.parent
        if not parent or parent in {"Top", "None"}:
            break
        current = parent
    return chain


def clean_alt(label: str, command_id: str) -> str:
    if not label or label == command_id:
        return ""
    if label.startswith(("B ", "S ", "M ")):
        return ""
    return label


def rows(headers: dict[str, Entry], items: list[Entry]) -> list[Row]:
    result: list[Row] = []
    for item in items:
        chain = chain_for(item.group, headers)
        menu = "(unresolved)"
        command = item.group
        if chain:
            reversed_chain = list(reversed(chain))
            menu = reversed_chain[0].label
            command = " > ".join(h.label for h in reversed_chain[1:]) or "(root)"
        scripts = item.scripts
        default_script = scripts[-1] if scripts else ""
        alt_script = " ; ".join(scripts[:-1])
        result.append(
            Row(
                menu=menu,
                command=command,
                option=item.label,
                alt_label=clean_alt(item.alt_label, item.command_id),
                command_id=item.command_id,
                script=default_script,
                alt_script=alt_script,
                source=item.source,
                line=item.line,
            )
        )
    return result


def menu_key(menu: str) -> tuple[int, str]:
    try:
        return (MENU_ORDER.index(menu), menu)
    except ValueError:
        return (len(MENU_ORDER), menu)


def write_csv(path: Path, rows_in: list[Row]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.writer(fh)
        writer.writerow(
            ["menu", "command", "option", "alt_label", "command_id", "script", "alt_script", "source", "line"]
        )
        for row in rows_in:
            writer.writerow(
                [
                    row.menu,
                    row.command,
                    row.option,
                    row.alt_label,
                    row.command_id,
                    row.script,
                    row.alt_script,
                    row.source,
                    row.line,
                ]
            )
    print(f"wrote {path} ({len(rows_in)} rows)")


def write_markdown(path: Path, rows_in: list[Row]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    by_menu: dict[str, list[Row]] = {}
    for row in rows_in:
        by_menu.setdefault(row.menu, []).append(row)
    lines = [
        "# Matrix command layers",
        "",
        "Derived from the Matrix VB6 command registry in `ref/matrix/vb6-lite/Matrix90`",
        "(`modMillC_Defaults.bas` and `modTopMenu.bas`). Each entry is represented as",
        "**Menu -> Command -> Option**; `Command` is the submenu path, and `Option` is",
        "the item that invokes a Rhino script. `Script` is the default invocation;",
        "`Alt script` is the alternate/right-click invocation carried by the same item.",
        "`Alt label` is the secondary label/tooltip carried by that item.",
        "",
        f"- Options: **{len(rows_in)}**",
        f"- Menus: **{len(by_menu)}**",
        "- Generated by `tools/extract_matrix_command_layers.py`.",
        "",
        "## Scope",
        "",
        "- Static registry entries registered through `01A5B610h` in",
        "  `modMillC_Defaults.bas` and `modTopMenu.bas`.",
        "- Entries duplicated across the two modules are merged by",
        "  menu/command/option/command-id.",
        "- `Builders`, `Others` and `Top Icons` are root groups, shown with a",
        "  `(root)` command.",
        "- Dynamic/context menus and commands not registered through this function",
        "  are outside this table.",
        "",
        "## Summary",
        "",
        "| Menu | Commands | Options |",
        "|---|---:|---:|",
    ]
    for menu in sorted(by_menu, key=menu_key):
        menu_rows = by_menu[menu]
        commands = {row.command for row in menu_rows}
        lines.append(f"| {menu} | {len(commands)} | {len(menu_rows)} |")
    lines.append("")
    for menu in sorted(by_menu, key=menu_key):
        menu_rows = by_menu[menu]
        lines.append(f"## {menu}")
        lines.append("")
        commands: dict[str, list[Row]] = {}
        for row in menu_rows:
            commands.setdefault(row.command, []).append(row)
        for command, command_rows in commands.items():
            lines.append(f"### {command}")
            lines.append("")
            lines.append("| Option | Alt label | Command ID | Script | Alt script |")
            lines.append("|---|---|---|---|---|")
            for row in command_rows:
                script = row.script.replace("|", "\\|")
                alt_script = row.alt_script.replace("|", "\\|")
                alt = row.alt_label.replace("|", "\\|")
                option = row.option.replace("|", "\\|")
                lines.append(
                    f"| {option} | {alt} | `{row.command_id}` | `{script}` | `{alt_script}` |"
                )
            lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {path} ({len(rows_in)} rows, {len(by_menu)} menus)")


def write_json(path: Path, rows_in: list[Row]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    menus: dict[str, dict[str, object]] = {}
    for row in rows_in:
        menu = menus.setdefault(row.menu, {"name": row.menu, "commands": {}})
        command_map = menu["commands"]
        assert isinstance(command_map, dict)
        command = command_map.get(row.command)
        if command is None:
            is_root = row.command == "(root)"
            segments = [] if is_root else row.command.split(" > ")
            command = {
                "name": segments[-1] if segments else "(root)",
                "path": [row.menu, *segments],
                "root": is_root,
                "options": [],
            }
            command_map[row.command] = command
        options = command["options"]
        assert isinstance(options, list)
        options.append(
            {
                "label": row.option,
                "alt_label": row.alt_label,
                "command_id": row.command_id,
                "script": row.script,
                "alt_script": row.alt_script,
                "source": row.source,
                "line": row.line,
            }
        )

    menu_list = []
    for menu_name in sorted(menus, key=menu_key):
        menu = menus[menu_name]
        commands = list(menu["commands"].values())
        menu_list.append(
            {
                "name": menu_name,
                "command_count": len(commands),
                "option_count": sum(len(c["options"]) for c in commands),
                "commands": commands,
            }
        )
    payload = {
        "schema": "om9.matrix-command-layers.v1",
        "source": [
            "ref/matrix/vb6-lite/Matrix90/modMillC_Defaults.bas",
            "ref/matrix/vb6-lite/Matrix90/modTopMenu.bas",
        ],
        "counts": {
            "menus": len(menu_list),
            "commands": sum(m["command_count"] for m in menu_list),
            "options": len(rows_in),
        },
        "notes": [
            "Grouped as menus -> commands -> options.",
            "alt_label and alt_script are secondary label/invocation of the same option.",
            "root=true means the menu has no submenu; path is [menu].",
        ],
        "menus": menu_list,
    }
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {path} ({len(menu_list)} menus, {len(rows_in)} options)")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--csv", type=Path)
    parser.add_argument("--md", type=Path)
    parser.add_argument("--json", type=Path)
    parser.add_argument("--summary", action="store_true")
    args = parser.parse_args()

    headers, items = load()
    catalog = rows(headers, items)

    if args.summary:
        menus: dict[str, int] = {}
        commands: dict[tuple[str, str], int] = {}
        for row in catalog:
            menus[row.menu] = menus.get(row.menu, 0) + 1
            commands[(row.menu, row.command)] = commands.get((row.menu, row.command), 0) + 1
        print(f"headers={len(headers)} unique_items={len(catalog)} menus={len(menus)} commands={len(commands)}")
        for menu in sorted(menus, key=menu_key):
            print(f"{menu:20s} items={menus[menu]:4d} commands={sum(1 for m, _ in commands if m == menu):3d}")

    if args.csv:
        write_csv(args.csv, catalog)
    if args.md:
        write_markdown(args.md, catalog)
    if args.json:
        write_json(args.json, catalog)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
