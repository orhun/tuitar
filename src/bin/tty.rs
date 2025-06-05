use std::sync::mpsc;

use ratatui::crossterm::event::{self, Event, KeyCode};
use tuitar::input::Recorder;
use tuitar::transform::Transform;
use tuitar::ui::*;

fn main() {
    let (tx, rx) = mpsc::channel::<Vec<i16>>();

    let recorder = Recorder::init("pipewire", move |data: &[i16], _| {
        tx.send(data.to_vec()).unwrap();
    });

    recorder.start();

    let mut terminal = ratatui::init();
    let mut samples = Vec::new();
    let mut mode = 0;

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
                        draw_frequency_chart(frame, &transform, recorder.sample_rate() as f64);
                    }
                    _ => {}
                }
                draw_note(frame, &transform.note(recorder.sample_rate()));
            })
            .unwrap();

        if let Ok(v) = rx.try_recv() {
            samples = v;
        }

        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Tab => {
                        mode = (mode + 1) % 3;
                    }
                    _ => break,
                }
            }
        }
    }
    ratatui::restore();
}
