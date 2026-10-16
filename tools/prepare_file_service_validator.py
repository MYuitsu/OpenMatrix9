"""Create the new service validator from the historical modal harness; never overwrite it."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
old = ROOT / "tools/validate_owned_layer_modals.py"
new = ROOT / "tools/validate_owned_file_service.py"
assert not new.exists(), "new validator already exists"
source = old.read_text(encoding="utf-8")
source = source.replace('"""Freeze and verify owned Layers modal scope; complete Layers/RCORE gates stay open."""',
                        '"""Freeze Task1 File lifecycle service; File UI/codecs/full RCORE gates stay open."""')
source = source.replace("2026-10-14-owned-layer-modals.json", "2026-10-14-owned-file-service.json")
source = source.replace("owned-layer-modals-", "owned-file-service-")
source = source.replace("owned-layer-dialogs-ui", "owned-file-service-layer-dialogs-regression")
source = source.replace("owned-layer-modals-layers-regression", "owned-file-service-layers-regression")
source = source.replace("owned-layer-modals-viewport-regression", "owned-file-service-viewport-regression")
source = source.replace('"--all-targets", "--locked", "--"', '"--all-targets", "--locked", "-j", "2", "--"')
source = source.replace('["cargo", "test", "--locked", "-p"', '["cargo", "test", "--locked", "-j", "2", "-p"')
source = source.replace('["cargo", "test", "--workspace", "--locked", "--"', '["cargo", "test", "--workspace", "--locked", "-j", "2", "--"')
source = source.replace('["cargo", "build", "--release", "--locked", "-p"', '["cargo", "build", "--release", "--locked", "-j", "2", "-p"')
source = source.replace('("om9-app", "viewport_title_menu", 17)]',
                        '("om9-app", "viewport_title_menu", 17), ("om9-app", "file_lifecycle", 17),\n'
                        '               ("om9-core", "coordinator_held_startup", 1), ("om9-app", "file_lifecycle_cold", 1)]')
source = source.replace('if "OM9_LAYER_RECEIPT " in line]', 'if "OM9_LAYER_RECEIPT " in line]\n'
                        '            cold += [json.loads(line.split("OM9_FILE_RECEIPT ", 1)[1]) for line in output.splitlines()\n'
                        '                     if "OM9_FILE_RECEIPT " in line]')
source = source.replace('["passed"] == 439', '["passed"] == 458')
source = source.replace('len(cold) == 5', 'len(cold) == 7')
source = source.replace('"status": "validated_owned_layer_modals_slice"', '"status": "validated_owned_file_lifecycle_service_slice"')
source = source.replace('"full_layers_acceptance": False,', '"full_layers_acceptance": False, "full_file_acceptance": False, "file_ui_connected": False,')
source = source.replace('"Current-layer output acceptance still covers foundation Point/Line only."',
                        '"Task1 File lifecycle service is wired to controller APIs only; File menu/CMD/picker UI is not enabled or accepted."')
source = source.replace('"Cold fixtures validate actual modal commits and archive reopen, not interactive native save/open."',
                        '"Cold fixtures validate controller Save/New/Open/SaveAs and archive state; interactive native picker/save/open, complete view/material persistence and .3dm remain pending."')
new.write_text(source, encoding="utf-8")
print(new.relative_to(ROOT).as_posix())
