mod child;
mod config;
mod cursor;
mod output;
mod state;

use anyhow::Result;
use config::get_config;
use cursor::Cursor;
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
// use output::{print_buffer, OutputType};
use state::{
    command_mode, insert_mode, normal_mode, History, Mode, Source, State,
};
use std::{
    env, error,
    io::{stdin, stdout, Write},
    os::unix::prelude::AsRawFd,
    sync::mpsc::channel,
    time::Duration,
};
use termion::{
    clear,
    // event::{Event, Key},
    input::TermRead,
    raw::IntoRawMode,
    terminal_size,
};
use tokio::{runtime::Runtime, sync::mpsc::unbounded_channel};

fn main() -> Result<(), Box<dyn error::Error + 'static>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let term_in = stdin();
    let mut term_events = stdin().events();
    let mut output = stdout().into_raw_mode()?;

    let (child_send, child_recv) = unbounded_channel();
    let (parent_send, parent_recv) = channel();

    let rt = Runtime::new()?;
    // let handle = rt.handle().clone();
    let output_t = std::thread::spawn(move || -> Result<()> {
        child::run_child(
            args[0].clone(),
            &args[1..],
            child_recv,
            parent_send,
            &rt,
        )
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
        input: Cursor::new(),
        command: Cursor::new(),
        keys: get_config("_filename")?,
        history: History::new(),
    };

    write!(output, "{}", clear::All)?;
    output.flush()?;

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

    const TERM_EVENT: Token = Token(0);

    poll.registry().register(
        &mut SourceFd(&term_in.as_raw_fd()),
        TERM_EVENT,
        Interest::READABLE,
    )?;

    'main: loop {
        if let Mode::Quit = state.mode {
            break 'main;
        }

        if let Ok(cout) = parent_recv.try_recv() {
            state
                .history
                .push(Source::ChildOutput(Cursor::from(cout, String::new())));
            output::print_buffer(&mut output, &state)?;
        }

        poll.poll(&mut events, Some(Duration::from_millis(30)))?;

        for event in events.iter() {
            match event.token() {
                TERM_EVENT => {
                    if let Some(term_event) = term_events.next() {
                        let term_event = term_event?;
                        state = match state.mode {
                            Mode::Execute => {
                                let cin = format!("{}\n", state.input);
                                child_send
                                    .send(child::Message::Exec(cin.clone()))?;

                                state.history.push(Source::ChildInput(
                                    state.input.to_owned(),
                                ));
                                state.mode = Mode::Normal;
                                state.input = Cursor::new();
                                state
                            }

                            Mode::Command => command_mode(term_event, state),
                            Mode::Insert => insert_mode(term_event, state),
                            Mode::Normal => normal_mode(term_event, state),
                            Mode::Quit => break 'main,
                        };

                        output::print_buffer(&mut output, &state)?;

                        state = match state.mode {
                            Mode::Execute => {
                                let cin = format!("{}\n", state.input);
                                child_send
                                    .send(child::Message::Exec(cin.clone()))?;

                                state.history.push(Source::ChildInput(
                                    state.input.to_owned(),
                                ));
                                state.mode = Mode::Normal;
                                state.input = Cursor::new();
                                state
                            }
                            Mode::Quit => break 'main,
                            _ => {
                                child_send.send(child::Message::Input(
                                    state.input.clone(),
                                ))?;
                                state
                            }
                        };
                    }
                }
                _ => break 'main,
            }
        }
    }

    // loop {
    /*
    for event in term_in.events() {
        let event = event?;
        state = match state.mode {
            Mode::Execute => {
                let cin = format!("{}\n", state.input);
                child_send.send(child::Message::Exec(cin.clone()))?;

                state.history.push(state.input.to_owned());
                state.mode = Mode::Normal;
                state.input = Cursor::new();
                state
            }

            Mode::Command => command_mode(event, state),
            Mode::Insert => insert_mode(event, state),
            Mode::Normal => normal_mode(event, state),
            Mode::Quit => break,
        };

        // output::print_buffer(&mut output, &state)?;

        state = match state.mode {
            Mode::Execute => {
                let cin = format!("{}\n", state.input);
                child_send.send(child::Message::Exec(cin.clone()))?;

                state.history.push(state.input.to_owned());
                state.mode = Mode::Normal;
                state.input = Cursor::new();
                state
            }
            Mode::Quit => break,
            _ => {
                child_send.send(child::Message::Input(state.input.clone()))?;
                state
            }
        };

        /*
        let cloned = state.clone();
        handle.spawn(async move {
            print_buffer(OutputType::Input(cloned)).await.unwrap();
        });
        */
    }
    */

    child_send.send(child::Message::Kill)?;
    output_t.join().unwrap()?;
    drop(output);

    println!("\nfinished\n\n",);

    Ok(())
}
