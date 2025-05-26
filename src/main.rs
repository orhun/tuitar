mod input;
mod transform;
mod ui;

use ratatui::crossterm::event::{self, Event, KeyCode};
use std::sync::mpsc;
use transform::Transform;
use ui::*;

const SAMPLES: &str = include_str!("samples.txt");

fn mock_data(tx: mpsc::Sender<Vec<i16>>) {
    // Parse the string into a Vec<i16>
    let parsed_samples: Vec<i16> = SAMPLES
        .trim_matches(&['[', ']'][..])
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    std::thread::spawn(move || {
        let mut index = 0;

        loop {
            // Create a 1024-sample chunk from parsed_samples, wrapping if needed
            let chunk: Vec<i16> = (0..1024)
                .map(|i| {
                    let sample_index = (index + i) % parsed_samples.len();
                    parsed_samples[sample_index]
                })
                .collect();

            if tx.send(chunk).is_err() {
                break; // Exit if receiver is gone
            }

            index = (index + 1024) % parsed_samples.len(); // Advance position

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
}

fn main() {
    let (tx, rx) = mpsc::channel::<Vec<i16>>();

    // let recorder = input::Recorder::init("pipewire", move |data: &[i16], _| {
    //     tx.send(data.to_vec()).unwrap();
    // });
    // recorder.start();

    mock_data(tx);

    let sample_rate = 384000.;
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
                        draw_frequency_chart(frame, &transform, sample_rate);
                    }
                    _ => {}
                }
                draw_note(frame, &transform.note(sample_rate as u32));
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
