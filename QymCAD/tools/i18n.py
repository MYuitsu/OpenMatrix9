#!/usr/bin/env python3
"""TRANSLATION CATALOGUE TOOL: check coverage, find untranslated keys, and generate stubs.

English (`i18n/en`) is the reference language. Other languages live in `i18n/<lang_code>`
and must provide translations for all keys defined in the reference.

Usage:
    tools/i18n.py                  Summary of translation coverage across all languages
    tools/i18n.py <lang>           List missing keys for a specific language with English reference
    tools/i18n.py --stub <lang>    Append missing translation stubs to i18n/<lang>/*.ftl
    tools/i18n.py --export <lang>  Print missing translation stubs to stdout
    tools/i18n.py --check [lang]   Exit with code 1 if any keys are missing or extra (for CI/hooks)
"""
import argparse
import os
import re
import signal
import sys
from pathlib import Path

# Clean exit on broken pipes (e.g. `tools/i18n.py | head`)
if hasattr(signal, "SIGPIPE"):
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)

ROOT = Path(__file__).resolve().parent.parent
I18N_DIR = ROOT / "i18n"
REFERENCE_LANG = "en"

# BCP-47 tag format (e.g. de, uk, zh-CN, pt-BR)
LANG_CODE_RE = re.compile(r"^[a-z]{2,3}(-[A-Z]{2})?$")


def validate_lang_code(lang: str) -> None:
    """Validate that language code matches standard BCP-47 tag format to prevent path traversal."""
    if not LANG_CODE_RE.match(lang):
        print(red(f"Error: invalid language code '{lang}'. Expected format like 'de', 'uk', or 'zh-CN'."), file=sys.stderr)
        sys.exit(1)

# Support colored output if running in a terminal
USE_COLOR = sys.stdout.isatty() and not os.environ.get("NO_COLOR")


def color(text: str, code: str) -> str:
    if not USE_COLOR:
        return text
    return f"\033[{code}m{text}\033[0m"


def green(text: str) -> str:
    return color(text, "32")


def red(text: str) -> str:
    return color(text, "31")


def yellow(text: str) -> str:
    return color(text, "33")


def cyan(text: str) -> str:
    return color(text, "36")


def bold(text: str) -> str:
    return color(text, "1")


def dim(text: str) -> str:
    return color(text, "2")


PLACEHOLDER_RE = re.compile(r"\{\s*\$([a-zA-Z0-9_-]+)\s*\}")
# A message of Fluent: its id, `=`, and its value on the same line or starting on the next ("key =" alone). Matched
# by " = " instead, a message whose value starts on the next line was not seen, and its lines went to the message
# before it: `par-name-taken` was missing from English and "extra" in the language that wrote it on one line.
MESSAGE_RE = re.compile(r"^([A-Za-z][A-Za-z0-9_-]*)\s*=(.*)$")


def parse_ftl(path: Path) -> dict:
    """Parse a Fluent (.ftl) file into a dict of key -> {value, line, comments, placeholders}."""
    entries = {}
    if not path.is_file():
        return entries

    comments = []
    cur_key = None
    cur_val = []
    cur_line = 0

    with open(path, "r", encoding="utf-8") as f:
        for line_no, raw_line in enumerate(f, 1):
            line = raw_line.rstrip("\r\n")
            trimmed = line.strip()

            if not trimmed:
                comments = []
                continue

            if trimmed.startswith("#"):
                comments.append(trimmed)
                continue

            if line.startswith((" ", "\t")):
                if cur_key is not None:
                    cur_val.append(trimmed)
            elif MESSAGE_RE.match(line):
                if cur_key is not None:
                    val_str = " ".join(cur_val)
                    entries[cur_key] = {
                        "value": val_str,
                        "line": cur_line,
                        "comments": comments,
                        "placeholders": sorted(set(PLACEHOLDER_RE.findall(val_str))),
                    }
                    comments = []

                m = MESSAGE_RE.match(line)
                cur_key = m.group(1)
                cur_val = [m.group(2).strip()] if m.group(2).strip() else []
                cur_line = line_no

    if cur_key is not None:
        val_str = " ".join(cur_val)
        entries[cur_key] = {
            "value": val_str,
            "line": cur_line,
            "comments": comments,
            "placeholders": sorted(set(PLACEHOLDER_RE.findall(val_str))),
        }

    return entries


def load_language(lang_code: str) -> dict:
    """Load all .ftl files for a language: returns {rel_filename: {key: entry}}."""
    lang_dir = I18N_DIR / lang_code
    if not lang_dir.is_dir():
        return {}

    files = {}
    for p in sorted(lang_dir.glob("*.ftl")):
        files[p.name] = parse_ftl(p)
    return files


