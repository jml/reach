use futures::stream::TryStreamExt;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs;
use tokio::process::Command;
use tokio_stream::wrappers::ReadDirStream;

#[derive(Debug)]
pub struct Each {
    shell: String,
    command: String,
    source_dir: PathBuf,
    destination_dir: PathBuf,
    num_processes: usize,
}

// TODO: Add support for both input modes
// TODO: Add progress bar

// TODO: Add support for source "dir" being a filename with a bunch of lines.
// Consider instead making a separate command that turns a filename with a
// bunch of lines into a bunch of directories with the lines as contents.

impl Each {
    pub fn new(
        shell: String,
        command: String,
        source_dir: PathBuf,
        destination_dir: PathBuf,
        num_processes: usize,
        _recreate: bool,
        _retries: u32,
    ) -> Self {
        Each {
            shell,
            command,
            source_dir,
            destination_dir,
            num_processes,
        }
    }

    pub async fn run(&self) -> io::Result<()> {
        let source_dir = fs::read_dir(&self.source_dir).await?;
        let stream = ReadDirStream::new(source_dir);
        stream
            .try_for_each_concurrent(self.num_processes, |source_file| async move {
                let metadata = source_file.metadata().await?;
                if metadata.is_file() {
                    run_process(
                        &source_file,
                        &self.destination_dir,
                        &self.shell,
                        &self.command,
                    )
                    .await
                } else {
                    Ok(())
                }
            })
            .await?;
        Ok(())
    }
}

async fn run_process(
    source_file: &fs::DirEntry,
    destination_dir: &Path,
    shell: &str,
    command: &str,
) -> io::Result<()> {
    // TODO(jml): Understand whether this actually has any benefit over directly opening the standard file.
    let source_path = source_file.path();
    let in_file = fs::File::open(source_path).await?.into_std().await;

    let mut base_directory = destination_dir.to_path_buf();
    base_directory.push(source_file.file_name());

    ensure_directory(&base_directory).await?;
    let mut out_path = base_directory.clone();
    out_path.push("out");

    // TODO(jml): 'create' truncates. Actual desired behaviour depends on 'recreate' setting.
    let out_file = fs::File::create(out_path).await?.into_std().await;
    let mut err_path = base_directory.clone();
    err_path.push("err");
    let err_file = fs::File::create(err_path).await?.into_std().await;

    let mut child_process = Command::new(shell)
        .arg("-c")
        .arg(command)
        .stdin(in_file)
        .stdout(out_file)
        .stderr(err_file)
        .spawn()?;
    child_process.wait().await?;
    Ok(())
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

/// Asynchronously ensure a directory exists.
async fn ensure_directory(p: &Path) -> io::Result<()> {
    let result = fs::create_dir_all(p).await;
    match result {
        Ok(()) => Ok(()),
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => Ok(()),
            _ => Err(error),
        },
    }
}
