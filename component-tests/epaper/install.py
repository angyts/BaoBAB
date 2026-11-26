#!/usr/bin/env python3
"""Install the BaoBAB Waveshare V4 test command into a Xous checkout.

Copies one Rust source file and registers a stateless REPL command.
Does not build, flash, or alter hardware.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python3 install.py /path/to/xous-core")

root = Path(sys.argv[1]).resolve()
source = Path(__file__).resolve().parent
cmd_dir = root / "apps-dabao/dabao-console/src/cmds"
cmd_file = cmd_dir / "epaper.rs"
registry = cmd_dir.parent / "cmds.rs"

if not registry.is_file():
    raise SystemExit(f"Missing console command registry: {registry}")
if cmd_file.exists():
    raise SystemExit(f"Refusing to overwrite existing file: {cmd_file}")

registry_text = registry.read_text()
module_anchor = "mod bio;\nuse bio::*;"
dispatch_anchor = "        let mut i2cdetect_cmd = I2cDetect {};"
array_anchor = "            &mut i2cdetect_cmd,"

if "mod epaper;" in registry_text or "&mut epaper_cmd" in registry_text:
    raise SystemExit("An epaper command is already registered; inspect manually.")
if registry_text.count(module_anchor) != 1:
    raise SystemExit("Unexpected cmds.rs module layout; expected one i2cdetect module declaration.")
if registry_text.count(dispatch_anchor) != 1:
    raise SystemExit("Unexpected cmds.rs dispatch layout; expected i2cdetect command declaration.")
if registry_text.count(array_anchor) != 1:
    raise SystemExit("Unexpected cmds.rs command array layout; expected i2cdetect entry.")

source_file = source / "epaper.rs"
if not source_file.is_file():
    raise SystemExit(f"Missing bundled command source: {source_file}")

new_registry = registry_text.replace(
    module_anchor,
    module_anchor + "\nmod epaper;\nuse epaper::*;",
)
new_registry = new_registry.replace(
    dispatch_anchor,
    dispatch_anchor + "\n        let mut epaper_cmd = EpaperCmd::new();",
)
new_registry = new_registry.replace(
    array_anchor,
    array_anchor + "\n            &mut epaper_cmd,",
)

# All validation above is complete before we mutate the checkout.
cmd_file.write_text(source_file.read_text())
registry.write_text(new_registry)

print("Installed the initial Waveshare V4 e-Paper console command.")
print("Review with: git diff --check && git status --short && git diff")
print("Build with: cargo xtask dabao dabao-console")
print("No build or flash was performed.")
