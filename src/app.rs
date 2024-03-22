use std::collections::HashMap;
use std::error;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum HostState {
    Connecting,
    Up(String),
    Down(String),
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,

    pub hosts: HashMap<String, HostState>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            hosts: HashMap::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }

    pub fn set_host_connecting(&mut self, host: &str) {
        let state = HostState::Connecting;
        self.hosts
            .entry(host.to_string())
            .and_modify(|v| *v = state)
            .or_insert(HostState::Connecting);
    }

    pub fn set_host_status(&mut self, host: &str, load: &str) {
        let state = HostState::Up(load.to_string());
        self.hosts
            .entry(host.to_string())
            .and_modify(|v| *v = state)
            .or_insert(HostState::Up(load.to_string()));
    }

    pub fn set_host_error(&mut self, host: &str, error: &str) {
        let state = HostState::Down(error.to_string());
        self.hosts
            .entry(host.to_string())
            .and_modify(|v| *v = state)
            .or_insert(HostState::Down(error.to_string()));
    }
}
