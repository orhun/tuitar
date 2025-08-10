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
use tui_big_text::BigText;

use crate::state::State;
use crate::transform::Transformer;

pub fn draw_waveform<T: Transformer>(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &State<T>,
    bounds: (f64, f64),
    titles: (&'static str, &'static str),
) {
    let duration = state.samples.len() as f64 / state.sample_rate;
    let data_points: Vec<(f64, f64)> = state
        .samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let time_in_seconds = (i as f64) / state.sample_rate;
            (time_in_seconds, sample as f64)
        })
        .collect();

    let label_count = 5;
    let x_labels: Vec<String> = (0..label_count)
        .map(|i| {
            format!("{:.2}", i as f64 * duration / (label_count - 1) as f64)
                .trim_start_matches("0")
                .to_string()
        })
        .collect();

    let x_axis = Axis::default()
        .title(titles.1.red())
        .style(Style::default().white())
        .bounds([0.0, duration])
        .labels(x_labels);

    let y_axis = Axis::default()
        .title(titles.0.red())
        .style(Style::default().white())
        .bounds([bounds.0, bounds.1])
        .labels(vec![
            format!("{:.1}", bounds.0 / 1000.0),
            format!("{:.1}", (bounds.0 + (bounds.1 - bounds.0) * 0.25) / 1000.0),
            format!("{:.1}", (bounds.0 + (bounds.1 - bounds.0) * 0.5) / 1000.0),
            format!("{:.1}", (bounds.0 + (bounds.1 - bounds.0) * 0.75) / 1000.0),
            format!("{:.1}", bounds.1 / 1000.0),
        ]);

    let dataset = Dataset::default()
        .name("Audio Waveform")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Scatter)
        .style(Style::default().white())
        .data(&data_points);

    let chart = Chart::new(vec![dataset]).x_axis(x_axis).y_axis(y_axis);

    frame.render_widget(chart, area);
}

pub fn draw_frequency<T: Transformer>(frame: &mut Frame<'_>, area: Rect, state: &State<T>) {
    let data_points = state.transform.fft_data();

    let fft_size = data_points.len();
    let freq_per_bin = state.sample_rate as f32 / fft_size as f32;

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
        .with_gradient(colorgrad::preset::reds())
        .with_bar_style(BarStyle::Braille)
        .with_color_mode(ColorMode::VerticalGradient);

    frame.render_widget(bar_graph, area);
}

// TODO: needs fixing
pub fn draw_frequency_chart<T: Transformer>(frame: &mut Frame<'_>, area: Rect, state: &State<T>) {
    let fft_data = state.transform.fft_data();

    let fft_len = fft_data.len();
    let freq_step = state.sample_rate / (2.0 * fft_len as f64); // Nyquist range

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

    frame.render_widget(chart, area);
}

pub fn draw_fretboard<T: Transformer>(frame: &mut Frame<'_>, area: Rect, state: &State<T>) {
    let fretboard = Fretboard::default()
        .with_frets(0..=state.fret_count)
        .with_active_note_symbol('⬤')
        .with_active_note_style(Color::Yellow.into());

    let mut fretboard_state = FretboardState::default();
    if let Some(note) = state
        .get_current_note()
        .and_then(|(note, _)| note.try_into().ok())
    {
        fretboard_state.set_active_note(note);
    }

    frame.render_stateful_widget(fretboard, area, &mut fretboard_state);
}

pub fn draw_cents<T: Transformer>(frame: &mut Frame<'_>, area: Rect, state: &State<T>) {
    let Some((note, cents)) = state.get_current_note() else {
        return;
    };

    let cents = cents.clamp(-50.0, 50.0);
    let ratio = ((cents + 50.0) / 100.0).clamp(0.0, 1.0);

    let label = if cents.abs() < 1.0 {
        "✓ in tune".to_string()
    } else if cents > 0.0 {
        format!("+{cents:.1}c")
    } else {
        format!("{cents:.1}c")
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

    let area = area.inner(Margin {
        horizontal: area.width / 5,
        vertical: 0,
    });
    frame.render_widget(gauge, area);

    if let Some(note_name) = note.name() {
        let mut area = area;
        area.x = area.right() + 1;
        frame.render_widget(note_name.bold(), area);
    }
}

pub fn draw_note_name<T: Transformer>(frame: &mut Frame<'_>, area: Rect, state: &State<T>) {
    let Some((note, cents)) = state.get_current_note() else {
        return;
    };

    let Some(note_name) = note.name() else {
        return;
    };

    // One character represents 10 cents, max 5 characters
    let padding = (cents.abs() / 10.0).round() as usize;
    let padding = padding.min(5);
    let mut spans = Vec::new();
    if cents.abs() < 1.0 {
        spans.push(Span::raw("["));
        spans.push(note_name.to_string().green());
        spans.push(Span::raw("]"));
    } else if cents > 0.0 {
        spans.push(Span::raw(" ".repeat(padding)));
        spans.push(note_name.to_string().blue());
        spans.push(Span::styled(".".repeat(padding), Color::Green));
    } else {
        spans.push(Span::styled(".".repeat(padding), Color::Red));
        spans.push(note_name.to_string().blue());
        spans.push(Span::raw(" ".repeat(padding)));
    };

    let text = vec![Line::from(spans)];
    let area = area.offset(Offset {
        x: 0,
        y: (area.height / 2).saturating_sub(state.bottom_padding) as i32,
    });
    let big_text = BigText::builder()
        .pixel_size(state.text_size)
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
            y: state.bottom_padding as i32 + 2,
        })
        .inner(Margin {
            horizontal: area.width.saturating_sub(freq_text.len() as u16) / 2,
            vertical: 0,
        });
    frame.render_widget(freq_line, text_area);
}
