mod app;
mod input;
mod transform;

use app::Application;
use ratatui::crossterm::event;

fn main() {
    let mut app = Application::new();
    app.start_recording();

    let mut terminal = ratatui::init();
    let mut samples = Vec::new();

    while app.is_running {
        app.state
            .process_samples(&samples, app.recorder.sample_rate() as f64);

        terminal.draw(|frame| app.render(frame)).unwrap();

        if let Ok(v) = app.receiver.try_recv() {
            samples = v;
        }

        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            let event = event::read().unwrap();
            app.handle_event(event);
        }
    }
    ratatui::restore();
}
