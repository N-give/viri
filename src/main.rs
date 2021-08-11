mod child;
mod config;
mod lib;

use anyhow::Result;
use config::get_config;
use lib::{command_mode, insert_mode, normal_mode, Mode, State};
use std::{
    env, error,
    io::{stdin, stdout, BufReader, BufWriter, Stdout, Write},
    process::{Command, Stdio},
    sync::mpsc,
};
use termion::{
    clear, cursor, input::TermRead, raw::IntoRawMode, terminal_size,
};
use tokio::sync::mpsc::unbounded_channel;

fn main() -> Result<(), Box<dyn error::Error + 'static>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let term_in = stdin();
    let mut output = stdout().into_raw_mode()?;

    let (child_send, child_recv) = unbounded_channel();

    let output_t = std::thread::spawn(move || -> Result<()> {
        child::run_child(args[0].clone(), &args[1..], child_recv)
    });

    // let error_t = std::thread::spawn(move || -> Result<()> {
    //     loop {
    //         match child_stderr.read_line() {
    //             Ok(Some(buf)) => {
    //                 err_send.send(buf)?;
    //             }
    //             Ok(None) => break,
    //             Err(_e) => break,
    //         }
    //     }
    //     Ok(())
    // });

    let size = terminal_size()?;
    let mut state: State = State {
        mode: Mode::Normal,
        size,
        input_before: String::new(),
        input_after: String::new(),
        cmd_before: String::new(),
        cmd_after: String::new(),
        output: Vec::with_capacity(size.0 as usize),
        error: String::new(),
        keys: get_config("_filename")?,
        previous: Vec::with_capacity(100),
    };

    write!(output, "{}", clear::All)?;
    output.flush()?;

    // loop {
    for event in term_in.events() {
        let event = event?;
        state = match state.mode {
            Mode::Execute => {
                let cin =
                    format!("{}{}\n", state.input_before, state.input_after);

                child_send.send(child::Message::Exec(cin))?;

                State {
                    mode: Mode::Normal,
                    input_before: String::new(),
                    input_after: String::new(),
                    ..state
                }
            }
            Mode::Command => command_mode(event, state),
            Mode::Insert => insert_mode(event, state),
            Mode::Normal => normal_mode(event, state),
            Mode::Quit => break,
        };

        print_buffer(&mut output, &state)?;

        state = match state.mode {
            Mode::Execute => {
                let cin =
                    format!("{}{}\n", state.input_before, state.input_after);

                child_send.send(child::Message::Exec(cin))?;

                State {
                    mode: Mode::Normal,
                    input_before: String::new(),
                    input_after: String::new(),
                    ..state
                }
            }
            Mode::Quit => break,
            _ => state,
        }
    }

    child_send.send(child::Message::Kill)?;
    output_t.join().unwrap()?;

    write!(
        output,
        "\n{}{}finished\n\r",
        clear::CurrentLine,
        cursor::Goto(1, state.size.1),
    )?;
    output.flush()?;

    Ok(())
}

fn print_buffer(
    output: &mut Stdout,
    state: &State,
) -> Result<(), std::io::Error> {
    write!(output, "{clear_all}", clear_all = clear::All,)?;
    for (i, o) in state.output.iter().enumerate() {
        write!(
            output,
            "{output_pos}",
            output_pos = cursor::Goto(1, 1 + i as u16)
        )?;
        write!(output, "{}", o).unwrap();
    }
    write!(
        output,
        "{state_pos}: {state_mode:?}{command_pos}command: {command}{before_pos}{before}{after}{cursor_pos}",
        state_pos = cursor::Goto(1, state.size.1 - 2),
        state_mode = state.mode,
        command_pos = cursor::Goto(1, state.size.1 - 1),
        command = format!("{}{}", state.cmd_before, state.cmd_after),
        before_pos = cursor::Goto(1, state.size.1),
        before = state.input_before,
        after = state.input_after,
        cursor_pos = cursor::Goto(state.input_before.len() as u16 + 1, state.size.1),
    )?;
    output.flush()?;
    Ok(())
}
