use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

use pitchy::Note;
use tui_big_text::PixelSize;

use crate::transform::Transformer;

const MAX_HISTORY: usize = 2;

/// The application state.
pub struct State<T: Transformer> {
    /// FFT transformer.
    pub transform: T,

    /// The audio samples to display.
    pub samples: Vec<i16>,

    /// The sample rate.
    pub sample_rate: f64,

    /// The number of frets to display on the fretboard.
    pub fret_count: u8,

    /// The pixel size for the note name display.
    pub text_size: PixelSize,

    /// The y offset for the note name display.
    /// It takes the center of the frame as reference point.
    pub bottom_padding: u16,

    /// A history of recent notes.
    note_history: VecDeque<f64>,
}

impl<T: Transformer> State<T> {
    pub fn new(transform: T, fret_count: u8, text_size: PixelSize, bottom_padding: u16) -> Self {
        Self {
            transform,
            samples: Vec::new(),
            sample_rate: 0.0,
            fret_count,
            text_size,
            bottom_padding,
            note_history: VecDeque::new(),
        }
    }

    pub fn process_samples(&mut self, samples: &[i16], sample_rate: f64) {
        self.samples = samples.to_vec();
        self.transform.process(samples);
        self.sample_rate = sample_rate;
        let fundamental_frequency = self.transform.find_fundamental_frequency(sample_rate);

        if Note::new(fundamental_frequency).name().is_some() {
            self.note_history.push_back(fundamental_frequency);
            if self.note_history.len() > MAX_HISTORY {
                self.note_history.pop_front();
            }
        }

        log::info!(
            "Sampled {} samples at {:.2} Hz | Fundamental frequency = {:.2} Hz",
            samples.len(),
            sample_rate,
            fundamental_frequency
        );
    }

    fn get_most_frequent_note(&self) -> Option<f64> {
        self.note_history
            .iter()
            .map(|f| *f as i32)
            .fold(HashMap::new(), |mut acc, freq| {
                *acc.entry(freq).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(freq_hz, _)| freq_hz as f64)
            .and_then(|f| {
                if f > 70.0 && f < 3000.0 {
                    Some(f)
                } else {
                    None
                }
            })
    }

    pub fn get_current_note(&self) -> Option<(Note, f64)> {
        let Some(frequency) = self.get_most_frequent_note() else {
            return None;
        };

        let note = Note::new(frequency);
        let Some(note_name) = note.name() else {
            return None;
        };

        let perfect_note = Note::from_str(&note_name).expect("failed to get perfect note");

        // 1 semitone = 100 cents
        // 1200 cents = 1 octave
        // cents = 1200 * log2(note.frequency() / target.frequency())
        let cents = 1200.0 * (note.frequency() / perfect_note.frequency()).log2();

        Some((note, cents))
    }
}
