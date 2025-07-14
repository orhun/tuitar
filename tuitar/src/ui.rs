use std::str::FromStr;

use pitchy::Note;
use ratatui::layout::{Alignment, Margin, Offset, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, LineGauge};
use ratatui::Frame;
use ratatui::{
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Chart, Dataset, GraphType},
};
use ratatui_fretboard::{Fretboard, FretboardState};
use tui_bar_graph::{BarGraph, BarStyle, ColorMode};
use tui_big_text::{BigText, PixelSize};

use crate::transform::Transformer;

pub fn draw_waveform(frame: &mut Frame<'_>, samples: &[i16], sample_rate: f64, bounds: (f64, f64)) {
    let duration = samples.len() as f64 / sample_rate;
    let data_points: Vec<(f64, f64)> = samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let time_in_seconds = (i as f64) / sample_rate;
            (time_in_seconds, sample as f64)
        })
        .collect();

    let label_count = 5;
    let x_labels: Vec<String> = (0..label_count)
        .map(|i| format!("{:.2}", i as f64 * duration / (label_count - 1) as f64))
        .collect();

    let x_axis = Axis::default()
        .title("Time(s)".red())
        .style(Style::default().white())
        .bounds([0.0, duration])
        .labels(x_labels);

    let y_axis = Axis::default()
        .title("Amplitude".red())
        .style(Style::default().white())
        .bounds([bounds.0, bounds.1])
        .labels(vec![
            format!("{:.0}", bounds.0),
            format!("{:.0}", bounds.0 + (bounds.1 - bounds.0) * 0.25),
            format!("{:.0}", bounds.0 + (bounds.1 - bounds.0) * 0.5),
            format!("{:.0}", bounds.0 + (bounds.1 - bounds.0) * 0.75),
            format!("{:.0}", bounds.1),
        ]);

    let dataset = Dataset::default()
        .name("Audio Waveform")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Scatter)
        .style(Style::default().white())
        .data(&data_points);

    let chart = Chart::new(vec![dataset]).x_axis(x_axis).y_axis(y_axis);

    frame.render_widget(chart, frame.area());
}

pub fn draw_frequency<T: Transformer>(frame: &mut Frame<'_>, transform: &T, sample_rate: f64) {
    let data_points = transform.fft_data();

    let fft_size = data_points.len();
    let freq_per_bin = sample_rate as f32 / fft_size as f32;

    // Calculate bin indices for 20Hz and 20kHz
    let start_bin = (20.0 / freq_per_bin).ceil() as usize;
    let end_bin = ((20_000.0 / freq_per_bin).floor() as usize).min(fft_size);

    // Slice out the desired frequency range, skip the first spike
    let filtered_points = &data_points[start_bin..end_bin];

    // Normalize data to 0..1 range for colorgrad
    let max_value = filtered_points
        .iter()
        .cloned()
        .fold(0.0_f64, f64::max)
        .max(1e-8); // Avoid division by zero

    let scaled_points: Vec<f64> = filtered_points
        .iter()
        .map(|&x| ((x / max_value).clamp(0.0, 1.0)))
        .collect();

    let bar_graph = BarGraph::new(scaled_points)
        .with_gradient(colorgrad::preset::rainbow())
        .with_bar_style(BarStyle::Braille)
        .with_color_mode(ColorMode::VerticalGradient);

    frame.render_widget(bar_graph, frame.area());
}

// TODO: needs fixing
pub fn draw_frequency_chart<T: Transformer>(
    frame: &mut Frame<'_>,
    transform: &T,
    sample_rate: f64,
) {
    let fft_data = transform.fft_data();

    let fft_len = fft_data.len();
    let freq_step = sample_rate / (2.0 * fft_len as f64); // Nyquist range

    let data_points: Vec<(f64, f64)> = fft_data
        .iter()
        .enumerate()
        .map(|(i, &mag)| {
            let freq = i as f64 * freq_step;
            let db = 20.0 * mag.max(1e-10).log10(); // avoid log(0)
            (freq, db)
        })
        .collect();

    let dataset = Dataset::default()
        .name("Frequency Spectrum")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().cyan())
        .data(&data_points);

    let x_max = data_points.last().map(|v| v.0).unwrap_or(0.0);

    let x_axis = Axis::default()
        .title("Frequency (Hz)".red())
        .style(Style::default().white())
        .bounds([0.0, x_max])
        .labels(vec!["0", "1k", "5k", "10k", "20k"]);

    let y_axis = Axis::default()
        .title("Gain (dB)".red())
        .style(Style::default().white())
        .bounds([-100.0, 100.0])
        .labels(vec!["-100", "-50", "0", "50", "100"]);

    let chart = Chart::new(vec![dataset])
        .x_axis(x_axis)
        .y_axis(y_axis)
        .block(Block::bordered().title("Frequency Spectrum"));

    frame.render_widget(chart, frame.area());
}

