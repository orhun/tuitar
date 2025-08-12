use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget};
use ratatui_fretboard::{note::Note, Fretboard, FretboardState};

#[test]
fn test_track_notes() {
    let area = Rect::new(0, 0, 36, 7);
    let mut buf = Buffer::empty(area);
    let fretboard = Fretboard::default()
        .with_active_note_style(Style::default())
        .with_fret_number_style(Style::default())
        .with_note_name_style(Style::default())
        .with_ghost_note_style(Style::default());
    let mut state = FretboardState::new(0..=6);

    fretboard.render(area, &mut buf, &mut state);
    let expected = Buffer::with_lines([
        "E4 ║─┼─────┼─────┼─────┼─────┼─────║",
        "B3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "G3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "D3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "A2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "E2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "     1     2     3     4     5     6",
    ]);
    assert_eq!(buf, expected);

    state.set_ghost_note(Note::F(4));
    state.set_ghost_note(Note::FSharp(3));
    assert_eq!(vec![Note::F(4), Note::FSharp(3)], state.ghost_notes);
    assert_eq!(Vec::<Note>::new(), state.active_notes);

    fretboard.render(area, &mut buf, &mut state);
    let expected = Buffer::with_lines([
        "E4 ║─┼──✖──┼─────┼─────┼─────┼─────║",
        "B3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "G3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "D3 ║─┼─────┼─────┼─────┼──✖──┼─────║",
        "A2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "E2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "     1     2     3     4     5     6",
    ]);
    assert_eq!(buf, expected);

    state.set_active_note(Note::F(4));
    state.set_active_note(Note::F(3));
    assert_eq!(vec![Note::FSharp(3)], state.ghost_notes);
    assert_eq!(vec![Note::F(4), Note::F(3)], state.active_notes);

    state.clear_active_notes();
    state.set_active_note(Note::FSharp(3));
    fretboard.render(area, &mut buf, &mut state);
    let expected = Buffer::with_lines([
        "E4 ║─┼─────┼─────┼─────┼─────┼─────║",
        "B3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "G3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "D3 ║─┼─────┼─────┼─────┼──⬤──┼─────║",
        "A2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "E2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "     1     2     3     4     5     6",
    ]);
    assert_eq!(buf, expected);
}

#[test]
fn test_scroll() {
    let area = Rect::new(0, 0, 36, 7);
    let mut buf = Buffer::empty(area);
    let fretboard = Fretboard::default()
        .with_active_note_style(Style::default())
        .with_fret_number_style(Style::default())
        .with_note_name_style(Style::default())
        .with_ghost_note_style(Style::default())
        .with_active_string_style(Style::default());
    let mut state = FretboardState::new(0..=6);

    state.set_active_note(Note::FSharp(3));
    fretboard.render(area, &mut buf, &mut state);
    let expected = Buffer::with_lines([
        "E4 ║─┼─────┼─────┼─────┼─────┼─────║",
        "B3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "G3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "D3 ║─┼─────┼─────┼─────┼──⬤──┼─────║",
        "A2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "E2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "     1     2     3     4     5     6",
    ]);
    assert_eq!(buf, expected);

    state.set_frets(7..=13);
    fretboard.render(area, &mut buf, &mut state);
    let expected = Buffer::with_lines([
        "E4 ║─┼─────┼─────┼─────┼─────┼─────║",
        "B3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "G3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "D3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "A2 ║─┼─────┼──⬤──┼─────┼─────┼─────║",
        "E2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "     8     9    10    11    12    13",
    ]);
    assert_eq!(buf, expected);

    state.clear_active_notes();
    state.set_active_note(Note::B(3));
    state.set_frets(10..=16);
    fretboard.render(area, &mut buf, &mut state);
    let expected = Buffer::with_lines([
        "E4 ║─┼─────┼─────┼─────┼─────┼─────║",
        "B3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "G3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "D3 ║─┼─────┼─────┼─────┼─────┼─────║",
        "A2 ║─┼─────┼─────┼─────┼──⬤──┼─────║",
        "E2 ║─┼─────┼─────┼─────┼─────┼─────║",
        "    11    12    13    14    15    16",
    ]);
    assert_eq!(buf, expected);
}
