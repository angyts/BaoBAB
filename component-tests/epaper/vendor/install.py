from pathlib import Path
import sys, shutil
src=Path(__file__).resolve().parent
app=Path(sys.argv[1]).resolve()/"apps-dabao/dabao-console"
reg=app/"src/cmds.rs"
s=reg.read_text()
anchors=[("mod i2cdetect;\nuse i2cdetect::*;","\nmod epapervendor;\nuse epapervendor::*;"),("        let mut i2cdetect_cmd = I2cDetect {};","\n        let mut epapervendor_cmd = EpaperVendorCmd;"),("            &mut i2cdetect_cmd,","\n            &mut epapervendor_cmd,")]
if "mod epapervendor;" not in s:
    for a,b in anchors:
        if s.count(a)!=1: raise SystemExit("Unexpected registry: "+a)
        s=s.replace(a,a+b)
build=app/"build.rs"
if build.exists() and build.read_bytes()!=(src/"build_vendor.rs").read_bytes(): raise SystemExit("Existing build.rs; merge manually")
dest=app/"vendor_epaper"
dest.mkdir(exist_ok=True)
for name in ["EPD_2in13_V4.c","EPD_2in13_V4.h","DEV_Config.h","Debug.h"]:
    shutil.copyfile(src/name,dest/name)
shutil.copyfile(src/"build_vendor.rs",build)
shutil.copyfile(src/"epapervendor.rs",app/"src/cmds/epapervendor.rs")
reg.write_text(s)
print("Installed vendor command; build required, no flash performed")
