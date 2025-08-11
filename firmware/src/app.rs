use std::{fmt::Display, time::Instant};

use mousefood::prelude::*;
use tui_big_text::PixelSize;
use tuitar_core::fps::FpsWidget;

use crate::{Transform, MAX_ADC_VALUE};
use tuitar_core::state::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    SwitchTab,
    SwitchInputMode,
    UpdateControlValue(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Mic,
    Jack,
}

impl InputMode {
    pub fn as_line(&self) -> Line {
        let label = match self {
            InputMode::Mic => "M".red(),
            InputMode::Jack => "J".blue(),
        };
        Line::from(vec!["[".gray(), label, "]".gray()])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Frequency,
    Waveform,
    Spectrum,
    Fretboard,
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Tab::Frequency => "Frequency",
            Tab::Waveform => "Waveform",
            Tab::Spectrum => "Spectrum",
            Tab::Fretboard => "Fretboard",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FretboardMode {
    #[default]
    Live,
    Scale,
    Exercise,
    Song,
}

impl Display for FretboardMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FretboardMode::Live => "Live",
            FretboardMode::Scale => "Scale",
            FretboardMode::Exercise => "Exercise",
            FretboardMode::Song => "Song",
        };
        write!(f, "{name}")
    }
}

impl FretboardMode {
    pub fn as_line(&self) -> Line {
        let label = match self {
            FretboardMode::Live => "Live".green(),
            FretboardMode::Scale => "Scale".yellow(),
            FretboardMode::Exercise => "Exercise".cyan(),
            FretboardMode::Song => "Song".red(),
        };
        Line::from(vec!["<".gray(), label, ">".gray()])
    }
}

pub struct Application {
    pub is_running: bool,
    pub state: State<Transform>,
    pub input_mode: InputMode,
    pub control_value: u16,
    pub fps_widget: FpsWidget,
    pub splash_timestamp: Instant,
    pub tab: Tab,
    pub fretboard_mode: FretboardMode,
}

impl Application {
    pub fn new(buffer_size: usize) -> Self {
        let transform = Transform::new();
        let state = State::new(transform, buffer_size, 6, PixelSize::Quadrant, 2);

        Self {
            is_running: true,
            state,
            input_mode: InputMode::default(),
            control_value: MAX_ADC_VALUE / 2,
            fps_widget: FpsWidget::default().with_style(
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ),
            splash_timestamp: Instant::now(),
            tab: Tab::default(),
            fretboard_mode: FretboardMode::Live,
        }
    }

    pub fn switch_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Frequency => Tab::Waveform,
            Tab::Waveform => Tab::Spectrum,
            Tab::Spectrum => Tab::Fretboard,
            Tab::Fretboard => Tab::Frequency,
        };
    }

    pub fn switch_input_mode(&mut self) {
        self.input_mode = match self.input_mode {
            InputMode::Mic => InputMode::Jack,
            InputMode::Jack => InputMode::Mic,
        };
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::SwitchTab => self.switch_tab(),
            Event::SwitchInputMode => self.switch_input_mode(),
            Event::UpdateControlValue(value) => {
                self.control_value = value;
                #[cfg(feature = "logging")]
                log::info!("Control value updated: {value}");

                if self.tab == Tab::Fretboard {
                    let step = MAX_ADC_VALUE / 4;
                    if value < step {
                        self.fretboard_mode = FretboardMode::Live;
                    } else if value < step * 2 {
                        self.fretboard_mode = FretboardMode::Scale;
                    } else if value < step * 3 {
                        self.fretboard_mode = FretboardMode::Exercise;
                    } else {
                        self.fretboard_mode = FretboardMode::Song;
                    }
                }
            }
        }
    }
}
