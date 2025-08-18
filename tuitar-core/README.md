# Tuitar Core

The core logic and UI of **Tuitar**.

With using this library you can implement your own Tuitar-like applications. The current implementations are:

- [`tuitar`](https://github.com/orhun/tuitar/tree/main/tuitar): The terminal application for **Tuitar**.
- [`tuitar-firmware`](https://github.com/orhun/tuitar/tree/main/firmware): The ESP32 firmware for **Tuitar**.

The library is currently capable of:

- Draw the UI using [Ratatui](https://ratatui.rs) and track FPS via `FpsWidget`
- Process raw audio samples and apply FFT with the provided backend (`impl Tranformer`)
- Track application state and provide methods suchs as pitch detection (`State`)
- Load songs as MIDI or Guitar Pro format (see the `songs` module)

See the [main documentation](https://github.com/orhun/tuitar) for more information.
