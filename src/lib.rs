use async_trait::async_trait;
use futures::stream::TryStreamExt;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs;
use tokio::process::Command;
use tokio_stream::wrappers::ReadDirStream;

pub struct Each {
    source_dir: PathBuf,
    num_processes: usize,
}

// TODO: Add support for both input modes
// TODO: Add progress bar

// TODO: Add support for source "dir" being a filename with a bunch of lines.
// Consider instead making a separate command that turns a filename with a
// bunch of lines into a bunch of directories with the lines as contents.

impl Each {
    pub fn new(source_dir: PathBuf, num_processes: usize, _recreate: bool, _retries: u32) -> Self {
        Each {
            source_dir,
            num_processes,
        }
    }

    pub async fn run<R: Runner>(&self, runner: &R) -> io::Result<()> {
        let source_dir = fs::read_dir(&self.source_dir).await?;
        let stream = ReadDirStream::new(source_dir);
        stream
            .try_for_each_concurrent(self.num_processes, |source_file| async move {
                let metadata = source_file.metadata().await?;
                if metadata.is_file() {
                    runner.run(&source_file).await
                } else {
                    Ok(())
                }
            })
            .await?;
        Ok(())
    }
}

#[async_trait]
pub trait Runner {
    async fn run(&self, source_file: &fs::DirEntry) -> io::Result<()>;
}

#[derive(Debug)]
pub struct StdinRunner {
    shell: String,
    command: String,
    destination_dir: PathBuf,
}

impl StdinRunner {
    pub fn new(shell: String, command: String, destination_dir: PathBuf) -> Self {
        StdinRunner {
            shell,
            command,
            destination_dir,
        }
    }
}

#[async_trait]
impl Runner for StdinRunner {
    async fn run(&self, source_file: &fs::DirEntry) -> io::Result<()> {
        // TODO(jml): This function has potential for internal parallelism.
        // Better understand how join! and .await work and see if there's any benefit.

        // TODO(jml): Understand whether this actually has any benefit over directly opening the standard file.
        let source_path = source_file.path();
        let in_file = fs::File::open(source_path).await?.into_std().await;

        let mut base_directory = self.destination_dir.clone();
        base_directory.push(source_file.file_name());

        ensure_directory(&base_directory).await?;

        // TODO(jml): 'create' truncates. Actual desired behaviour depends on 'recreate' setting.
        let mut out_path = base_directory.clone();
        out_path.push("out");
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
        Ok(())
    }
}

pub struct FilenameRunner {
    shell: String,
    command: String,
    destination_dir: PathBuf,
}

impl FilenameRunner {
    pub fn new(shell: String, command: String, destination_dir: PathBuf) -> Self {
        FilenameRunner {
            shell,
            command,
            destination_dir,
        }
    }
}

#[async_trait]
impl Runner for FilenameRunner {
    async fn run(&self, source_file: &fs::DirEntry) -> io::Result<()> {
        // TODO(jml): This function duplicates way too much from StdinRunner.
        // I think we probably want to make a factory for Command.
        let source_path = source_file.path();
        let source_path = source_path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Non-unicode filename: {:?}", source_path),
            )
        })?;
        let command = self.command.replace("{}", source_path);

        let mut base_directory = self.destination_dir.clone();
        base_directory.push(source_file.file_name());

        ensure_directory(&base_directory).await?;

        // TODO(jml): 'create' truncates. Actual desired behaviour depends on 'recreate' setting.
        let mut out_path = base_directory.clone();
        out_path.push("out");
        let out_file = fs::File::create(out_path).await?.into_std().await;

        let mut err_path = base_directory.clone();
        err_path.push("err");
        let err_file = fs::File::create(err_path).await?.into_std().await;

        let mut child_process = Command::new(&self.shell)
            .arg("-c")
            .arg(&command)
            .stdout(out_file)
            .stderr(err_file)
            .spawn()?;
        child_process.wait().await?;
        Ok(())
    }
}

/// How the command given to `reach` gets at its input.
#[derive(Debug, PartialEq)]
pub enum InputMode {
    /// The contents of the input file are sent to standard input.
    Stdin,
    /// The name of the input file is passed as a command-line argument.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_mode_parse() {
        assert_eq!(Ok(InputMode::Stdin), "stdin".parse());
        assert_eq!(Ok(InputMode::Filename), "filename".parse());
    }
}
