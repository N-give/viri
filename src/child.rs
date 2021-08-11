use anyhow::Result;
use std::process::Stdio;
use tokio::{
    io::{
        AsyncBufReadExt,
        AsyncWriteExt,
        BufReader,
        BufWriter,
    },
    join,
    process::{
        Command,
        ChildStdout,
        ChildStdin
    },
    runtime::Runtime,
    sync::mpsc::UnboundedReceiver
};

#[derive(Debug)]
pub enum Message {
    Exec(String),
    Kill,
}

pub fn run_child(
    proc: String,
    args: &[String],
    commander: UnboundedReceiver<Message>
) -> Result<()> {
    let mut cmd = Command::new(&proc);
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());

    let rt = Runtime::new()?;

    rt.block_on(async {
        let mut child = cmd.spawn()?;
        let cin = BufWriter::new(
            child.stdin.take().expect("failed to get child input")
        );
        let cout = BufReader::new(
            child.stdout.take().expect("failed to get child output")
        );

        let cin = rt.spawn(child_input(cin, commander));
        let cout = rt.spawn(child_output(cout));

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
    mut commander: UnboundedReceiver<Message>
) -> Result<()> {
    loop {
        let message = commander.recv().await.expect("no message");
        print!("recieved: {:?}\n\r", message);
        match message {
            Message::Exec(i) => {
                // cin.write_all(b"abc").await?;
                cin.write_all(i.as_bytes()).await?;
                cin.flush().await?;
                print!("sent bytes: {:?}\n\r", i.as_bytes());
            }

            Message::Kill => break,
        }
    }
    Ok(())
}

async fn child_output(cout: BufReader<ChildStdout>) -> Result<()> {
    let mut cout = cout.lines();
    while let Some(line) = cout.next_line().await? {
        print!("child output: {}\n\r", line);
    }
    print!("no more entries...\n\r");
    Ok(())
}

