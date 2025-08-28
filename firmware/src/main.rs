mod app;
mod transform;
mod ui;
mod utils;

use std::num::NonZeroUsize;
use std::time::Instant;

use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{ADCPin, Gpio27, Gpio33, InputPin, OutputPin};
use esp_idf_svc::hal::gpio::{InterruptType, Pull};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi::SpiAnyPins;
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
use mousefood::prelude::*;
use st7735_lcd::{Orientation, ST7735};

use app::{Application, Button, ButtonState, Event, InputMode};
use transform::Transform;

pub(crate) const MAX_ADC_VALUE: u16 = 3129;

type DisplayResult<'a> = anyhow::Result<
    ST7735<
        SpiDeviceDriver<'a, esp_idf_svc::hal::spi::SpiDriver<'a>>,
        PinDriver<'a, Gpio27, esp_idf_svc::hal::gpio::Output>,
        PinDriver<'a, Gpio33, esp_idf_svc::hal::gpio::Output>,
    >,
>;

pub fn init_display<'a>(
    spi: impl Peripheral<P = impl SpiAnyPins> + 'a,
    sclk: impl Peripheral<P = impl OutputPin + InputPin> + 'a,
    sdo: impl Peripheral<P = impl OutputPin> + 'a,
    sdi: Option<impl Peripheral<P = impl OutputPin + InputPin> + 'a>,
    cs: Option<impl Peripheral<P = impl OutputPin> + 'a>,
    dc: impl Peripheral<P = Gpio27> + 'a,
    rst: impl Peripheral<P = Gpio33> + 'a,
) -> DisplayResult<'a> {
    let rst = PinDriver::output(rst)?;
    let dc = PinDriver::output(dc)?;
    let driver_config = Default::default();
    let spi_config = SpiConfig::new()
        .baudrate(80u32.MHz().into())
        .write_only(true);
    // let spi_config = SpiConfig::new().baudrate(26.MHz().into()).data_mode(embedded_hal::spi::MODE_3);
    let spi = SpiDeviceDriver::new_single(spi, sclk, sdo, sdi, cs, &driver_config, &spi_config)?;
    let rgb = true;
    let inverted = false;
    let width = 160;
    let height = 128;
    let mut display = ST7735::new(spi, dc, rst, rgb, inverted, width, height);

    let mut delay = FreeRtos;
    display.init(&mut delay).unwrap();
    display
        .set_orientation(&Orientation::LandscapeSwapped)
        .unwrap();

    Ok(display)
}

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

    Layout::init_cache(NonZeroUsize::new(20).unwrap()); // default is 500

    let peripherals = Peripherals::take()?;

    let mut display = init_display(
        // SPI2 is used for the display
        peripherals.spi2,
        // HSPI SCK — default SPI2 clock pin, safe at boot
        peripherals.pins.gpio14,
        // HSPI MOSI — default SPI2 data pin, safe at boot
        peripherals.pins.gpio13,
        // MISO not used for display
        None::<esp_idf_svc::hal::gpio::Gpio0>,
        // CS — general-purpose pin, safe at boot
        Some(peripherals.pins.gpio25),
        // DC (AO) — general-purpose pin, safe at boot
        peripherals.pins.gpio27,
        // RESET — unused GPIO, safe and stable
        peripherals.pins.gpio33,
    )?;

    let mut mode_button = PinDriver::input(peripherals.pins.gpio21).unwrap();
    mode_button
        .set_interrupt_type(InterruptType::NegEdge)
        .unwrap();
    mode_button.set_pull(Pull::Up).unwrap();
    let mut mode_button_state = ButtonState::new();

    let mut menu_button = PinDriver::input(peripherals.pins.gpio22).unwrap();
    menu_button
        .set_interrupt_type(InterruptType::NegEdge)
        .unwrap();
    menu_button.set_pull(Pull::Up).unwrap();
    let mut menu_button_state = ButtonState::new();

    let adc1_driver = AdcDriver::new(peripherals.adc1).unwrap();
    let mut jack_adc_channel = init_adc_channel(&adc1_driver, peripherals.pins.gpio36)?;
    let mut mic_adc_channel = init_adc_channel(&adc1_driver, peripherals.pins.gpio32)?;
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

        let control_value = pot.read().unwrap_or_default();
        if control_value / 100 != app.control_value / 100 {
            app.handle_event(Event::UpdateControlValue(control_value));
        }

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
