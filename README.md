# TrustZone-M on the Raspberry Pi Pico 2 in Rust

Quickly get started with [TrustZone-M](
    https://www.arm.com/technologies/trustzone-for-cortex-m)
on the [Raspberry Pi Pico 2](
    https://www.raspberrypi.com/products/raspberry-pi-pico-2/) in Rust.
Inspiration was taken from [`IoTS-P/trustzone-m-rs`](
    https://github.com/IoTS-P/trustzone-m-rs) which was adapted
to work on the Pi Pico 2 and made to make use of modern Rust TrustZone-M syntax.

## Project lay-out

TrustZone-M divides the chip into two worlds:
the "secure world" (in [`app-s/`](./app-s/))
and the "non-secure world" (in [`app-ns/`](./app-ns/)).
These are separate binaries and flashed onto the chip separately, too.
To help in this, [`xtask/`](./xtask/) provides some helper functions for this.

## Prerequisites

To work with this repository, install the following tools:
- `rustc` and `cargo`, preferably installed via `rustup`, to compile the Rust code
- an ARM cross-compiler, e.g. `arm-none-eabi-gcc`, to link ARM binaries
- `probe-rs` to simplify flashing the code

## How to run

To quickly run this code, power up your Raspberry Pi Pico 2,
attach the [Raspberry Pi Debug Probe](
    https://www.raspberrypi.com/documentation/microcontrollers/debug-probe.html),
and run
```sh
cargo xtask run --release
```
to build, flash, and run the code.
You will see the semihosted `hprint!()` and `hprintln!()` output in your `stdout`.

## Debugging

Because we are dealing with two separate binaries, debugging the code is not the easiest.
We basically need to choose whether we want to debug the secure or the non-secure world.
The process is as follows:
- flash **secure** binary
- flash and run **non-secure** binary and attach debugger to it

OR:
- flash **non-secure** binary
- flash and run **secure** binary and attach debugger to it

If you use `vscode`, there are some helper tasks in [`.vscode/launch.json`](
    .vscode/launch.json) to simplify the order of doing things.
