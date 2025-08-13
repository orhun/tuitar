# Tuitar Firmware

This directory contains the firmware that runs on [ESP-WROOM-32D], the main microcontroller of Tuitar.

🦀 _Powered with Rust & built with [Ratatui] via [Mousefood] backend_ 🧀

> [!NOTE]
> We use the ESP-IDF which is the official development framework for ESP32 devices. This means that the standard library of Rust is accessible throughout the codebase.
>
> For more information, see [ESP-IDF Rust Book].

<!-- vim-markdown-toc GFM -->

- [Setting up the toolchain](#setting-up-the-toolchain)
- [Flashing](#flashing)
- [Monitoring](#monitoring)

<!-- vim-markdown-toc -->

## Setting up the toolchain

Assuming you already have Rust installed (via [rustup](https://rustup.rs)), you need the `xtensa-esp32-espidf` target to be able to build the firmware. You can set it up by following the [instructions here], or simply:

1. Install `espup`

```sh
cargo install espup --locked
```

2. Install the toolchain

```sh
espup install
```

3. Set up the [environment variables].

Or you can export this file on Linux when you want to build the firmware:

```sh
. ~/export-esp.sh
```

Now you can stick to your typical Rust development workflow (i.e. `cargo build`, `cargo check` commands) and simply run `cargo run` to flash the firmware to the device.

> [!TIP]
> If you get compilation errors, make sure that `cmake` is installed and run `cargo` with `-vv` flag to get a more verbose output.

## Flashing

Plug the ESP-WROOM-32D to your computer via USB.

Make sure that you have the [`espflash`] command-line tool installed. If not:

```sh
cargo install espflash --locked
```

Then simply run:

```sh
cargo run --release
```

Enjoy!

## Monitoring

There are a lot of serial monitoring tools available, but you can use something like [`comchan`] because it's configurable and auto-detect the port and it's cool.

[ESP-WROOM-32D]: https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32d_esp32-wroom-32u_datasheet_en.pdf
[Ratatui]: https://ratatui.rs
[mousefood]: https://github.com/j-g00da/mousefood
[ESP-IDF Rust Book]: https://docs.espressif.com/projects/rust/book/overview/using-the-standard-library.html#current-support
[instructions here]: https://docs.espressif.com/projects/rust/book/installation/riscv-and-xtensa.html
[environment variables]: https://docs.espressif.com/projects/rust/book/installation/riscv-and-xtensa.html#3-set-up-the-environment-variables
[`espflash`]: https://crates.io/crates/espflash
[`comchan`]: https://github.com/Vaishnav-Sabari-Girish/ComChan
