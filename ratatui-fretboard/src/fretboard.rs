//! Widget implementation.

use std::ops::RangeInclusive;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::StatefulWidget,
};

use crate::note::{Note, STANDARD_TUNING};

/// State for the fretboard widget.
#[derive(Clone, Debug)]
pub struct FretboardState {
    /// The currently active notes on the fretboard.
    pub active_notes: Vec<Note>,
    /// The notes that are being used for tracking.
    pub ghost_notes: Vec<Note>,
    /// The range of frets to display on the fretboard.
    pub frets: RangeInclusive<u8>,
}

impl Default for FretboardState {
    /// Creates a default `FretboardState` with no active notes and an empty ghost notes list.
    fn default() -> Self {
        Self {
            active_notes: Vec::new(),
            ghost_notes: Vec::new(),
            frets: 0..=12,
        }
    }
}

impl FretboardState {
    /// Creates a new `FretboardState` with no active notes.
    pub fn new(frets: RangeInclusive<u8>) -> Self {
        Self {
            active_notes: Vec::new(),
            ghost_notes: Vec::new(),
            frets,
        }
    }

    /// Sets the active note on the fretboard.
    ///
    /// If the note is already a ghost note, it will be removed from the ghost notes.
    pub fn set_active_note(&mut self, note: Note) {
        if let Some(pos) = self.ghost_notes.iter().position(|n| *n == note) {
            self.ghost_notes.remove(pos);
        }

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

    /// Sets a ghost note on the fretboard, which is a note
    /// that is not currently active but is being tracked.
    pub fn set_ghost_note(&mut self, note: Note) {
        if !self.ghost_notes.contains(&note) {
            self.ghost_notes.push(note);
        }
    }

    /// Clears all active notes on the fretboard.
    pub fn clear_active_notes(&mut self) {
        self.active_notes.clear();
    }

    /// Clears all ghost notes on the fretboard.
    pub fn clear_ghost_notes(&mut self) {
        self.ghost_notes.clear();
    }

    /// Sets the range of frets to display on the fretboard.
    pub fn set_frets(&mut self, frets: RangeInclusive<u8>) {
        self.frets = frets;
    }
}

/// Represents a fretboard widget for displaying musical notes
/// and their positions on a guitar fretboard.
pub struct Fretboard {
    /// The names of the strings on the fretboard.
    string_names: Vec<Note>,
    /// The style for fret numbers.
    fret_number_style: Style,
    /// The style for note names.
    note_name_style: Style,
    /// The style for the active note.
    active_note_style: Style,
    /// The symbol used to represent the active note on the fretboard.
    active_note_symbol: char,
    /// The style for the active string.
    active_string_style: Style,
    /// The style for ghost notes on the fretboard.
    ghost_note_style: Style,
    /// The symbol used to represent ghost notes on the fretboard.
    ghost_note_symbol: char,
}

impl Default for Fretboard {
    /// Creates a default `Fretboard` with standard guitar tuning.
    fn default() -> Self {
        Self {
            string_names: STANDARD_TUNING.to_vec(),
            fret_number_style: Style::default().fg(Color::Magenta),
            note_name_style: Style::default().fg(Color::Green),
            active_note_style: Style::default().fg(Color::Yellow),
            active_note_symbol: '⬤',
            active_string_style: Style::default().fg(Color::Yellow),
            ghost_note_style: Style::default().fg(Color::Blue),
            ghost_note_symbol: '✖',
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

    /// Sets the style for the active string.
    pub fn with_active_string_style(mut self, style: Style) -> Self {
        self.active_string_style = style;
        self
    }

    /// Sets the style for ghost notes on the fretboard.
    pub fn with_ghost_note_style(mut self, style: Style) -> Self {
        self.ghost_note_style = style;
        self
    }

    /// Sets the symbol used to represent ghost notes on the fretboard.
    pub fn with_ghost_note_symbol(mut self, symbol: char) -> Self {
        self.ghost_note_symbol = symbol;
        self
    }
}

impl StatefulWidget for &Fretboard {
    type State = FretboardState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let fret_labels: Vec<_> = state.frets.clone().collect();
        let available_width = area.width as usize;
        let fret_width = available_width / fret_labels.len();

        // Draw top border
        for (i, string_note) in self.string_names.iter().rev().enumerate() {
            let y = area.y + i as u16;
            let base_note = string_note.clone();

            // Draw string name
            let name = base_note.to_string();
            let mut spans: Vec<Span> = vec![
                Span::from(format!("{name:<3}")).style(self.note_name_style),
                "║".into(),
            ];

            let string_style = if state.active_notes.contains(&base_note) {
                self.active_string_style
            } else {
                Style::default()
            };

            // Draw fret symbols for each fret
            for (j, fret_num) in fret_labels.iter().enumerate() {
                let note = base_note.clone() + *fret_num;

                let fret_width = if j == 0 { 1 } else { fret_width };
                let highlight_active = state.active_notes.contains(&note);
                let highlight_ghost = state.ghost_notes.contains(&note);

                let symbol: Vec<Span> = if highlight_active || highlight_ghost {
                    let fret_width = fret_width.max(1);
                    let left_pad = (fret_width - 1) / 2;
                    let right_pad = fret_width - 1 - left_pad;
                    vec![
                        Span::styled("─".repeat(left_pad), string_style),
                        if j == 0 {
                            Span::styled("─", string_style)
                        } else if highlight_active {
                            Span::styled(
                                self.active_note_symbol.to_string(),
                                self.active_note_style,
                            )
                        } else if highlight_ghost {
                            Span::styled(self.ghost_note_symbol.to_string(), self.ghost_note_style)
                        } else {
                            unreachable!("Unexpected note state")
                        },
                        Span::styled("─".repeat(right_pad), string_style),
                    ]
                } else {
                    vec![Span::styled("─".repeat(fret_width), string_style)]
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
        let mut label_line = String::from("   ");
        for (j, fret_num) in fret_labels.iter().skip(1).enumerate() {
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
        FretboardState {
            active_notes: vec![Note::A(4)],
            ghost_notes: Vec::new(),
            frets: 0..=12,

        },
        Buffer::with_lines([
            "E4 ║─┼───┼───┼───┼───┼─⬤─┼───┼───┼───┼───┼───┼───║ ",
            "B3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼─⬤─┼───║ ",
            "G3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
            "D3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
            "A2 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
            "E2 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
            "     1   2   3   4   5   6   7   8   9  10  11  12 ",
        ])
    )]
    #[case::narrow_display(
        Rect::new(0, 0, 36, 13),
        Fretboard::default()
            .with_active_note_style(Style::default())
            .with_fret_number_style(Style::default())
            .with_note_name_style(Style::default()),
        FretboardState {
            active_notes: vec![Note::F(4)],
            ghost_notes: Vec::new(),
            frets: 0..=6,

        },
        Buffer::with_lines([
            "E4 ║─┼──⬤──┼─────┼─────┼─────┼─────║",
            "B3 ║─┼─────┼─────┼─────┼─────┼─────║",
            "G3 ║─┼─────┼─────┼─────┼─────┼─────║",
            "D3 ║─┼─────┼─────┼─────┼─────┼─────║",
            "A2 ║─┼─────┼─────┼─────┼─────┼─────║",
            "E2 ║─┼─────┼─────┼─────┼─────┼─────║",
            "     1     2     3     4     5     6",
            "                                    ",
            "                                    ",
            "                                    ",
            "                                    ",
            "                                    ",
            "                                    ",
        ])
    )]
    #[case::single_string_open_note(
        Rect::new(0, 0, 20, 3),
        Fretboard {
            string_names: vec![Note::E(2)],
            ..Fretboard::default()
        }
        .with_active_note_style(Style::default())
        .with_fret_number_style(Style::default())
        .with_note_name_style(Style::default())
        .with_active_string_style(Style::default()),
        FretboardState {
            active_notes: vec![Note::F(2)],
            ghost_notes: Vec::new(),
            frets: 0..=3,

        },
        Buffer::with_lines([
            "E2 ║─┼──⬤──┼─────║  ",
            "     1     2     3  ",
            "                    ",
        ])
    )]
    #[case::two_strings_custom_frets(
        Rect::new(0, 0, 24, 4),
        Fretboard {
            string_names: vec![Note::A(2), Note::D(3)],
            ..Fretboard::default()
        }
        .with_active_note_style(Style::default())
        .with_fret_number_style(Style::default())
        .with_note_name_style(Style::default()),
        FretboardState {
            active_notes: vec![Note::F(3)],
            ghost_notes: Vec::new(),
            frets: 2..=5,
        },
        Buffer::with_lines([
            "D3 ║─┼──⬤───┼──────║    ",
            "A2 ║─┼──────┼──────║    ",
            "     3      4      5    ",
            "                        ",
        ])
    )]
    #[case::custom_tuning_bass_style(
        Rect::new(0, 0, 34, 6),
        Fretboard {
            string_names: vec![Note::B(1), Note::E(2), Note::A(2), Note::D(3)],
            ..Fretboard::default()
        }
        .with_active_note_style(Style::default())
        .with_fret_number_style(Style::default())
        .with_note_name_style(Style::default()),
        FretboardState {
            active_notes: vec![Note::E(3)],
            ghost_notes: Vec::new(),
            frets: 0..=4,
        },
        Buffer::with_lines([
            "D3 ║─┼──────┼──⬤───┼──────║       ",
            "A2 ║─┼──────┼──────┼──────║       ",
            "E2 ║─┼──────┼──────┼──────║       ",
            "B1 ║─┼──────┼──────┼──────║       ",
            "     1      2      3      4       ",
            "                                  ",
        ])
    )]
    #[case::compact_display_limited_width(
        Rect::new(0, 0, 26, 6),
        Fretboard {
            string_names: vec![Note::E(4), Note::B(3), Note::G(3)],
            ..Fretboard::default()
        }
        .with_active_note_style(Style::default())
        .with_fret_number_style(Style::default())
        .with_note_name_style(Style::default()),
        FretboardState {
            active_notes: vec![Note::FSharp(4)],
            ghost_notes: Vec::new(),
            frets: 0..=3,
        },
        Buffer::with_lines([
            "G3 ║─┼──────┼──────║      ",
            "B3 ║─┼──────┼──────║      ",
            "E4 ║─┼──────┼──⬤───║      ",
            "     1      2      3      ",
            "                          ",
            "                          ",
        ])
    )]
    #[case::extremely_long_fretboard(
        Rect::new(0, 0, 120, 7),
        Fretboard {
            string_names: STANDARD_TUNING.to_vec(),
            ..Fretboard::default()
        }
        .with_active_note_style(Style::default())
        .with_fret_number_style(Style::default())
        .with_note_name_style(Style::default()),
        FretboardState {
            active_notes: vec![Note::FSharp(4), Note::F(3)],
            ghost_notes: Vec::new(),
            frets: 0..=20,
        },
        Buffer::with_lines([
            "E4 ║─┼─────┼──⬤──┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────║",
            "B3 ║─┼─────┼─────┼─────┼─────┼─────┼─────┼──⬤──┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────║",
            "G3 ║─┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──⬤──┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────║",
            "D3 ║─┼─────┼─────┼──⬤──┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──⬤──┼─────┼─────┼─────║",
            "A2 ║─┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──⬤──┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────║",
            "E2 ║─┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──⬤──┼─────┼─────┼─────┼─────┼─────┼─────║",
            "     1     2     3     4     5     6     7     8     9    10    11    12    13    14    15    16    17    18    19    20",
        ])
    )]

    fn render_fretboard(
        #[case] area: Rect,
        #[case] fretboard: Fretboard,
        #[case] mut state: FretboardState,
        #[case] expected: Buffer,
    ) {
        let mut buf = Buffer::empty(area);
        fretboard.render(area, &mut buf, &mut state);
        assert_eq!(buf, expected);
    }
}
