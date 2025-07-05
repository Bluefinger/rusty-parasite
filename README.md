# Rusty Parasite

[`b-parasite`](https://github.com/rbaron/b-parasite), the open source soil moisture and ambient temperature/humidity/light sensor, but in Rust!

## Notes

Currently, this project has been tested only with the v2.0 of the sensor, making use of the NRF52840 chip. Other variations have not been used, though this could be added by others should they wish. Also, soil moisture calculations have been calibrated against boards that have had conformal coating applied to the capacitive sensor part of the board and further tweaking my still happen, so YMMV.

## How to install / Flash to the board

To compile, [you'll the latest rustc compiler](https://www.rust-lang.org/tools/install), `clang` 9.0 or later C compiler (due to C deps in the codebase), and a SWD debug probe to flash with (A Pico Debug Probe flashed with yapico firmware will work). Optional but highly recommended tool for flashing this board is a pogo pin clamp (6 pin version). These can be found cheap on AliExpress.

Once rust is installed, you'll need to use `rustup` to download the `nightly` toolchain with the `thumbv7em-none-eabihf` target:

```
rustup toolchain install nightly
rustup +nightly target install thumbv7em-none-eabihf
```

Additionally, there's also at least one utility needed for compilation, `flip-link`. To install, run:

```
cargo install flip-link
```

For flashing, you'll need `probe-rs`, so be sure to [follow the install instructions here](https://probe.rs/docs/getting-started/installation/). There will be [further setup required after installation](https://probe.rs/docs/getting-started/probe-setup/).

This should be enough to get you all the necessary tools in place for compiling and flashing your b-parasite board. To check everything compiles correctly, `cd` into the `para-firmware` folder and run the following:

```
cargo build
```

This should build the firmware in debug mode and with `defmt` logging in place. To flash and log the debug build, ensure your debug probe is connected to the machine and the board and simply run the command:

```
cargo run
```

This will run the compiler and then execute `probe-rs` to flash the chip with the compiled binary. All of the debug info/symbols are kept on the machine and are not flashed, only the actual executable will be flashed. This allows for very efficient debug logging via `defmt`, which decodes the format using the debug symbols.

To compile and flash without debugging/logging, run:

```
cargo run --release --no-default-features
```

This will ensure all debugging/logging code is not generated, nor any debug symbols. Once `probe-rs` flashes the device, you should see no logging output once the chip begins to run.

## Licenses

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option and for maximum compatibility with the Rust Ecosystem.
