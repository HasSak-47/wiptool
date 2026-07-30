use std::sync::atomic::{AtomicUsize, Ordering};

use env_logger::fmt::style::{AnsiColor, Style};

use log::{Level, LevelFilter};

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

pub fn init_log() {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(LevelFilter::Trace)
        .write_style(env_logger::WriteStyle::Always)
        .format(|f, record| {
            use std::io::Write;
            let target = record.target();
            let max_target_width = max_target_width(target);
            let level = record.level();
            let line = record.line().unwrap_or(0);

            let level_style = Style::new().fg_color(Some(
                match level {
                    Level::Trace => AnsiColor::Magenta,
                    Level::Debug => AnsiColor::Blue,
                    Level::Info => AnsiColor::Green,
                    Level::Warn => AnsiColor::Yellow,
                    Level::Error => AnsiColor::Red,
                }
                .into(),
            ));
            let target_style = Style::new().bold();

            writeln!(
                f,
                "{level_style}{level}{level_style:#} {target_style}{target:>max_target_width$}{target_style:#}:{line:4<} > {}",
                record.args(),
            )
        })
        .init();
}
