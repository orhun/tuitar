use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::StatefulWidget,
};
use std::{
    fmt,
    ops::{Add, RangeInclusive},
};

/// Error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Indicates that the note letter is missing.
    MissingLetter,
    /// Indicates that the note octave is missing.
    MissingOctave,
    /// Indicates that the note letter is invalid.
    InvalidLetter(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingLetter => write!(f, "note letter is missing"),
            Error::MissingOctave => write!(f, "note octave is missing"),
            Error::InvalidLetter(letter) => {
                write!(f, "invalid note letter: {letter}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Represents a musical note with its pitch and octave.
#[derive(Clone, Debug, PartialEq, Eq)]
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

/// State for the fretboard widget.
#[derive(Default, Clone, Debug)]
pub struct FretboardState {
    /// The currently active notes on the fretboard.
    active_notes: Vec<Note>,
}

impl FretboardState {
    /// Creates a new `FretboardState` with no active notes.
    ///
    /// This is equivalent to `FretboardState::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the active note on the fretboard.
    pub fn set_active_note(&mut self, note: Note) {
        if !self.active_notes.contains(&note) {
            self.active_notes.push(note);
        }
    }

    /// Sets multiple active notes on the fretboard.
    pub fn set_active_notes(&mut self, notes: Vec<Note>) {
        for note in notes {
            self.set_active_note(note);
        }
    }

    /// Clears all active notes on the fretboard.
    pub fn clear_active_notes(&mut self) {
        self.active_notes.clear();
    }
}

/// Represents a fretboard widget for displaying musical notes
/// and their positions on a guitar fretboard.
pub struct Fretboard {
    /// The names of the strings on the fretboard.
    string_names: Vec<Note>,
    /// The range of frets to display on the fretboard.
    frets: RangeInclusive<u8>,
    /// The style for fret numbers.
    fret_number_style: Style,
    /// The style for note names.
    note_name_style: Style,
    /// The style for the active note.
    active_note_style: Style,
    /// The symbol used to represent the active note on the fretboard.
    active_note_symbol: char,
}

impl Default for Fretboard {
    /// Creates a default `Fretboard` with standard guitar tuning.
    fn default() -> Self {
        Self {
            string_names: vec![
                Note::E(2),
                Note::A(2),
                Note::D(3),
                Note::G(3),
                Note::B(3),
                Note::E(4),
            ],
            frets: 0..=12,
            fret_number_style: Style::default().fg(Color::Magenta),
            note_name_style: Style::default().fg(Color::Green),
            active_note_style: Style::default().fg(Color::Red),
            active_note_symbol: '●',
        }
    }
}

impl Fretboard {
    /// Creates a new `Fretboard` with default settings.
    ///
    /// This is equivalent to `Fretboard::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the names of the strings on the fretboard.
    pub fn with_string_names(mut self, string_names: Vec<Note>) -> Self {
        self.string_names = string_names;
        self
    }

    /// Sets the range of frets to display on the fretboard.
    pub fn with_frets(mut self, frets: RangeInclusive<u8>) -> Self {
        self.frets = frets;
        self
    }

    /// Sets the style for fret numbers.
    pub fn with_fret_number_style(mut self, style: Style) -> Self {
        self.fret_number_style = style;
        self
    }

    /// Sets the style for note names.
    pub fn with_note_name_style(mut self, style: Style) -> Self {
        self.note_name_style = style;
        self
    }

    /// Sets the style for the active note.
    pub fn with_active_note_style(mut self, style: Style) -> Self {
        self.active_note_style = style;
        self
    }

    /// Sets the symbol used to represent the active note on the fretboard.
    pub fn with_active_note_symbol(mut self, symbol: char) -> Self {
        self.active_note_symbol = symbol;
        self
    }
}

impl StatefulWidget for Fretboard {
    type State = FretboardState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let fret_labels: Vec<_> = self.frets.clone().collect();
        let available_width = area.width as usize - 4;
        let fret_width = available_width / fret_labels.len();

        // Draw top border
        for (i, string_note) in self.string_names.iter().rev().enumerate() {
            let y = area.y + i as u16;
            let base_note = string_note.clone();

            // Draw string name
            let name = base_note.name();
            let mut spans: Vec<Span> = vec![
                Span::from(format!("{name:<2}")).style(self.note_name_style),
                "║".into(),
            ];

            // Draw fret symbols for each fret
            for (j, fret_num) in fret_labels.iter().enumerate() {
                let note = base_note.clone() + *fret_num;

                let symbol: Vec<Span> = if state.active_notes.contains(&note) {
                    let fret_width = fret_width.max(1);
                    let left_pad = (fret_width - 1) / 2;
                    let right_pad = fret_width - 1 - left_pad;
                    vec![
                        "─".repeat(left_pad).into(),
                        Span::styled(self.active_note_symbol.to_string(), self.active_note_style),
                        "─".repeat(right_pad).into(),
                    ]
                } else {
                    Line::from("─".repeat(fret_width)).spans
                };

                if j == 0 {
                    spans.extend(symbol);
                } else if j == fret_labels.len() - 1 {
                    spans.push("║".into());
                } else {
                    spans.push("┼".into());
                    spans.extend(symbol);
                }
            }

            // Set the line in the buffer
            buf.set_line(area.x, y, &Line::from(spans), buf.area.width);
        }

        // Draw fret number row
        let label_y = area.y + self.string_names.len() as u16;
        let mut label_line = String::new();
        for (j, fret_num) in fret_labels.iter().enumerate() {
            if j == 0 {
                label_line.push_str(&format!("{fret_num:>3}"));
            } else {
                label_line.push_str(&format!("{:>width$}", fret_num, width = fret_width + 1));
            }
        }
        buf.set_string(area.x, label_y, label_line, self.fret_number_style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect};
    use rstest::*;

    #[rstest]
    #[case::wide_display(
        Rect::new(0, 0, 51, 7),
        Fretboard::default()
            .with_active_note_style(Style::default())
            .with_fret_number_style(Style::default())
            .with_note_name_style(Style::default()),
        Note::A(4),
        Buffer::with_lines([
            "E ║───┼───┼───┼───┼───┼─●─┼───┼───┼───┼───┼───┼───║",
            "B ║───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼─●─┼───║",
            "G ║───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║",
            "D ║───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║",
            "A ║───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║",
            "E ║───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║",
            "  0   1   2   3   4   5   6   7   8   9  10  11  12",
        ])
    )]
    #[case::narrow_display(
        Rect::new(0, 0, 40, 13),
        Fretboard::default()
            .with_active_note_style(Style::default())
            .with_fret_number_style(Style::default())
            .with_note_name_style(Style::default())
            .with_frets(0..=6),
        Note::F(4),
        Buffer::with_lines([
            "E ║─────┼──●──┼─────┼─────┼─────┼─────║ ",
            "B ║─────┼─────┼─────┼─────┼─────┼─────║ ",
            "G ║─────┼─────┼─────┼─────┼─────┼─────║ ",
            "D ║─────┼─────┼─────┼─────┼─────┼─────║ ",
            "A ║─────┼─────┼─────┼─────┼─────┼─────║ ",
            "E ║─────┼─────┼─────┼─────┼─────┼─────║ ",
            "  0     1     2     3     4     5     6 ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
        ])
    )]
    #[case::single_string_open_note(
        Rect::new(0, 0, 20, 3),
        Fretboard {
            string_names: vec![Note::E(2)],
            frets: 0..=3,
            ..Fretboard::default()
        }
        .with_active_note_style(Style::default())
        .with_fret_number_style(Style::default())
        .with_note_name_style(Style::default()),
        Note::E(2), // open string
        Buffer::with_lines([
            "E ║─●──┼────┼────║  ",
            "  0    1    2    3  ",
            "                    ",
        ])
    )]
    fn render_fretboard(
        #[case] area: Rect,
        #[case] fretboard: Fretboard,
        #[case] active_note: Note,
        #[case] expected: Buffer,
    ) {
        let mut buf = Buffer::empty(area);
        let mut state = FretboardState::default();
        state.set_active_note(active_note);
        fretboard.render(area, &mut buf, &mut state);
        assert_eq!(buf, expected);
    }

    // TODO: add more tests for different fretboard configurations
    //
    //     #[case::two_strings_custom_frets(
    //     Rect::new(0, 0, 26, 4),
    //     Fretboard {
    //         string_names: vec![Note::A(2), Note::D(3)],
    //         frets: 2..=5,
    //         ..Fretboard::default()
    //     }
    //     .with_active_note_style(Style::default())
    //     .with_fret_number_style(Style::default())
    //     .with_note_name_style(Style::default()),
    //     Note::B(3), // D string, 4th fret
    //     Buffer::with_lines([
    //         "D ║─────┼──●──┼─────┼─────║",
    //         "A ║─────┼─────┼─────┼─────║",
    //         "  2     3     4     5     ",
    //         "                          ",
    //     ])
    // )]
    //
    // #[case::custom_tuning_bass_style(
    //     Rect::new(0, 0, 34, 6),
    //     Fretboard {
    //         string_names: vec![Note::B(1), Note::E(2), Note::A(2), Note::D(3)],
    //         frets: 0..=4,
    //         ..Fretboard::default()
    //     }
    //     .with_active_note_style(Style::default())
    //     .with_fret_number_style(Style::default())
    //     .with_note_name_style(Style::default()),
    //     Note::G(3), // D string, fret 2
    //     Buffer::with_lines([
    //         "D ║─────┼─────┼──●──┼─────┼─────║",
    //         "A ║─────┼─────┼─────┼─────┼─────║",
    //         "E ║─────┼─────┼─────┼─────┼─────║",
    //         "B ║─────┼─────┼─────┼─────┼─────║",
    //         "  0     1     2     3     4     ",
    //         "                                ",
    //     ])
    // )]
    //
    // #[case::compact_display_limited_width(
    //     Rect::new(0, 0, 26, 6),
    //     Fretboard {
    //         string_names: vec![Note::E(4), Note::B(3), Note::G(3)],
    //         frets: 0..=3,
    //         ..Fretboard::default()
    //     }
    //     .with_active_note_style(Style::default())
    //     .with_fret_number_style(Style::default())
    //     .with_note_name_style(Style::default()),
    //     Note::G(4), // E string, fret 3
    //     Buffer::with_lines([
    //         "E ║─────┼─────┼─────┼──●──║",
    //         "B ║─────┼─────┼─────┼─────║",
    //         "G ║─────┼─────┼─────┼─────║",
    //         "  0     1     2     3     ",
    //         "                          ",
    //         "                          ",
    //     ])
    // )]
}
