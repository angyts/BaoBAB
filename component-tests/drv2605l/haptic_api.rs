//! Reusable Xous DRV2605L register transport. No motor activation in this revision.
use bao1x_api::{I2cApi,I2cResult};
use bao1x_hal::i2c::I2c;
pub const ADDRESS:u8=0x5a;
#[derive(Debug)]
pub enum Error { Transport(xous::Error), Response(I2cResult), UnexpectedId(u8) }
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
}
