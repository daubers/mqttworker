use std::{collections::HashMap, error::Error, io, time::Duration};
use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use paho_mqtt as mqtt;
use messages::messages::process_message;

struct App {
    messages: Vec<(DateTime<Local>, String)>,
    scroll_vertical: u16,
    scroll_horizontal: u16,
    topic_counts: HashMap<String, usize>,
    last_pane_height: u16,
}

impl App {
    fn new() -> App {
        App {
            messages: Vec::new(),
            scroll_vertical: 0,
            scroll_horizontal: 0,
            topic_counts: HashMap::new(),
            last_pane_height: 0,
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
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

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

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
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
        app.messages.push((Local::now(), format!("Error connecting to MQTT: {:?}", e)));
    } else {
        app.messages.push((Local::now(), "Connected to MQTT broker".to_string()));
        client.subscribe("#", 1).wait().expect("Error subscribing");
    }

    loop {
        terminal.draw(|f| ui(f, app)).expect("Can't draw on terminal");

        // Check for UI events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => {
                        app.scroll_vertical = app.scroll_vertical.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        app.scroll_vertical = app.scroll_vertical.saturating_add(1);
                    }
                    KeyCode::Left => {
                        app.scroll_horizontal = app.scroll_horizontal.saturating_sub(1);
                    }
                    KeyCode::Right => {
                        app.scroll_horizontal = app.scroll_horizontal.saturating_add(1);
                    }
                    _ => {}
                }
            } else if let Event::Mouse(mouse_event) = event::read()? {
                match mouse_event.kind {
                    event::MouseEventKind::ScrollUp => {
                        app.scroll_vertical = app.scroll_vertical.saturating_sub(1);
                    }
                    event::MouseEventKind::ScrollDown => {
                        app.scroll_vertical = app.scroll_vertical.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }

        // Check for MQTT messages
        match stream.try_recv() {
            Ok(Some(msg)) => {
                let topic = msg.topic();
                let payload = msg.payload_str();

                *app.topic_counts.entry(topic.to_string()).or_insert(0) += 1;

                match process_message(&msg) {
                    Some(msg_type) => {
                        app.messages.push((Local::now(), format!("[{}] {:?}", topic, msg_type)));
                    }
                    None => {
                        app.messages.push((Local::now(), format!("[{}] {}", topic, payload)));
                    }
                }
                
                // Auto-scroll to the bottom
                if app.messages.len() > 0 {
                    let total_messages = app.messages.len() as u16;
                    if total_messages > app.last_pane_height {
                        app.scroll_vertical = total_messages - app.last_pane_height;
                    } else {
                        app.scroll_vertical = 0;
                    }
                }

                // Keep only last 100 messages to avoid overflow
                if app.messages.len() > 100 {
                    app.messages.remove(0);
                }
            }
            Ok(None) => {}
            Err(_) => {}
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.area());

    let title = Block::default()
        .borders(Borders::ALL)
        .title("MQTT WebWatcher TUI (q: quit, arrows/mouse: scroll)");
    f.render_widget(title, chunks[0]);

    let mut stats: Vec<_> = app.topic_counts.iter().collect();
    stats.sort_by(|a, b| a.0.cmp(b.0));
    let stats_text = stats
        .iter()
        .map(|(topic, count)| format!("{}: {}", topic, count))
        .collect::<Vec<_>>()
        .join(" | ");

    let stats_paragraph = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Topic Counts"));
    f.render_widget(stats_paragraph, chunks[1]);

    let messages_text = app
        .messages
        .iter()
        .map(|(ts, msg)| format!("[{}] {}", ts.format("%H:%M:%S%.6f"), msg))
        .collect::<Vec<_>>()
        .join("\n");
    let messages_block = Block::default().borders(Borders::ALL).title("Messages");
    let messages_inner_area = messages_block.inner(chunks[2]);
    app.last_pane_height = messages_inner_area.height;
    let messages_paragraph = Paragraph::new(messages_text)
        .block(messages_block)
        .scroll((app.scroll_vertical, app.scroll_horizontal));
    f.render_widget(messages_paragraph, chunks[2]);
}
