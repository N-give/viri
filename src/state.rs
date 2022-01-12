use super::cursor::Cursor;
use phf::phf_map;
use std::{clone, collections::HashMap, slice::Iter};
use termion::event::{Event, Key};

static FUNCTIONS: phf::Map<&'static str, fn(State) -> State> = phf_map! {
    "Enter" => enter,
    "Insert" => insert,
    "InsertStart" => insert_start,
    "Append" => append,
    "AppendEnd" => append_end,
    "DeletePosInsert" => delete_pos_insert,
    "DeleteLineInsert" => delete_line_insert,
    "CursorLeft" => |mut state: State| {
        state.input.left_char();
        state
    },
    "CursorRight" => |mut state: State| {
        state.input.right_char();
        state
    },
    "CursorRightWord" => |mut state: State| {
        state.input.right_word();
        state
    },
    "CursorLeftWord" => |mut state: State| {
        state.input.left_word();
        state
    },
    "CursorRightAll" => |mut state: State| {
        state.input.right_all();
        state
    },
    "CursorLeftAll" => |mut state: State| {
        state.input.left_all();
        state
    },
    "ClearAfterCursor" => clear_after_insert,
    "Quit" => quit,
};

#[derive(Clone, Debug)]
pub struct State {
    pub mode: Mode,
    pub size: (u16, u16),
    pub input: Cursor,
    pub command: Cursor,
    pub keys: HashMap<String, String>,
    pub history: History,
}

#[derive(Clone, Debug)]
pub enum Source {
    ChildInput(Cursor),
    ChildOutput(Cursor),
}

#[derive(Clone, Debug)]
pub struct History {
    history: Vec<Source>,
    pos: usize,
}

impl History {
    pub fn new() -> Self {
        let history = Vec::with_capacity(100);
        History { history, pos: 0 }
    }

    pub fn push(&mut self, value: Source) {
        self.history.push(value);
    }

    pub fn values(&self) -> HistoryIterator {
        HistoryIterator {
            iter: self.history.iter(),
        }
    }

    pub fn get_prev(&mut self) -> Cursor {
        // if self.pos < self.history.len() {
        //     self.pos += 1;
        // }
        let hist = self
            .history
            .iter()
            .filter_map(|h| match h {
                Source::ChildInput(c) => Some(c),
                Source::ChildOutput(_) => None,
            })
            .rev()
            .take(self.pos + 1)
            .collect::<Vec<&Cursor>>();

        if self.pos < hist.len() {
            self.pos += 1;
        }
        (*hist.last().unwrap_or(&&Cursor::new())).clone()
    }

    pub fn get_next(&mut self) -> Cursor {
        if self.pos > 0 {
            self.pos -= 1;
        }
        self.history
            .iter()
            .filter_map(|h| match h {
                Source::ChildInput(c) => Some(c),
                Source::ChildOutput(_) => None,
            })
            .rev()
            .take(self.pos)
            .last()
            .unwrap_or(&Cursor::new())
            .clone()
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }
}

pub struct HistoryIterator<'a> {
    iter: std::slice::Iter<'a, Source>,
}

impl<'a> Iterator for HistoryIterator<'a> {
    type Item = &'a Source;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/*
impl IntoIterator for History {
    type Item = Cursor;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.history.into_iter()
    }
}
*/

