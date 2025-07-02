mod input;
mod transform;

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;

use input::Recorder;
use pitchy::Note;
use ratatui::crossterm::event::{self, Event, KeyCode};
use transform::Transform;
use tui_big_text::PixelSize;
use tuitar::transform::Transformer;
use tuitar::ui::*;

fn main() {
    let (tx, rx) = mpsc::channel::<Vec<i16>>();

    let recorder = Recorder::init("pipewire", move |data: &[i16], _| {
        tx.send(data.to_vec()).unwrap();
    });

    recorder.start();

    let mut terminal = ratatui::init();
    let mut samples = Vec::new();
    let mut mode = 1;

    let mut note_history: VecDeque<f64> = VecDeque::new();
    let max_history = 2;
    let mut transform = Transform::new();

    loop {
        transform.process(&samples);
        let fundamental_freq = transform.find_fundamental_frequency(recorder.sample_rate() as f64);
        if Note::new(fundamental_freq).name().is_some() {
            note_history.push_back(fundamental_freq);
            if note_history.len() > max_history {
                note_history.pop_front();
            }
        }
        terminal
            .draw(|frame| {
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

                let most_frequent_note = note_history
                    .iter()
                    .map(|f| *f as i32)
                    .fold(HashMap::new(), |mut acc, freq| {
                        *acc.entry(freq).or_insert(0) += 1;
                        acc
                    })
                    .into_iter()
                    .max_by_key(|&(_, count)| count)
                    .map(|(freq_hz, _)| freq_hz as f64);
                if let Some(most_frequent_note) = most_frequent_note {
                    if most_frequent_note > 70.0 && most_frequent_note < 3000.0 {
                        draw_note(frame, most_frequent_note, PixelSize::Full, 5);
                    }
                }
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
