use pitchy::Note;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::widgets::Block;
use ratatui::{
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Chart, Dataset, GraphType},
};
use tui_bar_graph::{BarGraph, BarStyle, ColorMode};
use tui_big_text::{BigText, PixelSize};

use crate::transform::Transform;

pub fn draw_waveform(frame: &mut Frame<'_>, samples: &Vec<i16>) {
    let data_points: Vec<(f64, f64)> = samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let time_in_seconds = (i as f64) / 16384.;
            (time_in_seconds, sample as f64)
        })
        .collect();

    let dataset = Dataset::default()
        .name("Audio Waveform")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Scatter)
        .style(Style::default().white())
        .data(&data_points);

    let x_axis = Axis::default()
        .title("Time(s)".red())
        .style(Style::default().white())
        .bounds([0.0, data_points.last().map(|v| v.0).unwrap_or_default()])
        .labels(vec!["0", "1", "2", "3", "4"]);

    let y_axis = Axis::default()
        .title("Amplitude".red())
        .style(Style::default().white())
        .bounds([-32768.0, 32767.0])
        .labels(vec!["-32768", "-16384", "0", "16384", "32767"]);

    let chart = Chart::new(vec![dataset]).x_axis(x_axis).y_axis(y_axis);

    frame.render_widget(chart, frame.area());
}

pub fn draw_frequency(frame: &mut Frame<'_>, transform: &Transform) {
    let data_points = transform.fft_data();
    let bar_graph = BarGraph::new(data_points)
        .with_gradient(colorgrad::preset::cool())
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

pub fn draw_note(frame: &mut Frame<'_>, note: &Note) {
    if let Some(name) = note.name() {
        let big_text = BigText::builder()
            .pixel_size(PixelSize::Full)
            .style(Style::new().blue())
            .lines(vec![name.into()])
            .build();

        let [area] = Layout::horizontal([Constraint::Percentage(20)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [area] = Layout::vertical([Constraint::Percentage(20)])
            .flex(Flex::Center)
            .areas(area);

        frame.render_widget(big_text, area);
    }
}
