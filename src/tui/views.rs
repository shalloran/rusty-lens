// draw event list, detail panel, command bar (hacker theme)

use crate::timeline::TimelineEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Wrap,
};
use std::rc::Rc;

use super::app::App;
use super::theme::Theme;

fn theme() -> Theme {
    Theme
}

pub fn layout_chunks(area: Rect) -> Rc<[Rect]> {
    let vertical = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .direction(Direction::Vertical)
        .split(area);
    let main_area = vertical[0];
    let bar_area = vertical[1];
    let horizontal = Layout::default()
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)].as_ref())
        .direction(Direction::Horizontal)
        .split(main_area);
    let mut out = vec![horizontal[0], horizontal[1]];
    out.push(bar_area);
    out.into()
}

pub fn draw_list(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    use super::app::{Mode, TimeFilterSub};

    if app.mode == Mode::ActionTypeFilter {
        draw_action_type_picker(f, area, app);
        return;
    }
    if app.mode == Mode::TimeFilter && app.time_filter_sub == TimeFilterSub::Picker {
        draw_time_picker(f, area, app);
        return;
    }
    if app.mode == Mode::TimeFilter
        && matches!(
            app.time_filter_sub,
            TimeFilterSub::CustomRangeStart
                | TimeFilterSub::CustomRangeStartHour(_)
                | TimeFilterSub::CustomRangeEnd(_)
                | TimeFilterSub::CustomRangeEndHour(_, _)
        )
    {
        draw_date_range_picker(f, area, app);
        return;
    }

    let t = theme();
    let has_time = app.time_range_start.is_some() || app.time_range_end.is_some();
    let has_filter = !app.search.is_empty() || app.action_type_filter.is_some() || has_time;
    let empty = app.filtered_indices.is_empty();

    if empty && has_filter {
        draw_no_results(f, area, app);
        return;
    }

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .take(5000)
        .filter_map(|&idx| app.events.get(idx))
        .map(|ev| {
            let line = ev.list_line();
            let line = truncate_for_display(&line, area.width.saturating_sub(4) as usize);
            ListItem::new(Line::from(Span::raw(line)))
        })
        .collect();

    let title = format!(" Events ({}) ", app.filtered_indices.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border_color()))
        .style(Style::default().bg(t.background_color()))
        .title(Span::styled(
            title,
            Style::default()
                .fg(t.title_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        ));

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(t.text_color()).bg(t.background_color()))
        .highlight_style(
            Style::default()
                .fg(t.highlight_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_no_results(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = theme();
    let mut lines = vec!["No events match.".to_string(), String::new()];
    if !app.search.is_empty() {
        lines.push(format!("Search: \"{}\"", app.search));
    }
    if let Some(ref at) = app.action_type_filter {
        lines.push(format!("Action type filter: {}", at));
    }
    if app.time_range_start.is_some() || app.time_range_end.is_some() {
        let tr = match (&app.time_range_start, &app.time_range_end) {
            (Some(s), None) => format!("after {}", s.format("%Y-%m-%d %H:%M")),
            (None, Some(e)) => format!("before {}", e.format("%Y-%m-%d %H:%M")),
            (Some(s), Some(e)) => format!(
                "{} to {}",
                s.format("%Y-%m-%d %H:%M"),
                e.format("%Y-%m-%d %H:%M")
            ),
            _ => String::new(),
        };
        if !tr.is_empty() {
            lines.push(format!("Time range: {}", tr));
        }
    }
    lines.push(String::new());
    lines.push("Try different terms or press [ x ] to clear search & filter.".to_string());
    let text = lines.join("\n");

    let title = " Events (0) — no results ";
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border_color()))
        .style(Style::default().bg(t.background_color()))
        .title(Span::styled(
            title,
            Style::default()
                .fg(t.title_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        ));

    let para = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(t.text_color()).bg(t.background_color()))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_time_picker(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    use super::app::TIME_PRESETS;

    let t = theme();
    let items: Vec<ListItem> = TIME_PRESETS
        .iter()
        .map(|s| {
            let line = truncate_for_display(s, area.width.saturating_sub(4) as usize);
            ListItem::new(Line::from(Span::raw(line)))
        })
        .collect();

    let title = " Esc back — Time range presets (Enter apply) ";
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border_color()))
        .style(Style::default().bg(t.background_color()))
        .title(Span::styled(
            title,
            Style::default()
                .fg(t.title_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        ));

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(t.text_color()).bg(t.background_color()))
        .highlight_style(
            Style::default()
                .fg(t.highlight_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.time_picker_list_state);
}

