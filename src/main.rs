use jbtop::app::{App, AppResult};
use jbtop::event::{Event, EventHandler};
use jbtop::handler::{handle_host_events, handle_key_events, handle_load_events};
use jbtop::ssh;
use jbtop::tui::Tui;
use log::LevelFilter;
use ratatui::{backend::CrosstermBackend, Terminal};
use simple_logger::SimpleLogger;
use std::{collections::HashMap, error::Error, io, sync::Arc};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> AppResult<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_module_level("russh", LevelFilter::Info)
        .init()
        .unwrap();

    let noderange = std::env::args()
        .nth(1)
        .ok_or("Missing the nodeset argument !")?;
    let nodes: Vec<String> = noderange
        .split(',')
        .map(|set| nodeset::node::node_to_vec_string(set))
        .collect::<Result<Vec<Vec<String>>, Box<dyn Error>>>()?
        .iter_mut()
        .flat_map(|set| set.drain(..))
        .collect();

    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);

    let mut events: Vec<EventHandler> = vec![EventHandler::terminal(tui.channel(), 250)];

    let mut session_pool = HashMap::<String, Arc<Mutex<Option<ssh::Session>>>>::new();
    for node in nodes.iter() {
        let connection = Arc::new(Mutex::new(None));
        events.push(EventHandler::connection(
            tui.channel(),
            node,
            Arc::clone(&connection),
        ));
        events.push(EventHandler::load(
            tui.channel(),
            node,
            Arc::clone(&connection),
        ));
        session_pool.insert(node.clone(), connection);
    }

    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::HostStatus(host, event) => handle_host_events(&host, event, &mut app)?,
            Event::LoadStatus(host, event) => handle_load_events(&host, event, &mut app)?,
        }
    }

    // Exit the user interface.
    tui.exit()?;

    Ok(())
}
