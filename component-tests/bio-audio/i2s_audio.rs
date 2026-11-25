//! Bounded 440 Hz I2S tone generator using Xous' BIO service.
//!
//! Streams 16-bit mono PCM (duplicated to L/R) in 32-bit standard-I2S slots.
//! Initial level is 512/32767 (~1.6% full scale), with 10 ms fade-in/out.
//! This is a short speaker check, not a general streaming player yet.

use bao1x_api::bio::*;
use bao1x_api::bio_resources::*;
use bao1x_hal::bio::{Bio, CoreCsr};
use std::time::{Duration, Instant};
use utralib::utra::bio_bdma;

use crate::i2s_audio_kernel;

const CLAIMER: &str = "BaoBAB-I2S-audio";
const PIN_MASK: u32 = (1 << 24) | (1 << 26) | (1 << 27);
const TARGET_SAMPLE_RATE: u32 = 48_000;
const TONE_HZ: u32 = 440;
const TONE_MS: u32 = 250;
const FADE_MS: u32 = 10;
const FIFO_HIGH_WATER: u32 = 6;
const FIFO_TIMEOUT: Duration = Duration::from_secs(3);

// Peak amplitude 512/32767, before fade envelope.
const SINE_64: [i16; 64] = [0, 50, 100, 149, 196, 241, 284, 325, 362, 396, 426, 452, 473, 490, 502, 510, 512, 510, 502, 490, 473, 452, 426, 396, 362, 325, 284, 241, 196, 149, 100, 50, 0, -50, -100, -149, -196, -241, -284, -325, -362, -396, -426, -452, -473, -490, -502, -510, -512, -510, -502, -490, -473, -452, -426, -396, -362, -325, -284, -241, -196, -149, -100, -50];

#[derive(Debug)]
pub enum AudioError {
    Bio(BioError),
    Timeout(&'static str),
}

impl From<BioError> for AudioError {
    fn from(error: BioError) -> Self { Self::Bio(error) }
}

#[derive(Debug, Clone, Copy)]
pub struct AudioReport {
    pub quantum_hz: u32,
    pub bclk_hz: u32,
    pub lrclk_hz: u32,
    pub frames: u32,
    pub peak_sample: i16,
}

pub struct I2sTone {
    bio: Bio,
    grant: Option<ResourceGrant>,
    core: BioCore,
    actual_quantum_hz: u32,
    // Keep handles alive for the lifetime of the direct CSR mappings.
    _tx_handle: CoreHandle,
    _rx_handle: CoreHandle,
    tx: CoreCsr,
    rx: CoreCsr,
}

impl Resources for I2sTone {
    fn resource_spec() -> ResourceSpec {
        ResourceSpec {
            claimer: CLAIMER.to_string(),
            cores: vec![CoreRequirement::Any],
            fifos: vec![Fifo::Fifo1, Fifo::Fifo2],
            static_pins: vec![24, 26, 27],
            dynamic_pin_count: 0,
        }
    }
}

impl I2sTone {
    fn new() -> Result<Self, AudioError> {
        let mut bio = Bio::new();
        let grant = bio.claim_resources(&Self::resource_spec()).map_err(|_| AudioError::Bio(BioError::ResourceInUse))?;
        let core = grant.cores[0];
        let config = CoreConfig { clock_mode: ClockMode::TargetFreqInt(6_144_000) };

        let actual_quantum_hz = match bio.init_core(core, i2s_audio_kernel(), config) {
            Ok(Some(freq)) => freq,
            Ok(None) => {
                let _ = bio.de_init_core(core);
                let _ = bio.release_resources(grant.grant_id);
                return Err(AudioError::Bio(BioError::InternalError));
            }
            Err(error) => {
                let _ = bio.release_resources(grant.grant_id);
                return Err(AudioError::Bio(error));
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
            return Err(AudioError::Bio(error));
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
                return Err(AudioError::Bio(BioError::NoFreeMachines));
            }
            Err(error) => {
                let _ = bio.de_init_core(core);
                let mut clear = IoConfig::default();
                clear.mode = IoConfigMode::ClearOnly;
                clear.mapped = !PIN_MASK;
                let _ = bio.setup_io_config(clear);
                let _ = bio.release_resources(grant.grant_id);
                return Err(AudioError::Bio(error));
            }
        };

        let rx_handle = match unsafe { bio.get_core_handle(Fifo::Fifo2) } {
            Ok(Some(handle)) => handle,
            Ok(None) => {
                let _ = bio.de_init_core(core);
                let mut clear = IoConfig::default();
                clear.mode = IoConfigMode::ClearOnly;
                clear.mapped = !PIN_MASK;
                let _ = bio.setup_io_config(clear);
                let _ = bio.release_resources(grant.grant_id);
                return Err(AudioError::Bio(BioError::NoFreeMachines));
            }
            Err(error) => {
                let _ = bio.de_init_core(core);
                let mut clear = IoConfig::default();
                clear.mode = IoConfigMode::ClearOnly;
                clear.mapped = !PIN_MASK;
                let _ = bio.setup_io_config(clear);
                let _ = bio.release_resources(grant.grant_id);
                return Err(AudioError::Bio(error));
            }
        };

        let tx = CoreCsr::from_handle(&tx_handle);
        let rx = CoreCsr::from_handle(&rx_handle);

        Ok(Self {
            bio,
            grant: Some(grant),
            core,
            actual_quantum_hz,
            _tx_handle: tx_handle,
            _rx_handle: rx_handle,
            tx,
            rx,
        })
    }

