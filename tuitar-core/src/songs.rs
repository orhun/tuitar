include!(concat!(env!("OUT_DIR"), "/songs.rs"));

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget};
    use ratatui_fretboard::{note::Note, Fretboard, FretboardState};

    use super::*;

    #[test]
    fn render_smoke_on_the_water() {
        let song = SMOKE_ON_THE_WATER.clone();

        let area = Rect::new(0, 0, 51, 7);
        let mut buf = Buffer::empty(area);

        let fretboard = Fretboard::default()
            .with_active_note_style(Style::default())
            .with_fret_number_style(Style::default())
            .with_note_name_style(Style::default())
            .with_active_string_style(Style::default());

        let mut state = FretboardState::default();

        assert_eq!(vec![Note::G(3), Note::D(3)], song.notes[0]);
        state.set_active_notes(song.notes[0].to_vec());

        fretboard.render(area, &mut buf, &mut state);

        assert_eq!(
            buf,
            Buffer::with_lines([
                "E4 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "B3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "G3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "D3 ║─┼───┼───┼───┼───┼─⬤─┼───┼───┼───┼───┼───┼───║ ",
                "A2 ║─┼───┼───┼───┼───┼─⬤─┼───┼───┼───┼───┼─⬤─┼───║ ",
                "E2 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼─⬤─┼───║ ",
                "     1   2   3   4   5   6   7   8   9  10  11  12 ",
            ])
        );

        state.clear_active_notes();
        assert_eq!(vec![Note::ASharp(3), Note::F(3)], song.notes[1]);
        state.set_active_notes(song.notes[1].to_vec());
        fretboard.render(area, &mut buf, &mut state);

        assert_eq!(
            buf,
            Buffer::with_lines([
                "E4 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "B3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "G3 ║─┼───┼───┼─⬤─┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "D3 ║─┼───┼───┼─⬤─┼───┼───┼───┼───┼─⬤─┼───┼───┼───║ ",
                "A2 ║─┼───┼───┼───┼───┼───┼───┼───┼─⬤─┼───┼───┼───║ ",
                "E2 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "     1   2   3   4   5   6   7   8   9  10  11  12 ",
            ])
        );
    }

    #[test]
    fn render_unscripted_violence() {
        let song = UNSCRIPTED_VIOLENCE.clone();

        let area = Rect::new(0, 0, 51, 7);
        let mut buf = Buffer::empty(area);

        let fretboard = Fretboard::default()
            .with_active_note_style(Style::default())
            .with_fret_number_style(Style::default())
            .with_note_name_style(Style::default())
            .with_active_string_style(Style::default());

        let mut state = FretboardState::default();

        assert_eq!(vec![Note::B(3)], song.notes[0]);
        state.set_active_notes(song.notes[0].to_vec());
        fretboard.render(area, &mut buf, &mut state);

        assert_eq!(
            buf,
            Buffer::with_lines([
                "E4 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "B3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "G3 ║─┼───┼───┼───┼─⬤─┼───┼───┼───┼───┼───┼───┼───║ ",
                "D3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼─⬤─┼───┼───║ ",
                "A2 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "E2 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "     1   2   3   4   5   6   7   8   9  10  11  12 ",
            ])
        );

        assert_eq!(vec![Note::D(4)], song.notes[2]);
        state.set_active_notes(song.notes[2].to_vec());
        fretboard.render(area, &mut buf, &mut state);

        assert_eq!(
            buf,
            Buffer::with_lines([
                "E4 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "B3 ║─┼───┼───┼─⬤─┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "G3 ║─┼───┼───┼───┼─⬤─┼───┼───┼─⬤─┼───┼───┼───┼───║ ",
                "D3 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼─⬤─┼───┼───║ ",
                "A2 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "E2 ║─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───║ ",
                "     1   2   3   4   5   6   7   8   9  10  11  12 ",
            ])
        );
    }
}
