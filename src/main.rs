use crate::components::metrics::build_metrics_table;
use crate::components::ranking_rightbar::{build_ranking, build_right_bar};
use charybdis::operations::New;
use charybdis::types::Timestamp;
use chrono::{Datelike, NaiveDate, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Terminal,
};
use scylla::frame::value::CqlDate;
use scylla::SessionBuilder;
use std::ops::{Add, Deref};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use scylla::statement::PagingState;
use tokio::sync::mpsc;
use tokio::time::interval;

mod components;
mod models;

// Define the application state
struct App {
    // Tabs for the right bar
    tabs: Vec<&'static str>,
    active_tab: u32,
    // Data for the sidebar (list of active channels)
    active_channels: Vec<(String, String, usize)>,
    active_channel: usize,
    // Data for the main chat
    chat_messages: Vec<String>,
    // Data for connected users
    connected_users: Vec<(String, u32)>, // (Username, Messages Sent)
    // Data for rankings
    rankings: Vec<(String, u32)>, // (Username, Messages Sent)
    // Focused component
    focused: Focus,
}

impl App {
    fn new() -> Self {
        App {
            tabs: vec!["Connected Users", "Rankings"],
            active_tab: 0,
            active_channels: vec![
                ("danielhe4rt".to_string(), "daniel".to_string(), 0),
            ],
            active_channel: 0,
            chat_messages: vec![
                "User1: Hello!".to_string(),
                "User2: Hi there!".to_string(),
                "User3: Welcome to the chat.".to_string(),
            ],
            connected_users: vec![
                ("User1".to_string(), 150),
                ("User2".to_string(), 120),
                ("User3".to_string(), 100),
                ("User4".to_string(), 80),
            ],
            rankings: vec![
                ("User1".to_string(), 150),
                ("User2".to_string(), 120),
                ("User3".to_string(), 100),
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

    pub fn update_connected_users(&mut self, users: Vec<(String, u32)>) {
        self.connected_users = users;
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
                Constraint::Length(5), // Top Bar (increased height for Metrics Table)
                Constraint::Min(0),    // Main Content
            ]
            .as_ref(),
        )
        .split(f.size());

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

    let metrics_table = build_metrics_table(&app);
    let chat_paragraph = components::chat::build_chat(&app);
    let channels_table = components::channels_sidebar::build_sidebar(&app);

    f.render_widget(metrics_table, outer_chunks[0]);
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Clone the Arc for the input handler
    let app_clone = Arc::clone(&app);

    // Spawn a task to handle input
    tokio::spawn(async move {
        loop {
            // Poll for an event with a timeout
            if event::poll(Duration::from_millis(20)).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(key).await.unwrap();
                }
            }
        }
    });

    let session = SessionBuilder::new()
        .known_node("localhost:9042")
        .build()
        .await
        .unwrap();
    session.use_keyspace("twitch", true).await.unwrap();
    let session = Arc::new(session);
    // Create a channel for tick events.
    let (tick_tx, mut tick_rx) = mpsc::channel(100);

    // Spawn a task to send tick events every 250ms.
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(5));
        loop {
            interval.tick().await;
            if tick_tx.send(()).await.is_err() {
                break;
            }
        }
    });

    tokio::spawn(async move {
        // Initialize ScyllaDB session
        let mut prepared_connected_users = session
            .prepare("SELECT chatter_id FROM twitch.connected_users_to_channel WHERE streamer_id = ? LIMIT 30")
            .await
            .unwrap();

        let mut prepared_chat = session
            .prepare("SELECT sent_at, chatter_username, content, chatter_color FROM twitch.messages WHERE streamer_id = ? LIMIT 40")
            .await
            .unwrap();

        let mut prepared_streamers_events_leaderboard = session
            .prepare("SELECT streamer_id, events_count FROM twitch.streamers_events_leaderboard WHERE day = ? LIMIT 50")
            .await
            .unwrap();

        loop {
            // Fetch connected users
            let channel = {
                let app = app_clone.lock().unwrap();
                app.active_channels[app.active_channel].0.clone()
            };

            let mut paging_state = PagingState::start();
            let result = session.execute_unpaged(&prepared_connected_users, (channel.clone(),)).await.unwrap();
            let mut connected_users = vec![];
            let iter = result.rows_typed::<(String,)>().unwrap().enumerate();
            for (idx, row) in iter {
                connected_users.push((row.unwrap().0, idx as u32));
            }
            {
                let mut app = app_clone.lock().unwrap();
                app.connected_users = connected_users;
            }

            // Fetch chat
            let mut paging_state = PagingState::start();
            let (result, _) = session.execute_single_page(&prepared_chat, (channel,), paging_state).await.unwrap();
            let mut chat_messages = vec![];
            let iter = result
                .rows_typed::<(Timestamp, String, String, Option<String>)>()
                .unwrap()
                .enumerate();
            for (idx, row) in iter {
                let (sent_at, chatter_id, message, chatter_color) = row.unwrap();
                let chatter_color = chatter_color.unwrap_or("white".to_string());
                let message = format!("[{}] {}: {}", sent_at, chatter_id, message);
                chat_messages.push(message);
            }
            {
                let mut app = app_clone.lock().unwrap();
                app.chat_messages = chat_messages;
            }

            // channels
            let mut active_channels = vec![];


            let epoch_naive = Utc::now().date_naive();

            let mut paging_state = PagingState::start();
            let (result, _) = session.execute_single_page(&prepared_streamers_events_leaderboard, (epoch_naive,), paging_state).await.unwrap();
            let iter = result.rows_typed::<(String, i32)>().unwrap().enumerate();
            for (idx, row) in iter {
                let (streamer_id, events_count) = row.unwrap();
                let formatted = format!("{} ({})", streamer_id, events_count);
                active_channels.push((streamer_id, formatted, idx));
            }
            {
                let mut app = app_clone.lock().unwrap();
                if app.active_channels.len() != active_channels.len() {
                    app.active_channel = 0;
                }
                app.active_channels = active_channels;
            }

            // Sleep before the next fetch
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    });
    // Main loop
    // Main event loop.
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

                match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Tab => {
                        app.focus_next();
                    }
                    KeyCode::BackTab => {
                        app.focus_previous();
                    }
                    KeyCode::Up => {
                        if app.focused == Focus::Sidebar {
                            if app.active_channel > 0 {
                                app.active_channel -= 1;
                            }
                        }
                        // Optionally, handle up arrow for other components
                    }
                    KeyCode::Down => {
                        if app.focused == Focus::Sidebar {
                            if app.active_channel < app.active_channels.len() - 1 {
                                app.active_channel += 1;
                            }
                        }
                        // Optionally, handle down arrow for other components
                    }
                    KeyCode::Enter => {
                        if app.focused == Focus::Sidebar {
                            // Handle channel selection
                            let selected_channel = app.active_channels[app.active_channel].clone();
                            // Update app state to load the selected channel
                            app.active_channel = app.active_channel;
                            // Optionally, trigger data fetching for the new channel
                            // You might need to implement additional logic here
                        }
                    }
                    KeyCode::Left => {
                        app.previous_tab()
                    }
                    KeyCode::Right => {
                        app.next_tab()
                    }
                    // ... other key events ...
                    _ => {}
                }
            }

            else => {
                break;
            }
        }
    }
    // Restore the terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
