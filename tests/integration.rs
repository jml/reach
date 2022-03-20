use reach;
use std::io::Write;
use std::{env, fs, io};
use tempfile;
use tokio;

/// Smoke test for each.
///
/// If we run against an empty input directory, we do not get any errors.
/// Not actually an interesting test in itself, but rather a pattern for tests to come,
/// and a way of exploring our public API.
#[tokio::test]
async fn test_stdin_empty() -> io::Result<()> {
    let source = tempfile::tempdir()?;
    let source_path = source.path().to_path_buf();
    let destination = tempfile::tempdir()?;
    let destination_path = destination.path().to_path_buf();
    let each = reach::Each::new(source_path, 1, true, 0);
    let shell = env::var("SHELL").unwrap();
    let runner = reach::StdinRunner::new(shell, "cat".to_string(), destination_path);
    each.run(&runner).await
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
    let destination_path = destination.path();
    let each = reach::Each::new(source_path.to_path_buf(), 1, true, 0);
    let shell = env::var("SHELL").unwrap();
    let runner = reach::StdinRunner::new(shell, "cat".to_string(), destination_path.to_path_buf());
    each.run(&runner).await?;

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
