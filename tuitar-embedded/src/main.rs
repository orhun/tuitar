mod transform;

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::time::Instant;

use esp_idf_svc::hal::gpio::{InterruptType, Pull};
use esp_idf_svc::hal::{
    adc::{
        attenuation::DB_11,
        oneshot::{
            config::{AdcChannelConfig, Calibration},
            AdcChannelDriver, AdcDriver,
        },
        Resolution,
    },
    delay::Ets,
    delay::FreeRtos,
    gpio::PinDriver,
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver},
    units::*,
};
use mousefood::prelude::*;
use mousefood::ratatui::layout::Offset;
use pitchy::Note;
use st7735_lcd::{Orientation, ST7735};
use tui_big_text::PixelSize;
use tuitar::transform::Transformer;

use transform::Transform;
use tuitar::ui::*;

fn main() -> Result<(), Box<dyn Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let spi = peripherals.spi2;
    // HSPI SCK — default SPI2 clock pin, safe at boot
    let sclk = peripherals.pins.gpio14;
    // HSPI MOSI — default SPI2 data pin, safe at boot
    let sdo = peripherals.pins.gpio13;
    // MISO not used for display
    let sdi = Option::<esp_idf_svc::hal::gpio::Gpio0>::None;
    // CS — general-purpose pin, safe at boot
    let cs = Some(peripherals.pins.gpio25);
    // RESET — unused GPIO, safe and stable
    let rst = PinDriver::output(peripherals.pins.gpio33)?;
    // DC (AO) — general-purpose pin, safe at boot
    let dc = PinDriver::output(peripherals.pins.gpio27)?;

    let driver_config = Default::default();
    let spi_config = SpiConfig::new().baudrate(40u32.MHz().into());

    let spi = SpiDeviceDriver::new_single(spi, sclk, sdo, sdi, cs, &driver_config, &spi_config)?;

    let rgb = true;
    let inverted = false;
    let width = 160;
    let height = 128;

    let mut delay = FreeRtos;
    let mut display = ST7735::new(spi, dc, rst, rgb, inverted, width, height);

    display.init(&mut delay).unwrap();
    display
        .set_orientation(&Orientation::LandscapeSwapped)
        .unwrap();

    let mut button1 = PinDriver::input(peripherals.pins.gpio22).unwrap();
    button1.set_interrupt_type(InterruptType::NegEdge).unwrap();
    button1.set_pull(Pull::Up).unwrap();

    let mut button2 = PinDriver::input(peripherals.pins.gpio21).unwrap();
    button2.set_interrupt_type(InterruptType::NegEdge).unwrap();
    button2.set_pull(Pull::Up).unwrap();

    let adc1_driver = AdcDriver::new(peripherals.adc1).unwrap();
    let mut jack_adc_channel = AdcChannelDriver::new(
        &adc1_driver,
        peripherals.pins.gpio36,
        &AdcChannelConfig {
            attenuation: DB_11,
            calibration: Calibration::Line,
            // Sample unsigned 12-bit integers (0-4095)
            resolution: Resolution::Resolution12Bit,
        },
    )
    .unwrap();

    let mut mic_adc_channel = AdcChannelDriver::new(
        &adc1_driver,
        peripherals.pins.gpio32,
        &AdcChannelConfig {
            attenuation: DB_11,
            calibration: Calibration::Line,
            // Sample unsigned 12-bit integers (0-4095)
            resolution: Resolution::Resolution12Bit,
        },
    )
    .unwrap();

    let mut pot = AdcChannelDriver::new(
        &adc1_driver,
        peripherals.pins.gpio39,
        &AdcChannelConfig {
            attenuation: DB_11,
            calibration: Calibration::Line,
            // Sample unsigned 12-bit integers (0-4095)
            resolution: Resolution::Resolution12Bit,
        },
    )
    .unwrap();

    let buffer_size = 1024;
    let mut samples = Vec::with_capacity(buffer_size);
    let mut mode = 0;

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend)?;

    let mut note_history: VecDeque<f64> = VecDeque::new();
    let max_history = 2;
    let mut transform = Transform::new();

    let mut input_mode = 0;

    loop {
        let instant = Instant::now();
        while samples.len() < buffer_size {
            let raw = match input_mode {
                0 => mic_adc_channel.read().unwrap_or(0),
                1 => jack_adc_channel.read().unwrap_or(0),
                _ => mic_adc_channel.read().unwrap_or(0),
            };
            samples.push(raw as i16);
        }

        let elapsed = instant.elapsed();

        transform.process(&samples);

        let sample_rate = samples.len() as f64 / elapsed.as_secs_f64();

        let fundamental_freq = transform.find_fundamental_frequency(sample_rate);
        if Note::new(fundamental_freq).name().is_some() {
            note_history.push_back(fundamental_freq);
            if note_history.len() > max_history {
                note_history.pop_front();
            }
        }

        log::info!(
            "Sampled {} samples at {:.2} Hz | Fundamental frequency = {:.2} Hz",
            samples.len(),
            sample_rate,
            fundamental_freq
        );

        let pot_val = pot.read().unwrap_or(1000) as f64;

        terminal
            .draw(|frame| {
                match mode {
                    0 => draw_waveform(frame, &samples, sample_rate, (pot_val, pot_val + 500.)),
                    1 => draw_frequency(frame, &transform, sample_rate),
                    2 => draw_frequency_chart(frame, &transform, buffer_size as f64),
                    3 => draw_fretboard(
                        frame,
                        fundamental_freq,
                        frame.area().offset(Offset { x: 0, y: 3 }),
                        6,
                    ),
                    _ => {}
                }
                let most_frequent_note = note_history
                    .iter()
                    .map(|f| *f as i32)
                    .fold(HashMap::new(), |mut acc, freq| {
                        *acc.entry(freq).or_insert(0) += 1;
                        acc
                    })
                    .into_iter()
                    .max_by_key(|&(_, count)| count)
                    .map(|(freq_hz, _)| freq_hz as f64);
                if let Some(most_frequent_note) = most_frequent_note {
                    if most_frequent_note > 70.0 && most_frequent_note < 3000.0 {
                        let pixel_size = (mode != 3).then_some(PixelSize::Quadrant);
                        draw_note(frame, most_frequent_note, pixel_size, 2, false);
                    }
                }
                let input_mode_letter = match input_mode {
                    0 => "M",
                    1 => "J",
                    _ => "?",
                };
                frame.render_widget(
                    input_mode_letter,
                    Rect::new(
                        frame.area().right().saturating_sub(1),
                        frame.area().top().saturating_sub(1),
                        1,
                        1,
                    ),
                );
            })
            .unwrap();

        if button1.is_low() {
            Ets::delay_ms(10);
            mode = (mode + 1) % 4;
        }

        if button2.is_low() {
            Ets::delay_ms(10);
            input_mode = (input_mode + 1) % 2;
        }

        samples.clear();
    }
}