def all_languages() -> list:
    """List all available language codes in i18n/ matching BCP-47 pattern."""
    if not I18N_DIR.is_dir():
        return []
    langs = []
    for d in sorted(I18N_DIR.iterdir()):
        if d.is_dir() and not d.name.startswith(".") and LANG_CODE_RE.match(d.name):
            langs.append(d.name)
    return langs


def get_native_name(files: dict, fallback: str) -> str:
    """Extract native language name from key `language-name` if present."""
    for entries in files.values():
        if "language-name" in entries:
            return entries["language-name"]["value"]
    return fallback


def print_summary(ref_files: dict, langs: list):
    """Print overall summary table of coverage across all languages."""
    ref_total = sum(len(entries) for entries in ref_files.values())
    file_names = ", ".join(ref_files.keys())

    print(f"\n{bold('QymCAD Translation Catalogue')}")
    print(f"Reference language: {bold(REFERENCE_LANG)} ({ref_total} keys across {file_names})\n")

    header = f"{'Language':<24} {'Coverage':<22} {'Missing':<10} {'Extra':<8} {'Status'}"
    print(bold(header))
    print("-" * len(header))

    for lang in langs:
        lang_files = load_language(lang)
        native = get_native_name(lang_files, lang)
        display_name = f"{lang} ({native})"

        all_lang_keys = set()
        for entries in lang_files.values():
            all_lang_keys.update(entries.keys())

        all_ref_keys = set()
        for entries in ref_files.values():
            all_ref_keys.update(entries.keys())

        have = len(all_lang_keys & all_ref_keys)
        missing = len(all_ref_keys - all_lang_keys)
        extra = len(all_lang_keys - all_ref_keys)
        pct = (have / ref_total * 100.0) if ref_total else 100.0

        lang_col = f"{display_name:<24}"
        cov_col = f"{have}/{ref_total} ({pct:5.1f}%)"
        cov_col_padded = f"{cov_col:<22}"
        miss_col = f"{missing:<10}"
        extra_col = f"{extra:<8}"

        if lang == REFERENCE_LANG:
            status = dim("[reference]")
            miss_col = dim(miss_col)
            extra_col = dim(extra_col)
        elif missing == 0 and extra == 0:
            status = green("complete")
            miss_col = green(miss_col)
            extra_col = dim(extra_col)
        elif missing == 0:
            status = yellow(f"{extra} extra")
            extra_col = yellow(extra_col)
        else:
            status = red(f"{missing} missing")
            miss_col = red(miss_col)
            if extra:
                extra_col = yellow(extra_col)

        print(f"{lang_col} {cov_col_padded} {miss_col} {extra_col} {status}")

    print("\nRun " + cyan("tools/i18n.py <lang>") + " to inspect missing keys for a specific language.")
    print("Run " + cyan("tools/i18n.py --stub <lang>") + " to generate translation stubs.\n")


