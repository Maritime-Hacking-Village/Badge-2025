# Differential Destroyer
This document attempts to provide an overview of the MHV badge this year. We're going to go over the features of the board, walk through the board layout, and dive into the firmware interfaces and scriping engine. Please note that this documentation, along with the firmware itself, is very much a work-in-progress. As we continue to develop the board's firmware, we will be updating these docs and the scripting interface. *All voltage glitching functions are experimental and may cause damage if used improperly. We assume no liability for damages caused by performing voltage glitching.*

## *Aug. 15 UPDATE*
We'll be taking a short break from firmware development to decompress and catch up on some other work. In a week or so, we'll come back to this repo and start cleaning up the firmware, making optimization passes for heap usage, and improving the project documentation. Please try to be patient with us.

If you notice anything missing from the repo, features you'd like added, or bugs you've found, please open an issue on the GitHub page, or reach out directly to nick@maritimehackingvillage.com.

## *Important Notice!*
If there is a missing feature that you desperately want implemented during DEF CON, or if you run into bugs, please approach @nick-halt and I'd be happy to help get your board working as desired, time permitting.

## *SAFETY WARNING*
The diferential injector circuit is *not* electrially isolated from the connected device or bus. You should not connect this device directly to a boat without additional safety precautions.

The injector circuit can drive 400mA maximum output current. This is enough to be dangerous in certain circumstances. Please do not use the injector mode without a robust understanding of what you're doing and assurance that you're operating the device safely. Please refer to the provided schematics.

Absolutely do not attempt fault injection using the injector circuit on the CTF equipment at the village, or without explicit and informed permission from the device's handler.

## Features
### Hardware
The hardware contains the following:

- Pico 2 compatible headers.
  - All boards sold through the village will come bundled with a Pico 2 flashed with the current version of firmware. If you want to BYOD here, please approach village staff and we'll gladly flash your board.
- 1.9" 320x170 TFT LCD display
- 5-bit joystick input
- 2 buttons
- 9 WS2812B NeoPixels
- Accelerometer
- SAO connector bridged to the Pico 2 and I2C bus
- SD card slot
- Battery charging capability with a 2-pin JST-PH connector
- Two 3-pin (H, L, GND) screw terminal ports for differentially modulated protocols
  - One Trx, one Rx connector
    - Digitally controlled Trx-Rx tie switch to switch between full-duplex and half-duplex mode
  - EMI protection circuit
  - Digitally controlled termination resistors (120R, 220R, 13R, or open)
  - MCP2518FD CAN-FD transceiver
  - Differential receiver circuit to decode arbitrary differential signals
  - Differential injector circuit to encode arbitrary differential signals
    - Operates up to 5 MHz
    - Digitally switched voltage references (0V, 1V, 1V5, 2V, 2V5, 3V, 3V5, 4V)
    - Digitally switched high impedance mode
- Licensed under the CERN-OHL-W

#### LED Behavior
Along with the 9 NeoPixels, we have 6 additional single-color LEDs.

- Battery: to the left of the Pico 2, there is a battery charge status indicator LED. This will be solid amber when charging, off when discharging, and oscillating when the battery is unplugged (we are aware this blinks like crazy)
- Differential Injector: on the Pico 2, by the USB connector, there is a green LED that will blink when the differential injector circuit is actively transmitting
- CAN Tx/Rx: by the right lanyard strap, there are two LEDs indicating whether the CAN SPI transceiver is transmitting or receiving frames. Note that this will *not* blink when using the differential injector/receiver circuit directly, but rather the CAN SPI transceiver.
- Differential Receiver: to the right of the CAN Tx/Rx LEDs, there is an LED indicating the presence of data on the differential receiver circuit.

### Firmware
The firmware contains the following:

- Available Rust source code under the Apache 2.0 & MIT licenses
- Cooperative scheduler using the embassy-rs runtime
- Scrolling console display drivers for log output
- USB serial device pass-through for log output
- Embedded scripting engine using Rhai (rhai.rs)
  - USB serial device pass-through for embedded scripting engine
  - Script bindings to inputs, peripherals, and the Trx/Rx circuit
- Some secrets :^)

#### TODOs
The firmware is not complete yet. We'll be updating this repository as we have time to expand firmware capability. We are currently absent an implementation for the following:

