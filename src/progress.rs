use indicatif::{ProgressBar, ProgressStyle};
use std::io;
use std::process::ExitStatus;

/// How `reach` reports progress.
///
/// Exists so we can have a "real" implementation that delegates to indicatif,
/// and a "fake" implementation that does nothing and is used only in tests.
pub trait Progress {
    fn set_num_tasks(&self, tasks: usize);
    fn task_completed(&self, result: io::Result<ExitStatus>);
}

impl Progress for ProgressBar {
    fn set_num_tasks(&self, tasks: usize) {
        self.set_length(tasks as u64);
    }

    fn task_completed(&self, result: io::Result<ExitStatus>) {
        // TODO(jml): Actually use `result` to communicate whether the run succeeded.
        match result {
            Ok(_) => self.inc(1),
            Err(e) => {
                self.println(format!("Error: {:?}", e));
                self.inc(1);
            }
        }
    }
}

impl Progress for () {
    fn set_num_tasks(&self, _tasks: usize) {}
    fn task_completed(&self, _result: io::Result<ExitStatus>) {}
}

/// Construct a real progress bar for rendering to users.
pub fn default_progress_bar() -> impl Progress {
    ProgressBar::new(0).with_style(
        ProgressStyle::default_bar()
            .template("{prefix}{wide_bar} {pos}/{len} [{elapsed}<{eta}, {per_sec}]"),
    )
}
