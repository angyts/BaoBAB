#!/usr/bin/env python3
"""Apply the Baochip HAL IFRAM unmap offset fix to a Xous checkout.

The allocator returns page offsets (page_index * 4096), but the unmap
handler was using that byte offset as an index into a page table.
This fixes the unit mismatch. Review the diff and build before flashing.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python3 fix_xous_ifram_unmap.py /path/to/xous-core")

root = Path(sys.argv[1]).resolve()
path = root / "services/bao1x-hal-service/src/main.rs"
if not path.is_file():
    raise SystemExit(f"Cannot find {path}")

text = path.read_text()

old = """                        let bank: usize;
                        let offset = if utralib::generated::HW_IFRAM0_MEM <= phys_addr
                            && phys_addr
                                < utralib::generated::HW_IFRAM0_MEM + utralib::generated::HW_IFRAM0_MEM_LEN
                        {
                            bank = 0;
                            phys_addr - utralib::generated::HW_IFRAM0_MEM
                        } else if utralib::generated::HW_IFRAM1_MEM <= phys_addr
                            && phys_addr
                                < utralib::generated::HW_IFRAM1_MEM + utralib::generated::HW_IFRAM1_MEM_LEN
                        {
                            bank = 1;
                            phys_addr - utralib::generated::HW_IFRAM1_MEM
                        } else {
                            log::error!("Mapped IFRAM address 0x{:x} is invalid", phys_addr);
                            panic!("Mapped IFRAM address is invalid");
                        };
"""
new = """                        let bank: usize;
                        let offset_bytes = if utralib::generated::HW_IFRAM0_MEM <= phys_addr
                            && phys_addr
                                < utralib::generated::HW_IFRAM0_MEM + utralib::generated::HW_IFRAM0_MEM_LEN
                        {
                            bank = 0;
                            phys_addr - utralib::generated::HW_IFRAM0_MEM
                        } else if utralib::generated::HW_IFRAM1_MEM <= phys_addr
                            && phys_addr
                                < utralib::generated::HW_IFRAM1_MEM + utralib::generated::HW_IFRAM1_MEM_LEN
                        {
                            bank = 1;
                            phys_addr - utralib::generated::HW_IFRAM1_MEM
                        } else {
                            log::error!("Mapped IFRAM address 0x{:x} is invalid", phys_addr);
                            panic!("Mapped IFRAM address is invalid");
                        };
                        // IFRAM allocator entries represent 4-KiB pages, not bytes.
                        debug_assert_eq!(offset_bytes % 4096, 0);
                        let offset = offset_bytes / 4096;
"""
if new in text:
    print("IFRAM unmap offset fix is already applied; no changes made.")
    raise SystemExit(0)
if text.count(old) != 1:
    raise SystemExit("Unexpected HAL source layout; no changes made. Inspect main.rs manually.")

path.write_text(text.replace(old, new))
print("Applied IFRAM unmap byte-offset to page-index fix.")
print("Review with: git diff --check && git diff -- services/bao1x-hal-service/src/main.rs")
print("Then rebuild with: cargo xtask dabao dabao-console")
print("Do not flash until the build succeeds.")