pub fn draw_fretboard(frame: &mut Frame<'_>, frequency: f64, area: Rect, frets: u8) {
    let note = Note::new(frequency);
    let fretboard = Fretboard::default()
        .with_frets(0..=frets)
        .with_active_note_symbol('⬤')
        .with_active_note_style(Color::Yellow.into());
    let mut state = FretboardState::default();
    if let Ok(note) = note.try_into() {
        state.set_active_note(note);
    }
    frame.render_stateful_widget(fretboard, area, &mut state);
}

pub fn draw_note(
    frame: &mut Frame<'_>,
    frequency: f64,
    pixel_size: Option<PixelSize>,
    y_offset: u16,
    fretboard: bool,
) {
    let note = Note::new(frequency);
    let Some(name) = note.name() else {
        return;
    };

    let target = Note::from_str(&name).expect("failed to get perfect note");
    // 1 semitone = 100 cents
    // 1200 cents = 1 octave
    // cents = 1200 * log2(note.frequency() / target.frequency())
    let cents = 1200.0 * (note.frequency() / target.frequency()).log2();

    // One character represents 10 cents, max 5 characters
    let n = (cents.abs() / 10.0).round() as usize;
    let n = n.min(5);

    let mut spans = Vec::new();
    if cents.abs() < 1.0 {
        spans.push(Span::raw("["));
        spans.push(name.to_string().green());
        spans.push(Span::raw("]"));
    } else if cents > 0.0 {
        spans.push(Span::raw(" ".repeat(n)));
        spans.push(name.to_string().blue());
        spans.push(Span::styled(".".repeat(n), Color::Green));
    } else {
        spans.push(Span::styled(".".repeat(n), Color::Red));
        spans.push(name.to_string().blue());
        spans.push(Span::raw(" ".repeat(n)));
    };

    let text = vec![Line::from(spans)];

    let cents = cents.clamp(-50.0, 50.0);
    let ratio = ((cents + 50.0) / 100.0).clamp(0.0, 1.0);

    let label = if cents.abs() < 1.0 {
        "✓ in tune".to_string()
    } else if cents > 0.0 {
        format!("+{:.1}c", cents)
    } else {
        format!("{:.1}c", cents)
    };

    let (filled_style, unfilled_style) = if cents < 0.0 {
        (
            Style::new().white().on_red().bold(),
            Style::new().gray().on_black(),
        )
    } else {
        (
            Style::new().white().on_green().bold(),
            Style::new().gray().on_black(),
        )
    };

    let gauge = LineGauge::default()
        .filled_style(filled_style)
        .unfilled_style(unfilled_style)
        .label(Line::from(label).italic())
        .ratio(ratio);

    let gauge_area = frame.area().inner(Margin {
        horizontal: frame.area().width / 5,
        vertical: 0,
    });

    frame.render_widget(gauge, gauge_area);

    frame.render_widget(name.bold(), {
        let mut area = gauge_area.clone();
        area.x = gauge_area.right() + 1;
        area
    });

    let area = frame.area().offset(Offset {
        x: 0,
        y: (frame.area().height / 2).saturating_sub(y_offset) as i32,
    });
    if let Some(pixel_size) = pixel_size {
        let big_text = BigText::builder()
            .pixel_size(pixel_size)
            .style(Color::Blue)
            .lines(text)
            .alignment(Alignment::Center)
            .build();

        frame.render_widget(big_text, area);

        let freq_text = format!("{:.2} Hz", note.frequency());
        let freq_line =
            Line::styled(&freq_text, Style::new().bold().white()).alignment(Alignment::Center);
        let text_area = area
            .offset(Offset {
                x: 0,
                y: y_offset as i32 + 2,
            })
            .inner(Margin {
                horizontal: area.width.saturating_sub(freq_text.len() as u16) / 2,
                vertical: 0,
            });
        frame.render_widget(freq_line, text_area);
    }

    if fretboard {
        let mut area = area.offset(Offset {
            x: (frame.area().width as i32 - 51) / 2,
            y: y_offset as i32 + 4,
        });
        area.width = 51;

        draw_fretboard(frame, note.frequency(), area, 12);
    }
}
