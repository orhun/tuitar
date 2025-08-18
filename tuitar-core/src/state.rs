use std::collections::VecDeque;
use std::str::FromStr;

use pitchy::Note;
use tui_big_text::PixelSize;

use crate::transform::Transformer;

const DEFAULT_MAX_HISTORY: usize = 2;
const MIN_FREQ_HZ: f64 = 80.0;
const MAX_FREQ_HZ: f64 = 1320.0;

#[derive(Debug, Clone)]
struct NoteHistory {
    name: String,
    fundamental_frequency: f64,
}

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
    note_history: VecDeque<NoteHistory>,

    /// The maximum number of notes to keep in history.
    pub max_history: usize,
}

impl<T: Transformer> State<T> {
    pub fn new(
        transform: T,
        buffer_size: usize,
        fret_count: u8,
        text_size: PixelSize,
        bottom_padding: u16,
        max_history: Option<usize>,
    ) -> Self {
        Self {
            transform,
            samples: Vec::with_capacity(buffer_size),
            sample_rate: 0.0,
            fret_count,
            text_size,
            bottom_padding,
            note_history: VecDeque::with_capacity(max_history.unwrap_or(DEFAULT_MAX_HISTORY)),
            max_history: max_history.unwrap_or(DEFAULT_MAX_HISTORY),
        }
    }

    pub fn process_samples(&mut self, samples: &[i16], sample_rate: f64) {
        self.samples = samples.to_vec();
        self.transform.process(samples);
        self.sample_rate = sample_rate;
        let fundamental_frequency = self.transform.find_fundamental_frequency(sample_rate);

        if !(MIN_FREQ_HZ..=MAX_FREQ_HZ).contains(&fundamental_frequency) {
            #[cfg(feature = "logging")]
            log::warn!(
                "Fundamental frequency out of range: {:.2} Hz (expected between {:.2} and {:.2} Hz)",
                fundamental_frequency,
                MIN_FREQ_HZ,
                MAX_FREQ_HZ
            );
            return;
        }

        if let Some(name) = Note::new(fundamental_frequency).name() {
            self.note_history.push_back(NoteHistory {
                name,
                fundamental_frequency,
            });
            if self.note_history.len() > self.max_history {
                self.note_history.pop_front();
            }
        }

        #[cfg(feature = "logging")]
        log::info!(
            "Sampled {} samples at {:.2} Hz | Fundamental frequency = {:.2} Hz",
            samples.len(),
            sample_rate,
            fundamental_frequency
        );
    }

    /// Returns the last note if all notes in the history are the same.
    fn get_most_frequent_note(&self) -> Option<f64> {
        let h = &self.note_history;
        if h.is_empty() {
            return None;
        }

        let first_name = &h[0].name;
        if h.iter().all(|e| e.name == *first_name) {
            Some(h.iter().last()?.fundamental_frequency)
        } else {
            None
        }
    }

    pub fn get_current_note(&self) -> Option<(Note, f64)> {
        let frequency = self.get_most_frequent_note()?;

        let note = Note::new(frequency);
        let note_name = note.name()?;

        let perfect_note = Note::from_str(&note_name).expect("failed to get perfect note");

        // 1 semitone = 100 cents
        // 1200 cents = 1 octave
        // cents = 1200 * log2(note.frequency() / target.frequency())
        let cents = 1200.0 * (note.frequency() / perfect_note.frequency()).log2();

        Some((note, cents))
    }
}
