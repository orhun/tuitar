use std::error::Error;
use std::thread;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use mousefood::prelude::*;

use embedded_hal::spi::MODE_3;

use esp_idf_svc::hal::{
    delay::Ets,
    gpio::{AnyIOPin, PinDriver},
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig},
    units::*,
};
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::{interface::SpiInterface, models::ST7789, Builder};

use std::sync::mpsc;
use tuitar::transform::Transform;
use tuitar::ui::*;

fn mock_data(tx: mpsc::Sender<Vec<i16>>) {
    let builder = thread::Builder::new()
        .name("mock_data_thread".into())
        .stack_size(10 * 1024); // 64KB stack

    builder
        .spawn(move || {
            let mut samples = vec![0i16; 200];

            let mut i = 0;

            loop {
                for sample in &mut samples {
                    *sample = (i % 32768) as i16;
                    i += 1;
                }

                if tx.send(samples.clone()).is_err() {
                    break;
                }

                thread::sleep(std::time::Duration::from_millis(100));
            }
        })
        .expect("Failed to spawn mock data thread with custom stack size");
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

    let (tx, rx) = mpsc::channel::<Vec<i16>>();

    mock_data(tx);

    let sample_rate = 384000.;
    let mut samples = Vec::new();
    let mode = 0;

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend)?;
    loop {
        terminal
            .draw(|frame| {
                let transform = Transform::new(samples.clone());
                match mode {
                    0 => {
                        draw_waveform(frame, &samples);
                    }
                    1 => {
                        draw_frequency(frame, &transform);
                    }
                    2 => {
                        draw_frequency_chart(frame, &transform, sample_rate);
                    }
                    _ => {}
                }
                draw_note(frame, &transform.note(sample_rate as u32));
            })
            .unwrap();

        if let Ok(v) = rx.try_recv() {
            samples = v;
        }
    }
}
