use std::time::Instant;

use mousefood::prelude::*;
use mousefood::ratatui::layout::Offset;
use mousefood::ratatui::widgets::Paragraph;
use tui_big_text::PixelSize;
use tuitar_core::fps::FpsWidget;

use crate::{Transform, MAX_ADC_VALUE};
use tuitar_core::state::State;
use tuitar_core::ui::*;

const LOGO_ASCII: &str = r#"
              ████  █████    
       █████  ████  █████    
       ████    ██     ██     
         ██████████████████  
███████████████████████████  
███████████████████████████  
████████████████   ███████   
██████             ██████    
                   █████     
                    ███      
"#;

pub enum Event {
    SwitchTab,
    SwitchInputMode,
}

pub struct Application {
    pub is_running: bool,
    pub state: State<Transform>,
    pub input_mode: usize,
    pub control_value: u16,
    pub fps_widget: FpsWidget,
    pub splash_timestamp: Instant,
    tab: usize,
}

impl Application {
    pub fn new(buffer_size: usize) -> Self {
        let transform = Transform::new();
        let state = State::new(transform, buffer_size, 6, PixelSize::Quadrant, 2);

        Self {
            is_running: true,
            state,
            input_mode: 0,
            control_value: 0,
            fps_widget: FpsWidget::default().with_style(
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ),
            splash_timestamp: Instant::now(),
            tab: 0,
        }
    }

    pub fn switch_tab(&mut self) {
        self.tab = (self.tab + 1) % 4;
    }

    pub fn switch_input_mode(&mut self) {
        self.input_mode = (self.input_mode + 1) % 2;
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::SwitchTab => self.switch_tab(),
            Event::SwitchInputMode => self.switch_input_mode(),
        }
    }

    fn render_splash(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let logo = Paragraph::new(LOGO_ASCII).style(Color::Red);
        frame.render_widget(logo, area);

        frame.render_widget(
            Paragraph::new("━━━━━━━┫ Tuitar v0 ┣━━━━━━━")
                .alignment(Alignment::Center)
                .style(Color::White),
            // One line above the bottom of the screen
            Rect::new(area.left(), area.bottom().saturating_sub(1), area.width, 1),
        );
    }

    fn render_fps(&mut self, frame: &mut Frame<'_>) {
        self.fps_widget.fps.tick();
        let area = frame.area();

        frame.render_widget(
            &self.fps_widget,
            // Bottom right corner of the screen
            Rect::new(
                area.right().saturating_sub(3),
                area.bottom().saturating_sub(1),
                3,
                1,
            ),
        );
    }

    fn render_input_mode(&mut self, frame: &mut Frame<'_>) {
        frame.render_widget(
            Line::from(vec![
                "[".gray(),
                match self.input_mode {
                    0 => "M".red(),
                    1 => "J".blue(),
                    _ => "?".gray(),
                },
                "]".gray(),
            ]),
            // Bottom right corner of the screen
            Rect::new(
                frame.area().left().saturating_sub(3),
                frame.area().bottom().saturating_sub(1),
                3,
                1,
            ),
        );
    }

    fn render_menus(&mut self, frame: &mut Frame<'_>) {
        let frame_area = frame.area();
        draw_cents(frame, frame_area, &self.state);

        // Move the area up by one line to make space for the bottom area
        let area = frame_area.inner(Margin {
            horizontal: 0,
            vertical: 1,
        });

        match self.tab {
            0 => {
                draw_frequency(frame, area, &self.state);
                draw_note_name(frame, area, &self.state);
            }
            1 => {
                let value = MAX_ADC_VALUE.saturating_sub(self.control_value);
                let min_bound = (value / 100 * 100) as f64;
                draw_waveform(
                    frame,
                    area,
                    &self.state,
                    (min_bound, min_bound + 300.),
                    ("Amp", "Time"),
                )
            }
            2 => draw_dbfs_spectrum(frame, area, &self.state, ("dBFS", "Hz")),
            3 => draw_fretboard(
                frame,
                frame.area().offset(Offset { x: 0, y: 3 }),
                &self.state,
            ),
            _ => {}
        }

        let menu_name = match self.tab {
            0 => "Frequency",
            1 => "Waveform",
            2 => "Spectrum",
            3 => "Fretboard",
            _ => "Unknown",
        };

        frame.render_widget(
            Paragraph::new(menu_name).alignment(Alignment::Center),
            // One line above the bottom of the screen
            Rect::new(
                frame_area.left(),
                frame_area.bottom().saturating_sub(1),
                frame_area.width,
                1,
            ),
        )
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        if self.splash_timestamp.elapsed().as_secs() < 1 {
            self.render_splash(frame);
            return;
        }

        self.render_menus(frame);
        self.render_fps(frame);
        self.render_input_mode(frame);
    }
}
