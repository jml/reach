use reach;
use std::env;
use tempfile;
use tokio;

/// Smoke test for each.
///
/// If we run against an empty input directory, we do not get any errors.
/// Not actually an interesting test in itself, but rather a pattern for tests to come,
/// and a way of exploring our public API.
#[tokio::test]
async fn test_stdin() {
    let source = tempfile::tempdir().unwrap();
    let source_path = source.path().to_path_buf();
    let destination = tempfile::tempdir().unwrap();
    let destination_path = destination.path().to_path_buf();
    let each = reach::Each::new(source_path, 1, true, 0);
    let shell = env::var("SHELL").unwrap();
    let runner = reach::StdinRunner::new(shell, "cat".to_string(), destination_path);
    let result = each.run(&runner).await;
    result.unwrap();
}
