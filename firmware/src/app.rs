use std::{fmt::Display, time::Instant};

use mousefood::prelude::*;
use ratatui_fretboard::FretboardState;
use tui_big_text::PixelSize;
use tuitar_core::fps::FpsWidget;

use crate::{utils, Transform, MAX_ADC_VALUE};
use tuitar_core::state::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Mode,
    Menu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Tick,
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
    Random,
    Song,
}

impl Display for FretboardMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FretboardMode::Live => "Live",
            FretboardMode::Scale => "Scale",
            FretboardMode::Random => "Random",
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
            FretboardMode::Random => "Random".cyan(),
            FretboardMode::Song => "Song".red(),
        };
        Line::from(vec!["[".gray(), label, "]".gray()])
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
    pub fretboard_state: FretboardState,
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
            fretboard_state: FretboardState::default(),
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

    pub fn switch_fretboard_mode(&mut self) {
        self.fretboard_state.clear_ghost_notes();
        match self.fretboard_mode {
            FretboardMode::Live => {
                self.fretboard_mode = FretboardMode::Scale;
            }
            FretboardMode::Scale => {
                self.fretboard_mode = FretboardMode::Random;
            }
            FretboardMode::Random => {
                self.fretboard_mode = FretboardMode::Song;
            }
            FretboardMode::Song => {
                self.fretboard_mode = FretboardMode::Live;
            }
        }
    }

    pub fn scroll_fretboard(&mut self) {
        const MAX_FRET: u8 = 24;
        const WINDOW_SIZE: u8 = 6;
        let max_start_fret = MAX_FRET - WINDOW_SIZE;
        let start_fret =
            ((self.control_value as u32 * max_start_fret as u32) / MAX_ADC_VALUE as u32) as u8;

        let end_fret = start_fret + WINDOW_SIZE;
        self.fretboard_state.set_frets(start_fret..=end_fret);
    }

    pub fn tick(&mut self) {
        if self.tab == Tab::Fretboard
            && self.fretboard_mode == FretboardMode::Random
            && self.fretboard_state.ghost_notes.is_empty()
        {
            self.fretboard_state
                .set_ghost_note(utils::generate_random_note(0..=self.state.fret_count));
        }
    }

    pub fn handle_press(&mut self, button: Button, long_press: bool) {
        if button == Button::Mode && long_press {
            self.handle_event(Event::SwitchInputMode);
            return;
        }

        if button == Button::Menu && !long_press {
            self.handle_event(Event::SwitchTab);
            return;
        }

        if button == Button::Mode && !long_press && self.tab == Tab::Fretboard {
            self.switch_fretboard_mode();
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Tick => {
                self.tick();
            }
            Event::SwitchTab => self.switch_tab(),
            Event::SwitchInputMode => self.switch_input_mode(),
            Event::UpdateControlValue(value) => {
                self.control_value = value;
                #[cfg(feature = "logging")]
                log::info!("Control value updated: {value}");

                if self.tab == Tab::Fretboard {
                    self.scroll_fretboard();
                }
            }
        }
    }
}
