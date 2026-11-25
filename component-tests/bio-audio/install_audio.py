#!/usr/bin/env python3
"""Install the bounded BIO 440 Hz I2S tone test into a local Xous checkout.

Prerequisites:
- The earlier BIO clock test has already been installed and compiled.
- The local Xous checkout contains bio-lib/src/i2s_clock.rs and the
  i2sclock console command.

This script does not build or flash firmware. Review git diff afterwards.
"""
from pathlib import Path
import shutil
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python3 install_audio.py /path/to/xous-core")

root = Path(sys.argv[1]).resolve()
source = Path(__file__).resolve().parent
bio_src = root / "libs/bio-lib/src"
console_src = root / "apps-dabao/dabao-console/src"
lib_rs = bio_src / "lib.rs"
cmd_file = console_src / "cmds/i2s_clock.rs"

required = [lib_rs, cmd_file, bio_src / "i2s_clock.rs"]
for path in required:
    if not path.is_file():
        raise SystemExit(f"Missing prerequisite: {path}")

kernel_dest = bio_src / "i2s_audio_kernel.rs"
api_dest = bio_src / "i2s_audio.rs"
for path in (kernel_dest, api_dest):
    if path.exists():
        raise SystemExit(f"Refusing to overwrite existing file: {path}")

lib_text = lib_rs.read_text()
cmd_text = cmd_file.read_text()

if "pub mod i2s_audio_kernel;" in lib_text or "pub mod i2s_audio;" in lib_text:
    raise SystemExit("Audio modules already appear registered; inspect manually.")

if "i2sclock" not in cmd_text or "tone440" in cmd_text:
    raise SystemExit("Expected old i2sclock command not found, or tone440 already installed.")

if lib_text.count("pub mod i2s_clock;") != 1:
    raise SystemExit("Unexpected bio-lib module layout; expected one pub mod i2s_clock;")

# Validate before writing any files.
new_lib_text = lib_text.replace(
    "pub mod i2s_clock;",
    "pub mod i2s_clock;\npub mod i2s_audio_kernel;\npub mod i2s_audio;",
)
new_cmd_text = (source / "i2s_clock_cmd.rs").read_text()
if "tone440" not in new_cmd_text:
    raise SystemExit("Bundled i2s_clock_cmd.rs does not include tone440.")

shutil.copyfile(source / "i2s_audio_kernel.rs", kernel_dest)
shutil.copyfile(source / "i2s_audio.rs", api_dest)
lib_rs.write_text(new_lib_text)
cmd_file.write_text(new_cmd_text)

print("Installed bounded BIO I2S tone test.")
print("No build or flash was performed.")
print("Review: git diff --check && git status --short && git diff")
