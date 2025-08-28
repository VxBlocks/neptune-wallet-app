use once_cell::sync::OnceCell;
use std::{collections::VecDeque, io::Write, str::FromStr, sync::Mutex};
use tracing_subscriber::{
    filter,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    reload,
    util::SubscriberInitExt,
};

#[cfg(feature = "gui")]
use crate::config::Config;

static LOG_HANDLER: OnceCell<
    Mutex<
        tracing_subscriber::reload::Handle<
            tracing_subscriber::filter::Targets,
            tracing_subscriber::Registry,
        >,
    >,
> = OnceCell::new();
static LOGS: OnceCell<Mutex<VecDeque<Vec<u8>>>> = OnceCell::new();

struct MemoryLogger {
    max_lines: usize,
}

impl MemoryLogger {
    fn new(max_lines: usize) -> Self {
        MemoryLogger { max_lines }
    }
}

impl Write for MemoryLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let log_entry = buf.to_owned();

        #[cfg(any(feature = "dev-release", not(feature = "gui"), debug_assertions))]
        std::io::stdout().write_all(buf)?;

        let mut logs = unsafe { LOGS.get_unchecked() }.lock().unwrap();
        logs.push_back(log_entry);
        while logs.len() > self.max_lines {
            logs.pop_front();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for MemoryLogger {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        MemoryLogger::new(self.max_lines)
    }
}

pub fn setup_logger(level: Option<String>) -> anyhow::Result<()> {
    let target_filter = build_target_filter(&level.unwrap_or("info".to_string()));
    let (filter, reload_handle) = reload::Layer::new(target_filter);
    let _ = LOG_HANDLER.set(Mutex::new(reload_handle));

    let fmt = {
        let max_lines = 1000;
        let _ = LOGS.set(Mutex::new(VecDeque::with_capacity(max_lines)));
        let logger = MemoryLogger::new(max_lines);
        fmt::Layer::default().with_writer(logger)
    };

    tracing_subscriber::registry().with(filter).with(fmt).init();

    Ok(())
}

fn build_target_filter(level: &str) -> tracing_subscriber::filter::Targets {
    let level_filter = match filter::LevelFilter::from_str(level) {
        Ok(level) => level,
        Err(_) => filter::LevelFilter::INFO,
    };
    filter::Targets::new().with_targets(vec![
        ("neptune_cash", level_filter),
        ("neptune_wallet", level_filter),
        ("wallet", level_filter),
        ("hyper_util", filter::LevelFilter::WARN),
        ("tower_http", filter::LevelFilter::WARN),
        ("reqwest", filter::LevelFilter::WARN),
    ])
}

#[cfg_attr(feature = "gui", tauri::command)]
#[cfg_attr(not(feature = "gui"), allow(unused))]
pub fn get_logs() -> Vec<String> {
    let logs = unsafe { LOGS.get_unchecked() }.lock().unwrap();
    logs.iter()
        .map(|s| unsafe { String::from_utf8_unchecked(s.to_owned()) })
        .collect()
}

#[cfg_attr(feature = "gui", tauri::command)]
#[cfg(feature = "gui")]
pub fn clear_logs() {
    let mut logs = unsafe { LOGS.get_unchecked() }.lock().unwrap();
    logs.clear();
}

#[cfg_attr(feature = "gui", tauri::command)]
#[cfg(feature = "gui")]
pub fn set_log_level(level: &str) {
    let handler = LOG_HANDLER.get().unwrap().lock().unwrap();
    if let Err(e) = handler.reload(build_target_filter(level)) {
        tracing::error!("Failed to reload log handler: {}", e);
    };
    let config = crate::service::get_state::<std::sync::Arc<Config>>();
    let _ = config.set_log_level(level);
}

#[cfg_attr(feature = "gui", tauri::command)]
#[cfg(feature = "gui")]
pub fn get_log_level() -> String {
    let handler = LOG_HANDLER.get().unwrap().lock().unwrap();
    let current = handler.clone_current();
    if let Some(level) = current {
        return level
            .iter()
            .find(|(target, _)| *target == "neptune_wallet")
            .unwrap()
            .1
            .to_string();
    }
    return "info".to_string();
}

#[cfg_attr(feature = "gui", tauri::command)]
#[cfg(feature = "gui")]
pub fn log(level: &str, message: &str) {
    match level {
        "trace" => tracing::trace!("{}", message),
        "debug" => tracing::debug!("{}", message),
        "info" => tracing::info!("{}", message),
        "warn" => tracing::warn!("{}", message),
        "error" => tracing::error!("{}", message),
        _ => return,
    };
}
