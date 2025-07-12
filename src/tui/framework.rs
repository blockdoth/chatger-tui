use std::io::{self, stdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::vec;

use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{Event, poll, read};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use log::{LevelFilter, debug, error, info};
use ratatui::prelude::CrosstermBackend;
use ratatui::{Frame, Terminal};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use super::logs::LogEntry;
use crate::network::client::Client;
use crate::tui::logs;

/// A configurable and generic runner that manages the entire lifecycle of a TUI application.
/// It handles input events, log streaming, periodic ticks, and state updates.
pub struct TuiRunner<T: Tui<Update>, Update> {
    app: T,
    client: Client,
    update_recv: Receiver<Update>,
    update_send: Sender<Update>,
    log_send: Sender<LogEntry>,
    log_recv: Receiver<LogEntry>,
    event_send: Sender<Event>,
    event_recv: Receiver<Event>,
    log_level: LevelFilter,
}

const LOG_CHANNEL_CAPACITY: usize = 100;
const EVENT_CHANNEL_CAPACITY: usize = 10;
const EVENT_POLL_DELAY: u64 = 100;

impl<T, U> TuiRunner<T, U>
where
    U: FromLog + Send + 'static,
    T: Tui<U>,
{
    /// Creates a new `TuiRunner` with required communication channels and configuration.
    ///
    /// # Parameters
    /// - `app`: The application state implementing the `Tui` trait.
    /// - `command_send`: Channel to send commands to async task handlers.
    /// - `update_recv`: Channel to receive updates for the TUI.
    /// - `update_send`: Channel to send updates (e.g., from logs or external sources).
    /// - `log_level`: Logging level for filtering logs.
    pub fn new(app: T, client: Client, update_recv: Receiver<U>, update_send: Sender<U>, log_level: LevelFilter) -> Self {
        let (log_send, log_recv) = mpsc::channel::<LogEntry>(LOG_CHANNEL_CAPACITY);
        let (event_send, event_recv) = mpsc::channel::<Event>(EVENT_CHANNEL_CAPACITY);
        Self {
            app,
            client,
            update_recv,
            update_send,
            log_send,
            log_recv,
            event_send,
            event_recv,
            log_level,
        }
    }

    /// Starts the main event loop for the TUI and runs any background async tasks.
    ///
    /// This function sets up the terminal, handles logs, polls for keyboard events,
    /// and applies periodic updates.
    ///
    /// # Arguments
    /// - `tasks`: Background async tasks to spawn during runtime.
    ///
    /// # Returns
    /// A `Result` indicating success or any terminal-related errors.
    pub async fn run<F>(mut self, tasks: Vec<F>) -> Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let log_handle = Self::init_log_handler_task(self.log_recv, self.update_send.clone()).await;
        let stop_flag = Arc::new(AtomicBool::new(false)); // TODO make more elegant

        let update_send = self.update_send.clone();

        Self::init_event_handler_thread(self.event_send, stop_flag.clone()).await;
        logs::init_logger(self.log_level, self.log_send)?;

        let mut handles: Vec<JoinHandle<()>> = vec![];
        for task in tasks {
            handles.push(tokio::spawn(task));
        }

        let mut terminal = Self::setup_terminal()?;
        loop {
            terminal.draw(|f| self.app.draw_ui(f))?;

            tokio::select! {

              Some(event) = self.update_recv.recv() => {
                  if let Err(e) = self.app.handle_event(event, &update_send, &mut self.client).await {
                      error!("Failed to handle update from update_recv: {e:?}");
                  }
              }

              Some(event) = self.event_recv.recv() => {
                  if let Some(update) = self.app.process_event(event)
                    && let Err(e) = self.app.handle_event(update, &update_send, &mut self.client).await {
                    error!("Failed to handle update from keyboard: {e:?}");
                  }

                  if self.app.should_quit() {
                    break;
                  }

              }

              _ = tokio::time::sleep(Duration::from_millis(10)) => {

                  if let Err(e) = self.app.on_tick().await {
                      error!("Failed during tick handler: {e:?}");
                  }
              }
            }
        }
        stop_flag.store(true, Ordering::Relaxed);
        for handle in &handles {
            handle.abort();
        }
        log_handle.abort();

        Self::restore_terminal(&mut terminal)?;

        Ok(())
    }

    /// Launches an async task that listens for log entries and converts them to updates
    /// using the `FromLog` trait. These updates are then forwarded into the update stream.
    async fn init_log_handler_task(mut log_recv_channel: Receiver<LogEntry>, update_send_channel: Sender<U>) -> JoinHandle<()> {
        tokio::spawn(async move {
            info!("Started log handler task");
            while let Some(log_msg) = log_recv_channel.recv().await {
                if update_send_channel.send(U::from_log(log_msg)).await.is_err() {
                    debug!("Log processor: Update channel closed.");
                    break;
                }
            }
            info!("Log processor task stopped.");
        })
    }

    async fn init_event_handler_thread(event_send: Sender<Event>, stop_signal: Arc<AtomicBool>) {
        std::thread::spawn(move || {
            info!("Started event handler thread");
            while !stop_signal.load(Ordering::Relaxed) {
                if poll(Duration::from_millis(EVENT_POLL_DELAY)).unwrap_or(false) {
                    match read() {
                        Ok(event) => {
                            if event_send.blocking_send(event).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("{e}");
                            break;
                        }
                    }
                }
            }

            info!("Event handler stopped");
        });
    }

    /// Prepares the terminal for raw mode and alternate screen usage.
    fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend).map_err(Into::into)
    }

    /// Restores the terminal to its original state after exiting the application.
    fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), Clear(ClearType::All), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }
}

/// Trait that any TUI application must implement to work with `TuiRunner`.
#[async_trait]
pub trait Tui<E> {
    /// Draws the UI using the current state. Should be purely visual with no side effects.
    fn draw_ui(&self, f: &mut Frame);

    /// Handles a keyboard event and optionally returns an update to process.
    /// Should not mutate state directly.
    fn process_event(&self, event: Event) -> Option<E>;

    /// Main update handler that reacts to updates from events, logs, or commands.
    /// This is where all state mutations should occur.
    async fn handle_event(&mut self, event: E, event_send: &Sender<E>, client: &mut Client) -> Result<()>;

    /// Periodic tick handler that gets called every loop iteration.
    /// Suitable for lightweight background updates like animations or polling.
    async fn on_tick(&mut self) -> Result<()>;

    /// Determines if the TUI application should terminate.
    fn should_quit(&self) -> bool;
}

/// Trait for converting from a `LogEntry` into an implementing type,
/// allowing integration of logs into other components such as update/event enums.
pub trait FromLog {
    fn from_log(log: LogEntry) -> Self;
}
