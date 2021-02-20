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

    pub async fn run(&self) -> io::Result<()> {
        // TODO(jml): Figure out how to separate 'iterate through files' from 'process files'.

        // TODO(jml): Currently retrieving each file in sequence, and then
        // running each command and waiting for each command. Need instead to
        // run things in parallel.
        let mut source_dir = fs::read_dir(&self.source_dir).await?;
        while let Some(source_file) = source_dir.next_entry().await? {
            let metadata = source_file.metadata().await?;
            if metadata.is_file() {
                // TODO(jml): Either format the command or pass stdin.
                let path = source_file.path();
                // TODO(jml): Understand whether this actually has any benefit over directly opening the standard file.
                let async_file = fs::File::open(path).await?;
                let in_file = async_file.into_std().await;

                let mut base_directory = self.destination_dir.clone();
                base_directory.push(source_file.file_name());
                // TODO(jml): Instead of looking before leaping, check the error and only re-raise if file exists.
                if !base_directory.exists() {
                    // TODO(jml): create_dir_all is probably inefficient,
                    // since we can probably assume that the destination directory exists.
                    fs::create_dir_all(&base_directory).await?;
                }
                let mut out_path = base_directory.clone();
                out_path.push("out");
                // TODO(jml): 'create' truncates. Actual desired behaviour depends on 'recreate' setting.
                let out_file = fs::File::create(out_path).await?.into_std().await;
                let mut err_path = base_directory.clone();
                err_path.push("err");
                let err_file = fs::File::create(err_path).await?.into_std().await;

                let mut child_process = Command::new(&self.shell)
                    .arg("-c")
                    .arg(&self.command)
                    .stdin(in_file)
                    .stdout(out_file)
                    .stderr(err_file)
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
