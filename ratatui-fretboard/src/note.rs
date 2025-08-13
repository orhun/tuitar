//! Error handling.

use std::{fmt, ops::Add};

use crate::error::Error;

/// Represents the standard tuning of a 6-string guitar.
pub const STANDARD_TUNING: [Note; 6] = [
    Note::E(2),
    Note::A(2),
    Note::D(3),
    Note::G(3),
    Note::B(3),
    Note::E(4),
];

/// Represents a musical note with its pitch and octave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Note {
    C(u8),
    CSharp(u8),
    D(u8),
    DSharp(u8),
    E(u8),
    F(u8),
    FSharp(u8),
    G(u8),
    GSharp(u8),
    A(u8),
    ASharp(u8),
    B(u8),
}

impl TryFrom<pitchy::Note> for Note {
    type Error = Error;
    /// Converts a `pitchy::Note` to a `Note`.
    fn try_from(note: pitchy::Note) -> Result<Self, Self::Error> {
        let letter = note.note_letter().ok_or(Error::MissingLetter)?;
        let octave = note.octave().ok_or(Error::MissingOctave)? as u8;
        match letter {
            "C" => Ok(Note::C(octave)),
            "C#" => Ok(Note::CSharp(octave)),
            "D" => Ok(Note::D(octave)),
            "D#" => Ok(Note::DSharp(octave)),
            "E" => Ok(Note::E(octave)),
            "F" => Ok(Note::F(octave)),
            "F#" => Ok(Note::FSharp(octave)),
            "G" => Ok(Note::G(octave)),
            "G#" => Ok(Note::GSharp(octave)),
            "A" => Ok(Note::A(octave)),
            "A#" => Ok(Note::ASharp(octave)),
            "B" => Ok(Note::B(octave)),
            _ => Err(Error::InvalidLetter(letter.to_string())),
        }
    }
}

impl Note {
    /// Returns the name of the note as a string.
    pub fn name(&self) -> &str {
        match self {
            Note::C(_) => "C",
            Note::CSharp(_) => "C#",
            Note::D(_) => "D",
            Note::DSharp(_) => "D#",
            Note::E(_) => "E",
            Note::F(_) => "F",
            Note::FSharp(_) => "F#",
            Note::G(_) => "G",
            Note::GSharp(_) => "G#",
            Note::A(_) => "A",
            Note::ASharp(_) => "A#",
            Note::B(_) => "B",
        }
    }

    /// Returns the octave of the note.
    pub fn semitone_index(&self) -> u8 {
        let (offset, octave) = match self {
            Note::C(o) => (0, *o),
            Note::CSharp(o) => (1, *o),
            Note::D(o) => (2, *o),
            Note::DSharp(o) => (3, *o),
            Note::E(o) => (4, *o),
            Note::F(o) => (5, *o),
            Note::FSharp(o) => (6, *o),
            Note::G(o) => (7, *o),
            Note::GSharp(o) => (8, *o),
            Note::A(o) => (9, *o),
            Note::ASharp(o) => (10, *o),
            Note::B(o) => (11, *o),
        };
        offset + octave * 12
    }

    /// Converts a semitone index (0-127) to a `Note`.
    pub fn from_semitone_index(semitones: u8) -> Self {
        let octave = semitones / 12;
        match semitones % 12 {
            0 => Note::C(octave),
            1 => Note::CSharp(octave),
            2 => Note::D(octave),
            3 => Note::DSharp(octave),
            4 => Note::E(octave),
            5 => Note::F(octave),
            6 => Note::FSharp(octave),
            7 => Note::G(octave),
            8 => Note::GSharp(octave),
            9 => Note::A(octave),
            10 => Note::ASharp(octave),
            11 => Note::B(octave),
            _ => unreachable!(),
        }
    }
}

impl Add<u8> for Note {
    type Output = Self;

    /// Adds a number of semitones to the note.
    fn add(self, other: u8) -> Self::Output {
        Note::from_semitone_index(self.semitone_index() + other)
    }
}

impl fmt::Display for Note {
    /// Formats the note as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Note::C(o) => write!(f, "C{o}"),
            Note::CSharp(o) => write!(f, "C#{o}"),
            Note::D(o) => write!(f, "D{o}"),
            Note::DSharp(o) => write!(f, "D#{o}"),
            Note::E(o) => write!(f, "E{o}"),
            Note::F(o) => write!(f, "F{o}"),
            Note::FSharp(o) => write!(f, "F#{o}"),
            Note::G(o) => write!(f, "G{o}"),
            Note::GSharp(o) => write!(f, "G#{o}"),
            Note::A(o) => write!(f, "A{o}"),
            Note::ASharp(o) => write!(f, "A#{o}"),
            Note::B(o) => write!(f, "B{o}"),
        }
    }
}

impl TryFrom<String> for Note {
    type Error = Error;
    /// Converts a string to a `Note`.
    fn try_from(s: String) -> Result<Self, Error> {
        let mut chars = s.chars();
        let note = match chars.next() {
            Some('C') => Ok(Note::C(0)),
            Some('D') => Ok(Note::D(0)),
            Some('E') => Ok(Note::E(0)),
            Some('F') => Ok(Note::F(0)),
            Some('G') => Ok(Note::G(0)),
            Some('A') => Ok(Note::A(0)),
            Some('B') => Ok(Note::B(0)),
            _ => return Err(Error::MissingLetter),
        };

        let octave = chars
            .next()
            .and_then(|c| c.to_digit(10))
            .map(|d| d as u8)
            .unwrap_or(0);

        if chars.next() == Some('#') {
            match note.clone()? {
                Note::C(_) => Ok(Note::CSharp(octave)),
                Note::D(_) => Ok(Note::DSharp(octave)),
                Note::F(_) => Ok(Note::FSharp(octave)),
                Note::G(_) => Ok(Note::GSharp(octave)),
                Note::A(_) => Ok(Note::ASharp(octave)),
                _ => note,
            }
        } else {
            note
        }
    }
}
