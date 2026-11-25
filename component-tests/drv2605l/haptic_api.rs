//! Reusable Xous DRV2605L register transport. Bounded experimental ERM pulses.
use bao1x_api::{I2cApi,I2cResult};
use bao1x_hal::i2c::I2c;
pub const ADDRESS:u8=0x5a;
#[derive(Debug)]
pub enum Error { Transport(xous::Error), Response(I2cResult), UnexpectedId(u8), InvalidPulse, Fault(u8), Readback }
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
            I2cResult::Ack(_)=>Ok(()),
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
    pub fn pulse(&mut self, percent:u8, ms:u64)->Result<(),Error> {
        if percent==0 || percent>30 || ms==0 || ms>100 { return Err(Error::InvalidPulse); }
        let status=self.probe()?;
        if status & 0x0b != 0 { return Err(Error::Fault(status)); }
        self.stop()?;
        let result=(|| {
            // ERM, open-loop, unsigned RTP; no calibration or ROM effects.
            self.write(0x01,0)?;
            let feedback=self.read_feedback_erm()?;
            self.write(0x1a,feedback)?;
            let control=self.read(0x1d)?;
            self.write(0x1d,control | 0x28)?;
            // Open-loop clamp: 0x40 corresponds to about 1.4 V nominal.
            self.write(0x17,0x40)?;
            if self.read(0x17)? != 0x40 || self.read(0x1d)? & 0x28 != 0x28 {
                return Err(Error::Readback);
            }
            self.write(0x02,0)?;
            self.write(0x01,5)?;
            self.write(0x02,((percent as u16*255)/100) as u8)?;
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(())
        })();
        let stopped=self.stop();
        stopped?;
        result?;
        let s=self.status()?;
        if s & 0x0b != 0 { return Err(Error::Fault(s)); }
        Ok(())
    }
    fn read_feedback_erm(&mut self)->Result<u8,Error> { Ok(self.read(0x1a)? & 0x7f) }
}
