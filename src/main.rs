mod heap;
mod io;
mod message;
mod model;
mod update;
mod view;

use std::io::Result;

use ratatui::DefaultTerminal;

use crate::{
    message::{Message, NormalMsg, handle_event},
    model::{Mode, Model},
    update::update,
    view::view,
};

fn main_loop(mut terminal: DefaultTerminal) -> Result<()> {
    let file_path = io::data_file_path()?;
    let (heap, file) = io::init(&file_path)?;
    let mut model = Model { heap, mode: Mode::Normal };
    loop {
        terminal.draw(|frame| view(&model, frame))?;
        let message = handle_event(model.mode)?;
        if let Message::Normal(NormalMsg::Quit) = message {
            std::mem::drop(file);
            return io::save(model.heap, &file_path);
        }
        model = update(message, model.heap);
    }
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let result = main_loop(terminal);
    ratatui::restore();
    result
}

