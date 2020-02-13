use indicatif::{ProgressBar, ProgressStyle};

pub fn progress_bar(len: u64, template: &str) -> ProgressBar {
    let bar = ProgressBar::new(len);
    bar.set_style(
        ProgressStyle::default_bar()
            .progress_chars("█▉▊▋▌▍▎▏  ")
            .template(template),
    );
    bar.enable_steady_tick(200);
    bar
}
