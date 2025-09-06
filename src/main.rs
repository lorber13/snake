use std::io;

use crate::app::App;

mod app;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new(terminal.get_frame().area()).run(&mut terminal);
    ratatui::restore();
    app_result
}
