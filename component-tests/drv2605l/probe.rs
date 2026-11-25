//! Read-only DRV2605L probe; requires a platform-specific I2C adapter.
#![no_std]

pub const ADDRESS: u8 = 0x5a;
pub const STATUS: u8 = 0x00;

pub trait I2c {
    type Error;
    fn write_read(&mut self, address: u8, tx: &[u8], rx: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Status(pub u8);
impl Status {
    pub fn device_id(&self) -> u8 { (self.0 >> 5) & 7 }
    pub fn expected_device(&self) -> bool { self.device_id() == 7 }
}

pub fn probe<B: I2c>(bus: &mut B) -> Result<Status, B::Error> {
    let mut rx = [0u8; 1];
    bus.write_read(ADDRESS, &[STATUS], &mut rx)?;
    Ok(Status(rx[0]))
}

// Platform harness: call probe ten times, log raw byte and device ID,
// fail on transport errors or unexpected ID. Never trigger the motor.
