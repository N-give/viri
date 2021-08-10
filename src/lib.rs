use phf::phf_map;
use std::collections::HashMap;
use termion::event::{Event, Key};
// use tokio::{
//     io::{BufReader, BufWriter, AsyncBufReadExt},
//     process::Command,
// };

static FUNCTIONS: phf::Map<&'static str, fn(State) -> State> = phf_map! {
    "Enter" => enter,
    "Insert" => insert,
    "InsertStart" => insert_start,
    "Append" => append,
    "AppendEnd" => append_end,
    "DeleteLineInsert" => delete_line_insert,
    "CursorLeft" => cursor_left,
    "CursorRight" => cursor_right,
    "CursorRightWord" => cursor_right_word,
    "CursorLeftWord" => cursor_left_word,
    "CursorRightAll" => cursor_right_all,
    "CursorLeftAll" => cursor_left_all,
    "Quit" => quit,
};

#[derive(Debug)]
pub struct State {
    pub mode: Mode,
    pub size: (u16, u16),
    pub input_before: String,
    pub input_after: String,
    pub cmd_before: String,
    pub cmd_after: String,
    pub output: Vec<String>,
    pub error: String,
    pub keys: HashMap<String, String>,
    pub previous: Vec<String>,
}

#[derive(Debug)]
pub enum Mode {
    Execute,
    Command,
    Insert,
    Normal,
    Quit,
}

pub fn normal_mode(evt: Event, mut state: State) -> State {
    match evt {
        Event::Key(Key::Esc) => {
            state.cmd_before = String::new();
            state.cmd_after = String::new();
            state
        }

        Event::Key(Key::Char('\n')) => {
            state.mode = Mode::Execute;
            state
        }

        Event::Key(Key::Char(':')) => {
            state.mode = Mode::Command;
            state
        }

        Event::Key(Key::Char(c)) => {
            state.cmd_before.push(c);
            if let Some(cmd) = state
                .keys
                .get(&format!("{}{}", state.cmd_before, state.cmd_after))
            {
                match FUNCTIONS.get::<str>(cmd) {
                    Some(f) => {
                        state.cmd_before = String::new();
                        state.cmd_after = String::new();
                        state = f(state);
                    }
                    None => unreachable!(),
                }
            }
            state
        }

        _ => {
            unimplemented!("currently unimplemented key entered");
        }
    }
}

pub fn command_mode(evt: Event, mut state: State) -> State {
    match evt {
        Event::Key(Key::Esc) => {
            state.cmd_before = String::new();
            state.cmd_after = String::new();
            state.mode = Mode::Normal;
        }

        Event::Key(Key::Char('\n')) => match state
            .keys
            .get(&format!("{}{}", state.cmd_before, state.cmd_after))
        {
            Some(cmd) => match FUNCTIONS.get::<str>(cmd) {
                Some(f) => {
                    state.cmd_before = String::new();
                    state.cmd_after = String::new();
                    state = f(state);
                }
                None => unreachable!(),
            },

            None => {
                state.cmd_before = String::new();
                state.cmd_after = String::new();
                state.mode = Mode::Normal;
            }
        },

        Event::Key(Key::Backspace) => {
            state.cmd_before = backspace(state.cmd_before);
        }

        Event::Key(Key::Delete) => {
            state.cmd_after = delete(state.cmd_after);
        }

        Event::Key(Key::Left) => {
            let (before, after) =
                _cursor_left(state.cmd_before, state.cmd_after);
            state.cmd_before = before;
            state.cmd_after = after;
        }

        Event::Key(Key::Right) => {
            let (before, after) =
                _cursor_right(state.cmd_before, state.cmd_after);
            state.cmd_before = before;
            state.cmd_after = after;
        }

        Event::Key(Key::Char(c)) => {
            state.cmd_before.push(c);
        }

        _ => {}
    }
    state
}

