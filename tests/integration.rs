use reach;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs, io};
use tempfile;
use tokio;

fn new_test_config<C, S, D>(
    command: C,
    source_dir: S,
    dest_dir: D,
    input_mode: reach::InputMode,
) -> reach::Config
where
    C: Into<String>,
    S: Into<PathBuf>,
    D: Into<PathBuf>,
{
    reach::Config {
        command: command.into(),
        shell: env::var("SHELL").unwrap(),
        source_dir: source_dir.into(),
        destination_dir: dest_dir.into(),
        input_mode,
        num_processes: 1,
        recreate: true,
        retries: 1,
    }
}

/// Smoke test for each.
///
/// If we run against an empty input directory, we do not get any errors.
/// Not actually an interesting test in itself, but rather a pattern for tests to come,
/// and a way of exploring our public API.
#[tokio::test]
async fn test_stdin_empty() -> io::Result<()> {
    let source = tempfile::tempdir()?;
    let destination = tempfile::tempdir()?;
    reach::run(new_test_config(
        "cat",
        source.path(),
        destination.path(),
        reach::InputMode::Stdin,
    ))
    .await
}

/// Basic test for stdin processing happy path.
///
/// We use `cat` as our command.
/// The destination directory has a file in `out` matching each file in our source directory.
/// All of the `err` files are empty,
/// and the `status` files don't exist, because we haven't implemented them.
#[tokio::test]
async fn test_stdin() -> io::Result<()> {
    let source = tempfile::tempdir()?;
    let source_path = source.path();
    let file1_path = source_path.join("file1.txt");
    let mut file1 = fs::File::create(file1_path)?;
    writeln!(file1, "Arbitrary content for file one")?;
    let file2_path = source_path.join("file2.txt");
    let mut file2 = fs::File::create(file2_path)?;
    writeln!(file2, "Arbitrary content for file two")?;

    let destination = tempfile::tempdir()?;
    reach::run(new_test_config(
        "cat",
        source.path(),
        destination.path(),
        reach::InputMode::Stdin,
    ))
    .await?;

    let destination_path = destination.path();
    let mut filenames = fs::read_dir(destination_path)?
        .map(|res| res.map(|e| e.file_name()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    filenames.sort();

    assert_eq!(vec!["file1.txt", "file2.txt"], filenames);
    assert_eq!(
        "Arbitrary content for file one\n",
        String::from_utf8_lossy(&fs::read(destination_path.join("file1.txt/out"))?)
    );
    assert_eq!(
        "",
        String::from_utf8_lossy(&fs::read(destination_path.join("file1.txt/err"))?)
    );
    assert!(!destination_path.join("file1/status").exists());
    assert_eq!(
        "Arbitrary content for file two\n",
        String::from_utf8_lossy(&fs::read(destination_path.join("file2.txt/out"))?)
    );
    assert_eq!(
        "",
        String::from_utf8_lossy(&fs::read(destination_path.join("file2.txt/err"))?)
    );
    assert!(!destination_path.join("file2/status").exists());
    Ok(())
}

/// Basic test for filename processing happy path.
///
/// We use `echo {}` as our command.
/// The destination directory has a file in `out` matching each file in our source directory.
/// All of the `err` files are empty,
/// and the `status` files don't exist, because we haven't implemented them.
#[tokio::test]
async fn test_filename() -> io::Result<()> {
    let source = tempfile::tempdir()?;
    let source_path = source.path();
    let file1_path = source_path.join("file1.txt");
    let mut file1 = fs::File::create(file1_path)?;
    writeln!(file1, "Arbitrary content for file one")?;
    let file2_path = source_path.join("file2.txt");
    let mut file2 = fs::File::create(file2_path)?;
    writeln!(file2, "Arbitrary content for file two")?;

    let destination = tempfile::tempdir()?;
    reach::run(new_test_config(
        "echo -n {}",
        source.path(),
        destination.path(),
        reach::InputMode::Filename,
    ))
    .await?;

    let destination_path = destination.path();
    let mut filenames = fs::read_dir(destination_path)?
        .map(|res| res.map(|e| e.file_name()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    filenames.sort();

    assert_eq!(vec!["file1.txt", "file2.txt"], filenames);
    assert_eq!(
        source_path.join("file1.txt").to_string_lossy(),
        String::from_utf8_lossy(&fs::read(destination_path.join("file1.txt/out"))?)
    );
    assert_eq!(
        "",
        String::from_utf8_lossy(&fs::read(destination_path.join("file1.txt/err"))?)
    );
    assert!(!destination_path.join("file1/status").exists());
    assert_eq!(
        source_path.join("file2.txt").to_string_lossy(),
        String::from_utf8_lossy(&fs::read(destination_path.join("file2.txt/out"))?)
    );
    assert_eq!(
        "",
        String::from_utf8_lossy(&fs::read(destination_path.join("file2.txt/err"))?)
    );
    assert!(!destination_path.join("file2/status").exists());
    Ok(())
}
