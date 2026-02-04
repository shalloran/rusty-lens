// app state for timeline tui

use crate::csv_parser;
use crate::filters::{unique_action_types, unique_dates_from_events, unique_hours_for_date};
use crate::timeline::{now_for_relative, parse_relative_range, parse_time, TimelineEvent};
use chrono::{NaiveDate, NaiveDateTime, Timelike};
use std::path::PathBuf;

const MAX_LOAD_ROWS: usize = 100_000;

/// preset labels for time picker; last two are custom (date picker, then type range)
pub const TIME_PRESETS: &[&str] = &[
    "Today",
    "Yesterday",
    "Last 24 hours",
    "Last 7 days",
    "Last 30 days",
    "Custom (pick dates from data)",
    "Custom (type range)...",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    SearchInput,
    ActionTypeFilter,
    TimeFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFilterSub {
    Picker,
    CustomRangeStart,
    CustomRangeStartHour(NaiveDate),
    CustomRangeEnd(NaiveDateTime),
    CustomRangeEndHour(NaiveDateTime, NaiveDate),
    Custom,
}

#[derive(Debug)]
pub struct App {
    pub path: PathBuf,
    pub events: Vec<TimelineEvent>,
    pub action_types: Vec<String>,
    pub filtered_indices: Vec<usize>,
    pub list_state: ratatui::widgets::ListState,
    pub action_type_filter: Option<String>,
    pub search: String,
    pub search_input: String,
    /// time range filter (inclusive)
    pub time_range_start: Option<NaiveDateTime>,
    pub time_range_end: Option<NaiveDateTime>,
    /// buffer while in TimeFilter mode (Custom sub)
    pub time_input: String,
    /// when TimeFilter: show preset list, date picker, or custom text input
    pub time_filter_sub: TimeFilterSub,
    pub time_picker_list_state: ratatui::widgets::ListState,
    /// unique dates in loaded events (for custom date range picker)
    pub unique_dates: Vec<NaiveDate>,
    /// when CustomRangeEnd: dates >= start to choose end from
    pub date_picker_end_dates: Vec<NaiveDate>,
    /// when CustomRangeStartHour or CustomRangeEndHour: hours to pick (0–23)
    pub date_picker_hours: Vec<u32>,
    pub date_picker_list_state: ratatui::widgets::ListState,
    pub should_quit: bool,
    pub detail_scroll: u16,
    pub theme: crate::tui::theme::Theme,
    pub flash: Option<String>,
    pub error: Option<String>,
    pub mode: Mode,
    pub action_type_list_state: ratatui::widgets::ListState,
}

impl App {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let events = csv_parser::load_timeline(&path, Some(MAX_LOAD_ROWS))?;
        let action_types = unique_action_types(&events);
        let unique_dates = unique_dates_from_events(&events);
        let filtered_indices = (0..events.len()).collect::<Vec<_>>();
        let mut list_state = ratatui::widgets::ListState::default();
        if !filtered_indices.is_empty() {
            list_state.select(Some(0));
        }
        Ok(Self {
            path,
            events,
            action_types,
            filtered_indices,
            list_state,
            action_type_filter: None,
            search: String::new(),
            search_input: String::new(),
            time_range_start: None,
            time_range_end: None,
            time_input: String::new(),
            time_filter_sub: TimeFilterSub::Picker,
            time_picker_list_state: ratatui::widgets::ListState::default(),
            unique_dates,
            date_picker_end_dates: Vec::new(),
            date_picker_hours: Vec::new(),
            date_picker_list_state: ratatui::widgets::ListState::default(),
            should_quit: false,
            detail_scroll: 0,
            theme: crate::tui::theme::Theme,
            flash: None,
            error: None,
            mode: Mode::Normal,
            action_type_list_state: ratatui::widgets::ListState::default(),
        })
    }

    /// recompute filtered indices from current filters
    pub fn apply_filters(&mut self) {
        let action_filter = self.action_type_filter.as_deref();
        let start = self.time_range_start;
        let end = self.time_range_end;
        let filtered: Vec<usize> = self
            .events
            .iter()
            .enumerate()
            .filter(|(_, ev)| {
                if let Some(at) = action_filter {
                    if ev.action_type.as_deref() != Some(at) {
                        return false;
                    }
                }
                if !ev.in_time_range(start, end) {
                    return false;
                }
                ev.matches_search(self.search.trim())
            })
            .map(|(i, _)| i)
            .collect();
        self.filtered_indices = filtered;
        self.list_state.select(if self.filtered_indices.is_empty() {
            None
        } else {
            Some(0)
        });
        self.detail_scroll = 0;
    }

    /// selected event (by filtered list index)
    pub fn selected_event(&self) -> Option<&TimelineEvent> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .and_then(|&idx| self.events.get(idx))
    }

    pub fn next(&mut self) {
        let i = self
            .list_state
            .selected()
            .map(|i| (i + 1).min(self.filtered_indices.len().saturating_sub(1)))
            .unwrap_or(0);
        self.list_state.select(if self.filtered_indices.is_empty() {
            None
        } else {
            Some(i)
        });
        self.detail_scroll = 0;
    }

    pub fn previous(&mut self) {
        let i = self
            .list_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.list_state.select(if self.filtered_indices.is_empty() {
            None
        } else {
            Some(i)
        });
        self.detail_scroll = 0;
    }

    pub fn scroll_detail_down(&mut self, amount: u16) {
        self.detail_scroll = self.detail_scroll.saturating_add(amount);
    }

    pub fn scroll_detail_up(&mut self, amount: u16) {
        self.detail_scroll = self.detail_scroll.saturating_sub(amount);
    }

    pub fn set_flash(&mut self, msg: String) {
        self.flash = Some(msg);
    }

    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// enter search mode; pre-fill with current search
    pub fn start_search(&mut self) {
        self.mode = Mode::SearchInput;
        self.search_input = self.search.clone();
    }

    /// apply search input and exit search mode
    pub fn commit_search(&mut self) {
        self.search = std::mem::take(&mut self.search_input).trim().to_string();
        self.apply_filters();
        self.mode = Mode::Normal;
        let n = self.filtered_indices.len();
        let flash = if n == 0 && !self.search.is_empty() {
            format!("No results for \"{}\"", self.search)
        } else if n == 0 && self.action_type_filter.is_some() {
            "No events match the current filter.".to_string()
        } else {
            format!("Search: \"{}\" ({} events)", self.search, n)
        };
        self.set_flash(flash);
    }

    /// cancel search mode, keep current search
    pub fn cancel_search(&mut self) {
        self.search_input.clear();
        self.mode = Mode::Normal;
    }

    pub fn push_search_char(&mut self, c: char) {
        self.search_input.push(c);
    }

    pub fn pop_search_char(&mut self) {
        self.search_input.pop();
    }

    /// enter time range filter mode; show preset picker first
    pub fn start_time_filter(&mut self) {
        self.mode = Mode::TimeFilter;
        self.time_filter_sub = TimeFilterSub::Picker;
        self.time_input.clear();
        self.time_picker_list_state
            .select(if TIME_PRESETS.is_empty() {
                None
            } else {
                Some(0)
            });
    }

    /// in TimeFilter+Picker: apply selected preset or switch to Custom (date picker or type)
    pub fn apply_time_picker_selection(&mut self) {
        let idx = match self.time_picker_list_state.selected() {
            Some(i) if i < TIME_PRESETS.len() => i,
            _ => return,
        };
        // "Custom (pick dates from data)"
        if idx == TIME_PRESETS.len() - 2 {
            if self.unique_dates.is_empty() {
                self.set_flash("No dates in data to pick from.".to_string());
                return;
            }
            if self.unique_dates.len() == 1 {
                let d = self.unique_dates[0];
                self.time_filter_sub = TimeFilterSub::CustomRangeStartHour(d);
                self.date_picker_hours = unique_hours_for_date(&self.events, d);
                self.date_picker_list_state
                    .select(if self.date_picker_hours.is_empty() {
                        None
                    } else {
                        Some(0)
                    });
            } else {
                self.time_filter_sub = TimeFilterSub::CustomRangeStart;
                self.date_picker_list_state
                    .select(Some(0.min(self.unique_dates.len().saturating_sub(1))));
            }
            return;
        }
        // "Custom (type range)..."
        if idx == TIME_PRESETS.len() - 1 {
            self.time_filter_sub = TimeFilterSub::Custom;
            if let Some(s) = self.time_range_start {
                self.time_input
                    .push_str(&s.format("%Y-%m-%d %H:%M").to_string());
            }
            if self.time_range_end.is_some() {
                self.time_input.push_str(" to ");
            }
            if let Some(e) = self.time_range_end {
                self.time_input
                    .push_str(&e.format("%Y-%m-%d %H:%M").to_string());
            }
            return;
        }
        let now = now_for_relative();
        let label = TIME_PRESETS[idx].to_lowercase();
        if let Some((start, end)) = parse_relative_range(&label, now) {
            self.time_range_start = start;
            self.time_range_end = end;
            self.apply_filters();
            self.mode = Mode::Normal;
            self.time_filter_sub = TimeFilterSub::Picker;
            let flash = match (start, end) {
                (Some(s), Some(e)) => format!(
                    "{} to {} ({} events)",
                    s.format("%Y-%m-%d %H:%M"),
                    e.format("%Y-%m-%d %H:%M"),
                    self.filtered_indices.len()
                ),
                (Some(s), None) => format!(
                    "From {} ({} events)",
                    s.format("%Y-%m-%d %H:%M"),
                    self.filtered_indices.len()
                ),
                (None, Some(e)) => format!(
                    "Before {} ({} events)",
                    e.format("%Y-%m-%d %H:%M"),
                    self.filtered_indices.len()
                ),
                _ => format!("{} events", self.filtered_indices.len()),
            };
            self.set_flash(flash);
        }
    }

    pub fn time_picker_next(&mut self) {
        let i = self
            .time_picker_list_state
            .selected()
            .map(|i| (i + 1).min(TIME_PRESETS.len().saturating_sub(1)))
            .unwrap_or(0);
        self.time_picker_list_state.select(Some(i));
    }

    pub fn time_picker_previous(&mut self) {
        let i = self
            .time_picker_list_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.time_picker_list_state.select(Some(i));
    }

    /// length of current picker list (dates or hours)
    fn date_picker_list_len(&self) -> usize {
        match &self.time_filter_sub {
            TimeFilterSub::CustomRangeStart => self.unique_dates.len(),
            TimeFilterSub::CustomRangeStartHour(_) => self.date_picker_hours.len(),
            TimeFilterSub::CustomRangeEnd(_) => self.date_picker_end_dates.len(),
            TimeFilterSub::CustomRangeEndHour(_, _) => self.date_picker_hours.len(),
            _ => 0,
        }
    }

    pub fn date_picker_next(&mut self) {
        let len = self.date_picker_list_len();
        if len == 0 {
            return;
        }
        let i = self
            .date_picker_list_state
            .selected()
            .map(|i| (i + 1).min(len.saturating_sub(1)))
            .unwrap_or(0);
        self.date_picker_list_state.select(Some(i));
    }

    pub fn date_picker_previous(&mut self) {
        let len = self.date_picker_list_len();
        if len == 0 {
            return;
        }
        let i = self
            .date_picker_list_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.date_picker_list_state.select(Some(i));
    }

    /// in CustomRangeStart: set start date and switch to start-hour pick
    pub fn apply_date_range_start(&mut self) {
        let dates = self.unique_dates.as_slice();
        let idx = match self.date_picker_list_state.selected() {
            Some(i) if i < dates.len() => i,
            _ => return,
        };
        let start_date = dates[idx];
        self.time_filter_sub = TimeFilterSub::CustomRangeStartHour(start_date);
        self.date_picker_hours = unique_hours_for_date(&self.events, start_date);
        self.date_picker_list_state
            .select(if self.date_picker_hours.is_empty() {
                None
            } else {
                Some(0)
            });
    }

    /// in CustomRangeStartHour: set start datetime and switch to end date or end hour (if single date)
    pub fn apply_date_range_start_hour(&mut self) {
        let date = match &self.time_filter_sub {
            TimeFilterSub::CustomRangeStartHour(d) => *d,
            _ => return,
        };
        let hours = &self.date_picker_hours;
        let idx = match self.date_picker_list_state.selected() {
            Some(i) if i < hours.len() => i,
            _ => return,
        };
        let hour = hours[idx];
        let start_dt = date.and_hms_opt(hour, 0, 0).unwrap();
        let single_date = self.unique_dates.len() == 1 && self.unique_dates.first() == Some(&date);
        if single_date {
            self.date_picker_hours = hours.iter().filter(|&&h| h >= hour).copied().collect();
            self.time_filter_sub = TimeFilterSub::CustomRangeEndHour(start_dt, date);
            self.date_picker_list_state
                .select(if self.date_picker_hours.is_empty() {
                    None
                } else {
                    Some(0)
                });
        } else {
            self.date_picker_end_dates = self
                .unique_dates
                .iter()
                .filter(|d| **d >= date)
                .cloned()
                .collect();
            self.time_filter_sub = TimeFilterSub::CustomRangeEnd(start_dt);
            self.date_picker_list_state
                .select(if self.date_picker_end_dates.is_empty() {
                    None
                } else {
                    Some(0)
                });
        }
    }

    /// in CustomRangeEnd: set end date and switch to end-hour pick
    pub fn apply_date_range_end(&mut self) {
        let start_dt = match &self.time_filter_sub {
            TimeFilterSub::CustomRangeEnd(dt) => *dt,
            _ => return,
        };
        let dates = &self.date_picker_end_dates;
        let idx = match self.date_picker_list_state.selected() {
            Some(i) if i < dates.len() => i,
            _ => return,
        };
        let end_date = dates[idx];
        let start_date = start_dt.date();
        self.date_picker_hours = if end_date == start_date {
            unique_hours_for_date(&self.events, end_date)
                .into_iter()
                .filter(|&h| h >= start_dt.hour())
                .collect()
        } else {
            unique_hours_for_date(&self.events, end_date)
        };
        self.time_filter_sub = TimeFilterSub::CustomRangeEndHour(start_dt, end_date);
        self.date_picker_list_state
            .select(if self.date_picker_hours.is_empty() {
                None
            } else {
                Some(0)
            });
    }

    /// in CustomRangeEndHour: set range and apply filter
    pub fn apply_date_range_end_hour(&mut self) {
        let (start_dt, end_date) = match &self.time_filter_sub {
            TimeFilterSub::CustomRangeEndHour(dt, d) => (*dt, *d),
            _ => return,
        };
        let hours = &self.date_picker_hours;
        let idx = match self.date_picker_list_state.selected() {
            Some(i) if i < hours.len() => i,
            _ => return,
        };
        let end_hour = hours[idx];
        self.time_range_start = Some(start_dt);
        self.time_range_end = Some(end_date.and_hms_opt(end_hour, 59, 59).unwrap());
        self.apply_filters();
        self.mode = Mode::Normal;
        self.time_filter_sub = TimeFilterSub::Picker;
        self.set_flash(format!(
            "{} to {} ({} events)",
            start_dt.format("%Y-%m-%d %H:%M"),
            end_date
                .and_hms_opt(end_hour, 59, 59)
                .unwrap()
                .format("%Y-%m-%d %H:%M"),
            self.filtered_indices.len()
        ));
    }

    /// parse time filter input and apply. supports relative ("today", "last 7 days"), "clear", "after/before <t>", "<t> to <t>"
    pub fn commit_time_filter(&mut self) {
        let raw = std::mem::take(&mut self.time_input);
        let s = raw.trim();
        let s_lower = s.to_lowercase();
        if s_lower == "clear" || s.is_empty() {
            self.time_range_start = None;
            self.time_range_end = None;
            self.apply_filters();
            self.mode = Mode::Normal;
            self.set_flash("Time range cleared".to_string());
            return;
        }
        // try relative first (today, yesterday, last 7 days, etc.)
        let now = now_for_relative();
        if let Some((start, end)) = parse_relative_range(&s_lower, now) {
            self.time_range_start = start;
            self.time_range_end = end;
            self.apply_filters();
            self.mode = Mode::Normal;
            self.time_filter_sub = TimeFilterSub::Picker;
            let flash = match (start, end) {
                (Some(s), Some(e)) => format!(
                    "{} to {} ({} events)",
                    s.format("%Y-%m-%d %H:%M"),
                    e.format("%Y-%m-%d %H:%M"),
                    self.filtered_indices.len()
                ),
                _ => format!("{} events", self.filtered_indices.len()),
            };
            self.set_flash(flash);
            return;
        }
        if let Some(rest) = s_lower
            .strip_prefix("after ")
            .or_else(|| s_lower.strip_prefix("from "))
        {
            if let Some(t) = parse_time(rest) {
                self.time_range_start = Some(t);
                self.time_range_end = None;
                self.apply_filters();
                self.mode = Mode::Normal;
                self.set_flash(format!(
                    "Events after {} ({} events)",
                    t.format("%Y-%m-%d %H:%M"),
                    self.filtered_indices.len()
                ));
                return;
            }
        }
        if let Some((a, b)) = s_lower.split_once(" to ") {
            if let (Some(t1), Some(t2)) = (parse_time(a), parse_time(b.trim())) {
                self.time_range_start = Some(t1);
                self.time_range_end = Some(t2);
                self.apply_filters();
                self.mode = Mode::Normal;
                self.set_flash(format!(
                    "{} to {} ({} events)",
                    t1.format("%Y-%m-%d %H:%M"),
                    t2.format("%Y-%m-%d %H:%M"),
                    self.filtered_indices.len()
                ));
                return;
            }
        }
        if let Some(rest) = s_lower.strip_prefix("before ") {
            if let Some(t) = parse_time(rest) {
                self.time_range_start = None;
                self.time_range_end = Some(t);
                self.apply_filters();
                self.mode = Mode::Normal;
                self.set_flash(format!(
                    "Events before {} ({} events)",
                    t.format("%Y-%m-%d %H:%M"),
                    self.filtered_indices.len()
                ));
                return;
            }
        }
        if let Some((a, b)) = raw.trim().split_once("..") {
            if let (Some(t1), Some(t2)) = (parse_time(a), parse_time(b.trim())) {
                self.time_range_start = Some(t1);
                self.time_range_end = Some(t2);
                self.apply_filters();
                self.mode = Mode::Normal;
                self.set_flash(format!(
                    "{} to {} ({} events)",
                    t1.format("%Y-%m-%d %H:%M"),
                    t2.format("%Y-%m-%d %H:%M"),
                    self.filtered_indices.len()
                ));
                return;
            }
        }
        if let Some(t) = parse_time(raw.trim()) {
            self.time_range_start = Some(t);
            self.time_range_end = None;
            self.apply_filters();
            self.mode = Mode::Normal;
            self.set_flash(format!(
                "Events from {} ({} events)",
                t.format("%Y-%m-%d %H:%M"),
                self.filtered_indices.len()
            ));
            return;
        }
        self.time_input = raw;
        self.set_flash(
            "Invalid time. Try: today, last 7 days, after <time>, <time> to <time>, clear"
                .to_string(),
        );
    }

    pub fn cancel_time_filter(&mut self) {
        match self.time_filter_sub {
            TimeFilterSub::Custom => {
                self.time_filter_sub = TimeFilterSub::Picker;
                self.time_input.clear();
                self.time_picker_list_state
                    .select(if TIME_PRESETS.is_empty() {
                        None
                    } else {
                        Some(0)
                    });
            }
            TimeFilterSub::CustomRangeEndHour(start_dt, _) => {
                self.date_picker_end_dates = self
                    .unique_dates
                    .iter()
                    .filter(|d| **d >= start_dt.date())
                    .cloned()
                    .collect();
                self.time_filter_sub = TimeFilterSub::CustomRangeEnd(start_dt);
                self.date_picker_list_state
                    .select(if self.date_picker_end_dates.is_empty() {
                        None
                    } else {
                        Some(0)
                    });
            }
            TimeFilterSub::CustomRangeEnd(start_dt) => {
                let start_date = start_dt.date();
                self.time_filter_sub = TimeFilterSub::CustomRangeStartHour(start_date);
                self.date_picker_hours = unique_hours_for_date(&self.events, start_date);
                let pos = self
                    .date_picker_hours
                    .iter()
                    .position(|&h| h == start_dt.hour())
                    .unwrap_or(0);
                self.date_picker_list_state
                    .select(if self.date_picker_hours.is_empty() {
                        None
                    } else {
                        Some(pos)
                    });
            }
            TimeFilterSub::CustomRangeStartHour(_) => {
                self.time_filter_sub = TimeFilterSub::CustomRangeStart;
                self.date_picker_hours.clear();
                self.date_picker_list_state
                    .select(Some(0.min(self.unique_dates.len().saturating_sub(1))));
            }
            TimeFilterSub::CustomRangeStart => {
                self.time_filter_sub = TimeFilterSub::Picker;
                self.time_picker_list_state
                    .select(if TIME_PRESETS.is_empty() {
                        None
                    } else {
                        Some(0)
                    });
            }
            TimeFilterSub::Picker => {
                self.time_input.clear();
                self.mode = Mode::Normal;
            }
        }
    }

    pub fn push_time_char(&mut self, c: char) {
        self.time_input.push(c);
    }

    pub fn pop_time_char(&mut self) {
        self.time_input.pop();
    }

    /// enter action-type filter mode; select current filter if any
    pub fn start_action_type_filter(&mut self) {
        self.mode = Mode::ActionTypeFilter;
        if self.action_types.is_empty() {
            self.action_type_list_state.select(None);
            return;
        }
        let idx = self
            .action_type_filter
            .as_ref()
            .and_then(|at| self.action_types.iter().position(|x| x == at))
            .unwrap_or(0);
        self.action_type_list_state
            .select(Some(idx.min(self.action_types.len().saturating_sub(1))));
    }

    /// set filter to selected action type and exit
    pub fn commit_action_type_filter(&mut self) {
        let at = self
            .action_type_list_state
            .selected()
            .and_then(|i| self.action_types.get(i))
            .cloned();
        if let Some(at) = at {
            self.action_type_filter = Some(at.clone());
            self.apply_filters();
            self.set_flash(format!(
                "Filter: {} ({} events)",
                at,
                self.filtered_indices.len()
            ));
        }
        self.mode = Mode::Normal;
    }

    /// clear action type filter and exit (from picker)
    pub fn clear_action_type_filter(&mut self) {
        self.action_type_filter = None;
        self.apply_filters();
        self.mode = Mode::Normal;
        self.set_flash("Filter cleared".to_string());
    }

    /// clear search, action-type filter, and/or time range from Normal mode
    pub fn clear_search_and_filter_in_normal(&mut self) {
        let had_search = !self.search.is_empty();
        let had_filter = self.action_type_filter.is_some();
        let had_time = self.time_range_start.is_some() || self.time_range_end.is_some();
        if had_search {
            self.search.clear();
        }
        if had_filter {
            self.action_type_filter = None;
        }
        if had_time {
            self.time_range_start = None;
            self.time_range_end = None;
        }
        if had_search || had_filter || had_time {
            self.apply_filters();
            let flash = match (had_search, had_filter, had_time) {
                (true, true, true) => "Search, filter & time range cleared",
                (true, true, false) => "Search and filter cleared",
                (true, false, true) => "Search and time range cleared",
                (true, false, false) => "Search cleared",
                (false, true, true) => "Filter and time range cleared",
                (false, true, false) => "Filter cleared",
                (false, false, true) => "Time range cleared",
                (false, false, false) => return,
            };
            self.set_flash(flash.to_string());
        }
    }

    pub fn action_type_next(&mut self) {
        let i = self
            .action_type_list_state
            .selected()
            .map(|i| (i + 1).min(self.action_types.len().saturating_sub(1)))
            .unwrap_or(0);
        self.action_type_list_state
            .select(if self.action_types.is_empty() {
                None
            } else {
                Some(i)
            });
    }

    pub fn action_type_previous(&mut self) {
        let i = self
            .action_type_list_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.action_type_list_state
            .select(if self.action_types.is_empty() {
                None
            } else {
                Some(i)
            });
    }
}