pub fn insert_mode(evt: Event, mut state: State) -> State {
    match evt {
        Event::Key(Key::Esc) => {
            state = cursor_left(state);
            state.mode = Mode::Normal;
        }

        Event::Key(Key::Char('\n')) => {
            state.mode = Mode::Execute;
        }

        Event::Key(Key::Backspace) => {
            state.input_before = backspace(state.input_before);
        }

        Event::Key(Key::Delete) => {
            state.input_after = delete(state.input_after);
        }

        Event::Key(Key::Left) => {
            state = cursor_left(state);
        }

        Event::Key(Key::Right) => {
            state = cursor_right(state);
        }

        Event::Key(Key::Char(c)) => {
            state.input_before.push(c);
        }

        _ => {
            panic!("Unrecognized event");
        }
    }
    state
}

fn enter(mut state: State) -> State {
    state.mode = Mode::Execute;
    state
}

/*
 * into insert mode
 */

fn insert(state: State) -> State {
    State {
        mode: Mode::Insert,
        ..state
    }
}

fn insert_start(mut state: State) -> State {
    state.input_before.push_str(&state.input_after);
    state.input_after = state.input_before;
    state.input_before = String::new();
    State {
        mode: Mode::Insert,
        ..state
    }
}

fn append(mut state: State) -> State {
    if state.input_after.len() > 0 {
        state.input_before.push(state.input_after.remove(0));
    }
    State {
        mode: Mode::Insert,
        ..state
    }
}

fn append_end(mut state: State) -> State {
    state.input_before.push_str(&state.input_after);
    state.input_after = String::new();
    State {
        mode: Mode::Insert,
        ..state
    }
}

fn delete_line_insert(mut state: State) -> State {
    state.input_before = String::new();
    state.input_after = String::new();
    State {
        mode: Mode::Insert,
        ..state
    }
}

/*
 * insert mode none character keys
 */

fn backspace(mut before: String) -> String {
    before.pop();
    before
}

/*
fn backspace(mut state: State) -> State {
    state.input_before.pop();
    state
}
*/

fn delete(mut after: String) -> String {
    if after.len() > 0 {
        after.remove(0);
    }
    after
}

/*
 * Normal mode movements
 */
fn cursor_left(mut state: State) -> State {
    let (before, after) = _cursor_left(state.input_before, state.input_after);
    state.input_before = before;
    state.input_after = after;
    state
}

fn _cursor_left(mut before: String, mut after: String) -> (String, String) {
    if let Some(prev) = before.pop() {
        after.insert(0, prev);
    }
    (before, after)
}

fn cursor_right(mut state: State) -> State {
    let (before, after) = _cursor_right(state.input_before, state.input_after);
    state.input_before = before;
    state.input_after = after;
    state
}

fn _cursor_right(mut before: String, mut after: String) -> (String, String) {
    if after.len() > 0 {
        before.push(after.remove(0));
    }
    (before, after)
}

fn cursor_left_word(mut state: State) -> State {
    let pos = match state.input_before.trim_end().rfind(char::is_whitespace) {
        Some(i) => i + 1,
        None => 0,
    };
    state
        .input_after
        .insert_str(0, &state.input_before[pos..state.input_before.len()]);
    state.input_before.truncate(pos);
    state
}

fn cursor_right_word(mut state: State) -> State {
    let shift = match state.input_after.find(char::is_whitespace) {
        Some(pos) => state.input_after.drain(..=pos).collect::<String>(),
        None => state
            .input_after
            .drain(..state.input_after.len() - 1)
            .collect::<String>(),
    };
    state.input_before.push_str(&shift);
    state
}

fn cursor_right_all(mut state: State) -> State {
    state.input_before.push_str(&state.input_after);
    state.input_after = String::new();
    state
}

fn cursor_left_all(mut state: State) -> State {
    state.input_after.insert_str(0, &state.input_before);
    state.input_before = String::new();
    state
}

/*
 * quit application
 */

fn quit(state: State) -> State {
    State {
        mode: Mode::Quit,
        ..state
    }
}
