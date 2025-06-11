use std::sync::mpsc;

use ratatui::crossterm::event::{self, Event, KeyCode};
use tui_big_text::PixelSize;
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

    let mut transform = Transform::new();

    loop {
        terminal
            .draw(|frame| {
                transform.process(&samples);
                match mode {
                    0 => {
                        draw_waveform(
                            frame,
                            &samples,
                            recorder.sample_rate() as f64,
                            (i16::MIN as f64, i16::MAX as f64),
                        );
                    }
                    1 => {
                        draw_frequency(frame, &transform, recorder.sample_rate() as f64);
                    }
                    2 => {
                        draw_frequency_chart(frame, &transform, recorder.sample_rate() as f64);
                    }
                    _ => {}
                }
                draw_note(
                    frame,
                    &transform.note(recorder.sample_rate()),
                    PixelSize::Full,
                    5,
                );
            })
            .unwrap();
        if let Ok(v) = rx.try_recv() {
            // println!("Received {} samples", v.len());
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
