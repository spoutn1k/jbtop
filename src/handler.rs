use crate::app::{App, AppResult};
use crate::event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        // Counter handlers
        KeyCode::Right => {
            app.increment_counter();
        }
        KeyCode::Left => {
            app.decrement_counter();
        }
        // Other handlers you could add here.
        _ => {}
    }

    Ok(())
}

pub fn handle_host_events(
    host: &str,
    event: event::ConnectionEvent,
    app: &mut App,
) -> AppResult<()> {
    match event {
        event::ConnectionEvent::Connected => {}
        event::ConnectionEvent::Connecting => app.set_host_connecting(host),
        event::ConnectionEvent::ConnectionError(error) => app.set_host_error(host, &error),
    }

    Ok(())
}
pub fn handle_load_events(host: &str, event: event::LoadEvent, app: &mut App) -> AppResult<()> {
    match event {
        event::LoadEvent::Load(status) => app.set_host_status(host, &status),
        event::LoadEvent::LoadError(error) => app.set_host_error(host, &error),
    }

    Ok(())
}
