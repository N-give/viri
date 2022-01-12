
use super::{cursor::Cursor, state::State, state::Source};
use anyhow::Result;
use termion::{clear, raw::RawTerminal, cursor as tcursor};
use std::io::Write;
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum OutputType {
    Child(String),
    Input(Cursor),
    Exec(String),
}

#[derive(Debug)]
pub struct Output {
    recv: Receiver<OutputType>,
    input: Cursor,
    output: Vec<String>,
}

impl Output {
    pub fn new(recv: Receiver<OutputType>) -> Self {
        Output { recv, output: Vec::new(), input: Cursor::new() }
    }

    pub async fn run_output(mut self) -> Result<()> {
        while let Some(m) = self.recv.recv().await {
            // self.print_buffer(m).await?;
            match m {
                OutputType::Child(s) => {
                    self.output.push(s.clone());
                }

                OutputType::Input(c) => {
                    self.input = c;
                    // self.print_input(&c).await?;
                    // self.input = c.clone();
                    // let mut buf = format!("{}{}", cursor::Goto(1, state.size.1), clear::All);
                    // buf.push_str(&format!("{}{:?}", cursor::Goto(1, state.size.1 - 1), state.mode));
                    // buf.push_str(&format!("{}", cursor::Goto(state.input.pos() as u16, state.size.1)));
                    // println!("{}", buf);
                }

                OutputType::Exec(s) => {
                    self.input = Cursor::new();
                    self.output.push(s.trim_end().to_string());
                }
            }
        self.print_buffer().await?;

        }
        Ok(())
    }

    /*
    async fn print_input(&self, input: &Cursor) -> Result<(), std::io::Error> {
        let mut buf = String::new();
        buf.push_str(&input.to_string());
        buf.push_str(&tcursor::Goto(input.pos() as u16, 25).to_string());
        println!("{}", buf);
        Ok(())
    }
    */

    async fn print_buffer(
        &self,
    ) -> Result<(), std::io::Error> {
        let mut buf = format!("{}{}", clear::All, tcursor::Goto(1, 1));

        self.output
            .iter()
            .skip(if self.output.len() > 24 { self.output.len() - 24 } else { 0 })
            .for_each(|o: &String| {
                buf.push_str(o);
                buf.push('\n');
                buf.push('\r');
            });

        buf.push_str(
            &tcursor::Goto(
                1,
                if self.output.len() > 24 {
                    25
                } else {
                    self.output.len() as u16  + 1
                }
            ).to_string()
        );
        buf.push_str(&self.input.to_string());
        buf.push_str(
            &tcursor::Goto(
                self.input.pos() as u16,
                if self.output.len() > 24 {
                    25
                } else {
                    self.output.len() as u16
                }
            ).to_string()
        );
        println!("{}", buf);

        // println!("{}\n\r", buf);

        // output.write_all(&mut buf.as_bytes()).await?;

        /*
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
           after = state.input_after.chars().rev().collect::<String>(),
           cursor_pos = cursor::Goto(state.input_before.len() as u16 + 1, state.size.1),
           )?;
           output.flush()?;
           */
        Ok(())
    }
}

pub fn print_buffer(
    output: &mut RawTerminal<std::io::Stdout>,
    state: &State,
) -> Result<(), std::io::Error> {
    let mut buf = format!("{}{}", clear::All, tcursor::Goto(1, 1));

    state.history
        .values()
        .skip(if state.history.len() > (state.size.1 as usize) {
            state.history.len() - (state.size.1 as usize)
        } else {
            0
        })
        .map(|o: &Source| match o {
            Source::ChildInput(c) => c,
            Source::ChildOutput(c) => c,
        })
        .for_each(|o: &Cursor| {
            buf.push_str(&o.to_string());
            buf.push('\n');
            buf.push('\r');
        });

    buf.push_str(
        &tcursor::Goto(
            1,
            if state.history.len() > (state.size.1 as usize) {
                state.size.1
            } else {
                state.history.len() as u16 + 1
            }
        ).to_string()
    );
    buf.push_str(&state.input.to_string());
    buf.push_str(
        &tcursor::Goto(
            state.input.pos() as u16,
            if state.history.len() > (state.size.1 as usize) {
                state.size.1
            } else {
                state.history.len() as u16 + 1
            }
        ).to_string()
    );
    write!(output, "{}", buf)?;
    output.flush()?;
    Ok(())

    // println!("{}\n\r", buf);

    // output.write_all(&mut buf.as_bytes()).await?;

    /*
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
       after = state.input_after.chars().rev().collect::<String>(),
       cursor_pos = cursor::Goto(state.input_before.len() as u16 + 1, state.size.1),
       )?;
       output.flush()?;
       */
}
