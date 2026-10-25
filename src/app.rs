use crate::calendar::{PDate, today_persian, month_days, persian_to_gregorian};
use chrono::Datelike;
use std::collections::BTreeMap;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub title: String,
    pub note: String,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Command,
    EventEdit,
}

#[derive(Clone, Default)]
pub struct EventInput {
    pub active: bool,
    pub field: u8, // 0 title, 1 note
    pub title: String,
    pub note: String,
}

pub struct App {
    pub today: PDate,
    pub selected: PDate,
    pub view_year: i32,
    pub view_month: u32,
    pub mode: Mode,
    pub command_buf: String,
    pub events: BTreeMap<(i32, u32, u32), Vec<Event>>,
    pub message: String,
    pub show_help: bool,
    pub event_input: EventInput,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let t = today_persian();
        let mut app = Self {
            today: t,
            selected: t,
            view_year: t.year,
            view_month: t.month,
            mode: Mode::Normal,
            command_buf: String::new(),
            events: BTreeMap::new(),
            message: String::new(),
            show_help: false,
            event_input: EventInput::default(),
            should_quit: false,
        };
        app.load_events();
        app
    }

    pub fn go_today(&mut self) {
        let t = today_persian();
        self.today = t;
        self.selected = t;
        self.view_year = t.year;
        self.view_month = t.month;
    }

    pub fn move_day(&mut self, delta: i32) {
        // convert selected to gregorian, add days, convert back
        let gd = persian_to_gregorian(self.selected);
        let new_gd = gd + chrono::Duration::days(delta as i64);
        self.selected = crate::calendar::gregorian_to_persian(new_gd.year(), new_gd.month(), new_gd.day());
        self.view_year = self.selected.year;
        self.view_month = self.selected.month;
    }

    pub fn move_month(&mut self, delta: i32) {
        let mut m = self.view_month as i32 + delta;
        let mut y = self.view_year;
        while m > 12 { m -= 12; y += 1; }
        while m < 1 { m += 12; y -= 1; }
        self.view_year = y;
        self.view_month = m as u32;
        // keep selected day valid
        let d = self.selected.day.min(month_days(y, m as u32));
        self.selected = PDate::new(y, m as u32, d);
    }

    pub fn move_year(&mut self, delta: i32) {
        self.view_year += delta;
        let d = self.selected.day.min(month_days(self.view_year, self.view_month));
        self.selected = PDate::new(self.view_year, self.view_month, d);
    }

    fn data_path() -> std::path::PathBuf {
        let mut p = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        p.push("taqvim");
        let _ = std::fs::create_dir_all(&p);
        p.push("events.json");
        p
    }

    pub fn load_events(&mut self) {
        if let Ok(s) = std::fs::read_to_string(Self::data_path()) {
            if let Ok(m) = serde_json::from_str(&s) {
                self.events = m;
            }
        }
    }

    pub fn save_events(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.events) {
            let _ = std::fs::write(Self::data_path(), s);
        }
    }

    pub fn begin_event_edit(&mut self) {
        self.event_input = EventInput::default();
        self.event_input.active = true;
        self.mode = Mode::EventEdit;
    }

    pub fn commit_event_edit(&mut self) {
        if !self.event_input.title.trim().is_empty() {
            self.events.entry((self.selected.year, self.selected.month, self.selected.day))
                .or_default()
                .push(Event { title: self.event_input.title.trim().to_string(), note: self.event_input.note.trim().to_string() });
            self.save_events();
            self.message = "Event added".into();
        }
        self.event_input = EventInput::default();
        self.mode = Mode::Normal;
    }

    pub fn selected_events(&self) -> Option<&Vec<Event>> {
        self.events.get(&(self.selected.year, self.selected.month, self.selected.day))
    }

    pub fn cancel_event_edit(&mut self) {
        self.event_input = EventInput::default();
        self.mode = Mode::Normal;
    }
}
