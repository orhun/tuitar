use mousefood::ratatui::layout::Offset;
use mousefood::{prelude::*, ratatui};
use tui_big_text::PixelSize;

use crate::Transform;
use tuitar::state::State;
use tuitar::ui::*;

pub enum Event {
    SwitchTab,
    SwitchInputMode,
}

pub struct Application {
    pub is_running: bool,
    pub state: State<Transform>,
    pub input_mode: usize,
    pub control_value: u16,
    tab: usize,
}

impl Application {
    pub fn new() -> Self {
        let transform = Transform::new();
        let state = State::new(transform, 6, PixelSize::Quadrant, 2);

        Self {
            is_running: true,
            state,
            input_mode: 0,
            control_value: 0,
            tab: 0,
        }
    }

    pub fn switch_tab(&mut self) {
        self.tab = (self.tab + 1) % 4;
    }

    pub fn switch_input_mode(&mut self) {
        self.input_mode = (self.input_mode + 1) % 2;
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::SwitchTab => self.switch_tab(),
            Event::SwitchInputMode => self.switch_input_mode(),
        }
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>) {
        match self.tab {
            0 => draw_waveform(
                frame,
                &self.state,
                (self.control_value as f64, self.control_value as f64 + 500.),
            ),
            1 => draw_frequency(frame, &self.state),
            2 => draw_frequency_chart(frame, &self.state),
            3 => draw_fretboard(
                frame,
                frame.area().offset(Offset { x: 0, y: 3 }),
                &self.state,
            ),
            _ => {}
        }

        if self.tab != 3 {
            draw_cents(frame, &self.state);
            draw_note_name(frame, &self.state);
        }

        let input_mode_letter = match self.input_mode {
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
    }
}
