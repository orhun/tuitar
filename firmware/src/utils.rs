use std::ops::RangeInclusive;

use ratatui_fretboard::note::{Note, STANDARD_TUNING};

/// Generates a random note within the specified range of frets.
///
/// # Notes
///
/// This functions only supports the standard tuning of a
/// 6-string guitar as of now.
pub fn generate_random_note(frets: &RangeInclusive<u8>) -> Note {
    let fret = fastrand::u8(*frets.start()..=*frets.end());
    let string_index = fastrand::usize(..STANDARD_TUNING.len());
    STANDARD_TUNING[string_index] + fret
}
