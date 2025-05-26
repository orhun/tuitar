use std::sync::mpsc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use pitchy::Note;
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::widgets::Block;
use ratatui::{
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Chart, Dataset, GraphType},
};
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use tui_bar_graph::{BarGraph, BarStyle, ColorMode};
use tui_big_text::{BigText, PixelSize};

struct Recorder {
    stream: Stream,
    config: StreamConfig,
}

impl Recorder {
    fn init<D>(device_name: &str, callback: D) -> Self
    where
        D: FnMut(&[i16], &cpal::InputCallbackInfo) + Send + 'static,
    {
        let host = cpal::default_host();

        let device = host
            .input_devices()
            .unwrap()
            .find(|d| d.name().unwrap().contains(device_name))
            .expect("No pipewire input device found");

        println!("Using input device: {:?}", device.name().unwrap());

        let mut supported_configs_range = device.supported_input_configs().unwrap();
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config")
            .with_max_sample_rate()
            .config();

        println!("Using config: {:?}", supported_config,);

        let stream = device
            .build_input_stream(
                &supported_config,
                callback,
                |err| {
                    panic!("Error: {err}");
                },
                None,
            )
            .unwrap();

        Self {
            stream,
            config: supported_config,
        }
    }

    pub fn start(&self) {
        self.stream.play().unwrap();
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }
}

struct Transform {
    fft_samples: Vec<Complex<f64>>,
}

impl Transform {
    fn new(samples: Vec<i16>) -> Self {
        let samples_f64: Vec<f64> = samples.iter().map(|&s| s as f64).collect();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(samples.len());

        let mut fft_input: Vec<Complex<f64>> =
            samples_f64.iter().map(|&s| Complex::new(s, 0.0)).collect();

        fft.process(&mut fft_input);

        Self {
            fft_samples: fft_input,
        }
    }

    fn find_fundamental_frequency(&self, sample_rate: f64) -> f64 {
        let mut max_magnitude = 0.0;
        let mut fundamental_freq = 0.0;

        for (i, &sample) in self.fft_samples.iter().enumerate() {
            let magnitude = sample.norm();
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                fundamental_freq = i as f64 * sample_rate / self.fft_samples.len() as f64;
            }
        }

        fundamental_freq
    }

    pub fn note(&self, sample_rate: u32) -> Note {
        let fundamental_freq = self.find_fundamental_frequency(sample_rate as f64);
        Note::new(fundamental_freq)
    }

    pub fn fft_data(&self) -> Vec<f64> {
        // Only take the first half of the FFT output (the positive frequencies)
        let half_len = self.fft_samples.len() / 2;
        let magnitude_spectrum: Vec<f64> = self
            .fft_samples
            .iter()
            .take(half_len)
            .map(|c| c.norm()) // magnitude = sqrt(re^2 + im^2)
            .collect();

        magnitude_spectrum
    }
}

fn draw_waveform(frame: &mut Frame<'_>, samples: &Vec<i16>) {
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

fn draw_frequency(frame: &mut Frame<'_>, transform: &Transform) {
    let data_points = transform.fft_data();
    let bar_graph = BarGraph::new(data_points)
        .with_gradient(colorgrad::preset::cool())
        .with_bar_style(BarStyle::Braille)
        .with_color_mode(ColorMode::VerticalGradient);
    frame.render_widget(bar_graph, frame.area());
}

// TODO: needs fixing
fn draw_frequency_chart(frame: &mut Frame<'_>, transform: &Transform, sample_rate: f64) {
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

fn draw_note(frame: &mut Frame<'_>, note: &Note) {
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

fn main() {
    let (tx, rx) = mpsc::channel::<Vec<i16>>();

    let recorder = Recorder::init("pipewire", move |data: &[i16], _| {
        tx.send(data.to_vec()).unwrap();
    });

    recorder.start();

    let mut terminal = ratatui::init();
    let mut samples = Vec::new();
    let mut mode = 0;

    loop {
        terminal
            .draw(|frame| {
                let transform = Transform::new(samples.clone());
                match mode {
                    0 => {
                        draw_waveform(frame, &samples);
                    }
                    1 => {
                        draw_frequency(frame, &transform);
                    }
                    2 => {
                        draw_frequency_chart(frame, &transform, recorder.sample_rate() as f64);
                    }
                    _ => {}
                }
                draw_note(frame, &transform.note(recorder.sample_rate()));
            })
            .unwrap();

        if let Ok(v) = rx.try_recv() {
            samples = v;
        }

        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Tab => {
                        mode = (mode + 1) % 3;
                    }
                    _ => break,
                }
            }
        }
    }
    ratatui::restore();
}
