//! BIO I2S clock-only bring-up for the MAX98357A.
//!
//! Uses Xous' BIO service/resource manager. This phase generates BCLK and
//! LRCLK while holding DIN low. It intentionally does not produce sound.

use bao1x_api::bio::*;
use bao1x_api::bio_resources::*;
use bao1x_hal::bio::Bio;

use crate::i2s_clock_kernel;

const CLAIMER: &str = "BaoBAB-I2S-clock";
const PIN_MASK: u32 = (1 << 24) | (1 << 26) | (1 << 27);

pub struct I2sClockTest {
    bio: Bio,
    grant: Option<ResourceGrant>,
    core: BioCore,
    actual_quantum_hz: u32,
}

impl Resources for I2sClockTest {
    fn resource_spec() -> ResourceSpec {
        ResourceSpec {
            claimer: CLAIMER.to_string(),
            cores: vec![CoreRequirement::Any],
            fifos: vec![],
            static_pins: vec![24, 26, 27],
            dynamic_pin_count: 0,
        }
    }
}

impl I2sClockTest {
    pub fn start() -> Result<Self, BioError> {
        let mut bio = Bio::new();
        let grant = bio.claim_resources(&Self::resource_spec()).map_err(BioError::from)?;
        let core = grant.cores[0];

        let config = CoreConfig {
            clock_mode: ClockMode::TargetFreqInt(3_072_000),
        };

        // The current Xous API accepts (program, optional pad word), not an
        // older (program, offset) pair.
        let actual_quantum_hz = match bio.init_core(core, i2s_clock_kernel(), config) {
            Ok(Some(freq)) => freq,
            Ok(None) => {
                let _ = bio.de_init_core(core);
                let _ = bio.release_resources(grant.grant_id);
                return Err(BioError::InternalError);
            }
            Err(error) => {
                let _ = bio.release_resources(grant.grant_id);
                return Err(error);
            }
        };

        let mut io = IoConfig::default();
        io.mode = IoConfigMode::SetOnly;
        io.mapped = PIN_MASK;
        if let Err(error) = bio.setup_io_config(io) {
            let _ = bio.de_init_core(core);
            let mut clear = IoConfig::default();
            clear.mode = IoConfigMode::ClearOnly;
            clear.mapped = !PIN_MASK;
            let _ = bio.setup_io_config(clear);
            let _ = bio.release_resources(grant.grant_id);
            return Err(error);
        }

        bio.set_core_run_state(&grant, true);

        Ok(Self {
            bio,
            grant: Some(grant),
            core,
            actual_quantum_hz,
        })
    }

    pub fn actual_quantum_hz(&self) -> u32 {
        self.actual_quantum_hz
    }

    pub fn actual_bclk_hz(&self) -> u32 {
        self.actual_quantum_hz / 2
    }

    pub fn actual_lrclk_hz(&self) -> u32 {
        self.actual_quantum_hz / 64
    }

    pub fn stop(&mut self) -> Result<(), BioError> {
        if let Some(grant) = self.grant.take() {
            self.bio.set_core_run_state(&grant, false);

            // Stop and de-initialize before returning the pins to normal GPIO muxing.
            let deinit_result = self.bio.de_init_core(self.core);

            // ClearOnly uses the inverse mask: zero bits in mapped are unmapped.
            let mut io = IoConfig::default();
            io.mode = IoConfigMode::ClearOnly;
            io.mapped = !PIN_MASK;
            let io_result = self.bio.setup_io_config(io);

            let release_result = self.bio.release_resources(grant.grant_id);

            deinit_result?;
            io_result?;
            release_result.map_err(BioError::from)?;
        }
        Ok(())
    }
}

impl Drop for I2sClockTest {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            log::warn!("I2S clock test cleanup failed: {:?}", error);
        }
    }
}