fn draw_date_range_picker(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    use super::app::TimeFilterSub;

    let t = theme();
    let (items, title): (Vec<ListItem>, String) = match &app.time_filter_sub {
        TimeFilterSub::CustomRangeStart => (
            app.unique_dates
                .iter()
                .map(|d| {
                    let line = truncate_for_display(
                        &d.format("%Y-%m-%d").to_string(),
                        area.width.saturating_sub(4) as usize,
                    );
                    ListItem::new(Line::from(Span::raw(line)))
                })
                .collect(),
            " Esc back — Pick start date (Enter) ".to_string(),
        ),
        TimeFilterSub::CustomRangeStartHour(date) => (
            app.date_picker_hours
                .iter()
                .map(|&h| {
                    let line = format!("{:02}:00", h);
                    ListItem::new(Line::from(Span::raw(line)))
                })
                .collect(),
            format!(
                " Esc back — Pick start hour for {} (Enter) ",
                date.format("%Y-%m-%d")
            ),
        ),
        TimeFilterSub::CustomRangeEnd(start_dt) => (
            app.date_picker_end_dates
                .iter()
                .map(|d| {
                    let line = truncate_for_display(
                        &d.format("%Y-%m-%d").to_string(),
                        area.width.saturating_sub(4) as usize,
                    );
                    ListItem::new(Line::from(Span::raw(line)))
                })
                .collect(),
            format!(
                " Esc back — Pick end date (>= {}) (Enter) ",
                start_dt.format("%Y-%m-%d")
            ),
        ),
        TimeFilterSub::CustomRangeEndHour(_start_dt, end_date) => (
            app.date_picker_hours
                .iter()
                .map(|&h| {
                    let line = format!("{:02}:00", h);
                    ListItem::new(Line::from(Span::raw(line)))
                })
                .collect(),
            format!(
                " Esc back — Pick end hour for {} (Enter apply) ",
                end_date.format("%Y-%m-%d")
            ),
        ),
        _ => return,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border_color()))
        .style(Style::default().bg(t.background_color()))
        .title(Span::styled(
            title.as_str(),
            Style::default()
                .fg(t.title_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        ));

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(t.text_color()).bg(t.background_color()))
        .highlight_style(
            Style::default()
                .fg(t.highlight_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.date_picker_list_state);
}

fn draw_action_type_picker(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let t = theme();
    let items: Vec<ListItem> = app
        .action_types
        .iter()
        .map(|s| {
            let line = truncate_for_display(s, area.width.saturating_sub(4) as usize);
            ListItem::new(Line::from(Span::raw(line)))
        })
        .collect();

    let title = " Esc to go back — Filter by action type (Enter apply) ";
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border_color()))
        .style(Style::default().bg(t.background_color()))
        .title(Span::styled(
            title,
            Style::default()
                .fg(t.title_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        ));

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(t.text_color()).bg(t.background_color()))
        .highlight_style(
            Style::default()
                .fg(t.highlight_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.action_type_list_state);
}

pub fn draw_detail(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = theme();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border_color()))
        .style(Style::default().bg(t.background_color()))
        .title(Span::styled(
            " Detail ",
            Style::default()
                .fg(t.title_color())
                .bg(t.background_color())
                .add_modifier(Modifier::BOLD),
        ));

    let content = if let Some(ev) = app.selected_event() {
        detail_content(ev, area.width.saturating_sub(4) as usize)
    } else {
        "Select an event.".to_string()
    };

    let para = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(t.text_color()).bg(t.background_color()))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    f.render_widget(para, area);

    // scrollbar for detail
    let total_lines = app
        .selected_event()
        .map(|e| e.detail_lines().len())
        .unwrap_or(0) as u16;
    let mut scroll_state = ScrollbarState::new(total_lines.saturating_add(2) as usize)
        .position(app.detail_scroll as usize);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .thumb_style(
            Style::default()
                .fg(t.highlight_color())
                .bg(t.background_color()),
        )
        .track_style(
            Style::default()
                .fg(t.border_color())
                .bg(t.background_color()),
        );
    f.render_stateful_widget(scrollbar, area, &mut scroll_state);
}

fn detail_content(ev: &TimelineEvent, width: usize) -> String {
    let lines = ev.detail_lines();
    let mut out = String::new();
    for (label, value) in lines {
        let full = format!("{}: {}", label, value);
        for chunk in wrap_at_width(&full, width) {
            out.push_str(&chunk);
            out.push('\n');
        }
    }
    if out.is_empty() {
        out.push_str("(no fields)");
    }
    out
}

fn wrap_at_width(s: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![s.to_string()];
    }
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        let trial = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };
        if trial.len() <= width {
            line = trial;
        } else {
            if !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            if word.len() > width {
                for c in word.chars() {
                    if line.len() >= width {
                        out.push(std::mem::take(&mut line));
                    }
                    line.push(c);
                }
            } else {
                line = word.to_string();
            }
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    if out.is_empty() && s.is_empty() {
        out.push(String::new());
    } else if out.is_empty() {
        out.push(s.to_string());
    }
    out
}

