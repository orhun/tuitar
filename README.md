<p align="center">
    <img src="https://raw.githubusercontent.com/orhun/tuitar/refs/heads/main/assets/tuitar-logo-dark.png#gh-dark-mode-only" width="400"></a>
    <img src="https://raw.githubusercontent.com/orhun/tuitar/refs/heads/main/assets/tuitar-logo-light.png#gh-light-mode-only" width="400"></a>
    <br>
    <b>"Learning how to play guitar with a TUI - hence <em>Tuitar</em>."</b>
    <br>
    <br>
    <a href="https://github.com/orhun/tuitar/releases">
        <img src="https://img.shields.io/github/v/release/orhun/tuitar?color=000000">
    </a>
    <a href="https://crates.io/crates/tuitar/">
        <img src="https://img.shields.io/crates/v/tuitar?color=000000">
    </a>
    <a href="https://github.com/orhun/tuitar/actions?query=workflow%3A%22Continuous+Integration%22">
        <img src="https://img.shields.io/github/actions/workflow/status/orhun/tuitar/ci.yml?branch=master&color=000000&label=CI">
    </a>
    <a href="https://github.com/orhun/tuitar/blob/master/LICENSE">
        <img src="https://img.shields.io/crates/l/tuitar?color=000000">
    </a>
</p>

---

**Tuitar** is a terminal-based guitar training tool that also lives in your pocket.

It currently supports real-time visualization of:

- **Pitch** – Perfect for tuning and identifying notes.
- **Waveform, frequency & gain** – Watch the sound as it moves.
- **Fretboard** – See detected notes mapped directly onto the guitar neck for instant feedback.

Also runs standalone on ESP32 hardware.

> [!NOTE]
> Building all of this [on livestream](https://www.youtube.com/@orhundev/streams).

> [!WARNING]
> This is a work in progress.

## Demo

With microphone input:

https://github.com/user-attachments/assets/1922a316-57ff-4f3d-92eb-5ba5ff0dfdd8

With jack input:

https://github.com/user-attachments/assets/cdbdc811-790d-4dac-8dc4-51d49589d3c0

## BOM

- ESP-WROOM-32 Devkit
- TFT 1.8 ST7735 128X160 Display
- MP1584 Buck Converter
- MAX4466
- LM358P
- 1 x On/off switch
- 2 x buttons
- 2 x 1M pot
- 3 x 1K resistor
- 2 x 10K resistor
- 1 x LED
- 1 x 1N5819 diode
- 1 x 1uF capacitor
- Mono jack
- A kickass guitar