- SD card FAT32 file system for packet captures and replay files
- Button-controlled GUI
- More/better protocol state machines
  - Missing Modbus Rx
- MCP2518FD drivers
- Better I2C/SPI pass-through
- Better docs (like this one)

## Scripting Engine
We chose Rhai for our scripting language for its seamless integration into our Rust firmware. Some functions require switching from the firmware's default application context to user control.

Let's enumerate the full set:

| Function | Args | Returns | Description | Requires Control? |
|--|--|--|--|--|
| `sys::assume_control` | `()` | `()` | Takes control of app context | false |
| `sys::release_control` | `()` | `()` | Releases control of app context | true |
| `sys::random` | `(bytes: INT)` | `Blob` | Samples the TRNG | false |
| `sys::time` | `()` | `FLOAT` | Samples RTC | false |
| `sys::ticks` | `()` | `INT` | Samples clock counter | false |
| `sys::sleep` | `(secs: FLOAT)` | `()` | Suspends script execution | false |
| `sys::heap` | `()` | `Array` | Probes the heap [used, free] | false |
| `input::read` | `()` | `Dynamic` | Polls user input (joystick, buttons) | false |
| `sao::get_direction` | `()` | `Array` | Returns SAO GPIO pin direction [pin 1, pin 2] | false |
| `sao::set_direction` | `(dir_1: bool, dir_2: bool)` | `()` | Sets SAO GPIO pin direction | false |
| `sao::read` | `()` | `Array` | Reads SAO GPIO | false |
| `sao::write` | `(out_1: bool, out_2: bool)` | `()` | Writes to SAO GPIO | false |
| `display::set_backlight` | `(percent: INT | float)` | `()` | Sets display backlight percentage | false |
| `display::set_pixel` | `(x: INT, y: INT, r: INT, g: INT, b: INT)` | `()` | Sets a pixel | true |
| `display::fill_region` | `(sx: INT, ex: INT, sy: INT, ey: INT, r: INT, g: INT, b: INT)` | `()` | Fills in a rectangle | true |
| `display::clear` | `()` | `()` | Clears the display | true |
| `display::flush` | `()` | `()` | Flushes the display | true |
| `display::reset` | `()` | `()` | Resets the display | false |
| `console::write` | `(text: &str \| Array)` | `()` | Writes to display as a console; supports ANSI escape codes | true |
| `led::set` | `(i: INT, r: INT, g: INT, b: INT)` | `()` | Sets the given LED | true |
| `accel::read` | `()` | `Dynamic` | Polls the accelerometer | false |
| `accel::set_reg8` | `(reg: INT, value: INT)` | `()` | Sets an 8-bit register | true |
| `accel::set_reg16` | `(reg: INT, value: INT)` | `()` | Sets a 16-bit register | true |
| `accel::set_int_enable` | `(tilt: bool, flip: bool, anym: bool, shake: bool, tilt35: bool, auto_clr: bool, acq: bool)` | `()` | Enables desired accelerometer interrupts | true |
| `accel::set_mode` | `(mode: INT, i2c_wdt_neg: bool, i2c_wdt_pos)` | `()` | Sets mode (sleep, standby, wake) | true |
| `accel::set_sample_rate` | `(rate: INT)` | `()` | Sets sample rate | true |
| `accel::set_motion_control` | `(reset: bool, raw_proc_stat: bool, z_axis_ort: bool, tilt35_en: bool, shake_en: bool, anym: bool, motion_latch: bool, tiltflip: bool)` | `()` | Enables motion capture | true |
| `accel::set_anym_threshold` | `(threshold: INT)` | `()` | Any motion threshold | true |
| `accel::set_anym_debounce` | `(debounce: INT)` | `()` | Any motion debounce | true | true |
| `accel::set_shake_threshold` | `(threshold: INT)` | `()` | Shake threshold | true |
| `accel::set_shake_duration` | `(cnt: INT, p2p: INT)` | `()` | Shake duration | true |
| `batt::status` | `()` | `Dynamic` | Reads the battery status registers | false |
| `trx::get_term` | `()` | `Array` | Get the termination switch values | true |
| `trx::set_term` | `(sel_0: bool, sel_1: bool)` | `()` | Set termintation resistor value | true |
| `trx::get_tie` | `()` | `bool` | Is the tie enabled? | true |
| `trx::set_tie` | `(tied: bool)` | `()` | Set the Trx-Rx tie | true |
| `tx::is_enabled` | `()` | `bool` | Is Tx enabled? | true |
| `tx::enable` | `()` | `()` | Enable Tx | true |
| `tx::disable` | `()` | `()` | Disable Tx | true |
| `tx::get_baud` | `()` | `INT` | Get Tx baud | true |
| `tx::set_baud` | `(baud: INT)` | `()` | Set Tx baud | true |
| `tx::set_baud` | `(baud: INT)` | `()` | Set the operating frequency of the Tx module | true |
| `tx::get_mode` | `()` | `ImmutableString` | Gets the current mode | false |
| `tx::set_mode` | `(mode: &str)` | `()` | Sets the operating mode ("can" or "inject") | true |
| `tx::send` | `(data: Blob)` | `()` | Sends the bytestream; behavior dependent on mode | true |
| `rx::is_enabled` | `()` | `bool` | Is Rx enabled? | true |
| `rx::get_baud` | `()` | `INT` | Get Rx baud | true |
| `rx::set_baud` | `(baud: INT)` | `()` | Set Rx baud | true |
| `rx::enable` | `()` | `()` | Enable Rx | true |
| `rx::disable` | `()` | `()` | Disable Rx | true |
| `rx::set_mode` | `(mode: &str)` | `()` | Sets the operating mode ("can" or "nmea0183") | true |
| `rx::get_mode` | `()` | `ImmutableString` | Gets the current mode | false |
| `rx::recv` | `(timeout_secs: FLOAT)` | `Blob` | Waits for a message to be received and returns the bytestream | true |
| `can::encode` | `(arb_id: INT, rtr: bool, payload: Blob)` | `Blob` | Encodes a CAN 2.0B message for use by the "can" Tx operating mode | false |

