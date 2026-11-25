# MAX98357A component bring-up

Status: research stage. No hardware wiring or firmware has been validated.

## Initial discoveries

- The user checkout of Xous contains a dedicated UDMA I2S peripheral definition in `utralib/bao1x/bao1x_peri.svd` and I2S event definitions in `libs/bao1x-api/src/udma.rs`.
- No implemented Baochip I2S driver was found in the user's searched `apps-dabao`, `libs/bao1x-hal`, or `libs/bio-lib` paths.
- The `services/codec` tree contains a TLV320 AIC3100 backend whose code may demonstrate the existing Betrusted I2S API, but it must be inspected for compatibility; do not assume it drives Baochip UDMA I2S.
- The `bio-lib` WS2812 implementation demonstrates BIO resource claims, CoreCsr and FIFO lifetimes, but is not an I2S driver.

## Proposed test sequence

1. Identify the exact Dabao breakout pin mux and I2S master signals from RTL, SVD, and board source.
2. Inspect UDMA I2S registers and event semantics.
3. Inspect `services/codec` and existing I2S transmit APIs.
4. Verify exact MAX98357A breakout pinout, VDD, SD_MODE, GAIN_SLOT, speaker output.
5. Connect only after pin and supply plan is proven.
6. Transmit digital silence; verify clock presence and amplifier state.
7. Play a low-level generated 440 Hz tone with conservative amplifier gain and 8-ohm 0.5 W speaker limits.
8. Test PCM playback and shutdown.

No GPIO assignments are approved yet. Earlier PC9/PC10/PC11 are only guesses and must not be soldered.
