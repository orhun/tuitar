use mousefood::prelude::*;
use mousefood::ratatui::layout::Offset;
use mousefood::ratatui::widgets::Paragraph;

use crate::{
    app::{Application, Tab},
    MAX_ADC_VALUE,
};
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

impl Application {
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
            self.input_mode.as_line(),
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
            Tab::Frequency => {
                draw_frequency(frame, area, &self.state);
                draw_note_name(frame, area, &self.state);
            }
            Tab::Waveform => {
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
            Tab::Spectrum => draw_dbfs_spectrum(frame, area, &self.state, ("dBFS", "Hz")),
            Tab::Fretboard => {
                frame.render_widget(
                    Paragraph::new(self.fretboard_mode.as_line()).alignment(Alignment::Center),
                    // Two lines above the bottom of the screen
                    Rect::new(
                        frame_area.left(),
                        frame_area.bottom().saturating_sub(2),
                        frame_area.width,
                        1,
                    ),
                );
                draw_fretboard(
                    frame,
                    frame.area().offset(Offset { x: 0, y: 3 }),
                    &self.state,
                    &mut self.fretboard_state,
                );
            }
        }

        frame.render_widget(
            Paragraph::new(self.tab.to_string()).alignment(Alignment::Center),
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
