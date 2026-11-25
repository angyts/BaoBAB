#!/usr/bin/env python3
"""Install BIO I2S external-line hold diagnostic into a local Xous checkout.

Requires the existing BaoBAB BIO audio, clock and pin-audit integration.
This script only copies sources and registers modules; it does not build or flash.
"""
from pathlib import Path
import shutil
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python3 install_line_hold.py /path/to/xous-core")

root = Path(sys.argv[1]).resolve()
source = Path(__file__).resolve().parent
bio_src = root / "libs/bio-lib/src"
lib_rs = bio_src / "lib.rs"
cmd_file = root / "apps-dabao/dabao-console/src/cmds/i2s_clock.rs"

for path in (lib_rs, cmd_file, bio_src / "i2s_audio.rs", bio_src / "i2s_pin_audit.rs"):
    if not path.is_file():
        raise SystemExit(f"Missing prerequisite: {path}")

new_files = [
    (source / "i2s_line_hold_kernel.rs", bio_src / "i2s_line_hold_kernel.rs"),
    (source / "i2s_line_hold.rs", bio_src / "i2s_line_hold.rs"),
]
for src, dest in new_files:
    if not src.is_file():
        raise SystemExit(f"Missing bundled source: {src}")
    if dest.exists():
        raise SystemExit(f"Refusing to overwrite existing file: {dest}")

lib_text = lib_rs.read_text()
cmd_text = (source / "i2s_clock_cmd.rs").read_text()
if "pub mod i2s_line_hold_kernel;" in lib_text or "pub mod i2s_line_hold;" in lib_text:
    raise SystemExit("Line-hold modules already registered; inspect manually.")
if lib_text.count("pub mod i2s_pin_audit;") != 1:
    raise SystemExit("Unexpected bio-lib module layout; expected one pub mod i2s_pin_audit;")
if "linehold" not in cmd_text:
    raise SystemExit("Bundled command does not include linehold.")

new_lib_text = lib_text.replace(
    "pub mod i2s_pin_audit;",
    "pub mod i2s_pin_audit;\npub mod i2s_line_hold_kernel;\npub mod i2s_line_hold;",
)

# All validations occur before writing to the checkout.
for src, dest in new_files:
    shutil.copyfile(src, dest)
lib_rs.write_text(new_lib_text)
shutil.copyfile(source / "i2s_clock_cmd.rs", cmd_file)

print("Installed BIO external-line hold diagnostic.")
print("Review: git diff --check && git status --short && git diff")
print("Then build with: cargo xtask dabao dabao-console")
print("No build or flash was performed.")
