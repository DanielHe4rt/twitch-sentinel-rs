use crate::models::channels::ConnectedUsersToChannel;
use crate::models::message::Message;
use crate::models::streams_metrics::StreamersEventsLeaderboard;
use crate::ui::components::metrics::build_metrics_table;
use crate::ui::components::ranking_rightbar::{build_ranking, build_right_bar};
use anyhow::Context;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Terminal,
};
use scylla::{CachingSession, SessionBuilder};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;
use tokio::time::interval;
use crate::models::metrics::StreamLeaderboard;

mod components;
mod hydration;
mod input;


// Define the application state
struct App {
    // Tabs for the right bar
    tabs: Vec<&'static str>,
    active_tab: u32,
    // Data for the sidebar (list of active channels)
    active_channels: Vec<StreamersEventsLeaderboard>,
    active_channel: usize,
    // Data for the main chat
    chat_messages: Vec<Message>,
    // Data for connected users
    connected_users: Vec<ConnectedUsersToChannel>, // (Username, Messages Sent)
    // Data for rankings
    rankings: Vec<StreamLeaderboard>, // (Username, Messages Sent)
    // Focused component
    focused: Focus,
}

impl App {
    fn new() -> Self {
        App {
            tabs: vec!["Connected Users", "Rankings"],
            active_tab: 0,
            active_channels: vec![StreamersEventsLeaderboard{
                streamer_id: "danielhe4rt".to_string(),
                day: Default::default(),
                events_count: 0,
            }],
            active_channel: 0,
            chat_messages: vec![Message::default()],
            connected_users: vec![
                ConnectedUsersToChannel {
                    streamer_id: "danielhe4rt".to_string(),
                    chatter_id: Some("User1".to_string()),
                    joined_at: None,
                },
            ],
            rankings: vec![
                StreamLeaderboard::default(),
            ],
            focused: Focus::Sidebar, // Default focus
        }
    }

    fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len() as u32;
    }

    fn previous_tab(&mut self) {
        if self.active_tab == 0 {
            self.active_tab = self.tabs.len() as u32 - 1;
        } else {
            self.active_tab -= 1;
        }
    }

    fn focus_next(&mut self) {
        self.focused = match self.focused {
            Focus::Sidebar => Focus::MainChat,
            Focus::MainChat => Focus::RightBar,
            Focus::RightBar => Focus::Sidebar,
        };
    }

    fn focus_previous(&mut self) {
        self.focused = match self.focused {
            Focus::Sidebar => Focus::RightBar,
            Focus::MainChat => Focus::Sidebar,
            Focus::RightBar => Focus::MainChat,
        };
    }

}

// Enum to represent focused component
#[derive(PartialEq)]
enum Focus {
    Sidebar,
    MainChat,
    RightBar,
}

fn draw_ui(f: &mut ratatui::Frame, app: &App) {
    // Define the outer layout (vertical split for top bar and main content)
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                // Constraint::Length(5), // Top Bar (increased height for Metrics Table)
                Constraint::Length(0), // Top Bar (increased height for Metrics Table)
                Constraint::Min(0),    // Main Content
            ]
                .as_ref(),
        )
        .split(f.area());

    // Define the main content layout (horizontal split for sidebar, chat, and right bar)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20), // Sidebar (20%)
                Constraint::Percentage(60), // Main Chat (60%)
                Constraint::Percentage(20), // Right Bar (20%)
            ]
                .as_ref(),
        )
        .split(outer_chunks[1]);
    let right_bar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
            ]
                .as_ref(),
        )
        .split(main_chunks[2]);
    let tabs = build_right_bar(&app);

    //let metrics_table = build_metrics_table(&app);
    let chat_paragraph = components::chat::build_chat(&app);
    let channels_table = components::channels_sidebar::build_sidebar(&app);

    //f.render_widget(metrics_table, outer_chunks[0]);
    f.render_widget(channels_table, main_chunks[0]);
    f.render_widget(chat_paragraph, main_chunks[1]);
    f.render_widget(tabs, right_bar_chunks[0]);
    match app.active_tab {
        0 => {
            // Connected Users as Table
            let users_table = components::chatters_list::build_chatters_list(&app);
            f.render_widget(users_table, right_bar_chunks[1]);
        }
        1 => {
            let build_ranking = build_ranking(&app);
            f.render_widget(build_ranking, right_bar_chunks[1]);
        }
        _ => {}
    }

    // Highlight the focused component by drawing a rectangle around it
    match app.focused {
        Focus::Sidebar => {
            let rect = main_chunks[0];
            let style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
            f.render_widget(
                Block::default().borders(Borders::ALL).border_style(style),
                rect,
            );
        }
        Focus::MainChat => {
            let rect = main_chunks[1];
            let style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
            f.render_widget(
                Block::default().borders(Borders::ALL).border_style(style),
                rect,
            );
        }
        Focus::RightBar => {
            let rect = main_chunks[2];
            let style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
            f.render_widget(
                Block::default().borders(Borders::ALL).border_style(style),
                rect,
            );
        }
    }
}


pub async fn start_terminal(terminal_session: Arc<CachingSession>) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create the application state
    let app = Arc::new(Mutex::new(App::new()));

    // Create a channel to communicate between the input handler and the main thread
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn a task to handle input
    tokio::spawn(async move {
        loop {
            // Poll for an event with a timeout
            if event::poll(Duration::from_millis(400)).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    let event = tx.send(key).await;
                    if event.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Create a channel for tick events.
    let (tick_tx, mut tick_rx) = mpsc::channel(100);

    // Spawn a task to send tick events every 250ms.
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(350));
        loop {
            interval.tick().await;
            if tick_tx.send(()).await.is_err() {
                break;
            }
        }
    });

    // Main loop
    // Main event loop.
    let mut database_interval = tokio::time::interval(Duration::from_millis(400));
    loop {
        tokio::select! {
            // Handle tick events to redraw the UI.
            _ = tick_rx.recv() => {
                let app = app.lock().unwrap();
                terminal.draw(|f| draw_ui(f, &app))?;
            }

            // Handle input events.
            Some(key) = rx.recv() => {
                let mut app = app.lock().unwrap();
                let exit = input::handle(&mut app, key)?;

                if exit {
                    break;
                }
            }
           _ = database_interval.tick() => {
                hydration::fetch_data(Arc::clone(&app), Arc::clone(&terminal_session)).await
                    .context("Failed to fetch data")?;
            }

            else => {
                break;
            }
        }
    }
    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    info!("Twitch client dashboard stopped");
    Ok(())
}
