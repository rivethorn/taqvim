mod app;
mod calendar;
mod tray;
mod ui;

use app::{App, Mode};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    let tray_tx = tray::start_tray();
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let _ = tray_tx.send(format!("Taqvim - {}", calendar::format_persian(app.today)));

    let mut last_tray = Instant::now();
    let mut pending_g = false;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if last_tray.elapsed() >= Duration::from_secs(60) {
            let _ = tray_tx.send(format!("{} - {}", calendar::weekday_name(app.today), calendar::format_persian(app.today)));
            last_tray = Instant::now();
        }

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key, &mut pending_g, &tray_tx);
            }
        }

        if app.should_quit {
            break;
        }
    }

    ratatui::restore();
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent, pending_g: &mut bool, tray_tx: &std::sync::mpsc::Sender<String>) {
    if app.mode == Mode::Command {
        match key.code {
            KeyCode::Esc => {
                app.mode = Mode::Normal;
                app.command_buf.clear();
            }
            KeyCode::Enter => {
                execute_command(app, tray_tx);
                app.mode = Mode::Normal;
                app.command_buf.clear();
            }
            KeyCode::Backspace => {
                app.command_buf.pop();
            }
            KeyCode::Char(c) => {
                app.command_buf.push(c);
            }
            _ => {}
        }
        return;
    }

    if app.mode == Mode::EventEdit {
        match key.code {
            KeyCode::Esc => app.cancel_event_edit(),
            KeyCode::Enter => {
                if app.event_input.field == 0 {
                    app.event_input.field = 1;
                } else {
                    app.commit_event_edit();
                }
            }
            KeyCode::Tab => {
                app.event_input.field = if app.event_input.field == 0 { 1 } else { 0 };
            }
            KeyCode::Backspace => {
                if app.event_input.field == 0 { app.event_input.title.pop(); }
                else { app.event_input.note.pop(); }
            }
            KeyCode::Char(c) => {
                if app.event_input.field == 0 { app.event_input.title.push(c); }
                else { app.event_input.note.push(c); }
            }
            _ => {}
        }
        return;
    }

    if app.show_help {
        if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc) {
            app.show_help = false;
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Char('t') => {
            app.go_today();
            let _ = tray_tx.send(format!("{} - {}", calendar::weekday_name(app.today), calendar::format_persian(app.today)));
        }
        KeyCode::Char('h') => app.move_day(-1),
        KeyCode::Char('l') => app.move_day(1),
        KeyCode::Char('k') => app.move_day(-7),
        KeyCode::Char('j') => app.move_day(7),
        KeyCode::Char('H') => app.move_month(-1),
        KeyCode::Char('L') => app.move_month(1),
        KeyCode::Char('Y') => app.move_year(1),
        KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => app.move_year(-1),
        KeyCode::Char('g') => {
            if *pending_g {
                app.selected = calendar::PDate::new(app.view_year, app.view_month, 1);
                *pending_g = false;
            } else {
                *pending_g = true;
            }
            return;
        }
        KeyCode::Char('G') => {
            let md = calendar::month_days(app.view_year, app.view_month);
            app.selected = calendar::PDate::new(app.view_year, app.view_month, md);
        }
        KeyCode::Char('a') => app.begin_event_edit(),
        KeyCode::Char('d') => {
            app.events.remove(&(app.selected.year, app.selected.month, app.selected.day));
            app.save_events();
            app.message = "Events cleared".into();
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Command;
            app.command_buf.clear();
        }
        _ => {}
    }
    if key.code != KeyCode::Char('g') {
        *pending_g = false;
    }
}

fn execute_command(app: &mut App, tray_tx: &std::sync::mpsc::Sender<String>) {
    let cmd = app.command_buf.trim().to_string();
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    match parts.as_slice() {
        ["quit"] | ["q"] => app.should_quit = true,
        ["today"] | ["t"] => {
            app.go_today();
            let _ = tray_tx.send(format!("{} - {}", calendar::weekday_name(app.today), calendar::format_persian(app.today)));
        }
        ["add", rest @ ..] => {
            let title = rest.join(" ");
            app.events.entry((app.selected.year, app.selected.month, app.selected.day))
                .or_default()
                .push(app::Event { title: if title.is_empty() { "Event".into() } else { title }, note: String::new() });
            app.save_events();
            app.message = "Event added".into();
        }
        ["del"] => {
            app.events.remove(&(app.selected.year, app.selected.month, app.selected.day));
            app.save_events();
            app.message = "Events cleared".into();
        }
        ["goto", y, m] => {
            if let (Ok(y), Ok(m)) = (y.parse::<i32>(), m.parse::<u32>()) {
                app.view_year = y;
                app.view_month = m.clamp(1, 12);
                app.selected = calendar::PDate::new(app.view_year, app.view_month, 1);
            }
        }
        ["help"] => app.show_help = true,
        _ => {
            app.message = format!("Unknown command: {}", cmd);
        }
    }
}