#[derive(Clone, Debug)]
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
            state.command = Cursor::new();
            state
        }

        Event::Key(Key::Char('\n')) => {
            state.history.pos = 0;
            state.mode = Mode::Execute;
            state
        }

        Event::Key(Key::Char(':')) => {
            state.mode = Mode::Command;
            state
        }

        Event::Key(Key::Char('k')) => {
            // state.history.push(state.input);
            state.input = state.history.get_prev();
            state
        }

        Event::Key(Key::Char('j')) => {
            // state.history.push(state.input);
            state.input = state.history.get_next();
            state
        }

        Event::Key(Key::Char(c)) => {
            state.command.insert(c);
            if let Some(cmd) = state.keys.get(&state.command.to_string()) {
                match FUNCTIONS.get::<str>(cmd) {
                    Some(f) => {
                        state.command = Cursor::new();
                        state = f(state);
                    }
                    None => unreachable!("function not found"),
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
            state.command = Cursor::new();
            state.mode = Mode::Normal;
        }

        Event::Key(Key::Char('\n')) => {
            match state.keys.get(&state.command.to_string()) {
                Some(cmd) => match FUNCTIONS.get::<str>(cmd) {
                    Some(f) => {
                        state.command = Cursor::new();
                        state = f(state);
                    }
                    None => unreachable!(),
                },

                None => {
                    state.command = Cursor::new();
                    state.mode = Mode::Normal;
                }
            }
        }

        Event::Key(Key::Backspace) => {
            state.command.backspace();
        }

        Event::Key(Key::Delete) => {
            state.command.delete();
        }

        Event::Key(Key::Left) => {
            state.command.left_char();
        }

        Event::Key(Key::Right) => {
            state.command.right_char();
        }

        Event::Key(Key::Char(c)) => {
            state.command.insert(c);
        }

        _ => {}
    }
    state
}

pub fn insert_mode(evt: Event, mut state: State) -> State {
    match evt {
        Event::Key(Key::Esc) => {
            state.input.left_char();
            state.mode = Mode::Normal;
        }

        Event::Key(Key::Char('\n')) => {
            state.history.pos = 0;
            state.mode = Mode::Execute;
        }

        Event::Key(Key::Backspace) => {
            state.input.backspace();
        }

        Event::Key(Key::Delete) => {
            state.input.delete();
        }

        Event::Key(Key::Left) => {
            state.input.left_char();
        }

        Event::Key(Key::Right) => {
            state.input.right_char();
        }

        Event::Key(Key::Char(c)) => {
            state.input.insert(c);
        }

        _ => {
            panic!("Unrecognized event");
        }
    }
    state
}

fn enter(mut state: State) -> State {
    state.history.pos = 0;
    state.mode = Mode::Execute;
    state
}

/*
 * into insert mode
 */

fn insert(mut state: State) -> State {
    state.mode = Mode::Insert;
    state
}

fn insert_start(mut state: State) -> State {
    state.input.left_all();
    state.mode = Mode::Insert;
    state
}

fn append(mut state: State) -> State {
    state.input.right_char();
    state.mode = Mode::Insert;
    state
}

fn append_end(mut state: State) -> State {
    state.input.right_all();
    state.mode = Mode::Insert;
    state
}

fn delete_pos_insert(mut state: State) -> State {
    state.input.delete_pos();
    state.mode = Mode::Insert;
    state
}

fn delete_line_insert(mut state: State) -> State {
    state.input = Cursor::new();
    state.mode = Mode::Insert;
    state
}

fn clear_after_insert(mut state: State) -> State {
    state.input.clear_after();
    state.mode = Mode::Insert;
    state
}

/*
 * quit application
 */

fn quit(mut state: State) -> State {
    state.mode = Mode::Quit;
    state
}

/*
 * Tests
 */

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_cursor_right() {
        let before = "console.log".to_string();
        let after = r#"("hello world");"#.to_string().chars().rev().collect();
        assert_eq!(
            (
                "console.log(".to_string(),
                r#""hello world");"#
                    .to_string()
                    .chars()
                    .rev()
                    .collect::<String>()
            ),
            _cursor_right(before, after)
        );
    }

    #[test]
    fn move_cursor_left() {
        let before = "console.log".to_string();
        let after = r#"("hello world");"#.to_string().chars().rev().collect();
        assert_eq!(
            (
                "console.lo".to_string(),
                r#"g("hello world");"#
                    .to_string()
                    .chars()
                    .rev()
                    .collect::<String>()
            ),
            _cursor_left(before, after)
        );
    }
}
*/
