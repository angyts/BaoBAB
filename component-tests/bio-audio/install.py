#!/usr/bin/env python3
"""Install the BIO I2S clock-only test into a local xous-core checkout.

This does not build or flash firmware. It refuses to overwrite existing files
or patch a source tree whose expected integration points differ.
"""
from pathlib import Path
import shutil
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python3 install.py /path/to/xous-core")

root = Path(sys.argv[1]).resolve()
src = Path(__file__).resolve().parent

bio_lib = root / "libs/bio-lib/src"
console_src = root / "apps-dabao/dabao-console/src"
cmds = console_src / "cmds.rs"

required = [
    root / "libs/bio-lib/Cargo.toml",
    bio_lib / "lib.rs",
    console_src / "cmds.rs",
]
for path in required:
    if not path.is_file():
        raise SystemExit(f"Missing expected Xous file: {path}")

dest_kernel = bio_lib / "i2s_clock_kernel.rs"
dest_api = bio_lib / "i2s_clock.rs"
dest_cmd = console_src / "cmds/i2s_clock.rs"

for path in (dest_kernel, dest_api, dest_cmd):
    if path.exists():
        raise SystemExit(f"Refusing to overwrite existing file: {path}")

lib_rs = bio_lib / "lib.rs"
lib_text = lib_rs.read_text()
cmds_text = cmds.read_text()

if "pub mod i2s_clock;" in lib_text or "mod i2s_clock;" in cmds_text:
    raise SystemExit("I2S clock module appears to be installed already; inspect manually.")

patches = [
    ("use i2cdetect::*;", "use i2cdetect::*;\nmod i2s_clock;\nuse i2s_clock::*;"),
    ("touch_cmd: Touch,", "touch_cmd: Touch,\n    i2s_clock_cmd: I2sClockCmd,"),
    ("touch_cmd: Touch::new(),", "touch_cmd: Touch::new(),\n            i2s_clock_cmd: I2sClockCmd::new(),"),
    ("let mut i2cdetect_cmd = I2cDetect {};", "let mut i2cdetect_cmd = I2cDetect {};"),
    ("&mut i2cdetect_cmd,", "&mut i2cdetect_cmd,\n            &mut self.i2s_clock_cmd,"),
]
for old, new in patches:
    if cmds_text.count(old) != 1:
        raise SystemExit(f"Expected exactly one integration anchor, found {cmds_text.count(old)}: {old!r}")

if lib_text.count("pub mod ws2812;") != 1:
    raise SystemExit("Unexpected bio-lib/src/lib.rs layout; expected one 'pub mod ws2812;' line.")

# Validate all anchors before making any changes.
lib_text = lib_text.replace("pub mod ws2812;", "pub mod ws2812;\npub mod i2s_clock_kernel;\npub mod i2s_clock;")
cmds_text = cmds_text.replace(patches[0][0], patches[0][1])
cmds_text = cmds_text.replace(patches[1][0], patches[1][1])
cmds_text = cmds_text.replace(patches[2][0], patches[2][1])
cmds_text = cmds_text.replace(patches[4][0], patches[4][1])

cmd_dir = console_src / "cmds"
cmd_dir.mkdir(exist_ok=True)
shutil.copyfile(src / "i2s_clock_kernel.rs", dest_kernel)
shutil.copyfile(src / "i2s_clock.rs", dest_api)
shutil.copyfile(src / "i2s_clock_cmd.rs", dest_cmd)
lib_rs.write_text(lib_text)
cmds.write_text(cmds_text)

print("Installed BIO I2S clock-only test.")
print("Review git diff before building. This is not an audio-tone generator.")