### Constants
We also expose some constants for ease-of-use:

| Const | Value | Description |
|--|--|--|
| `trx::TERM_OPEN_0` | `false` | H/L open |
| `trx::TERM_OPEN_1` | `false` | H/L open |
| `trx::TERM_120R_0` | `true` | 120R termination (CAN/NMEA-2000) |
| `trx::TERM_120R_1` | `false` | 120R termination (CAN/NMEA-2000) |
| `trx::TERM_220R_0` | `false` | 220R termination (Modbus) |
| `trx::TERM_220R_1` | `true` | 220R termination (Modbus) |
| `trx::TERM_13R_0` | `true` | 13R termination *(close to shorted)* |
| `trx::TERM_13R_1` | `true` | 13R termination *(close to shorted)* |
| `tx::L_Z0` | `0b000_0_000_1` | Set high-impedance mode on L |
| `tx::L_0V` | `0b000_0_000_0` | Set L to 0V |
| `tx::L_1V` | `0b000_0_100_0` | Set L to 1V |
| `tx::L_1V5` | `0b000_0_010_0` | Set L to 1.5V |
| `tx::L_2V` | `0b000_0_110_0` | Set L to 2V |
| `tx::L_2V5` | `0b000_0_001_0` | Set L to 2.5V |
| `tx::L_3V` | `0b000_0_101_0` | Set L to 3V |
| `tx::L_3V5` | `0b000_0_011_0` | Set L to 3.5V |
| `tx::L_4V` | `0b000_0_111_0` | Set L to 4V |
| `tx::H_Z0` | `0b000_1_000_0` | Set high-impedance mode on H |
| `tx::H_0V` | `0b000_0_000_0` | Set H to 0V |
| `tx::H_1V` | `0b100_0_000_0` | Set H to 1V |
| `tx::H_1V5` | `0b010_0_000_0` | Set H to 1.5V |
| `tx::H_2V` | `0b110_0_000_0` | Set H to 2V |
| `tx::H_2V5` | `0b001_0_000_0` | Set H to 2.5V |
| `tx::H_3V` | `0b101_0_000_0` | Set H to 3V |
| `tx::H_3V5` | `0b011_0_000_0` | Set H to 3.5V |
| `tx::H_4V` | `0b111_0_000_0` | Set H to 4V |
| `tx::LOW_Z` | `0b111_0_111_0` | Bitmask to turn off high-impedance mode |

### Caveats
The heap is pretty small on the stock Pico 2, and we still need to make a few optimization passes to reduce the firmware's memory footprint, so you'll likely run into memory problems with sufficienty complex Rhai scripts. Please approach village staff with any debugging -- we appreciate the feedback.

