## Architecture

The codebase consists of the following crates:

- [`tuitar-core`](./tuitar-core/README.md): The core logic and UI of **Tuitar**.
- [`firmware`](./firmware/README.md): The firmware for the ESP32 hardware.
- [`hardware`](./hardware/README.md): The hardware design files for the **Tuitar** kit.
- [`tuitar`](./tuitar/README.md): The terminal application for **Tuitar**.
- [`ratatui-fretboard`](./ratatui-fretboard/README.md): A crate for rendering fretboards in terminal applications using Ratatui.

The dependency relationship is as follows:

```
tuitar-core
 └── ratatui-fretboard

firmware
 ├── tuitar-core
 └── ratatui-fretboard

tuitar
 ├── tuitar-core
 └── ratatui-fretboard

hardware (no code deps)
```