fn truncate_for_display(s: &str, max_len: usize) -> String {
    let mut w = 0;
    for (i, c) in s.char_indices() {
        w += unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
        if w > max_len {
            return format!("{}…", &s[..i]);
        }
    }
    s.to_string()
}

/// vim-style mode label for the command bar
fn mode_label(mode: super::app::Mode) -> &'static str {
    match mode {
        super::app::Mode::Normal => " NORMAL ",
        super::app::Mode::SearchInput => " SEARCH ",
        super::app::Mode::ActionTypeFilter => " FILTER ",
        super::app::Mode::TimeFilter => " TIME ",
    }
}

pub fn draw_command_bar(f: &mut ratatui::Frame, area: Rect, app: &App) {
    use super::app::Mode;

    let t = theme();
    // split bar: fixed-width mode pill on the left, hints on the right
    let bar_chunks = Layout::default()
        .constraints([Constraint::Length(10), Constraint::Min(10)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);

    // mode indicator (vim-style): standout so it's obvious where you are
    let mode_style = Style::default()
        .fg(ratatui::style::Color::Black)
        .bg(t.highlight_color())
        .add_modifier(Modifier::BOLD);
    let mode_para = Paragraph::new(Line::from(Span::styled(mode_label(app.mode), mode_style)))
        .style(
            Style::default()
                .fg(t.command_bar_text_color())
                .bg(t.border_color()),
        );
    f.render_widget(mode_para, bar_chunks[0]);

    // right side: hints (and search buffer in SEARCH mode)
    let (hint_text, hint_align) = match app.mode {
        Mode::SearchInput => (
            format!(
                "Search: {}_  [ Enter ] apply  [ Esc ] cancel",
                app.search_input
            ),
            ratatui::layout::Alignment::Left,
        ),
        Mode::ActionTypeFilter => (
            " Esc to go back  |  j/k move  Enter apply".to_string(),
            ratatui::layout::Alignment::Left,
        ),
        Mode::TimeFilter => {
            use super::app::TimeFilterSub;
            let (hint, align) = match &app.time_filter_sub {
                TimeFilterSub::Picker => (
                    " j/k move  Enter apply or open Custom  Esc back".to_string(),
                    ratatui::layout::Alignment::Left,
                ),
                TimeFilterSub::CustomRangeStart => (
                    " j/k move  Enter pick start date  Esc back".to_string(),
                    ratatui::layout::Alignment::Left,
                ),
                TimeFilterSub::CustomRangeEnd(_) => (
                    " j/k move  Enter pick end date  Esc back".to_string(),
                    ratatui::layout::Alignment::Left,
                ),
                TimeFilterSub::CustomRangeStartHour(_) => (
                    " j/k move  Enter pick start hour  Esc back".to_string(),
                    ratatui::layout::Alignment::Left,
                ),
                TimeFilterSub::CustomRangeEndHour(_, _) => (
                    " j/k move  Enter pick end hour & apply  Esc back".to_string(),
                    ratatui::layout::Alignment::Left,
                ),
                TimeFilterSub::Custom => (
                    format!(
                        "Time: {}_  today, last 7 days, after/before, <t> to <t>  Enter apply  Esc back",
                        app.time_input
                    ),
                    ratatui::layout::Alignment::Left,
                ),
            };
            (hint, align)
        }
        Mode::Normal => {
            let has_time = app.time_range_start.is_some() || app.time_range_end.is_some();
            let mut s = match (
                !app.search.is_empty(),
                app.action_type_filter.is_some(),
                has_time,
            ) {
                (true, true, true) => "[ x ] clear all  |  ".to_string(),
                (true, true, false) => "[ x ] clear search & filter  |  ".to_string(),
                (true, false, true) => "[ x ] clear all  |  ".to_string(),
                (true, false, false) => "[ x ] clear search  |  ".to_string(),
                (false, true, true) => "[ x ] clear all  |  ".to_string(),
                (false, true, false) => "[ x ] clear filter  |  ".to_string(),
                (false, false, true) => "[ x ] clear time  |  ".to_string(),
                (false, false, false) => String::new(),
            };
            s.push_str("[ j/k ] up/down  [ / ] search  [ a ] filter  [ t ] time  [ q ] quit");
            if let Some(ref flash) = app.flash {
                s.push_str("  |  ");
                s.push_str(flash);
            }
            (s, ratatui::layout::Alignment::Left)
        }
    };
    let hint_para = Paragraph::new(Line::from(Span::raw(hint_text)))
        .style(
            Style::default()
                .fg(t.command_bar_text_color())
                .bg(t.border_color()),
        )
        .alignment(hint_align)
        .wrap(Wrap { trim: true });
    f.render_widget(hint_para, bar_chunks[1]);
}
