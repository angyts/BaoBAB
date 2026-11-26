# Waveshare 2.13-inch e-Paper V4 driver notes

These are porting notes based on Waveshare's upstream V4 Python and Arduino drivers. They are not a ready-to-build Xous driver.

## Interface behaviour

- Four-wire SPI.
- SPI mode 0.
- MSB-first.
- DC LOW = command; DC HIGH = data.
- CS is active-low.
- BUSY HIGH = busy; BUSY LOW = idle.
- Hardware RESET sequence: HIGH 20 ms, LOW 2 ms, HIGH 20 ms.

## Full initialisation sequence (upstream Python V4 driver)

```text
hardware reset
wait BUSY low
command 0x12                   # software reset
wait BUSY low
command 0x01, data F9 00 00    # driver output control
command 0x11, data 03          # data entry mode
command 0x44, data 00 0F       # X RAM address window
command 0x45, data 00 00 F9 00 # Y RAM address window
command 0x4E, data 00          # X address counter
command 0x4F, data 00 00       # Y address counter
command 0x3C, data 05          # border waveform
command 0x21, data 00 80       # display update control
command 0x18, data 80          # internal temperature sensor control
wait BUSY low
```

## Framebuffer and full refresh

- Driver constants: width 122, height 250.
- Row stride = ceil(122 / 8) = 16 bytes.
- Full frame size = 16 x 250 = 4,000 bytes.
- Write image memory using command 0x24 and 4,000 data bytes.
- Full update sequence: command 0x22, data 0xF7, command 0x20.
- Wait until BUSY reads LOW.

The Waveshare Python `Clear(color)` uses 0xFF for white. Test 0x00 for black after the white clear succeeds.

## Porting API shape

- `send_command(byte)`: set DC LOW, transfer command.
- `send_data(byte)`: set DC HIGH, transfer byte.
- `send_data_buffer(bytes)`: set DC HIGH, transfer buffer as a burst.
- `wait_idle(timeout)`: poll BUSY HIGH with a deadline; timeout is an error.
- `reset()`: perform documented pulse sequence.

Avoid sending each byte as an independent high-level SPI setup if the HAL can transfer a complete buffer. It wastes time and can create unwanted CS gaps. But ensure DC and CS semantics match the controller: command/data transitions must not be accidentally hidden inside one CS transaction with the wrong DC level.

Upstream links:
- https://github.com/waveshareteam/e-Paper/blob/master/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd2in13_V4.py
- https://github.com/waveshareteam/e-Paper/blob/master/Arduino/epd2in13_V4/epd2in13_V4.cpp
