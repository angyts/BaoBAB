//! Reusable Xous DRV2605L register transport. Register audit and signed RTP diagnostics.
use bao1x_api::{I2cApi,I2cResult};
use bao1x_hal::i2c::I2c;
pub const ADDRESS:u8=0x5a;
#[derive(Debug)]
pub enum Error { Transport(xous::Error), Response(I2cResult), UnexpectedId(u8), Fault(u8), Readback(u8,u8,u8), UnsafeState(u8), InvalidPulse }
pub struct Haptic { bus:I2c }
impl Haptic {
    pub fn new()->Self { Self { bus:I2c::new() } }
    pub fn read(&mut self, reg:u8)->Result<u8,Error> {
        let mut b=[0];
        match self.bus.i2c_read(ADDRESS,reg,&mut b,false).map_err(Error::Transport)? {
            I2cResult::Ack(1)=>Ok(b[0]),
            r=>Err(Error::Response(r)),
        }
    }
    pub fn status(&mut self)->Result<u8,Error> { self.read(0) }
    pub fn probe(&mut self)->Result<u8,Error> {
        let s=self.status()?;
        if s>>5==7 { Ok(s) } else { Err(Error::UnexpectedId(s>>5)) }
    }

    fn write(&mut self, reg:u8, value:u8)->Result<(),Error> {
        match self.bus.i2c_write(ADDRESS,reg,&[value]).map_err(Error::Transport)? {
            I2cResult::Ack(1)=>Ok(()),
            r=>Err(Error::Response(r)),
        }
    }
    pub fn stop(&mut self)->Result<(),Error> {
        // Attempt every shutdown operation even if an earlier one fails.
        let c=self.write(0x01,0x40);
        let a=self.write(0x02,0);
        let b=self.write(0x0c,0);
        a.and(b).and(c)
    }

    fn checked_write(&mut self, reg:u8, value:u8)->Result<(),Error> {
        self.write(reg,value)?;
        let actual=self.read(reg)?;
        if actual!=value { return Err(Error::Readback(reg,value,actual)); }
        Ok(())
    }
    pub fn audit(&mut self, out:&mut String)->Result<(),Error> {
        use core::fmt::Write;
        use std::{thread::sleep,time::Duration};
        let s=self.probe()?;
        if s & 0x0b != 0 { return Err(Error::Fault(s)); }
        let mode=self.read(1)?;
        let go=self.read(0x0c)?;
        if mode!=0x40 || go!=0 { return Err(Error::UnsafeState(mode)); }
        let original=self.read(0x04)?;
        // Waveform slot 1 is harmless while GO is zero and mode is internal trigger.
        let marker=if original==1 {2} else {1};
        let result=(|| {
            self.write(1,0)?;
            sleep(Duration::from_millis(5));
            let actual=self.read(1)?;
            writeln!(out,"wake MODE={:02X}",actual).unwrap();
            if actual!=0 {return Err(Error::Readback(1,0,actual));}
            self.checked_write(0x04,marker)?;
            writeln!(out,"slot write/readback={:02X}",marker).unwrap();
            sleep(Duration::from_millis(100));
            let v=self.read(4)?;
            writeln!(out,"after 100ms slot={:02X}",v).unwrap();
            if v!=marker {return Err(Error::Readback(4,marker,v));}
            self.checked_write(1,0x40)?;
            sleep(Duration::from_millis(20));
            let v=self.read(4)?;
            writeln!(out,"standby slot={:02X}",v).unwrap();
            if v!=marker {return Err(Error::Readback(4,marker,v));}
            self.write(1,0)?;
            sleep(Duration::from_millis(5));
            let v=self.read(4)?;
            writeln!(out,"rewake slot={:02X}",v).unwrap();
            if v!=marker {return Err(Error::Readback(4,marker,v));}
            Ok(())
        })();
        // Restore original slot without ever asserting GO.
        let restore=(|| {self.write(1,0)?;sleep(Duration::from_millis(5));self.checked_write(4,original)})();
        let stop=self.stop();
        writeln!(out,"restore={:?}; standby={:?}",restore,stop).unwrap();
        result?;
        restore?;
        stop?;
        let mode=self.read(1)?;
        if mode!=0x40 {return Err(Error::Readback(1,0x40,mode));}
        Ok(())
    }

    // TI section 8.5.8.1.1: signed open-loop RTP uses 0 as neutral,
    // 0x7f as full forward drive. Unsigned zero is NOT neutral.
    pub fn trace_rtp(&mut self, amplitude:u8, out:&mut String)->Result<(),Error> {
        use core::fmt::Write;
        use std::{thread::sleep,time::Duration};
        if amplitude>127 {return Err(Error::InvalidPulse);}
        let status=self.probe()?;
        if status & 0x0b != 0 {return Err(Error::Fault(status));}
        let mode=self.read(1)?;
        if mode!=0x40 || self.read(0x0c)?!=0 {return Err(Error::UnsafeState(mode));}
        let feedback=self.read(0x1a)?;
        let control=self.read(0x1d)?;
        let clamp=self.read(0x17)?;
        let boost=self.read(0x1b)?;
        let result=(|| {
            self.write(1,0)?;
            sleep(Duration::from_millis(5));
            self.checked_write(1,0)?;
            self.checked_write(0x1a,feedback & 0x7f)?;
            self.checked_write(0x1b,boost & 0x7f)?;
            self.checked_write(0x1d,(control | 0x20) & !0x08)?;
            self.checked_write(0x17,0x64)?;
            self.checked_write(2,0)?;
            self.checked_write(1,5)?;
            writeln!(out,"RTP ready: MODE=05 signed neutral=00 clamp=64").unwrap();
            self.checked_write(2,amplitude)?;
            sleep(Duration::from_millis(100));
            // Stop drive before collecting other registers.
            self.write(1,0x40)?;
            let s=self.status()?;
            let m=self.read(1)?;
            let c=self.read(0x17)?;
            let f=self.read(0x1d)?;
            writeln!(out,"after drive: STATUS={:02X} MODE={:02X} CLAMP={:02X} CTRL3={:02X}",s,m,c,f).unwrap();
            if s & 0x0b !=0 {return Err(Error::Fault(s));}
            if m!=0x40 {return Err(Error::Readback(1,0x40,m));}
            if c!=0x64 {return Err(Error::Readback(0x17,0x64,c));}
            if f!=((control | 0x20) & !0x08) {return Err(Error::Readback(0x1d,(control | 0x20) & !0x08,f));}
            Ok(())
        })();
        let stop=self.stop();
        writeln!(out,"shutdown={:?}",stop).unwrap();
        stop?;
        // Restore only after verified standby, using internal trigger with GO=0.
        let m=self.read(1)?;
        if m!=0x40 {return Err(Error::Readback(1,0x40,m));}
        let restore=(|| {
            self.write(1,0)?;
            sleep(Duration::from_millis(5));
            self.checked_write(0x1a,feedback)?;
            self.checked_write(0x1b,boost)?;
            self.checked_write(0x1d,control)?;
            self.checked_write(0x17,clamp)?;
            Ok::<(),Error>(())
        })();
        let final_stop=self.stop();
        writeln!(out,"restore={:?} final standby={:?}",restore,final_stop).unwrap();
        final_stop?;
        restore?;
        result
    }
}
