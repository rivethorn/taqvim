use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use crate::app::App;
use crate::calendar::{month_days, format_persian, weekday_name, MONTH_NAMES, WEEKDAYS, PDate, weekday_persian};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("Taqvim - Persian Calendar", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled(format!("View: {} {}", MONTH_NAMES[(app.view_month-1) as usize], app.view_year), Style::default().fg(Color::Yellow)),
        Span::raw("   "),
        Span::styled(format!("Today: {}", format_persian(app.today)), Style::default().fg(Color::Green)),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Header"));
    f.render_widget(header, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    draw_calendar(f, app, body[0]);
    draw_side(f, app, body[1]);

    let status = if app.mode == crate::app::Mode::Command {
        format!(":{} ", app.command_buf)
    } else {
        format!("Mode: {:?} | h/j/k/l move  gg/G top/bottom  H/L month  Y year  t today  ? help  q quit", app.mode)
    };
    let footer = Paragraph::new(status).block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(footer, chunks[2]);

    if app.show_help {
        draw_help(f);
    }
    if app.mode == crate::app::Mode::EventEdit {
        draw_event_edit(f, app);
    }
}

fn draw_calendar(f: &mut Frame, app: &App, area: Rect) {
    let mdays = month_days(app.view_year, app.view_month) as usize;
    let first = PDate::new(app.view_year, app.view_month, 1);
    let first_wd = weekday_persian(first);

    let mut lines: Vec<Line> = Vec::new();
    let mut header_spans = Vec::new();
    for w in WEEKDAYS {
        header_spans.push(Span::styled(format!("{:>3} ", w), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    }
    lines.push(Line::from(header_spans));

    let mut day = 1usize;
    let mut row_spans = Vec::new();
    for i in 0..7 {
        if i < first_wd {
            row_spans.push(Span::raw("    "));
        } else {
            row_spans.push(day_span(app, day as u32));
            day += 1;
        }
    }
    lines.push(Line::from(row_spans));

    while day <= mdays {
        let mut row_spans = Vec::new();
        for _ in 0..7 {
            if day > mdays {
                row_spans.push(Span::raw("    "));
            } else {
                row_spans.push(day_span(app, day as u32));
                day += 1;
            }
        }
        lines.push(Line::from(row_spans));
    }

    let para = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Calendar"));
    f.render_widget(para, area);
}

fn day_span(app: &App, day: u32) -> Span<'static> {
    let p = PDate::new(app.view_year, app.view_month, day);
    let is_today = p == app.today;
    let is_selected = p == app.selected;
    let has_event = app.events.contains_key(&(p.year, p.month, p.day));
    let mut style = Style::default();
    if is_selected {
        style = style.bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD);
    } else if is_today {
        style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
    }
    if has_event {
        style = style.fg(Color::Magenta);
    }
    Span::styled(format!("{:>3} ", day), style)
}
fn draw_side(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(area);

    let sel_info = format!(
        "Selected: {}\nWeekday: {}\n{} events",
        format_persian(app.selected),
        weekday_name(app.selected),
        app.selected_events().map(|v| v.len()).unwrap_or(0)
    );
    let p = Paragraph::new(sel_info).block(Block::default().borders(Borders::ALL).title("Selected"));
    f.render_widget(p, chunks[0]);

    let mut text = String::new();
    if let Some(ev) = app.selected_events() {
        for e in ev {
            text.push_str(&format!("- {}\n  {}\n", e.title, e.note));
        }
    } else {
        text.push_str("No events. Press 'a' to add.");
    }
    let p = Paragraph::new(text).wrap(Wrap { trim: true }).block(Block::default().borders(Borders::ALL).title("Events"));
    f.render_widget(p, chunks[1]);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    let help = "Vim motions:\n\
h/l: prev/next day   j/k: next/prev week (7 days)\ngg: first day of month   G: last day\n\
H/L: prev/next month   (year) move with Y/y? \n\
t: today   a: add event   d: delete event   /: command\n\
? : toggle help   q: quit\n";
    let p = Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_event_edit(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.area());
    let ei = &app.event_input;
    let title_style = if ei.field == 0 { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default() };
    let note_style = if ei.field == 1 { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default() };
    let lines = vec![
        Line::from(Span::styled(format!("Add Event for {}", crate::calendar::format_persian(app.selected)), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::raw("Title: "), Span::styled(ei.title.clone(), title_style)]),
        Line::from(vec![Span::raw("Note:  "), Span::styled(ei.note.clone(), note_style)]),
        Line::from(""),
        Line::from(Span::styled("Tab/Enter switch field. Enter on Note saves. Esc cancels.", Style::default().fg(Color::Gray))),
    ];
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("New Event"));
    f.render_widget(p, area);
}

