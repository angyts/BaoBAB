#!/usr/bin/env python3
"""Install read-only diagnostic into local Xous checkout; no build or flash."""
import pathlib, sys, shutil
root=pathlib.Path(sys.argv[1]).resolve()
base=root/"apps-dabao/dabao-console/src"
p=base/"cmds.rs"
s=p.read_text()
src=pathlib.Path(__file__).resolve().parent
replacements=[("use i2cdetect::*;","use i2cdetect::*;\nmod haptic;\nuse haptic::*;"),("let mut i2cdetect_cmd = I2cDetect {};","let mut i2cdetect_cmd = I2cDetect {};\n        let mut haptic_cmd = HapticCmd {};"),("&mut i2cdetect_cmd,","&mut i2cdetect_cmd,\n            &mut haptic_cmd,")]
dest=base/"cmds/haptic.rs"
api=base/"cmds/haptic/haptic_api.rs"
if dest.exists() or api.exists() or "mod haptic;" in s: raise SystemExit("Already installed or conflicting files; inspect manually")
for a,b in replacements:
    if s.count(a)!=1: raise SystemExit("Unexpected source: "+a)
    s=s.replace(a,b)
api.parent.mkdir(exist_ok=True)
shutil.copyfile(src/"haptic_api.rs",api)
shutil.copyfile(src/"haptic.rs",dest)
p.write_text(s)
print("Installed; inspect git diff before building.")
