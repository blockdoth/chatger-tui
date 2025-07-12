use chrono::{DateTime, Local};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use tokio::sync::mpsc::Sender;

/// Represents a single log entry captured for display in the TUI.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    /// Local timestamp when the log was generated.
    pub timestamp: DateTime<Local>,
    /// Log level (e.g., Info, Error, Debug).
    pub level: log::Level,
    /// The actual log message.
    pub message: String,
}

impl LogEntry {
    /// Converts a log entry into a styled `Line` for display in the terminal UI.
    pub fn format(&self) -> Line<'_> {
        use log::Level::*;
        let timestamp_str = self.timestamp.format("%H:%M:%S").to_string();
        let level_str = format!("[{}]", self.level);
        let message_str = &self.message;

        let style = match self.level {
            Error => Style::default().fg(Color::Red),
            Warn => Style::default().fg(Color::Yellow),
            Info => Style::default().fg(Color::Cyan),
            Debug => Style::default().fg(Color::Blue),
            Trace => Style::default().fg(Color::Magenta),
        };
        Line::from(vec![
            Span::raw(format!("{timestamp_str} ")),
            Span::styled(level_str, style),
            Span::styled(format!(" {message_str}"), style),
        ])
    }
}

/// Custom logger that implements the `log::Log` trait and sends log entries
/// over a Tokio channel to be handled by the TUI rendering system.
pub struct TuiLogger {
    /// Sender channel to pass log entries to the async TUI task.
    pub log_channel_send: Sender<LogEntry>,
    /// Minimum log level that should be recorded.
    pub log_level: Level,
}

impl log::Log for TuiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_entry = LogEntry {
                timestamp: Local::now(),
                level: record.level(),
                message: format!("{}", record.args()),
            };
            if self.log_channel_send.try_send(log_entry.clone()).is_err() {
                eprintln!(
                    "[TUI_LOG_FALLBACK] {}: {} [{}] - {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    std::thread::current().name().unwrap_or("unknown_thread"),
                    record.level(),
                    record.args()
                );
            }
        }
    }
    fn flush(&self) {}
}

/// Initializes the global logger with a custom `TuiLogger`, routing log messages
/// through a Tokio channel for asynchronous TUI display.
///
/// # Arguments
/// * `log_level_filter` - The maximum log level to be captured.
/// * `sender` - A Tokio `Sender` that receives `LogEntry` items.
///
/// # Returns
/// * `Ok(())` if the logger was successfully set.
/// * `Err(SetLoggerError)` if logger setup fails.
pub fn init_logger(log_level_filter: LevelFilter, sender: Sender<LogEntry>) -> Result<(), SetLoggerError> {
    let logger = TuiLogger {
        log_channel_send: sender,
        log_level: log_level_filter.to_level().unwrap_or(log::Level::Error),
    };

    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(log_level_filter);
    Ok(())
}
