use std::sync::mpsc;

use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Margin, Offset};
use ratatui::style::Modifier;
use ratatui_fretboard::FretboardState;
use tui_big_text::PixelSize;
use tuitar_core::fps::FpsWidget;
use tuitar_core::state::State;
use tuitar_core::ui::*;

use crate::input::Recorder;
use crate::transform::Transform;

pub struct Application {
    pub is_running: bool,
    pub state: State<Transform>,
    pub receiver: mpsc::Receiver<Vec<i16>>,
    pub recorder: Recorder,
    pub fps_widget: FpsWidget,
    tab: usize,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
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
            Some(5),
        );

        Self {
            is_running: true,
            state,
            receiver: rx,
            recorder,
            fps_widget: FpsWidget::default()
                .with_label(true)
                .with_style(Modifier::ITALIC),
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
        self.fps_widget.fps.tick();
        let area = frame.area();

        draw_cents(frame, area, &self.state);
        frame.render_widget(&self.fps_widget, frame.area());

        let area = area.inner(Margin {
            horizontal: 0,
            vertical: 1,
        });

        match self.tab {
            0 => draw_waveform(
                frame,
                area,
                &self.state,
                (i16::MIN as f64, i16::MAX as f64),
                ("Amplitudes", "Time(s)"),
            ),
            1 => {
                draw_frequency(frame, area, &self.state);
                draw_note_name(frame, area, &self.state);
            }
            2 => draw_dbfs_spectrum(frame, area, &self.state, ("Level (dBFS)", "Frequency (Hz)")),
            _ => {}
        }

        let fretboard_width = 51;
        let mut area = frame.area().offset(Offset {
            x: (frame.area().width as i32 - fretboard_width) / 2,
            y: (frame.area().height / 2) as i32 + 4,
        });
        area.width = fretboard_width.try_into().unwrap_or(0);
        draw_fretboard(frame, area, &self.state, &mut FretboardState::default());
    }
}
