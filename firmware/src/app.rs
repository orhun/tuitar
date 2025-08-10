use mousefood::ratatui::layout::Offset;
use mousefood::{prelude::*, ratatui};
use tui_big_text::PixelSize;
use tuitar_core::fps::FpsWidget;

use crate::{Transform, MAX_ADC_VALUE};
use tuitar_core::state::State;
use tuitar_core::ui::*;

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

    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>) {
        self.fps_widget.fps.tick();
        let frame_area = frame.area();

        frame.render_widget(
            &self.fps_widget,
            // Bottom left corner of the screen
            Rect::new(
                frame_area.right().saturating_sub(3),
                frame_area.bottom().saturating_sub(1),
                frame_area.width,
                1,
            ),
        );

        let input_mode_letter = match self.input_mode {
            0 => "M",
            1 => "J",
            _ => "?",
        };

        frame.render_widget(
            input_mode_letter.red(),
            // Bottom right corner of the screen
            Rect::new(
                frame_area.left().saturating_sub(1),
                frame_area.bottom().saturating_sub(1),
                frame_area.width,
                1,
            ),
        );

        // Move the area up by one line to make space for the bottom area
        let area = frame_area.inner(Margin {
            horizontal: 0,
            vertical: 1,
        });

        draw_cents(frame, frame_area, &self.state);

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
                    ("amp", "t"),
                )
            }
            2 => draw_frequency_chart(frame, area, &self.state),
            3 => draw_fretboard(
                frame,
                frame.area().offset(Offset { x: 0, y: 3 }),
                &self.state,
            ),
            _ => {}
        }
    }
}
