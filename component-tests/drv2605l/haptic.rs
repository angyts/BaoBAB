use crate::{CommonEnv,ShellCmdApi};
use core::fmt::Write;
use String;
mod haptic_api;
use haptic_api::Haptic;
pub struct HapticCmd {}
impl<'a> ShellCmdApi<'a> for HapticCmd {
    cmd_api!(haptic);
    fn process(&mut self,args:String,_env:&mut CommonEnv)->Result<Option<String>,xous::Error>{
        let mut out=String::new();
        let mut dev=Haptic::new();
        let words: Vec<&str> = args.split_whitespace().collect();
        match args.trim() {
            "info"=>match dev.probe(){
                Ok(s)=>{writeln!(out,"STATUS=0x{:02X} ID={} diagnostic_fail={} overtemp={} overcurrent={}",s,s>>5,(s>>3)&1,(s>>1)&1,s&1).unwrap();}
                Err(e)=>{writeln!(out,"Probe failed: {:?}",e).unwrap();}
            },
            "registers"=>for r in 0..=0x1e {
                match dev.read(r){
                    Ok(v)=>{writeln!(out,"{:02X}: {:02X}",r,v).unwrap();}
                    Err(e)=>{writeln!(out,"{:02X}: {:?}",r,e).unwrap();break;}
                }
            },
            "selftest"=> {
                let mut passed=0;
                for _ in 0..10 { match dev.probe(){Ok(_)=>passed+=1,Err(e)=>{writeln!(out,"{:?}",e).unwrap();}} }
                writeln!(out,"Identity checks: {}/10",passed).unwrap();
            },
            "stop"=>{writeln!(out,"Stop: {:?}",dev.stop()).unwrap();},
            _ if words.len()==3 && words[0]=="pulse"=>{
                match (words[1].parse::<u8>(),words[2].parse::<u64>()){
                    (Ok(p),Ok(ms))=>{writeln!(out,"Pulse: {:?}",dev.pulse(p,ms)).unwrap();}
                    _=>out.push_str("Invalid pulse arguments"),
                }
            },
            _=>out.push_str("haptic info | registers | selftest | pulse <1-30> <1-100 ms> | stop"),
        }
        Ok(Some(out))
    }
}
