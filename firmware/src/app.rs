use std::{fmt::Display, time::Instant};

use mousefood::prelude::*;
use ratatui_fretboard::{note::Note, scale::Scale, FretboardState};
use tui_big_text::PixelSize;
use tuitar_core::{fps::FpsWidget, songs::*};

use crate::{utils, Transform, MAX_ADC_VALUE};
use tuitar_core::state::State;

pub(crate) const MAX_RANDOM_INTERVAL: u64 = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonPressType {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Mode(ButtonPressType),
    Menu(ButtonPressType),
    Both,
}

impl Button {
    pub fn is_mode(&self) -> bool {
        matches!(self, Button::Mode(_))
    }

    pub fn is_menu(&self) -> bool {
        matches!(self, Button::Menu(_))
    }

    pub fn is_short_press(&self) -> bool {
        matches!(
            self,
            Button::Mode(ButtonPressType::Short) | Button::Menu(ButtonPressType::Short)
        )
    }

    pub fn is_long_press(&self) -> bool {
        matches!(
            self,
            Button::Mode(ButtonPressType::Long) | Button::Menu(ButtonPressType::Long)
        )
    }
}

pub struct ButtonState {
    pressed_at: Option<Instant>,
}

impl ButtonState {
    pub fn new() -> Self {
        Self { pressed_at: None }
    }
}

impl ButtonState {
    pub fn update<F>(&mut self, is_pressed: bool, on_press: F)
    where
        F: FnOnce(ButtonPressType),
    {
        if is_pressed {
            // Button is currently down
            if self.pressed_at.is_none() {
                self.pressed_at = Some(Instant::now());
            }
        } else if let Some(pressed_at) = self.pressed_at.take() {
            // Button just released
            let duration = pressed_at.elapsed().as_millis() as u64;
            let press_type = if (500..2000).contains(&duration) {
                Some(ButtonPressType::Long)
            } else if duration < 500 {
                Some(ButtonPressType::Short)
            } else {
                None
            };

            if let Some(press_type) = press_type {
                on_press(press_type);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Tick,
    SwitchTab,
    SwitchInputMode,
    UpdateControlValue(u16),
    ToggleRootNote,
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
    pub remove_ghost: bool,
    pub current_scale: Scale,
    pub current_root_note: Note,
    pub current_song_index: usize,
    pub song_note_index: usize,
    pub random_mode_points: usize,
    pub last_random: Instant,
}

impl Application {
    pub fn new(buffer_size: usize) -> Self {
        let transform = Transform::new();
        let state = State::new(transform, buffer_size, 6, PixelSize::Quadrant, 2, None);

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
            current_scale: Scale::MajorPentatonic,
            remove_ghost: true,
            current_root_note: Note::A(4),
            current_song_index: 0,
            song_note_index: 0,
            random_mode_points: 0,
            last_random: Instant::now(),
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
        self.remove_ghost = true;
        self.song_note_index = 0;
        self.random_mode_points = 0;
        match self.fretboard_mode {
            FretboardMode::Live => {
                self.fretboard_mode = FretboardMode::Scale;
                self.set_scale_notes();
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

    pub fn set_scale_notes(&mut self) {
        self.fretboard_state.clear_ghost_notes();
        self.remove_ghost = false;
        self.fretboard_state.set_ghost_notes(
            self.current_scale
                .fretboard_notes(self.current_root_note, &self.fretboard_state.frets),
        );
    }

    pub fn toggle_current_root_note(&mut self) {
        let index = self.current_root_note.semitone_index() % 12;
        self.current_root_note = Note::from_semitone_index(index + 1);
    }

    pub fn scroll_fretboard(&mut self) {
        const MAX_FRET: u8 = 24;
        const WINDOW_SIZE: u8 = 6;
        let max_start_fret = MAX_FRET - WINDOW_SIZE;
        let start_fret = max_start_fret
            - ((self.control_value as u32 * max_start_fret as u32) / MAX_ADC_VALUE as u32) as u8;
        let end_fret = start_fret + WINDOW_SIZE;
        self.fretboard_state.set_frets(start_fret..=end_fret);
    }

    pub fn toggle_current_song(&mut self) {
        self.current_song_index = (self.current_song_index + 1) % SONGS.len();
        self.song_note_index = 0;
        self.fretboard_state.clear_ghost_notes();
    }

    pub fn tick(&mut self) {
        if self.tab == Tab::Fretboard && self.fretboard_mode == FretboardMode::Random {
            if self.fretboard_state.ghost_notes.is_empty() {
                self.fretboard_state
                    .set_ghost_note(utils::generate_random_note(&self.fretboard_state.frets));
                self.last_random = Instant::now();
            } else if self.last_random.elapsed().as_millis() as u64 > MAX_RANDOM_INTERVAL {
                self.random_mode_points = self.random_mode_points.saturating_sub(1);
                self.fretboard_state.clear_ghost_notes();
            }
        }

        if self.tab == Tab::Fretboard
            && self.fretboard_mode == FretboardMode::Song
            && self.fretboard_state.ghost_notes.is_empty()
        {
            let song = &SONGS[self.current_song_index];
            self.fretboard_state
                .set_ghost_notes(song.notes[self.song_note_index % song.notes.len()].to_vec());
            self.song_note_index += 1;
        }

        self.fretboard_state.clear_active_notes();
        if let Some(note) = self
            .state
            .get_current_note()
            .and_then(|(note, _)| note.try_into().ok())
        {
            if self.remove_ghost {
                if let Some(pos) = self
                    .fretboard_state
                    .ghost_notes
                    .iter()
                    .position(|n| *n == note)
                {
                    self.fretboard_state.ghost_notes.remove(pos);
                    self.random_mode_points += 1;
                }
            }

            if !self.fretboard_state.active_notes.contains(&note) {
                self.fretboard_state.active_notes.push(note);
            }
        }
    }

    pub fn handle_press(&mut self, button: Button) {
        if button == Button::Both && self.tab == Tab::Fretboard {
            if self.fretboard_mode == FretboardMode::Scale {
                self.handle_event(Event::ToggleRootNote);
            } else if self.fretboard_mode == FretboardMode::Song {
                self.toggle_current_song();
            }
            return;
        }

        if button == Button::Mode(ButtonPressType::Long)
            || (button == Button::Mode(ButtonPressType::Short) && self.tab != Tab::Fretboard)
        {
            self.handle_event(Event::SwitchInputMode);
            return;
        }

        if button.is_menu() && button.is_short_press() {
            self.handle_event(Event::SwitchTab);
            return;
        }

        if button.is_mode() && button.is_short_press() && self.tab == Tab::Fretboard {
            self.switch_fretboard_mode();
            return;
        }

        if button.is_menu()
            && button.is_long_press()
            && self.tab == Tab::Fretboard
            && self.fretboard_mode == FretboardMode::Scale
        {
            self.current_scale = self.current_scale.next();
            self.set_scale_notes();
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
            Event::ToggleRootNote => {
                self.toggle_current_root_note();
                self.set_scale_notes();
                #[cfg(feature = "logging")]
                log::info!("Current root note changed: {}", self.current_root_note);
            }
        }
    }
}
