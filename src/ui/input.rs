use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::{App, Focus};

pub fn handle(app: &mut App, key: KeyEvent) -> anyhow::Result<bool> {
    match key.code {
        KeyCode::Char('q') => Ok(true),
        KeyCode::Tab => {
            app.focus_next();
            Ok(false)
        }
        KeyCode::BackTab => {
            app.focus_previous();
            Ok(false)
        }
        KeyCode::Up => {
            if app.focused == Focus::Sidebar && app.active_channel > 0 {
                app.active_channel -= 1;
            }
            Ok(false)
        }
        KeyCode::Down => {
            if app.focused == Focus::Sidebar && app.active_channel < app.active_channels.len() - 1 {
                app.active_channel += 1;
            }
            Ok(false)
        }
        KeyCode::Enter => {
            if app.focused == Focus::Sidebar {
                app.active_channel = app.active_channel;
            }
            Ok(false)
        }
        KeyCode::Left => {
            app.previous_tab();
            Ok(false)
        }
        KeyCode::Right => {
            app.next_tab();
            Ok(false)
        }
        _ => Ok(false),
    }
}