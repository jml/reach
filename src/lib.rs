use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;
use tokio::process::Command;

#[derive(Debug)]
pub struct Each {
    source_dir: PathBuf,
    command: String,
    destination_dir: PathBuf,
    num_processes: usize,
    recreate: bool,
    retries: u32,
    shell: String,
}

// TODO: Add support for both input modes
// TODO: Add progress bar

// TODO: Add support for source "dir" being a filename with a bunch of lines.
// Consider instead making a separate command that turns a filename with a
// bunch of lines into a bunch of directories with the lines as contents.

impl Each {
    pub fn new(
        source_dir: PathBuf,
        command: String,
        destination_dir: PathBuf,
        num_processes: usize,
        recreate: bool,
        retries: u32,
        shell: String,
    ) -> Self {
        Each {
            source_dir,
            command,
            destination_dir,
            num_processes,
            recreate,
            retries,
            shell,
        }
    }

    pub async fn run(self) -> io::Result<()> {
        // TODO(jml): Figure out how to separate 'iterate through files' from 'process files'.

        // TODO(jml): Currently retrieving each file in sequence, and then
        // running each command and waiting for each command. Need instead to
        // run things in parallel.
        let mut source_dir = fs::read_dir(self.source_dir).await?;
        while let Some(child) = source_dir.next_entry().await? {
            let metadata = child.metadata().await?;
            if metadata.is_file() {
                // TODO(jml): Either format the command or pass stdin.
                let mut child_process = Command::new(&self.shell)
                    .arg("-c")
                    .arg(&self.command)
                    .spawn()?;
                child_process.wait().await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum InputMode {
    Stdin,
    Filename,
}

impl FromStr for InputMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stdin" => Ok(InputMode::Stdin),
            "filename" => Ok(InputMode::Filename),
            _ => Err(format!("No such InputMode: {}", s)),
        }
    }
}
