use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct ConduitProgress;

impl ConduitProgress {
    pub fn download_style(total_size: u64) -> ProgressBar {
        let pb = ProgressBar::new(total_size);
        pb.enable_steady_tick(Duration::from_millis(100));

        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.cyan} {msg} ⟪{raw_bar:20.cyan/blue}⟫ {percent}%"
            )
            .unwrap()
            .with_key("raw_bar", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| {
                let pct = state.fraction();
                let fill_len = (pct * 20.0) as usize;
                let fill = "≋".repeat(fill_len);
                let empty = " ".repeat(20 - fill_len);
                write!(w, "{}{}", fill, empty).unwrap();
            })
            .tick_strings(&["○", "◎", "◉", "◎", "○"])
        );
        pb
    }

    #[allow(dead_code)]
    pub fn simple_spinner(msg: String) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["◦", "◌", "○", "◎", "◉", "⚡"]),
        );
        pb.set_message(msg);
        pb
    }
}