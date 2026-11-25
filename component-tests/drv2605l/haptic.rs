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
        let words:Vec<&str>=args.split_whitespace().collect();
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
            "audit"=>{let result=dev.audit(&mut out);writeln!(out,"Audit: {:?}",result).unwrap();},
            "rtpzero"=>{let result=dev.trace_rtp(0,&mut out);writeln!(out,"RTP zero: {:?}",result).unwrap();},
            "trace"=>{let result=dev.trace_rtp(127,&mut out);writeln!(out,"RTP pulse: {:?}",result).unwrap();},
            "ladder"=>{let result=dev.pattern(&[(32,100),(64,100),(96,100),(127,100)],&mut out);writeln!(out,"Ladder: {:?}",result).unwrap();},
            "double"=>{let result=dev.pattern(&[(127,100),(127,100)],&mut out);writeln!(out,"Double: {:?}",result).unwrap();},
            "warning"=>{let result=dev.pattern(&[(127,100),(127,100),(127,200)],&mut out);writeln!(out,"Warning: {:?}",result).unwrap();},
            _ if words.len()==3 && words[0]=="pulse"=>{
                match (words[1].parse::<u8>(),words[2].parse::<u64>()){
                    (Ok(a),Ok(ms))=>{let result=dev.pulse(a,ms,&mut out);writeln!(out,"Pulse: {:?}",result).unwrap();},
                    _=>out.push_str("Invalid arguments"),
                }
            },
            _=>out.push_str("haptic info | registers | selftest | audit | rtpzero | trace | pulse <0-127> <1-200 ms> | ladder | double | warning | stop"),
        }
        Ok(Some(out))
    }
}
