mod transform;

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::time::Instant;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use esp_idf_svc::hal::gpio::{InterruptType, Pull};
use mousefood::prelude::*;
use mousefood::ratatui::layout::Offset;
use pitchy::Note;

use embedded_hal::spi::MODE_3;

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
    gpio::{AnyIOPin, PinDriver},
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig},
    units::*,
};
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::{interface::SpiInterface, models::ST7789, Builder};
use tui_big_text::PixelSize;
use tuitar::transform::Transformer;

use transform::Transform;
use tuitar::ui::*;

const DISPLAY_OFFSET: (u16, u16) = (52, 40);
const DISPLAY_SIZE: (u16, u16) = (135, 240);

fn main() -> Result<(), Box<dyn Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // Turn on display backlight
    let mut backlight = PinDriver::output(peripherals.pins.gpio4)?;
    backlight.set_high()?;

    // Configure SPI
    let config = SpiConfig::new()
        .write_only(true)
        .baudrate(80u32.MHz().into())
        .data_mode(MODE_3);
    let spi_device = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio19,
        Option::<AnyIOPin>::None,
        Some(peripherals.pins.gpio5),
        &SpiDriverConfig::new(),
        &config,
    )?;
    let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0u8; 512]));
    let spi_interface = SpiInterface::new(
        spi_device,
        PinDriver::output(peripherals.pins.gpio16)?,
        buffer,
    );

    let mut button = PinDriver::input(peripherals.pins.gpio0).unwrap();
    button.set_interrupt_type(InterruptType::NegEdge).unwrap();
    button.set_pull(Pull::Up).unwrap();

    // Configure display
    let mut delay = Ets;
    let mut display = Builder::new(ST7789, spi_interface)
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(PinDriver::output(peripherals.pins.gpio23)?)
        .display_offset(DISPLAY_OFFSET.0, DISPLAY_OFFSET.1)
        .display_size(DISPLAY_SIZE.0, DISPLAY_SIZE.1)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut delay)
        .expect("Failed to init display");

    // Reset pixels
    display
        .clear(Rgb565::BLACK)
        .expect("Failed to clear display");

    let adc_driver = AdcDriver::new(peripherals.adc1).unwrap();
    let mut mic_adc_channel = AdcChannelDriver::new(
        &adc_driver,
        peripherals.pins.gpio36,
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

    loop {
        let instant = Instant::now();
        while samples.len() < buffer_size {
            let raw = mic_adc_channel.read().unwrap_or(0);
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

        terminal
            .draw(|frame| {
                match mode {
                    0 => draw_waveform(frame, &samples, sample_rate, (512., 1536.)),
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
                        draw_note(frame, most_frequent_note, PixelSize::Quadrant, 2, false);
                    }
                }
            })
            .unwrap();

        if button.is_low() {
            Ets::delay_ms(10);
            mode = (mode + 1) % 4;
        }

        samples.clear();
    }
}
