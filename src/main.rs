use reach::{Each, FilenameRunner, InputMode, StdinRunner};

use clap::Clap;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Jonathan M. Lange <jml@mumak.net>")]
struct Opts {
    #[clap(about = "The command to run on those source files")]
    command: String,

    #[clap(about = "The directory containing source files")]
    source: PathBuf,

    #[clap(about = "The destination directory. \
                 Defaults to the name of the input directory with '-results' appended to the end.")]
    destination: Option<PathBuf>,

    #[clap(
        long,
        about = "By default, reach will not attempt to recreate files that have already been successfully processed. \
                 If this is set, existing files will be overwritten."
    )]
    recreate: bool,

    #[clap(
        long,
        about = "How many times reach should retry a process if it fails (exits with a non-zero status). \
                 Previous runs of reach that failed will only ever be counted as a single failure no matter how many times they called the process.",
        default_value = "0"
    )]
    retries: u32,

    #[clap(
        long,
        about = "The shell to use to interpret the command.",
        env = "SHELL"
    )]
    shell: String,

    #[clap(
        short = 'j',
        long,
        about = "The number of child processes to run in parallel"
    )]
    processes: Option<usize>,

    #[clap(
        long,
        about = "How the input file should be passed to the command. \
                 'stdin' means the contents of the input file will be passed to the command's stdin. \
                 'filename' mean that its name will be substituted for the string '{}' in the command. \
                 The default is to use stdin unless '{}' is present in the command.",
        possible_values = &["stdin", "filename"],
    )]
    input_mode: Option<InputMode>,
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let opts: Opts = Opts::parse();
    let source = opts.source.canonicalize().unwrap_or_else(|error| {
        clap::Error::with_description(
            format!("Invalid source directory {:?}: {}", opts.source, error),
            clap::ErrorKind::Io,
        )
        .exit();
    });
    let destination = match opts.destination {
        Some(p) => p,
        None => {
            // TODO: Write a test for this bit.
            let mut file_name = source.file_name().unwrap_or_else(|| {
                clap::Error::with_description(format!("You must provide an explicit destination directory if source directory is {}", source.to_str().unwrap()), clap::ErrorKind::ValueValidation).exit();
            }).to_owned();
            file_name.push("-results");
            let mut dest = source.clone();
            dest.set_file_name(file_name);
            dest
        }
    };
    let destination = destination.canonicalize().unwrap_or_else(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            fs::create_dir_all(&destination).unwrap_or_else(|error| {
                clap::Error::with_description(
                    format!(
                        "Could not create destination directory {:?}: {}",
                        destination, error
                    ),
                    clap::ErrorKind::Io,
                );
            });
            destination
        } else {
            clap::Error::with_description(
                format!("Invalid destination directory {:?}: {}", destination, error),
                clap::ErrorKind::Io,
            )
            .exit();
        }
    });
    let num_processes = opts.processes.unwrap_or_else(num_cpus::get);
    let input_mode = opts.input_mode.unwrap_or(InputMode::Stdin);
    let each = Each::new(opts.source, num_processes, opts.recreate, opts.retries);
    match input_mode {
        InputMode::Stdin => {
            let runner = StdinRunner::new(opts.shell, opts.command, destination);
            each.run(&runner).await
        }
        InputMode::Filename => {
            let runner = FilenameRunner::new(opts.shell, opts.command, destination);
            each.run(&runner).await
        }
    }
}