### Guided Tour
So to get access to the scripting engine, first plug your Pico 2 into your machine over USB and connect to one of the serial ports that comes up (the other is the log output):

```
picocom --imap lfcrlf --echo /dev/ttyACM2
```

You should be greeted by a splash screen welcoming you to the REPL.

Now, let's take control of the application context:

```
sys::assume_control()
```

You can release control back to the firmware with (or Ctrl-D):

```
sys::release_control()
```

Great. Let's see what we can do.

#### LEDs
Let's write a simple loop to blink our LEDs:
```
for i in 0..10 { for j in 0..9 { led::set(j, i * 25, 127, j * 25) }; sys::sleep(0.5); }
```

#### CAN
```
let data = sys::random(8);
let frame = can::encode(0x8AA, false, data);
tx::set_mode("can")
tx::set_baud(250_000)
tx::enable()
tx::send(frame)
```

```
// Half-duplex mode
trx::set_tie(true)
// 120R termination resistor for CAN
trx::set_term(trx::TERM_120R_0, trx::TERM_120R_1)
rx::set_mode("can")
rx::set_baud(250_000)
rx::enable()
let frame = rx::recv(5.0)
print(frame)
```

Note that you can separate multi-line strings in the REPL with `\`.

#### Differential Injector
```
let data = blob();
data.push(tx::H_4V | rx::L_1V);
data.push(tx::H_3V5 | rx::L_1V5);
data.push(tx::H_3V | rx::L_2V);
data.push(tx::H_2V5 | rx::L_2V5);
data.push(tx::H_2V | rx::L_3V);
data.push(tx::H_1V5 | rx::L_3V5);
data.push(tx::H_1V | rx::L_4V);
data.push(tx::H_1V5 | rx::L_3V5);
data.push(tx::H_2V | rx::L_3V);
data.push(tx::H_2V5 | rx::L_2V5);
data.push(tx::H_3V | rx::L_2V);
data.push(tx::H_3V5 | rx::L_1V5);
data.push(tx::H_4V | rx::L_1V);
tx::set_mode("inject");
tx::set_baud(1_000_000);
tx::enable();
tx::send(data);
```

```
let flag = blob()
// SOF
flag.push(tx::H_1V | tx::L_1V); flag.push(tx::H_1V | tx::L_1V); flag.push(tx::H_1V | tx::L_1V);
// M
flag.push(tx::H_4V | tx::L_1V); flag.push(tx::H_3V5 | tx::L_2V5); flag.push(tx::H_3V | tx::L_2V); flag.push(tx::H_3V5 | tx::L_2V5); flag.push(tx::H_4V | tx::L_1V);
// IFS
flag.push(tx::H_1V | tx::L_1V); flag.push(tx::H_1V | tx::L_1V);
// H
flag.push(tx::H_4V | tx::L_1V); flag.push(tx::H_2V5 | tx::L_2V); flag.push(tx::H_2V5 | tx::L_2V); flag.push(tx::H_4V | tx::L_1V);
// IFS
flag.push(tx::H_1V | tx::L_1V); flag.push(tx::H_1V | tx::L_1V);
// V
flag.push(tx::H_4V | tx::L_1V); flag.push(tx::H_2V | tx::L_1V); flag.push(tx::H_1V | tx::L_1V); flag.push(tx::H_2V | tx::L_1V); flag.push(tx::H_4V | tx::L_1V);
// IFS
flag.push(tx::H_1V | tx::L_1V); flag.push(tx::H_1V | tx::L_1V); flag.push(tx::H_1V | tx::L_1V);
tx::set_mode("inject");
tx::set_baud(1_000_000);
tx::enable();
tx::send(flag);
```

#### Accelerometer
```
// Shake up the board to see measurements.
for i in 0..50 { print(accel::read()); sys::sleep(0.1); }
```

#### Input
```
for i in 0..50 { print(input::read()); sys::sleep(0.1); }
```

### Display/Console
```
display::clear();
display::flush();
console::write("\x1B[32Hack the planet!");
display::fill_region(50, 100, 50, 100, 127, 0, 127);
display::flush();
for i in 0..20 { for j in 0..20 { display::set_pixel(100 + i, 100 + j, i*10, j*10, 64); }}
display::flush();
```
