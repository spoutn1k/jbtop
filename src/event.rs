use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

use crate::ssh;

#[derive(Clone, Debug)]
pub enum LoadEvent {
    Load(String),
    LoadError(String),
}

#[derive(Clone, Debug)]
pub enum ConnectionEvent {
    Connecting,
    Connected,
}

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

    HostStatus(String, ConnectionEvent),
    LoadStatus(String, LoadEvent),
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

    pub fn load(
        sender: mpsc::UnboundedSender<Event>,
        hostname: &str,
        session: std::sync::Arc<Option<ssh::Session>>,
    ) -> Self {
        let _host = hostname.to_string();
        let handler = tokio::spawn(async move {
            sender
                .send(Event::HostStatus(
                    _host.to_string(),
                    ConnectionEvent::Connecting,
                ))
                .unwrap();
            let mut tick = tokio::time::interval(tokio::time::Duration::from_millis(1000));
            loop {
                tick.tick().await;

                let session = (*session).as_ref();
                if session.is_none() {
                    sender
                        .send(Event::LoadStatus(
                            _host.to_string(),
                            LoadEvent::LoadError("Failed to access session".to_string()),
                        ))
                        .unwrap();
                    continue;
                }

                let channel = session.unwrap().open_channel().await;
                if let Err(e) = channel {
                    sender
                        .send(Event::LoadStatus(
                            _host.to_string(),
                            LoadEvent::LoadError(e.to_string()),
                        ))
                        .unwrap();
                    continue;
                }

                let event = match channel.unwrap().block_exec("cat /proc/loadavg").await {
                    Ok((c, o, _)) if c == 0 => {
                        Event::LoadStatus(_host.to_string(), LoadEvent::Load(o))
                    }
                    Ok((_, _, e)) => Event::LoadStatus(_host.to_string(), LoadEvent::LoadError(e)),
                    Err(e) => {
                        Event::LoadStatus(_host.to_string(), LoadEvent::LoadError(e.to_string()))
                    }
                };

                sender.send(event).unwrap();
            }
        });

        Self { handler }
    }
}
