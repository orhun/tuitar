mod app;
mod transform;
mod ui;
mod utils;

use std::time::Instant;

use embedded_hal::spi::MODE_3;
use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{ADCPin, AnyIOPin, Gpio27, Gpio33, InputPin, OutputPin};
use esp_idf_svc::hal::gpio::{InterruptType, Pull};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi::{SpiAnyPins, SpiDriverConfig};
use esp_idf_svc::hal::{
    adc::{
        attenuation::DB_11,
        oneshot::{
            config::{AdcChannelConfig, Calibration},
            AdcChannelDriver, AdcDriver,
        },
        Resolution,
    },
    delay::FreeRtos,
    gpio::PinDriver,
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver},
    units::*,
};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::Builder;
use mousefood::embedded_graphics::draw_target::DrawTarget;
use mousefood::embedded_graphics::prelude::RgbColor;
use mousefood::prelude::*;

use app::{Application, Button, ButtonState, Event, InputMode};
use transform::Transform;

pub(crate) const MAX_ADC_VALUE: u16 = 3129;

pub fn init_adc_channel<'a, PIN>(
    adc1_driver: &'a AdcDriver<'a, ADC1>,
    pin: impl Peripheral<P = PIN> + 'a,
) -> anyhow::Result<AdcChannelDriver<'a, PIN, &'a AdcDriver<'a, ADC1>>>
where
    PIN: ADCPin<Adc = ADC1>,
{
    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: Calibration::Line,
        // Sample unsigned 12-bit integers (0-4095)
        resolution: Resolution::Resolution12Bit,
    };
    Ok(AdcChannelDriver::new(adc1_driver, pin, &config)?)
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    const DISPLAY_OFFSET: (u16, u16) = (52, 40);

    const DISPLAY_SIZE: (u16, u16) = (135, 240);

    let mut backlight = PinDriver::output(peripherals.pins.gpio4)?;

    backlight.set_high()?;

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

    let mut mode_button = PinDriver::input(peripherals.pins.gpio35).unwrap();
    let mut mode_button_state = ButtonState::new();

    let mut menu_button = PinDriver::input(peripherals.pins.gpio0).unwrap();
    let mut menu_button_state = ButtonState::new();

    let adc1_driver = AdcDriver::new(peripherals.adc1).unwrap();
    let mut jack_adc_channel = init_adc_channel(&adc1_driver, peripherals.pins.gpio32)?;
    let mut mic_adc_channel = init_adc_channel(&adc1_driver, peripherals.pins.gpio36)?;
    let mut pot = init_adc_channel(&adc1_driver, peripherals.pins.gpio39)?;

    let buffer_size = 1024;
    let mut samples = Box::new([0i16; 1024]);

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend)?;

    let mut app = Application::new(buffer_size);

    while app.is_running {
        let instant = Instant::now();
        let mut sample_len = 0;
        while sample_len < buffer_size {
            let raw_sample = match app.input_mode {
                InputMode::Mic => mic_adc_channel.read().unwrap_or(0),
                InputMode::Jack => jack_adc_channel.read().unwrap_or(0),
            };
            samples[sample_len] = raw_sample as i16;
            sample_len += 1;
        }
        let elapsed = instant.elapsed();
        let sample_rate = sample_len as f64 / elapsed.as_secs_f64();
        app.state
            .process_samples(&samples[..sample_len], sample_rate);
        terminal.draw(|frame| app.render(frame)).unwrap();

        // let control_value = pot.read().unwrap_or_default();
        // if control_value / 100 != app.control_value / 100 {
        //     app.handle_event(Event::UpdateControlValue(control_value));
        // }

        let mode_button_pressed = mode_button.is_low();
        let menu_button_pressed = menu_button.is_low();

        if mode_button_pressed && menu_button_pressed {
            app.handle_press(Button::Both);
            Ets::delay_ms(100);
        } else {
            mode_button_state.update(mode_button_pressed, |press_type| {
                app.handle_press(Button::Mode(press_type));
            });

            menu_button_state.update(menu_button_pressed, |press_type| {
                app.handle_press(Button::Menu(press_type));
            });
        }

        samples.fill(0);

        app.handle_event(Event::Tick);
    }

    Ok(())
}
