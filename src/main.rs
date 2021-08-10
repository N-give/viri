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

fn main() -> Result<(), Box<dyn error::Error + 'static>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let term_in = stdin();
    let mut output = stdout().into_raw_mode()?;

    let mut child = Command::new(&args[0]);
    child.stdout(Stdio::piped());
    // child.stderr(Stdio::piped());
    child.stdin(Stdio::piped());
    let mut child = child.spawn()?;

    let mut c_stdout = BufReader::new(
        child.stdout.take().expect("failed to get child stdout"),
    );
    // let mut c_stderr = BufReader::new(
    //     child.stderr.take().expect("faild to take child stderr"),
    // );
    let mut c_stdin =
        BufWriter::new(child.stdin.take().expect("failed to get child stdin"));

    let (output_send, output_recv) = mpsc::channel();
    // let (err_send, err_recv) = std::sync::mpsc::channel();
    let (kill, die) = mpsc::channel::<()>();

    let output_t = std::thread::spawn(move || -> Result<()> {
        loop {
            if let Ok(_) = die.try_recv() {
                break;
            }

            match c_stdout.read_line() {
                Ok(Some(buf)) => {
                    output_send.send(buf)?;
                }
                Ok(None) => break,
                Err(_e) => break,
            }
        }
        Ok(())
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
        if let Ok(out) = output_recv.try_recv() {
            state.output.push(out);
            print_buffer(&mut output, &state)?;
        }

        /*
        if let Ok(e) = err_recv.try_recv() {
            state.error = e;
            print_buffer(&mut output, &state).unwrap();
        }
        */

        state = match state.mode {
            Mode::Execute => {
                c_stdin.write_all(
                    format!("{}{}\n", state.input_before, state.input_after)
                        .as_bytes(),
                )?;
                c_stdin.flush()?;
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
        print_buffer(&mut output, &state).unwrap();
    }

    kill.send(())?;
    child.kill()?;
    output_t.join().unwrap()?;

    // output_t.join().expect("failed to join output thread")?;
    // error_t.join().expect("failed to join error thread")?;

    write!(
        output,
        "\n{}{}finished\n",
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