    fn fifo_write(&self, word: u32) -> Result<(), AudioError> {
        let start = Instant::now();
        while self.tx.csr.rf(bio_bdma::SFR_FLEVEL_PCLK_REGFIFO_LEVEL1) > FIFO_HIGH_WATER {
            if start.elapsed() > FIFO_TIMEOUT {
                return Err(AudioError::Timeout("BIO TX FIFO did not drain"));
            }
            xous::yield_slice();
        }
        self.tx.csr.wo(bio_bdma::SFR_TXF1, word);
        Ok(())
    }

    fn wait_complete(&self) -> Result<(), AudioError> {
        let start = Instant::now();
        while self.rx.csr.rf(bio_bdma::SFR_FLEVEL_PCLK_REGFIFO_LEVEL2) == 0 {
            if start.elapsed() > FIFO_TIMEOUT {
                return Err(AudioError::Timeout("BIO audio kernel did not signal completion"));
            }
            xous::yield_slice();
        }
        let _completion_token = self.rx.csr.r(bio_bdma::SFR_RXF2);
        Ok(())
    }

    pub fn play_440() -> Result<AudioReport, AudioError> {
        let mut audio = Self::new()?;
        let actual_sample_rate = audio.actual_quantum_hz / 128;
        if actual_sample_rate == 0 {
            return Err(AudioError::Bio(BioError::InternalError));
        }

        // The BIO kernel reads the frame count, followed by L and R sample words.
        let frames = (actual_sample_rate * TONE_MS) / 1000;
        let fade_frames = ((actual_sample_rate * FADE_MS) / 1000).max(1);
        let mut phase = 0u32;
        let first_sample = make_sample(0, frames, fade_frames, actual_sample_rate, &mut phase);
        audio.fifo_write(frames)?;
        audio.fifo_write((first_sample as u16) as u32)?;
        audio.fifo_write((first_sample as u16) as u32)?;

        audio.bio.set_core_run_state(audio.grant.as_ref().unwrap(), true);

        let mut peak_sample = first_sample.abs();
        for frame in 1..frames {
            let sample = make_sample(frame, frames, fade_frames, actual_sample_rate, &mut phase);
            peak_sample = peak_sample.max(sample.abs());
            let word = (sample as u16) as u32;
            audio.fifo_write(word)?;
            audio.fifo_write(word)?;
        }

        audio.wait_complete()?;

        let report = AudioReport {
            quantum_hz: audio.actual_quantum_hz,
            bclk_hz: audio.actual_quantum_hz / 2,
            lrclk_hz: actual_sample_rate,
            frames,
            peak_sample,
        };
        audio.stop()?;
        Ok(report)
    }

    fn stop(&mut self) -> Result<(), AudioError> {
        if let Some(grant) = self.grant.take() {
            self.bio.set_core_run_state(&grant, false);
            let deinit_result = self.bio.de_init_core(self.core);

            let mut io = IoConfig::default();
            io.mode = IoConfigMode::ClearOnly;
            io.mapped = !PIN_MASK;
            let io_result = self.bio.setup_io_config(io);
            let release_result = self.bio.release_resources(grant.grant_id);

            deinit_result.map_err(AudioError::Bio)?;
            io_result.map_err(AudioError::Bio)?;
            release_result.map_err(|_| AudioError::Bio(BioError::InternalError))?;
        }
        Ok(())
    }
}

impl Drop for I2sTone {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            log::warn!("I2S audio cleanup failed: {:?}", error);
        }
    }
}

fn make_sample(
    frame: u32,
    frames: u32,
    fade_frames: u32,
    sample_rate: u32,
    phase: &mut u32,
) -> i16 {
    let index = ((*phase as u64 * 64) / sample_rate as u64) as usize & 63;
    let mut sample = SINE_64[index] as i32;
    let fade_in = (frame + 1).min(fade_frames);
    let fade_out = (frames - frame).min(fade_frames);
    let envelope = fade_in.min(fade_out);
    sample = sample * envelope as i32 / fade_frames as i32;

    *phase += TONE_HZ;
    if *phase >= sample_rate {
        *phase -= sample_rate;
    }
    sample as i16
}
