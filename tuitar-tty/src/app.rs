use std::sync::mpsc;

use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Offset;
use tui_big_text::PixelSize;
use tuitar::state::State;
use tuitar::ui::*;

use crate::input::Recorder;
use crate::transform::Transform;

pub struct Application {
    pub is_running: bool,
    pub state: State<Transform>,
    pub receiver: mpsc::Receiver<Vec<i16>>,
    pub recorder: Recorder,
    tab: usize,
}

impl Application {
    pub fn new() -> Self {
        let transform = Transform::new();

        let (tx, rx) = mpsc::channel::<Vec<i16>>();
        let recorder = Recorder::init("pipewire", move |data: &[i16], _| {
            tx.send(data.to_vec()).unwrap();
        });
        let state = State::new(
            transform,
            recorder.sample_rate() as usize,
            12,
            PixelSize::Full,
            5,
        );

        Self {
            is_running: true,
            state,
            receiver: rx,
            recorder,
            tab: 0,
        }
    }

    pub fn start_recording(&mut self) {
        self.recorder.start();
    }

    pub fn switch_tab(&mut self) {
        self.tab = (self.tab + 1) % 3;
    }

    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Tab => self.switch_tab(),
                KeyCode::Char('q') | KeyCode::Esc => self.is_running = false,
                _ => {}
            }
        }
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>) {
        match self.tab {
            0 => draw_waveform(frame, &self.state, (i16::MIN as f64, i16::MAX as f64)),
            1 => draw_frequency(frame, &self.state),
            2 => draw_frequency_chart(frame, &self.state),
            _ => {}
        }

        draw_cents(frame, &self.state);
        draw_note_name(frame, &self.state);

        let fretboard_width = 51;
        let mut area = frame.area().offset(Offset {
            x: (frame.area().width as i32 - fretboard_width) / 2,
            y: (frame.area().height / 2) as i32 + 4,
        });
        area.width = fretboard_width.try_into().unwrap_or(0);
        draw_fretboard(frame, area, &self.state);
    }
}
