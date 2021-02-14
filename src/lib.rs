use std::path::PathBuf;
use std::str::FromStr;

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
