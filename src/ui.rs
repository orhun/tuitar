use pitchy::Note;
use ratatui::layout::Offset;
use ratatui::widgets::Block;
use ratatui::Frame;
use ratatui::{
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Chart, Dataset, GraphType},
};
use tui_bar_graph::{BarGraph, BarStyle, ColorMode};
use tui_big_text::{BigText, PixelSize};

#[cfg(feature = "tty")]
use crate::transform::Transform;

#[cfg(feature = "embedded")]
use crate::transform_esp::Transform;

pub fn draw_waveform(frame: &mut Frame<'_>, samples: &[i16], sample_rate: f64, bounds: (f64, f64)) {
    let duration = samples.len() as f64 / sample_rate;
    let data_points: Vec<(f64, f64)> = samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let time_in_seconds = (i as f64) / sample_rate as f64;
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

pub fn draw_frequency(frame: &mut Frame<'_>, transform: &Transform, sample_rate: f64) {
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
pub fn draw_frequency_chart(frame: &mut Frame<'_>, transform: &Transform, sample_rate: f64) {
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

pub fn draw_note(frame: &mut Frame<'_>, note: &Note, pixel_size: PixelSize) {
    if let Some(name) = note.name() {
        let big_text = BigText::builder()
            .pixel_size(pixel_size)
            .style(Style::new().blue())
            .lines(vec![name.into()])
            .build();

        let center_area = frame.area().offset(Offset {
            x: (frame.area().width / 2) as i32,
            y: (((frame.area().height / 2) - (big_text.lines.len() as u16)) as i32) + 1,
        });
        frame.render_widget(big_text, center_area);
    }
}
