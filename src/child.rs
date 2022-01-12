use super::{output, cursor::Cursor};
use anyhow::Result;
use std::{process::Stdio, sync::mpsc::Sender};
use tokio::{io::{
        AsyncBufReadExt,
        AsyncWriteExt,
        BufReader,
        BufWriter,
    }, join, process::{
        Command,
        ChildStdout,
        ChildStdin
    }, runtime::Runtime, sync::mpsc::{self, UnboundedReceiver}};

#[derive(Debug)]
pub enum Message {
    Exec(String),
    Input(Cursor),
    Kill,
}

pub fn run_child(
    proc: String,
    args: &[String],
    command_recv: UnboundedReceiver<Message>,
    output_send: Sender<String>,
    rt: &Runtime,
) -> Result<()> {
    let mut cmd = Command::new(&proc);
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());

    // let ( send, recv ) = mpsc::channel(25);
    // let console = output::Output::new(recv);

    rt.block_on(async {
        let mut child = cmd.spawn()?;
        let cin = BufWriter::new(
            child.stdin.take().expect("failed to get child input")
        );
        let cout = BufReader::new(
            child.stdout.take().expect("failed to get child output")
        );

        let cin = rt.spawn(child_input(cin, command_recv));
        let cout = rt.spawn(child_output(cout, output_send));
        // let console = rt.spawn(console.run_output());

        // let cin = join!(cin);
        let (cin, cout) = join!(cin, cout);
        cin??;
        cout??;
        child.kill().await?;
        Ok(())
    })
}

async fn child_input(
    mut cin: BufWriter<ChildStdin>,
    mut command_recv: UnboundedReceiver<Message>,
) -> Result<()> {
    loop {
        let message = command_recv.recv().await.expect("no message");
        match message {
            Message::Input(_i) => {
            }

            Message::Exec(i) => {
                cin.write_all(i.as_bytes()).await?;
                cin.flush().await?;
            }

            Message::Kill => break,
        }
    }
    Ok(())
}

async fn child_output(
    cout: BufReader<ChildStdout>,
    console: Sender<String>
) -> Result<()> {
    let mut cout = cout.lines();
    while let Some(line) = cout.next_line().await? {
        console.send(line)?;
    }
    print!("no more entries...\n\r");
    Ok(())
}

