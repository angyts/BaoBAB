//! Software readback audit for BIO24/26/27 using GPIO pad input sampling.
use bao1x_api::bio::*;
use bao1x_api::bio_resources::*;
use bao1x_hal::bio::{Bio, CoreCsr};
use std::time::{Duration, Instant};
use utralib::utra::bio_bdma;

use crate::i2s_pin_audit_kernel::i2s_pin_audit_kernel;

const CLAIMER: &str = "BaoBAB-I2S-pin-audit";
const PIN_MASK: u32 = (1 << 24) | (1 << 26) | (1 << 27);
const READBACK_TIMEOUT: Duration = Duration::from_millis(500);

pub struct I2sPinAudit {
    bio: Bio,
    grant: Option<ResourceGrant>,
    core: BioCore,
    _rx_handle: CoreHandle,
    rx: CoreCsr,
}

impl Resources for I2sPinAudit {
    fn resource_spec() -> ResourceSpec {
        ResourceSpec {
            claimer: CLAIMER.to_string(),
            cores: vec![CoreRequirement::Any],
            fifos: vec![Fifo::Fifo2],
            static_pins: vec![24, 26, 27],
            dynamic_pin_count: 0,
        }
    }
}

impl I2sPinAudit {
    pub fn run() -> Result<[u32; 4], BioError> {
        let mut bio = Bio::new();
        let grant = bio.claim_resources(&Self::resource_spec())?;
        let core = grant.cores[0];

        let config = CoreConfig { clock_mode: ClockMode::TargetFreqInt(1_000_000) };
        if let Err(error) = bio.init_core(core, i2s_pin_audit_kernel(), config) {
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

        let rx_handle = match unsafe { bio.get_core_handle(Fifo::Fifo2) } {
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

        let rx = CoreCsr::from_handle(&rx_handle);
        bio.set_core_run_state(&grant, true);

        let start = Instant::now();
        while rx.csr.rf(bio_bdma::SFR_FLEVEL_PCLK_REGFIFO_LEVEL2) < 4 {
            if start.elapsed() > READBACK_TIMEOUT {
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

        let mut snapshots = [0u32; 4];
        for item in snapshots.iter_mut() {
            *item = rx.csr.r(bio_bdma::SFR_RXF2) & PIN_MASK;
        }

        bio.set_core_run_state(&grant, false);
        bio.de_init_core(core)?;
        let mut clear = IoConfig::default();
        clear.mode = IoConfigMode::ClearOnly;
        clear.mapped = !PIN_MASK;
        bio.setup_io_config(clear)?;
        bio.release_resources(grant.grant_id).map_err(|_| BioError::InternalError)?;

        Ok(snapshots)
    }
}
