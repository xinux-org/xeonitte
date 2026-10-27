use anyhow::Context;
use gettextrs::LocaleCategory;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use size::Size;
use std::fs::File;

use crate::config::{GETTEXT_PACKAGE, LOCALEDIR};

pub mod config;
pub mod modules;
pub mod ui;
pub mod utils;

pub fn get_memory_size() -> Option<u64> {
    let contents = std::fs::read_to_string("/proc/meminfo").unwrap_or_else(|e| {
        eprintln!("Couldnʻt read the /proc/meminfo file: {e}");
        "".into()
    });

    contents
        .lines()
        .filter(|line| line.contains("MemTotal"))
        .map(|x| {
            x.chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .ok()
        })
        .collect::<Vec<_>>()
        .first()
        .copied()
        .flatten()
}

pub fn format_size(s: Size) -> String {
    let bytes = s.bytes();
    let tip = s
        .to_string()
        .split(" ")
        .nth(1)
        .unwrap_or_default()
        .to_string();

    let the = match tip.to_string().as_str() {
        "TiB" => 1024_u64.pow(4),
        "GiB" => 1024_u64.pow(3),
        "MiB" => 1024_u64.pow(2),
        "KiB" => 1024,
        _ => 1, // byte
    };

    let size = format!("{}", bytes as f64 / the as f64).to_string();
    let main = size.split('.').next().unwrap_or_default();
    format!("{main} {tip}")
}

pub fn get_storage_size(device: &str, logical_block_size: u64) -> Option<u64> {
    let device = if device.contains("/dev/") {
        &device[5..]
    } else {
        device
    };
    let contents = std::fs::read_to_string(format!("/sys/class/block/{device}/size"))
        .unwrap_or_else(|e| {
            eprintln!("Couldnʻt read the /sys/class/block/{device}/size file: {e}",);
            "".to_string()
        })
        .trim()
        .to_string();

    contents
        .parse::<u64>()
        .ok()
        .map(|x| x * logical_block_size / 1_000_000)
}

pub fn get_storage_size_for_disko(size: Size) -> String {
    let size = format_size(size);

    // extract number part from size: 100 MiB -> 100
    let number_part = size
        .split_ascii_whitespace()
        .next()
        .and_then(|float_number| float_number.split('.').next())
        .unwrap_or_default();

    // extract size part from size: 100 MiB -> M
    let size_part = size
        .split_ascii_whitespace()
        .next_back()
        .and_then(|x| x.chars().next())
        .unwrap_or_default();

    format!("{}{}", number_part, size_part) // e.g: 100M, 500G
}

pub fn setup_logger() -> anyhow::Result<()> {
    let mut log_level_filter = LevelFilter::Warn;

    if cfg!(debug_assertions) {
        log_level_filter = LevelFilter::Debug;
    }

    CombinedLogger::init(vec![
        TermLogger::new(
            log_level_filter,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("/tmp/xeonitte.log").context("Can't open log file: /tmp/xeonitte.log")?,
        ),
    ])
    .context("Failed to initialize loggers")?;

    Ok(())
}

pub fn setup_gettext() -> anyhow::Result<()> {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR)
        .context("Unable to bind the text domain")?;
    gettextrs::bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .context("Unable to bind the text domain codeset to UTF-8")?;
    gettextrs::textdomain(GETTEXT_PACKAGE).context("Unable to switch to the text domain")?;
    Ok(())
}
