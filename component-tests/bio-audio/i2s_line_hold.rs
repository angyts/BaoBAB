//! Static BIO output hold for checking external I2S jumper continuity with a DMM.
use bao1x_api::bio::*;
use bao1x_api::bio_resources::*;
use bao1x_hal::bio::{Bio, CoreCsr};
use std::time::{Duration, Instant};
use utralib::utra::bio_bdma;

use crate::i2s_line_hold_kernel::i2s_line_hold_kernel;

const CLAIMER: &str = "BaoBAB-I2S-line-hold";
const PIN_MASK: u32 = (1 << 24) | (1 << 26) | (1 << 27);

pub struct I2sLineHold {
    bio: Bio,
    grant: Option<ResourceGrant>,
    core: BioCore,
    _tx_handle: CoreHandle,
    tx: CoreCsr,
    held_mask: u32,
}

impl Resources for I2sLineHold {
    fn resource_spec() -> ResourceSpec {
        ResourceSpec {
            claimer: CLAIMER.to_string(),
            cores: vec![CoreRequirement::Any],
            fifos: vec![Fifo::Fifo1],
            static_pins: vec![24, 26, 27],
            dynamic_pin_count: 0,
        }
    }
}

impl I2sLineHold {
    pub fn start(held_mask: u32) -> Result<Self, BioError> {
        if ![1 << 24, 1 << 26, 1 << 27].contains(&held_mask) {
            return Err(BioError::InternalError);
        }
        let mut bio = Bio::new();
        let grant = bio.claim_resources(&Self::resource_spec())?;
        let core = grant.cores[0];
        let config = CoreConfig { clock_mode: ClockMode::TargetFreqInt(1_000_000) };
        if let Err(error) = bio.init_core(core, i2s_line_hold_kernel(), config) {
            let _ = bio.release_resources(grant.grant_id);
            return Err(error);
        }
        let mut io = IoConfig::default();
        io.mode = IoConfigMode::SetOnly;
        io.mapped = PIN_MASK;
        if let Err(error) = bio.setup_io_config(io) {
            let _ = bio.de_init_core(core);
            let _ = bio.release_resources(grant.grant_id);
            return Err(error);
        }
        let tx_handle = match unsafe { bio.get_core_handle(Fifo::Fifo1) } {
            Ok(Some(handle)) => handle,
            Ok(None) => {
                let _ = bio.de_init_core(core);
                let mut clear = IoConfig::default();
                clear.mode = IoConfigMode::ClearOnly;
                clear.mapped = !PIN_MASK;
                let _ = bio.setup_io_config(clear);
                let _ = bio.release_resources(grant.grant_id);
                return Err(BioError::NoFreeMachines);
            }
            Err(error) => {
                let _ = bio.de_init_core(core);
                let mut clear = IoConfig::default();
                clear.mode = IoConfigMode::ClearOnly;
                clear.mapped = !PIN_MASK;
                let _ = bio.setup_io_config(clear);
                let _ = bio.release_resources(grant.grant_id);
                return Err(error);
            }
        };
        let tx = CoreCsr::from_handle(&tx_handle);
        bio.set_core_run_state(&grant, true);
        let start = Instant::now();
        while tx.csr.rf(bio_bdma::SFR_FLEVEL_PCLK_REGFIFO_LEVEL1) >= 8 {
            if start.elapsed() > Duration::from_millis(250) {
                bio.set_core_run_state(&grant, false);
                let _ = bio.de_init_core(core);
                let mut clear = IoConfig::default();
                clear.mode = IoConfigMode::ClearOnly;
                clear.mapped = !PIN_MASK;
                let _ = bio.setup_io_config(clear);
                let _ = bio.release_resources(grant.grant_id);
                return Err(BioError::InternalError);
            }
            xous::yield_slice();
        }
        tx.csr.wo(bio_bdma::SFR_TXF1, held_mask);
        Ok(Self { bio, grant: Some(grant), core, _tx_handle: tx_handle, tx, held_mask })
    }

    pub fn held_mask(&self) -> u32 { self.held_mask }

    pub fn stop(&mut self) -> Result<(), BioError> {
        if let Some(grant) = self.grant.take() {
            self.bio.set_core_run_state(&grant, false);
            let deinit_result = self.bio.de_init_core(self.core);
            let mut clear = IoConfig::default();
            clear.mode = IoConfigMode::ClearOnly;
            clear.mapped = !PIN_MASK;
            let io_result = self.bio.setup_io_config(clear);
            let release_result = self.bio.release_resources(grant.grant_id);
            deinit_result?;
            io_result?;
            release_result.map_err(|_| BioError::InternalError)?;
        }
        Ok(())
    }
}

impl Drop for I2sLineHold {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            log::warn!("I2S line-hold cleanup failed: {:?}", error);
        }
    }
}
