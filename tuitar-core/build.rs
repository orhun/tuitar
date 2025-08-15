#![allow(unused)]
use guitarpro::{gp::Song as GpSong, note::Note as GpNote, track::Track};
use ratatui_fretboard::note::Note;
use std::{env, fs, path::PathBuf};

#[derive(Clone, Debug)]
struct SongData {
    name: &'static str,
    notes: Vec<Vec<Note>>,
}

#[derive(Debug, Clone, Copy)]
enum GpFormat {
    GP3,
    GP4,
    GP5,
}

fn gp_to_note(track: &Track, gp_note: &GpNote) -> Note {
    let string_index = (gp_note.string - 1) as usize;
    let tuning_midi = track.strings[string_index].1;
    let midi_value = (tuning_midi + gp_note.value as i8) as u8;
    let octave = (midi_value as i8 / 12) - 1;

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

fn parse_song_bytes(data: &[u8], format: GpFormat) -> Vec<Vec<Note>> {
    let mut song = GpSong::default();
    match format {
        GpFormat::GP3 => song.read_gp3(data),
        GpFormat::GP4 => song.read_gp4(data),
        GpFormat::GP5 => song.read_gp5(data),
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

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let songs_rs_path = out_dir.join("songs.rs");

    let songs = vec![
        (
            "SMOKE_ON_THE_WATER",
            "Smoke on the Water",
            "songs/deep-purple-smoke_on_the_water.gp3",
            GpFormat::GP3,
        ),
        (
            "MINECRAFT_SWEDEN",
            "Minecraft - Sweden",
            "songs/minecraft-sweden.gp5",
            GpFormat::GP5,
        ),
    ];

    let mut output = String::new();
    output.push_str("use ratatui_fretboard::note::Note;\n\n");
    output.push_str("#[derive(Clone, Debug)]\n");
    output.push_str("pub struct Song {\n");
    output.push_str("    pub name: &'static str,\n");
    output.push_str("    pub notes: &'static [ &'static [Note] ],\n");
    output.push_str("}\n\n");

    let mut song_idents = Vec::new();

    for (ident, name, path, fmt) in &songs {
        let data = fs::read(path).unwrap_or_else(|_| panic!("Missing file: {path}"));
        let parsed_notes = parse_song_bytes(&data, *fmt);
        song_idents.push(ident.to_string());

        output.push_str(&format!(
            "pub const {}: Song = Song {{\n    name: {:?},\n    notes: &[\n",
            ident, name
        ));
        for beat in parsed_notes {
            output.push_str("        &[");
            for note in beat {
                output.push_str(&format!("Note::{:?}, ", note));
            }
            output.push_str("],\n");
        }
        output.push_str("    ],\n};\n\n");

        println!("cargo:rerun-if-changed={}", path);
    }

    output.push_str("pub const SONGS: &[Song] = &[\n");
    for ident in song_idents {
        output.push_str(&format!("    {},\n", ident));
    }
    output.push_str("];\n");

    fs::write(songs_rs_path, output).expect("Failed to write songs.rs");
}
