use reach::{Config, InputMode};

use clap::Clap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

fn parse_options(opts: Opts) -> Result<Config, clap::Error> {
    let source = opts.source.canonicalize().map_err(|error| {
        clap::Error::with_description(
            format!("Invalid source directory {:?}: {}", opts.source, error),
            clap::ErrorKind::Io,
        )
    })?;
    // TODO(jml): There feels like there should be some more idiomatic way of doing this.
    let destination = match opts.destination {
        Some(p) => Ok(p),
        None => get_destination_dir(&source),
    }?;
    let destination = ensure_destination_directory(destination)?;
    let num_processes = opts.processes.unwrap_or_else(num_cpus::get);
    // TODO(jml): Automatically choose Filename input mode if {} present in command.
    let input_mode = opts.input_mode.unwrap_or(InputMode::Stdin);
    Ok(Config {
        command: opts.command,
        shell: opts.shell,
        source_dir: source,
        destination_dir: destination,
        num_processes,
        input_mode,
        recreate: opts.recreate,
        retries: opts.retries,
    })
}

/// Make up a path to the destination directory from the source directory.
/// `foo` becomes `foo-results`
fn get_destination_dir(source_dir: &Path) -> Result<PathBuf, clap::Error> {
    // TODO: Write a test for this bit.
    let mut file_name = source_dir
        .file_name()
        .ok_or_else(|| {
            clap::Error::with_description(
                format!(
                    "You must provide an explicit destination directory if source directory is {}",
                    source_dir.to_str().unwrap()
                ),
                clap::ErrorKind::ValueValidation,
            )
        })?
        .to_owned();
    file_name.push("-results");
    let mut dest = source_dir.to_path_buf();
    dest.set_file_name(file_name);
    Ok(dest)
}

/// Create the destination directory if it doesn't exist.
fn ensure_destination_directory(destination: PathBuf) -> Result<PathBuf, clap::Error> {
    destination.canonicalize().or_else(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            fs::create_dir_all(&destination).map_err(|error| {
                clap::Error::with_description(
                    format!(
                        "Could not create destination directory {:?}: {}",
                        destination, error
                    ),
                    clap::ErrorKind::Io,
                )
            })?;
            Ok(destination)
        } else {
            Err(clap::Error::with_description(
                format!("Invalid destination directory {:?}: {}", destination, error),
                clap::ErrorKind::Io,
            ))
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let opts: Opts = Opts::parse();
    let config = parse_options(opts).unwrap_or_else(|err| err.exit());
    let progress_bar = reach::default_progress_bar();
    reach::run(config, progress_bar).await
}
