use mousefood::ratatui::layout::Offset;
use mousefood::ratatui::widgets::Paragraph;
use mousefood::{prelude::*, ratatui::widgets::LineGauge};
use ratatui_fretboard::Fretboard;
use tachyonfx::{Duration, EffectRenderer};

use crate::{
    app::{Application, FretboardMode, Tab, MAX_RANDOM_INTERVAL},
    MAX_ADC_VALUE,
};
use tuitar_core::{songs::*, ui::*};

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

        frame.render_effect(&mut self.intro_effect, area, Duration::from_millis(400));
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

                frame.render_stateful_widget(
                    &Fretboard::default(),
                    frame.area().offset(Offset { x: 0, y: 3 }),
                    &mut self.fretboard_state,
                );

                if self.fretboard_mode == FretboardMode::Scale {
                    let scale_line = Line::from(vec![
                        "<".into(),
                        self.current_scale.to_string().yellow(),
                        " (".into(),
                        self.current_root_note.name().cyan(),
                        ")>".into(),
                    ]);
                    frame.render_widget(
                        Paragraph::new(scale_line).alignment(Alignment::Center),
                        // Third line from the top
                        frame_area.offset(Offset { x: 0, y: 2 }),
                    );
                } else if self.fretboard_mode == FretboardMode::Song {
                    let song_line = Line::from(vec![
                        "<".into(),
                        SONGS[self.current_song_index].name.yellow(),
                        ">".into(),
                    ]);
                    frame.render_widget(
                        Paragraph::new(song_line).alignment(Alignment::Center),
                        // Third line from the top
                        frame_area.offset(Offset { x: 0, y: 2 }),
                    );
                } else if self.fretboard_mode == FretboardMode::Random {
                    let random_line = Line::from(vec![
                        "Points: ".yellow(),
                        self.random_mode_points.to_string().cyan(),
                    ]);
                    frame.render_widget(
                        Paragraph::new(random_line).alignment(Alignment::Center),
                        // Third line from the top
                        frame_area.offset(Offset { x: 0, y: 2 }),
                    );
                    let ratio = 1.0
                        - (self.last_random.elapsed().as_millis() as f64
                            / MAX_RANDOM_INTERVAL as f64)
                            .clamp(0.0, 1.0);
                    frame.render_widget(
                        LineGauge::default()
                            .filled_style(Color::Green)
                            .unfilled_style(Color::Red)
                            .ratio(ratio),
                        // Second line from the top
                        frame_area.offset(Offset { x: 0, y: 1 }),
                    );
                }
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
        );
    }

    pub fn render_effects(&mut self, frame: &mut Frame<'_>) {
        if !self.menu_effect.done() {
            frame.render_effect(
                &mut self.menu_effect,
                frame.area(),
                Duration::from_millis(200),
            );
            return;
        }

        if !self.input_mode_effect.done() {
            frame.render_effect(
                &mut self.input_mode_effect,
                frame.area(),
                Duration::from_millis(100),
            );
            return;
        }

        if !self.mode_effect.done() {
            frame.render_effect(
                &mut self.mode_effect,
                frame.area(),
                Duration::from_millis(400),
            );
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        if self.splash_timestamp.elapsed().as_millis() < 1500 {
            self.render_splash(frame);
            return;
        }

        self.render_menus(frame);
        self.render_fps(frame);
        self.render_input_mode(frame);
        self.render_effects(frame);
    }
}
