#!/usr/bin/env python3
"""Install the BIO audio pin readback audit in a local Xous checkout.

Requires the earlier bio-audio clock/audio integration. Does not build or flash.
"""
from pathlib import Path
import shutil
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python3 install_pin_audit.py /path/to/xous-core")

root = Path(sys.argv[1]).resolve()
source = Path(__file__).resolve().parent
bio_src = root / "libs/bio-lib/src"
lib_rs = bio_src / "lib.rs"
cmd_file = root / "apps-dabao/dabao-console/src/cmds/i2s_clock.rs"

for p in (lib_rs, cmd_file, bio_src / "i2s_audio.rs"):
    if not p.is_file():
        raise SystemExit(f"Missing prerequisite: {p}")

new_files = [
    (source / "i2s_pin_audit_kernel.rs", bio_src / "i2s_pin_audit_kernel.rs"),
    (source / "i2s_pin_audit.rs", bio_src / "i2s_pin_audit.rs"),
]
for _, dest in new_files:
    if dest.exists():
        raise SystemExit(f"Refusing to overwrite: {dest}")

lib_text = lib_rs.read_text()
if "pub mod i2s_pin_audit;" in lib_text or "pub mod i2s_pin_audit_kernel;" in lib_text:
    raise SystemExit("Pin audit module is already registered; inspect manually.")
if lib_text.count("pub mod i2s_audio;") != 1:
    raise SystemExit("Unexpected bio-lib module layout; expected pub mod i2s_audio;")

cmd_text = cmd_file.read_text()
if '"pintest" =>' in cmd_text:
    raise SystemExit("pintest command already exists; inspect manually.")
cmd_source = (source / "i2s_clock_cmd.rs").read_text()
if '"pintest" =>' not in cmd_source:
    raise SystemExit("Bundled command file does not implement pintest.")

new_lib_text = lib_text.replace(
    "pub mod i2s_audio;",
    "pub mod i2s_audio;\npub mod i2s_pin_audit_kernel;\npub mod i2s_pin_audit;",
)

for src, dest in new_files:
    shutil.copyfile(src, dest)
lib_rs.write_text(new_lib_text)
shutil.copyfile(source / "i2s_clock_cmd.rs", cmd_file)

print("Installed BIO pin readback audit.")
print("Review git diff --check, git status --short and git diff.")
print("No build or flash performed.")
