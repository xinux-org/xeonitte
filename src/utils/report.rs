use anyhow::{Context, Result};
use log::warn;
use report::{JournalMode, ReportBuilder};
use reqwest::blocking::multipart;
use std::path::{Path, PathBuf};
use utils::config::{CONFIG, Config};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPhase {
    Setup,
    Partition,
    Configuration,
    Installation,
    PostInstall,
}

impl ErrorPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorPhase::Setup => "setup",
            ErrorPhase::Partition => "partition",
            ErrorPhase::Configuration => "configuration",
            ErrorPhase::Installation => "installation",
            ErrorPhase::PostInstall => "post-install",
        }
    }
}

impl std::fmt::Display for ErrorPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

const REPORT_CONFIG: &str = "/etc/xeonitte/relago/report.toml";

pub fn init() {
    let config = Config::get_config(REPORT_CONFIG).unwrap_or_else(|e| {
        warn!("Failed to load {REPORT_CONFIG}: {e}. Using built-in defaults.");
        relago_config()
    });

    CONFIG.set(move || config.clone());
}

fn relago_config() -> Config {
    Config {
        parallel_compression: 4,
        tmp_dir: PathBuf::from("/tmp/xeonitte-report"),
        data_dir: PathBuf::from("/tmp/xeonitte-report/data"),
        nix_config: PathBuf::from("/etc/nixos"),
        server: "https://relago.support.xinux.uz".to_string(),
        keys: PathBuf::default(),
    }
}

pub fn generate_report(phase: ErrorPhase, message: &str, log_files: &[&str]) -> Result<String> {
    let tmp_dir = CONFIG.get().tmp_dir.to_string_lossy().into_owned();
    std::fs::create_dir_all(&tmp_dir)
        .ok()
        .context("Failed to create file for report")?;

    let mut builder = ReportBuilder::new(&tmp_dir)
        .system_info()
        .journal(JournalMode::All)
        .meta("phase", phase.as_str())
        .meta("message", message);

    for log_file in log_files {
        if Path::new(log_file).exists() {
            builder = builder.log(log_file);
        } else {
            warn!("Log file not found, omitting from report: {log_file}");
        }
    }

    let report = builder.build().context("Failed to generate report")?;
    Ok(report.file.display().to_string())
}

pub fn upload_report(
    file_path: &str,
    phase: ErrorPhase,
    message: &str,
    log_files: &[&str],
) -> Result<()> {
    let server = CONFIG.get().server.clone();

    let log_files: Vec<String> = log_files
        .iter()
        .filter(|f| Path::new(f).exists())
        .filter_map(|f| {
            Path::new(f)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .collect();

    let meta = serde_json::json!({
        "phase": phase.as_str(),
        "logFiles": log_files,
        "errorMessage": message,
    });
    let meta = serde_json::to_string(&meta).unwrap_or_else(|_| "{}".to_string());

    let form = multipart::Form::new()
        .text("meta", meta)
        .file("report", file_path)
        .context("Failed to read report file")?;

    let url = format!("{}/reports/installation", &server);
    reqwest::blocking::Client::new()
        .post(&url)
        .multipart(form)
        .send()
        .context("Failed to upload report")?;

    Ok(())
}

pub fn send_report(phase: ErrorPhase, message: &str, log_files: &[&str]) -> Result<String> {
    let path = generate_report(phase, message, log_files).context("Generate report failed")?;
    upload_report(&path, phase, message, log_files)
        .with_context(|| format!("Failed to upload report from {path}"))?;
    Ok(path)
}
