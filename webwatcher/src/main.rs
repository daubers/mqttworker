use std::{error::Error, io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};
use paho_mqtt as mqtt;
use mqttworker::process_message;

struct App {
    messages: Vec<String>,
}

impl App {
    fn new() -> App {
        App {
            messages: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    // MQTT Setup
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri("tcp://localhost:1883")
        .client_id("webwatcher_tui")
        .finalize();

    let mut client = mqtt::AsyncClient::new(create_opts).expect("Error creating client");

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    let stream = client.get_stream(25);

    if let Err(e) = client.connect(conn_opts).wait() {
        app.messages.push(format!("Error connecting to MQTT: {:?}", e));
    } else {
        app.messages.push("Connected to MQTT broker".to_string());
        client.subscribe("workers/#", 1).wait().expect("Error subscribing");
    }

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Check for UI events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        // Check for MQTT messages
        match stream.try_recv() {
            Ok(Some(msg)) => {
                let topic = msg.topic();
                let payload = msg.payload_str();
                
                match process_message(&msg) {
                    Some(msg_type) => {
                        app.messages.push(format!("[{}] {:?}", topic, msg_type));
                    }
                    None => {
                        app.messages.push(format!("[{}] {}", topic, payload));
                    }
                }
                
                // Keep only last 20 messages to avoid overflow
                if app.messages.len() > 100 {
                    app.messages.remove(0);
                }
            }
            Ok(None) => {}
            Err(_) => {}
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(90),
            ]
            .as_ref(),
        )
        .split(f.area());

    let title = Block::default()
        .borders(Borders::ALL)
        .title("MQTT WebWatcher TUI (Press 'q' to quit)");
    f.render_widget(title, chunks[0]);

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .rev() // Show latest first
        .map(|m| {
            ListItem::new(m.as_str())
        })
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages_list, chunks[1]);
}
