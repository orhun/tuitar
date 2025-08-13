//! Common guitar scales.

use std::{fmt, ops::RangeInclusive};

use crate::note::{Note, STANDARD_TUNING};

/// Represents common guitar scales and their semitone intervals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    MajorPentatonic,
    MinorPentatonic,
    Major,
    NaturalMinor,
    Blues,
    Mixolydian,
    Dorian,
    Lydian,
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Scale::MajorPentatonic => "Major Pentatonic",
            Scale::MinorPentatonic => "Minor Pentatonic",
            Scale::Major => "Major",
            Scale::NaturalMinor => "Natural Minor",
            Scale::Blues => "Blues",
            Scale::Mixolydian => "Mixolydian",
            Scale::Dorian => "Dorian",
            Scale::Lydian => "Lydian",
        };
        write!(f, "{name}")
    }
}

impl Scale {
    /// Returns the pitch classes (0-11) of the scale, ignoring octave.
    pub fn pitch_classes(&self, root: Note) -> Vec<u8> {
        let root_pc = root.semitone_index() % 12;
        self.intervals()
            .iter()
            .map(|&interval| (root_pc + interval) % 12)
            .collect()
    }

    /// Returns the semitone steps for the scale, relative to the root note.
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            Scale::MajorPentatonic => &[0, 2, 4, 7, 9],
            Scale::MinorPentatonic => &[0, 3, 5, 7, 10],
            Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
            Scale::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::Blues => &[0, 3, 5, 6, 7, 10],
            Scale::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Scale::Lydian => &[0, 2, 4, 6, 7, 9, 11],
        }
    }

    /// Generates the notes of the scale starting from the given root note.
    pub fn notes(&self, root: Note) -> Vec<Note> {
        self.intervals()
            .iter()
            .map(|&interval| root.clone() + interval)
            .collect()
    }

    /// Returns all notes of this scale found on the given tuning and fret range.
    pub fn fretboard_notes(&self, root: Note, frets: &RangeInclusive<u8>) -> Vec<Note> {
        let scale_pcs = self.pitch_classes(root);

        STANDARD_TUNING
            .iter()
            .flat_map(|open_note| {
                frets.clone().filter_map(|fret| {
                    let note = open_note.clone() + fret;
                    if scale_pcs.contains(&(note.semitone_index() % 12)) {
                        Some(note)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Returns the next scale in a predefined sequence.
    pub fn next(&self) -> Scale {
        match self {
            Scale::MajorPentatonic => Scale::MinorPentatonic,
            Scale::MinorPentatonic => Scale::Blues,
            Scale::Major => Scale::NaturalMinor,
            Scale::NaturalMinor => Scale::MajorPentatonic,
            Scale::Blues => Scale::Mixolydian,
            Scale::Mixolydian => Scale::Dorian,
            Scale::Dorian => Scale::Lydian,
            Scale::Lydian => Scale::Major,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::Note;
    use rstest::rstest;
    use Note::*;

    #[rstest]
    #[case::major_pentatonic(Scale::MajorPentatonic, vec!["C4", "D4", "E4", "G4", "A4"])]
    #[case::minor_pentatonic(Scale::MinorPentatonic, vec!["C4", "D#4", "F4", "G4", "A#4"])]
    #[case::major(Scale::Major, vec!["C4", "D4", "E4", "F4", "G4", "A4", "B4"])]
    #[case::natural_minor(Scale::NaturalMinor, vec!["C4", "D4", "D#4", "F4", "G4", "G#4", "A#4"])]
    #[case::scales(Scale::Blues, vec!["C4", "D#4", "F4", "F#4", "G4", "A#4"])]
    #[case::mixolydian(Scale::Mixolydian, vec!["C4", "D4", "E4", "F4", "G4", "A4", "A#4"])]
    #[case::dorian(Scale::Dorian, vec!["C4", "D4", "D#4", "F4", "G4", "A4", "A#4"])]
    #[case::lydian(Scale::Lydian, vec!["C4", "D4", "E4", "F#4", "G4", "A4", "B4"])]
    fn test_scales(#[case] scale: Scale, #[case] expected: Vec<&str>) {
        let root = Note::C(4);
        let notes = scale.notes(root);
        let names: Vec<_> = notes.iter().map(|n| n.to_string()).collect();
        assert_eq!(names, expected);
    }

    #[rstest]
    #[case(Scale::MajorPentatonic, Note::A(4), 0..=5, vec![
        E(2), FSharp(2), A(2), A(2), B(2), CSharp(3),
        E(3), FSharp(3), A(3), B(3), B(3), CSharp(4),
        E(4), E(4), FSharp(4), A(4)
    ])]
    #[case(Scale::MinorPentatonic, Note::E(2), 0..=3, vec![
        E(2), G(2), A(2), B(2),
        D(3), E(3), G(3), A(3), B(3),
        D(4), E(4), G(4)
    ])]
    fn test_fretboard_notes(
        #[case] scale: Scale,
        #[case] root: Note,
        #[case] frets: std::ops::RangeInclusive<u8>,
        #[case] expected: Vec<Note>,
    ) {
        let result = scale.fretboard_notes(root, &frets);
        assert_eq!(result, expected);
    }
}