def inspect_language(ref_files: dict, lang: str, as_stubs: bool = False, append_to_files: bool = False) -> int:
    """Inspect or generate stubs for a specific language. Returns number of missing keys."""
    lang_files = load_language(lang)
    lang_dir = I18N_DIR / lang

    all_lang_keys = set()
    for entries in lang_files.values():
        all_lang_keys.update(entries.keys())

    all_ref_keys = set()
    for entries in ref_files.values():
        all_ref_keys.update(entries.keys())

    missing_keys = all_ref_keys - all_lang_keys
    extra_keys = all_lang_keys - all_ref_keys

    ref_total = len(all_ref_keys)
    have = len(all_lang_keys & all_ref_keys)
    pct = (have / ref_total * 100.0) if ref_total else 100.0
    native = get_native_name(lang_files, lang)

    if not as_stubs and not append_to_files:
        print(f"\n{bold(f'Language: {lang} ({native})')}")
        print(f"Coverage: {have}/{ref_total} ({pct:.1f}%), Missing: {len(missing_keys)}, Extra: {len(extra_keys)}\n")

    if append_to_files and not lang_dir.is_dir():
        lang_dir.mkdir(parents=True, exist_ok=True)
        print(f"Created new language directory: {lang_dir}")

    total_added = 0

    for fname, ref_entries in ref_files.items():
        curr_entries = lang_files.get(fname, {})
        missing_in_file = [k for k in ref_entries.keys() if k not in curr_entries]

        if not missing_in_file:
            continue

        if append_to_files:
            target_path = lang_dir / fname
            is_new = not target_path.exists()
            with open(target_path, "a", encoding="utf-8") as out:
                if is_new or target_path.stat().st_size == 0:
                    if fname == "main.ftl":
                        out.write(f"# Language name in itself\nlanguage-name = {lang}\n\n")
                out.write(f"\n# ── Missing translations ({len(missing_in_file)} keys) ──\n")
                for k in missing_in_file:
                    ref_info = ref_entries[k]
                    ref_val = ref_info["value"]
                    out.write(f"\n# en: {ref_val}\n")
                    if ref_info["placeholders"]:
                        vars_str = ", ".join(f"{{ ${p} }}" for p in ref_info["placeholders"])
                        out.write(f"# variables: {vars_str}\n")
                    out.write(f"{k} = \n")
            print(f"Added {bold(str(len(missing_in_file)))} stubs to {target_path}")
            total_added += len(missing_in_file)

        elif as_stubs:
            print(f"\n# ── {fname} ({len(missing_in_file)} keys) ──")
            for k in missing_in_file:
                ref_info = ref_entries[k]
                ref_val = ref_info["value"]
                print(f"\n# en: {ref_val}")
                if ref_info["placeholders"]:
                    vars_str = ", ".join(f"{{ ${p} }}" for p in ref_info["placeholders"])
                    print(f"# variables: {vars_str}")
                print(f"{k} = ")

        else:
            print(bold(f"[{fname}]") + f" — {len(missing_in_file)} missing:")
            for k in missing_in_file:
                ref_info = ref_entries[k]
                ref_val = ref_info["value"]
                line_info = f" (line {ref_info['line']})"
                print(f"  • {cyan(k)}{dim(line_info)}")
                print(f"    en: \"{ref_val}\"")
                if ref_info["placeholders"]:
                    vars_str = ", ".join(f"${p}" for p in ref_info["placeholders"])
                    print(f"    vars: {yellow(vars_str)}")
            print()

    if extra_keys and not as_stubs and not append_to_files:
        print(bold(yellow(f"Orphaned / extra keys in {lang} ({len(extra_keys)}):")))
        print(dim("These keys no longer exist in reference English and should be removed:"))
        for k in sorted(extra_keys):
            print(f"  • {yellow(k)}")
        print()

    if append_to_files:
        print(f"\n{green('Done!')} Generated {total_added} translation stubs in {lang_dir}/")
        print(f"Edit the files and fill in the translated strings right of the '=' sign.\n")
    elif not missing_keys and not extra_keys and not as_stubs:
        print(green(f"All {ref_total} keys are fully translated in {lang}! No missing or extra keys.\n"))

    return len(missing_keys) + len(extra_keys)


def main():
    try:
        parser = argparse.ArgumentParser(
            description="QymCAD Translation Catalogue Tool",
            formatter_class=argparse.RawDescriptionHelpFormatter,
            epilog=__doc__,
        )
        parser.add_argument("lang", nargs="?", default="", help="Language code to inspect (e.g. uk, ru, de)")
        parser.add_argument("--stub", metavar="LANG", help="Append missing translation stubs to i18n/<lang>/*.ftl")
        parser.add_argument("--export", metavar="LANG", help="Print missing translation stubs for <lang> to stdout")
        parser.add_argument("--check", nargs="?", const="ALL", default=None, help="Check that language(s) are 100%% complete")

        args = parser.parse_args()

        ref_files = load_language(REFERENCE_LANG)
        if not ref_files:
            print(red(f"Error: reference language '{REFERENCE_LANG}' not found in {I18N_DIR}"), file=sys.stderr)
            sys.exit(1)

        # 1. Stub generation mode
        if args.stub:
            validate_lang_code(args.stub)
            inspect_language(ref_files, args.stub, append_to_files=True)
            sys.exit(0)

        # 2. Export stubs to stdout
        if args.export:
            validate_lang_code(args.export)
            inspect_language(ref_files, args.export, as_stubs=True)
            sys.exit(0)

        # 3. Check mode (CI / exit code)
        if args.check is not None:
            target_lang = args.check
            if target_lang == "ALL":
                langs = [l for l in all_languages() if l != REFERENCE_LANG]
                total_issues = 0
                for l in langs:
                    issues = inspect_language(ref_files, l)
                    total_issues += issues
                sys.exit(1 if total_issues > 0 else 0)
            else:
                validate_lang_code(target_lang)
                issues = inspect_language(ref_files, target_lang)
                sys.exit(1 if issues > 0 else 0)

        # 4. Specific language inspection
        if args.lang:
            validate_lang_code(args.lang)
            inspect_language(ref_files, args.lang)
            sys.exit(0)

        # 5. Default: summary table
        langs = all_languages()
        print_summary(ref_files, langs)
    except (BrokenPipeError, KeyboardInterrupt):
        sys.exit(0)


if __name__ == "__main__":
    main()

