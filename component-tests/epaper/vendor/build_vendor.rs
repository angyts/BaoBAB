fn main(){
 use std::{env,path::PathBuf,process::Command};
 let out=PathBuf::from(env::var_os("OUT_DIR").unwrap());
 let cc=env::var("BAOBAB_CC").expect("Set BAOBAB_CC to absolute riscv-none-elf-gcc path");
 let ar=cc.strip_suffix("gcc").expect("compiler must end in gcc").to_owned()+"ar";
 let obj=out.join("epd_vendor.o");let lib=out.join("libepd_vendor.a");
 assert!(Command::new(&cc).args(["-march=rv32imac","-mabi=ilp32","-Os","-ffreestanding","-fno-builtin","-fno-stack-protector","-c","vendor_epaper/EPD_2in13_V4.c","-o"]).arg(&obj).status().unwrap().success());
 assert!(Command::new(ar).arg("crs").arg(&lib).arg(&obj).status().unwrap().success());
 println!("cargo:rustc-link-search=native={}",out.display());println!("cargo:rustc-link-lib=static=epd_vendor");
 println!("cargo:rerun-if-env-changed=BAOBAB_CC");println!("cargo:rerun-if-changed=vendor_epaper");
}
