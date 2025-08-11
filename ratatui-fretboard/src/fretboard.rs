//! Widget implementation.

use std::ops::RangeInclusive;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::StatefulWidget,
};

use crate::note::Note;

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
