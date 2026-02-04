// filter timeline events by action type and text search

use crate::timeline::TimelineEvent;
use chrono::{NaiveDate, Timelike};
use std::collections::HashSet;

/// filter events: optional action type exact match, optional substring search (case-insensitive)
pub fn filter_events<'a>(
    events: &'a [TimelineEvent],
    action_type_filter: Option<&str>,
    search: &str,
) -> Vec<&'a TimelineEvent> {
    let search = search.trim();
    let action_match = action_type_filter
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    events
        .iter()
        .filter(|ev| {
            if let Some(at) = action_match {
                if ev.action_type.as_deref() != Some(at) {
                    return false;
                }
            }
            ev.matches_search(search)
        })
        .collect()
}

/// collect unique action type strings from events (for filter dropdown/typeahead)
pub fn unique_action_types(events: &[TimelineEvent]) -> Vec<String> {
    let set: std::collections::HashSet<String> = events
        .iter()
        .filter_map(|e| e.action_type.clone())
        .filter(|s| !s.is_empty())
        .collect();
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v
}

/// collect unique dates that appear in event times (for time range picker)
pub fn unique_dates_from_events(events: &[TimelineEvent]) -> Vec<NaiveDate> {
    let set: HashSet<NaiveDate> = events
        .iter()
        .filter_map(|e| e.event_time_parsed().map(|dt| dt.date()))
        .collect();
    let mut v: Vec<NaiveDate> = set.into_iter().collect();
    v.sort();
    v
}

/// unique hours (0–23) that appear in event times on the given date
pub fn unique_hours_for_date(events: &[TimelineEvent], date: NaiveDate) -> Vec<u32> {
    let set: HashSet<u32> = events
        .iter()
        .filter_map(|e| e.event_time_parsed())
        .filter(|dt| dt.date() == date)
        .map(|dt| dt.hour())
        .collect();
    let mut v: Vec<u32> = set.into_iter().collect();
    v.sort();
    v
}
