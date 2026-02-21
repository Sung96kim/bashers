use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::thread;
use std::time::Duration;

const TICK_MS: u64 = 80;
const SPINNER_TICKS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""];
const SECTION_TICKS: &[&str] = &[""];
const SECTION_DIVIDER: &str = "────────────────────────────────────────";

pub fn multi_progress_stderr() -> MultiProgress {
    let draw_target = if atty::is(atty::Stream::Stderr) {
        ProgressDrawTarget::stderr()
    } else {
        ProgressDrawTarget::hidden()
    };
    MultiProgress::with_draw_target(draw_target)
}

pub fn run_header_spinner<F, T>(
    multi: &MultiProgress,
    loading_msg: &str,
    success_msg: impl AsRef<str>,
    failure_msg: impl AsRef<str>,
    op: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let style = ProgressStyle::default_spinner()
        .template("{spinner:.dim}{msg}")
        .unwrap()
        .tick_strings(SPINNER_TICKS);
    let pb = multi.add(
        ProgressBar::new_spinner()
            .with_style(style)
            .with_message(loading_msg.to_string()),
    );
    pb.enable_steady_tick(Duration::from_millis(TICK_MS));

    let result = op();

    if let Err(_) = result {
        pb.finish_with_message(failure_msg.as_ref().to_string());
        return result;
    }
    pb.finish_with_message(success_msg.as_ref().to_string());
    result
}

pub fn run_parallel_spinners<Item, R, FormatPrefix, FormatDone, PerItem>(
    multi: &MultiProgress,
    items: Vec<Item>,
    format_prefix: FormatPrefix,
    per_item: PerItem,
    format_done: FormatDone,
) -> Vec<R>
where
    Item: Send,
    R: Send,
    FormatPrefix: Fn(usize, usize, &Item) -> String + Sync,
    FormatDone: Fn(&R) -> String + Sync,
    PerItem: Fn(Item) -> R + Sync,
{
    let total = items.len();
    if total == 0 {
        return Vec::new();
    }

    let style = ProgressStyle::default_spinner()
        .template("{prefix}{spinner:.dim}{msg}")
        .unwrap()
        .tick_strings(SPINNER_TICKS);

    thread::scope(|s| {
        let per_item_ref = &per_item;
        let format_done_ref = &format_done;
        let mut handles = Vec::with_capacity(total);
        for (idx, item) in items.into_iter().enumerate() {
            let one_indexed = idx + 1;
            let prefix = format_prefix(one_indexed, total, &item);
            let pb = multi.add(
                ProgressBar::new_spinner()
                    .with_style(style.clone())
                    .with_prefix(prefix)
                    .with_message(""),
            );
            pb.enable_steady_tick(Duration::from_millis(TICK_MS));

            let handle = s.spawn(move || {
                let result = per_item_ref(item);
                let msg = format_done_ref(&result);
                pb.finish_with_message(msg);
                result
            });
            handles.push(handle);
        }

        handles
            .into_iter()
            .map(|h| h.join().expect("worker thread panicked"))
            .collect()
    })
}

pub fn run_parallel_spinners_sectioned<Item, R, FormatPrefix, FormatDone, PerItem>(
    multi: &MultiProgress,
    sections: Vec<(String, Vec<Item>)>,
    format_prefix: FormatPrefix,
    per_item: PerItem,
    format_done: FormatDone,
) -> Vec<R>
where
    Item: Send,
    R: Send,
    FormatPrefix: Fn(usize, usize, usize, &Item) -> String + Sync,
    FormatDone: Fn(&R) -> String + Sync,
    PerItem: Fn(Item) -> R + Sync,
{
    let section_style = ProgressStyle::default_spinner()
        .template("{msg}")
        .unwrap()
        .tick_strings(SECTION_TICKS);
    let item_style = ProgressStyle::default_spinner()
        .template("{prefix}{spinner:.dim}{msg}")
        .unwrap()
        .tick_strings(SPINNER_TICKS);

    let mut bars_and_items: Vec<(ProgressBar, Item)> = Vec::new();
    let section_count = sections.len();

    for (section_idx, (title, items)) in sections.into_iter().enumerate() {
        let title_pb = multi.add(
            ProgressBar::new_spinner()
                .with_style(section_style.clone())
                .with_message(""),
        );
        title_pb.finish_with_message(title);

        let total_in_section = items.len();
        for (one_indexed, item) in items.into_iter().enumerate() {
            let one_indexed = one_indexed + 1;
            let prefix = format_prefix(section_idx, one_indexed, total_in_section, &item);
            let pb = multi.add(
                ProgressBar::new_spinner()
                    .with_style(item_style.clone())
                    .with_prefix(prefix)
                    .with_message(""),
            );
            pb.enable_steady_tick(Duration::from_millis(TICK_MS));
            bars_and_items.push((pb, item));
        }

        if section_idx < section_count - 1 {
            let divider_pb = multi.add(
                ProgressBar::new_spinner()
                    .with_style(section_style.clone())
                    .with_message(""),
            );
            divider_pb.finish_with_message(SECTION_DIVIDER.to_string());
        }
    }

    thread::scope(|s| {
        let per_item_ref = &per_item;
        let format_done_ref = &format_done;
        let handles: Vec<_> = bars_and_items
            .into_iter()
            .map(|(pb, item)| {
                s.spawn(move || {
                    let result = per_item_ref(item);
                    let msg = format_done_ref(&result);
                    pb.finish_with_message(msg);
                    result
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("worker thread panicked"))
            .collect()
    })
}
