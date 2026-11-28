use crate::{CommonEnv, ShellCmdApi};
use bao1x_api::{IoSetup,IoxHal,IoxPort,IoxDir,IoxFunction,IoxEnable,IoxDriveStrength,IoxValue};
use std::{cell::RefCell,time::{Duration,Instant}};
thread_local! { static CTX: RefCell<Option<State>> = RefCell::new(None); }
struct State { io:IoxHal, failed:bool, busy:Option<Instant>, bytes:usize }
const PINS:[(IoxPort,u8);6]=[(IoxPort::PB,3),(IoxPort::PB,2),(IoxPort::PC,3),(IoxPort::PB,4),(IoxPort::PC,1),(IoxPort::PC,0)];
fn put(io:&IoxHal,p:usize,v:bool){let (port,pin)=PINS[p];io.set_gpio_pin_value(port,pin,if v {IoxValue::High}else{IoxValue::Low});}
fn get(io:&IoxHal,p:usize)->bool{let (port,pin)=PINS[p];io.get_gpio_pin_value(port,pin)==IoxValue::High}
fn setup(io:&IoxHal){
 for p in [2usize,0,1,4,5,3] {
 let (port,pin)=PINS[p];
 if p!=3 {put(io,p,p==2||p==0);}
 io.setup_pin(port,pin,Some(if p==3{IoxDir::Input}else{IoxDir::Output}),Some(IoxFunction::Gpio),Some(IoxEnable::Enable),Some(IoxEnable::Disable),None,Some(IoxDriveStrength::Drive2mA));
 }
}
#[no_mangle] extern "C" fn DEV_Digital_Write(pin:u16,v:u8){CTX.with(|c|{if let Some(s)=c.borrow_mut().as_mut(){if !s.failed && (pin as usize)<3 {put(&s.io,pin as usize,v!=0);}}});}
#[no_mangle] extern "C" fn DEV_Digital_Read(_pin:u16)->u8{CTX.with(|c|{let mut b=c.borrow_mut();let s=b.as_mut().unwrap();if s.failed{return 0} if !get(&s.io,3){s.busy=None;return 0} let start=s.busy.get_or_insert_with(Instant::now);if start.elapsed()>=Duration::from_secs(30){s.failed=true;return 0} 1})}
#[no_mangle] extern "C" fn DEV_SPI_WriteByte(v:u8){CTX.with(|c|{let mut b=c.borrow_mut();let s=b.as_mut().unwrap();if s.failed{return} for bit in (0..8).rev(){put(&s.io,4,(v>>bit)&1!=0);put(&s.io,5,true);put(&s.io,5,false);}s.bytes+=1;});}
#[no_mangle] extern "C" fn DEV_Delay_ms(ms:u32){if !failed(){std::thread::sleep(Duration::from_millis(ms as u64));}}
extern "C" {fn EPD_2in13_V4_Init();fn EPD_2in13_V4_Clear();fn EPD_2in13_V4_Clear_Black();fn EPD_2in13_V4_Display(p:*mut u8);fn EPD_2in13_V4_Init_Fast();fn EPD_2in13_V4_Display_Fast(p:*mut u8);fn EPD_2in13_V4_Display_Base(p:*mut u8);fn EPD_2in13_V4_Display_Partial(p:*mut u8);fn EPD_2in13_V4_Sleep();}
fn failed()->bool{CTX.with(|c|c.borrow().as_ref().map(|s|s.failed).unwrap_or(true))}
fn stage(name:&str,f:impl FnOnce())->bool{if failed(){return false} log::info!("VENDOR stage {}",name);f();!failed()}
pub struct EpaperVendorCmd;
impl<'a> ShellCmdApi<'a> for EpaperVendorCmd{
 cmd_api!(epapervendor);
 fn process(&mut self,args:String,_env:&mut CommonEnv)->Result<Option<String>,xous::Error>{
 let a:Vec<_>=args.split_whitespace().collect();
 if a==["info"] {return Ok(Some("Vendor V4 C / GPIO SPI mode 0 / 4000 bytes / no PWR GPIO / experimental".into()))}
 if !matches!(a.as_slice(),["gpio"]|["test"]|["full"]|["fast"]|["partial"]|["hold","mosi","high"]|["hold","mosi","low"]){return Ok(Some("epapervendor info|gpio|hold mosi high|hold mosi low|test|full|fast|partial".into()))}
 let io=IoxHal::new();setup(&io);
 if a[0]=="hold"{put(&io,4,a[2]=="high");return Ok(Some("MOSI held; CS inactive. Measure now; reset or run gpio to release.".into()))}
 let mut report=String::new();let mut ok=true;
 for p in [1usize,4,5] {for v in [false,true]{put(&io,p,v);std::thread::sleep(Duration::from_millis(2));let actual=get(&io,p);ok &= actual==v;report.push_str(&format!("pin {} requested {} read {}\n",p,v,actual));}put(&io,p,false);}
 log::info!("VENDOR GPIO: {}",report);
 if a[0]=="gpio" || !ok {return Ok(Some(format!("{}GPIO match={}; no refresh",report,ok)))}
 CTX.with(|c|*c.borrow_mut()=Some(State{io,failed:false,busy:None,bytes:0}));
 let mut image=[0u8;4000];for y in 0..250 {for x in 0..16 {image[y*16+x]=if (y/16+x/2)%2==0{0xff}else{0};}}
 unsafe {
 stage("init",||EPD_2in13_V4_Init());
 if a[0]=="fast"{stage("fast init",||EPD_2in13_V4_Init_Fast());stage("fast pattern",||EPD_2in13_V4_Display_Fast(image.as_mut_ptr()));}
 else if a[0]=="partial"{stage("base",||EPD_2in13_V4_Display_Base(image.as_mut_ptr()));image[0..160].fill(0xff);stage("partial",||EPD_2in13_V4_Display_Partial(image.as_mut_ptr()));}
 else {stage("white",||EPD_2in13_V4_Clear());if !failed(){std::thread::sleep(Duration::from_secs(2));}stage("black",||EPD_2in13_V4_Clear_Black());if !failed(){std::thread::sleep(Duration::from_secs(2));}stage("pattern",||EPD_2in13_V4_Display(image.as_mut_ptr()));}
 stage("sleep",||EPD_2in13_V4_Sleep());
 }
 let result=CTX.with(|c|{let s=c.borrow_mut().take().unwrap();put(&s.io,2,true);put(&s.io,5,false);put(&s.io,4,false);format!("Vendor sequence ended: bytes={}, timeout={}. {} Verify panel visually.",s.bytes,s.failed,if s.failed{"Power off display; sleep not confirmed."}else{"Sleep command sent."})});
 Ok(Some(result))
 }
}
