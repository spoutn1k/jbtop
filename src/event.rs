use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

use crate::ssh;

/// Terminal events.
#[derive(Clone, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),

    HostConnecting(String),
    Uptime(String, String),
    UptimeError(String, String),
    ConnectionError(String, String),
}

/// Terminal event handler.
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    /// Event handler thread.
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] for the running terminal.
    pub fn terminal(sender: mpsc::UnboundedSender<Event>, tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = sender.closed() => {
                    break;
                  }
                  _ = tick_delay => {
                    sender.send(Event::Tick).unwrap();
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    match evt {
                      CrosstermEvent::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                          sender.send(Event::Key(key)).unwrap();
                        }
                      },
                      CrosstermEvent::Mouse(mouse) => {
                        sender.send(Event::Mouse(mouse)).unwrap();
                      },
                      CrosstermEvent::Resize(x, y) => {
                        sender.send(Event::Resize(x, y)).unwrap();
                      },
                      CrosstermEvent::FocusLost => {
                      },
                      CrosstermEvent::FocusGained => {
                      },
                      CrosstermEvent::Paste(_) => {
                      },
                    }
                  }
                };
            }
        });
        Self { handler }
    }

    pub fn uptime(
        sender: mpsc::UnboundedSender<Event>,
        hostname: &str,
        session: std::sync::Arc<Option<ssh::Session>>,
    ) -> Self {
        let _host = hostname.to_string();
        let handler = tokio::spawn(async move {
            sender
                .send(Event::HostConnecting(_host.to_string()))
                .unwrap();
            let mut tick = tokio::time::interval(tokio::time::Duration::from_millis(1000));
            loop {
                let mut channel = (*session).as_ref().unwrap().open_channel().await.unwrap();
                let event = match channel.block_exec("cat /proc/loadavg").await {
                    Ok((c, o, _)) if c == 0 => Event::Uptime(_host.to_string(), o),
                    Ok((_, _, e)) => Event::UptimeError(_host.to_string(), e),
                    Err(e) => Event::UptimeError(_host.to_string(), e.to_string()),
                };

                sender.send(event).unwrap();

                tick.tick().await;
            }
        });

        Self { handler }
    }
}
