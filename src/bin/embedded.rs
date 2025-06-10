use std::thread::{self, ScopedJoinHandle};
use std::time::Instant;
use std::{error::Error, thread::Scope};

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use esp_idf_svc::hal::gpio::ADCPin;
use mousefood::prelude::*;

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
    peripheral::Peripheral,
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig},
    units::*,
};
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::{interface::SpiInterface, models::ST7789, Builder};
use pitchy::Note;
use tui_big_text::PixelSize;

use std::sync::mpsc;
use tuitar::transform::Transform;
use tuitar::ui::*;

pub fn spawn_reader_thread<'scope, T>(
    scope: &'scope Scope<'scope, '_>,
    adc: impl Peripheral<P = T::Adc> + 'scope + Send,
    battery_pin: impl Peripheral<P = T> + 'scope + Send,
    sender: mpsc::Sender<Vec<i16>>,
    sample_rate: f64,
) -> Result<ScopedJoinHandle<'scope, ()>, std::io::Error>
where
    T: ADCPin,
{
    thread::Builder::new()
        .stack_size(8192)
        .spawn_scoped(scope, move || {
            let adc_driver = AdcDriver::new(adc).unwrap();
            let mut adc_channel = AdcChannelDriver::new(
                &adc_driver,
                battery_pin,
                &AdcChannelConfig {
                    attenuation: DB_11,
                    calibration: Calibration::Line,
                    resolution: Resolution::Resolution12Bit,
                },
            )
            .unwrap();

            let interval = std::time::Duration::from_secs_f64(1.0 / sample_rate);

            loop {
                let mut samples = Vec::with_capacity(sample_rate as usize);

                for _ in 0..(sample_rate as usize) {
                    let raw = adc_channel.read().unwrap_or(0) as i32;
                    samples.push(raw as i16);
                    thread::sleep(interval);
                }

                if sender.send(samples).is_err() {
                    break;
                }
            }
        })
}

const DISPLAY_OFFSET: (u16, u16) = (52, 40);
const DISPLAY_SIZE: (u16, u16) = (135, 240);

fn main() -> Result<(), Box<dyn Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

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
            calibration: Calibration::None,
            // Sample unsigned 12-bit integers (0-4095)
            resolution: Resolution::Resolution12Bit,
        },
    )
    .unwrap();

    // let (tx, rx) = mpsc::channel::<Vec<i16>>();
    // read_mic_input(tx, &peripherals);
    // if let Ok(v) = rx.try_recv() {
    //     samples = v;
    // }
    //
    //

    let samples_needed = 512;
    let mut samples = Vec::with_capacity(samples_needed);
    let mode = 0;

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend)?;

    let mut transform = Transform::new();

    loop {
        let instant = Instant::now();
        while samples.len() < samples_needed {
            let raw = mic_adc_channel.read().unwrap_or(0);
            samples.push(raw as i16);
        }

        let elapsed = instant.elapsed();

        transform.process(&samples);

        let sample_rate = samples.len() as f64 / elapsed.as_secs_f64();
        let freq_nyquist = sample_rate / 2.0;

        let magnitudes = transform.normalized_fft_data();

        let (max_index, _) = magnitudes
            .iter()
            .enumerate()
            .skip(1) // skip DC bin
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        let bin_width = freq_nyquist / magnitudes.len() as f64;
        let fundamental_freq = max_index as f64 * bin_width;

        println!(
            "Sampled {} samples at {:.2} Hz | Nyquist: {:.2} Hz | Bin width: {:.2} Hz | Fundamental frequency = {:.2} Hz",
            samples.len(),
            sample_rate,
            freq_nyquist,
          bin_width,
            fundamental_freq
        );
        let note = Note::new(fundamental_freq);

        terminal
            .draw(|frame| {
                match mode {
                    0 => draw_waveform(frame, &samples, sample_rate, (0., 2048.0)),
                    1 => draw_frequency(frame, &transform),
                    2 => draw_frequency_chart(frame, &transform, samples_needed as f64),
                    _ => {}
                }

                draw_note(frame, &note, PixelSize::Quadrant);
            })
            .unwrap();

        samples.clear();
    }
}
