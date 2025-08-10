use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Paragraph, Widget},
};

#[derive(Debug)]
pub struct Fps {
    frame_count: usize,
    last_instant: Instant,
    fps: Option<f32>,
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            frame_count: 0,
            last_instant: Instant::now(),
            fps: None,
        }
    }
}

impl Fps {
    pub fn tick(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_instant.elapsed();
        // update the fps every second, but only if we've rendered at least 2 frames (to avoid
        // noise in the fps calculation)
        if elapsed > Duration::from_secs(1) && self.frame_count > 2 {
            self.fps = Some(self.frame_count as f32 / elapsed.as_secs_f32());
            self.frame_count = 0;
            self.last_instant = Instant::now();
        }
    }
}

#[derive(Default)]
pub struct FpsWidget {
    pub fps: Fps,
    pub show_label: bool,
    pub style: Style,
}

impl FpsWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_label(mut self, show_label: bool) -> Self {
        self.show_label = show_label;
        self
    }

    pub fn with_style<T: Into<Style>>(mut self, style: T) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for &FpsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(fps) = self.fps.fps {
            let text = if self.show_label {
                format!("{fps:.1} fps")
            } else {
                format!("{fps:.1}")
            };
            Paragraph::new(text).style(self.style).render(area, buf);
        }
    }
}
