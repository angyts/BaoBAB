//! Reusable Xous DRV2605L register transport. Non-actuating register audit.
use bao1x_api::{I2cApi,I2cResult};
use bao1x_hal::i2c::I2c;
pub const ADDRESS:u8=0x5a;
#[derive(Debug)]
pub enum Error { Transport(xous::Error), Response(I2cResult), UnexpectedId(u8), Fault(u8), Readback(u8,u8,u8), UnsafeState(u8) }
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
        let a=self.write(0x02,0);
        let b=self.write(0x0c,0);
        let c=self.write(0x01,0x40);
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
}
