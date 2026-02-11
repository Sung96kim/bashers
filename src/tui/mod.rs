use anyhow::Result;
use crossterm::event::{self as crossterm_event, EnableMouseCapture, Event};
use crossterm::ExecutableCommand;
use ratatui::layout::Rect;
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

pub trait TuiApp {
    fn update_layout(&mut self, term_size: Rect, available_height: u16);
    fn render(&self, frame: &mut Frame);
    fn poll_interval(&self) -> Duration;
    fn process_background(&mut self);
    fn handle_event(&mut self, event: Event) -> Result<bool>;
}

pub fn run<T: TuiApp>(mut app: T) -> Result<()> {
    let mut terminal = ratatui::init();
    std::io::stdout().execute(EnableMouseCapture)?;
    let result = run_loop(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run_loop<T: TuiApp>(terminal: &mut DefaultTerminal, app: &mut T) -> Result<()> {
    loop {
        app.process_background();
        let term_size = terminal.size()?;
        let available_height = term_size.height.saturating_sub(1);
        app.update_layout(term_size.into(), available_height);
        terminal.draw(|frame| app.render(frame))?;
        if crossterm_event::poll(app.poll_interval())?
            && app.handle_event(crossterm_event::read()?)?
        {
            break;
        }
    }
    Ok(())
}
