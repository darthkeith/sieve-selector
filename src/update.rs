use crate::{
    heap::HeapStatus,
    io::{self, LoadState},
    message::{
        CompareMsg,
        InputMsg,
        LoadMsg,
        Message,
        NormalMsg,
        SelectedMsg,
        SelectMsg,
    },
    model::{
        Choice,
        InputAction,
        InputState,
        Mode,
        Model,
        SessionState,
    },
};

// Append a digit to `index` if valid, otherwise return a fallback value.
fn append_index(index: usize, c: char, heap_size: usize) -> usize {
    if !c.is_ascii_digit() {
        return index;
    }
    let idx_str = format!("{index}{c}");
    if let Ok(new_index) = idx_str.parse::<usize>() {
        if new_index < heap_size {
            return new_index;
        }
    }
    let c_val = (c as usize) - ('0' as usize);
    if c_val < heap_size {
        return c_val;
    }
    return index;
}

// Return the next Model based on a message sent in Load mode.
fn update_load(
    msg: LoadMsg,
    load_state: LoadState,
    state: SessionState
) -> Model {
    let mode = match msg {
        LoadMsg::Decrement => Mode::Load(load_state.decrement()),
        LoadMsg::Increment => Mode::Load(load_state.increment()),
        LoadMsg::Open => {
            let path = load_state.get_path();
            return Model {
                state: io::init_session_state(path),
                mode: Mode::Normal,
            };
        }
        LoadMsg::New => Mode::Normal,
        LoadMsg::Delete => match load_state.delete() {
            Some(load_state) => Mode::Load(load_state),
            None => Mode::Normal,
        }
    };
    Model { state, mode }
}

// Return the next Model based on a message sent in Normal mode.
fn update_normal(msg: NormalMsg, state: SessionState) -> Model {
    let mode = match msg {
        NormalMsg::StartInput => Mode::Input(InputState::new_add()),
        NormalMsg::StartSelect => {
            match state.heap.size() > 0 {
                true => Mode::Select(0),
                false => Mode::Normal,
            }
        }
        NormalMsg::StartCompare => {
            match state.heap.status() {
                HeapStatus::MultiRoot(item1, item2) => Mode::Compare(
                    Choice {
                        item1: item1.to_string(),
                        item2: item2.to_string(),
                        first_selected: true,
                    }
                ),
                _ => Mode::Normal,
            }
        }
    };
    Model { state, mode }
}

// Return the next Model based on a message sent in Input mode.
fn update_input(
    msg: InputMsg,
    input_state: InputState,
    mut state: SessionState,
) -> Option<Model> {
    let mode = match msg {
        InputMsg::Append(c) => Mode::Input(input_state.append(c)),
        InputMsg::PopChar => Mode::Input(input_state.pop()),
        InputMsg::Submit => {
            let input_state = input_state.update_status();
            if input_state.is_valid() {
                let InputState { input, action } = input_state;
                let text = input.trim().to_string();
                match action {
                    InputAction::Add => {
                        state = state.add(text);
                        Mode::Normal
                    }
                    InputAction::Edit(index) => {
                        state = state.edit(index, text);
                        Mode::Normal
                    }
                    InputAction::Save(_) => {
                        match io::save_new(&state.heap, text) {
                            Ok(()) => return None,
                            Err(_) => {
                                let input_state = InputState::invalid(input);
                                Mode::Input(input_state)
                            }
                        }
                    }
                }
            } else {
                Mode::Input(input_state)
            }
        }
    };
    Some(Model { state, mode })
}

// Return the next Model based on a message sent in Select mode.
fn update_select(msg: SelectMsg, index: usize, state: SessionState) -> Model {
    let mode = match msg {
        SelectMsg::Append(c) => {
            let i = append_index(index, c, state.heap.size());
            Mode::Select(i)
        }
        SelectMsg::Decrement => {
            match index > 0 {
                true => Mode::Select(index - 1),
                false => Mode::Select(index),
            }
        }
        SelectMsg::Increment => {
            match index + 1 < state.heap.size() {
                true => Mode::Select(index + 1),
                false => Mode::Select(index),
            }
        }
        SelectMsg::Confirm => Mode::Selected(index),
    };
    Model { state, mode }
}

// Return the next Model based on a message sent in Selected mode.
fn update_selected(
    msg: SelectedMsg,
    index: usize,
    mut state: SessionState,
) -> Model {
    let mode = match msg {
        SelectedMsg::Edit => {
            let text = state.heap.label_at(index).to_string();
            let input_state = InputState::new_edit(text, index);
            Mode::Input(input_state)
        }
        SelectedMsg::Delete => {
            state = state.delete(index);
            Mode::Normal
        }
    };
    Model { state, mode }
}

// Return the next Model based on a message sent in Compare mode.
fn update_compare(
    msg: CompareMsg,
    choice: Choice,
    mut state: SessionState,
) -> Model {
    let Choice { item1, item2, first_selected } = choice;
    let mode = match msg {
        CompareMsg::Toggle => {
            let toggled = !first_selected;
            Mode::Compare(Choice { item1, item2, first_selected: toggled })
        }
        CompareMsg::Confirm => {
            state = state.merge_pair(first_selected);
            Mode::Normal
        }
    };
    Model { state, mode }
}

// Return the next Model if more action is needed after a Quit message.
fn update_quit(save: bool, state: SessionState) -> Option<Model> {
    if !save {
        return None;
    }
    match &state.maybe_file {
        Some(_) => {
            io::save(state);
            None
        }
        None => {
            let input_state = InputState::new_save();
            let mode = Mode::Input(input_state);
            Some(Model { state, mode })
        }
    }
}

/// Return the next Model based on the `message` and the session `state`.
pub fn update(message: Message, state: SessionState) -> Option<Model> {
    let model = match message {
        Message::Load(msg, load_state) => update_load(msg, load_state, state),
        Message::Normal(msg) => update_normal(msg, state),
        Message::Input(msg, input_state) => return update_input(msg, input_state, state),
        Message::Select(msg, index) => update_select(msg, index, state),
        Message::Selected(msg, index) => update_selected(msg, index, state),
        Message::Compare(msg, choice) => update_compare(msg, choice, state),
        Message::StartQuit => match state.is_changed() {
            true => Model { state, mode: Mode::Save(true) },
            false => return None,
        },
        Message::ToggleSave(save) => Model { state, mode: Mode::Save(!save) },
        Message::Quit(save) => return update_quit(save, state),
        Message::Continue(mode) => Model { state, mode },
    };
    Some(model)
}

