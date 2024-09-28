mod heap;
mod model;
mod msg;
mod update;
mod view;

use std::io;

use ratatui::DefaultTerminal;

use crate::model::Model;
use crate::msg::handle_event;
use crate::update::update;
use crate::view::view;

fn main_loop(mut terminal: DefaultTerminal) -> io::Result<()> {
    let mut model = Model::new();
    while !model.quit {
        terminal.draw(|frame| view(&model, frame))?;
        let msg = handle_event(&model.mode)?;
        model = update(model, msg);
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let result = main_loop(terminal);
    ratatui::restore();
    result
}

