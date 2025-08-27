use guitarpro::{gp::Song as GpSong, note::Note as GpNote, track::Track};
use midly::{MidiMessage, Smf, TrackEventKind};
use ratatui_fretboard::note::Note;
use std::{env, fs, path::PathBuf};

const SONGS: &[(&str, &str, &str, SongFormat)] = &[
    (
        "UNSCRIPTED_VIOLENCE",
        "Unscripted Violence",
        "songs/unscripted-violence.mid",
        SongFormat::Midi(0),
    ),
    (
        "MY_OWN_SUMMER",
        "My Own Summer",
        "songs/my-own-summer.mid",
        SongFormat::Midi(1),
    ),
    (
        "SMOKE_ON_THE_WATER",
        "Smoke on the Water",
        "songs/deep-purple-smoke_on_the_water.gp3",
        SongFormat::GP3,
    ),
    (
        "MINECRAFT_SWEDEN",
        "Minecraft - Sweden",
        "songs/minecraft-sweden.gp5",
        SongFormat::GP5,
    ),
];

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum SongFormat {
    GP3,
    GP4,
    GP5,
    Midi(usize),
}

/// Convert a Guitar Pro note to a `ratatui_fretboard::note::Note`
fn gp_to_note(track: &Track, gp_note: &GpNote) -> Note {
    let string_index = (gp_note.string - 1) as usize;
    let tuning_midi = track.strings[string_index].1;
    let midi_value = (tuning_midi + gp_note.value as i8) as u8;
    let octave = (midi_value as i8 / 12) - 1;

    midi_to_note(midi_value, octave)
}

/// Convert raw MIDI note number + octave into `Note`
fn midi_to_note(midi_value: u8, octave: i8) -> Note {
    match midi_value % 12 {
        0 => Note::C(octave as u8),
        1 => Note::CSharp(octave as u8),
        2 => Note::D(octave as u8),
        3 => Note::DSharp(octave as u8),
        4 => Note::E(octave as u8),
        5 => Note::F(octave as u8),
        6 => Note::FSharp(octave as u8),
        7 => Note::G(octave as u8),
        8 => Note::GSharp(octave as u8),
        9 => Note::A(octave as u8),
        10 => Note::ASharp(octave as u8),
        11 => Note::B(octave as u8),
        _ => unreachable!(),
    }
}

/// Parse Guitar Pro bytes
fn parse_gp_bytes(data: &[u8], format: SongFormat) -> Vec<Vec<Note>> {
    let mut song = GpSong::default();
    match format {
        SongFormat::GP3 => song.read_gp3(data),
        SongFormat::GP4 => song.read_gp4(data),
        SongFormat::GP5 => song.read_gp5(data),
        _ => unreachable!(),
    };

    let mut notes = Vec::new();
    for track in &song.tracks {
        for measure in &track.measures {
            for voice in &measure.voices {
                for beat in &voice.beats {
                    let mut beat_notes = Vec::new();
                    for gp_note in &beat.notes {
                        beat_notes.push(gp_to_note(track, gp_note));
                    }
                    if !beat_notes.is_empty() {
                        notes.push(beat_notes);
                    }
                }
            }
        }
    }
    notes
}

/// Parse MIDI bytes, only keeping events from a specific track index
fn parse_midi_bytes(data: &[u8], guitar_track_index: usize) -> Vec<Vec<Note>> {
    let smf = Smf::parse(data).expect("Invalid MIDI file");
    let mut beats = Vec::new();

    if let Some(track) = smf.tracks.get(guitar_track_index) {
        let mut current_beat = Vec::new();

        for event in track {
            if event.delta > 0 {
                // commit beat when time moves forward
                if !current_beat.is_empty() {
                    beats.push(current_beat.clone());
                    current_beat.clear();
                }
            }

            if let TrackEventKind::Midi { channel, message } = event.kind {
                // ignore drums (channel 9)
                if channel == 9 {
                    continue;
                }

                if let MidiMessage::NoteOn { key, vel } = message {
                    if vel > 0 {
                        let midi_val = key.as_int();
                        let octave = (midi_val as i8 / 12) - 1;
                        current_beat.push(midi_to_note(midi_val, octave));
                    }
                }
            }
        }

        // push the last group if not empty
        if !current_beat.is_empty() {
            beats.push(current_beat);
        }
    }

    beats
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let songs_rs_path = out_dir.join("songs.rs");

    let mut output = String::new();
    output.push_str("use ratatui_fretboard::note::Note;\n\n");
    output.push_str("#[derive(Clone, Debug)]\n");
    output.push_str("pub struct Song {\n");
    output.push_str("    pub name: &'static str,\n");
    output.push_str("    pub notes: &'static [ &'static [Note] ],\n");
    output.push_str("}\n\n");

    let mut song_idents = Vec::new();

    for (ident, name, path, fmt) in SONGS {
        let data = fs::read(path).unwrap_or_else(|_| panic!("Missing file: {path}"));
        let parsed_notes = match fmt {
            SongFormat::GP3 | SongFormat::GP4 | SongFormat::GP5 => parse_gp_bytes(&data, *fmt),
            SongFormat::Midi(index) => parse_midi_bytes(&data, *index),
        };

        song_idents.push(ident.to_string());

        output.push_str(&format!(
            "pub const {ident}: Song = Song {{\n    name: {name:?},\n    notes: &[\n"
        ));
        for beat in parsed_notes {
            output.push_str("        &[");
            for note in beat {
                output.push_str(&format!("Note::{note:?}, "));
            }
            output.push_str("],\n");
        }
        output.push_str("    ],\n};\n\n");

        println!("cargo:rerun-if-changed={path}");
    }

    output.push_str("pub const SONGS: &[Song] = &[\n");
    for ident in song_idents {
        output.push_str(&format!("    {ident},\n"));
    }
    output.push_str("];\n");

    fs::write(songs_rs_path, output).expect("Failed to write songs.rs");
}
