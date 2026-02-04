// tui entrypoint: load csv, run event loop, draw

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;

use rusty_lens::tui::{
    self,
    app::{App, Mode},
};

#[derive(Parser, Debug)]
#[command(author, version, about = "Defender device timeline TUI (DFIR)")]
struct Args {
    /// path to defender timeline csv
    #[arg(value_name = "FILE")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = App::new(args.path)?;

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let chunks = tui::views::layout_chunks(f.area());
            tui::views::draw_list(f, chunks[0], &mut app);
            tui::views::draw_detail(f, chunks[1], &app);
            tui::views::draw_command_bar(f, chunks[2], &app);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match app.mode {
                    Mode::SearchInput => match key.code {
                        KeyCode::Enter => app.commit_search(),
                        KeyCode::Esc => app.cancel_search(),
                        KeyCode::Backspace => app.pop_search_char(),
                        KeyCode::Char(c) => app.push_search_char(c),
                        _ => {}
                    },
                    Mode::ActionTypeFilter => match key.code {
                        KeyCode::Enter => app.commit_action_type_filter(),
                        KeyCode::Esc => app.clear_action_type_filter(),
                        KeyCode::Char('j') | KeyCode::Down => app.action_type_next(),
                        KeyCode::Char('k') | KeyCode::Up => app.action_type_previous(),
                        _ => {}
                    },
                    Mode::TimeFilter => match &app.time_filter_sub {
                        rusty_lens::tui::app::TimeFilterSub::Picker => match key.code {
                            KeyCode::Enter => app.apply_time_picker_selection(),
                            KeyCode::Esc => app.cancel_time_filter(),
                            KeyCode::Char('j') | KeyCode::Down => app.time_picker_next(),
                            KeyCode::Char('k') | KeyCode::Up => app.time_picker_previous(),
                            _ => {}
                        },
                        rusty_lens::tui::app::TimeFilterSub::CustomRangeStart => match key.code {
                            KeyCode::Enter => app.apply_date_range_start(),
                            KeyCode::Esc => app.cancel_time_filter(),
                            KeyCode::Char('j') | KeyCode::Down => app.date_picker_next(),
                            KeyCode::Char('k') | KeyCode::Up => app.date_picker_previous(),
                            _ => {}
                        },
                        rusty_lens::tui::app::TimeFilterSub::CustomRangeStartHour(_) => {
                            match key.code {
                                KeyCode::Enter => app.apply_date_range_start_hour(),
                                KeyCode::Esc => app.cancel_time_filter(),
                                KeyCode::Char('j') | KeyCode::Down => app.date_picker_next(),
                                KeyCode::Char('k') | KeyCode::Up => app.date_picker_previous(),
                                _ => {}
                            }
                        }
                        rusty_lens::tui::app::TimeFilterSub::CustomRangeEnd(_) => match key.code {
                            KeyCode::Enter => app.apply_date_range_end(),
                            KeyCode::Esc => app.cancel_time_filter(),
                            KeyCode::Char('j') | KeyCode::Down => app.date_picker_next(),
                            KeyCode::Char('k') | KeyCode::Up => app.date_picker_previous(),
                            _ => {}
                        },
                        rusty_lens::tui::app::TimeFilterSub::CustomRangeEndHour(_, _) => {
                            match key.code {
                                KeyCode::Enter => app.apply_date_range_end_hour(),
                                KeyCode::Esc => app.cancel_time_filter(),
                                KeyCode::Char('j') | KeyCode::Down => app.date_picker_next(),
                                KeyCode::Char('k') | KeyCode::Up => app.date_picker_previous(),
                                _ => {}
                            }
                        }
                        rusty_lens::tui::app::TimeFilterSub::Custom => match key.code {
                            KeyCode::Enter => app.commit_time_filter(),
                            KeyCode::Esc => app.cancel_time_filter(),
                            KeyCode::Backspace => app.pop_time_char(),
                            KeyCode::Char(c) => app.push_time_char(c),
                            _ => {}
                        },
                    },
                    Mode::Normal => match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                            app.should_quit = true;
                            break;
                        }
                        (KeyCode::Char('x'), _) => app.clear_search_and_filter_in_normal(),
                        (KeyCode::Char('/'), _) => app.start_search(),
                        (KeyCode::Char('t'), _) => app.start_time_filter(),
                        (KeyCode::Char('a'), _) => app.start_action_type_filter(),
                        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => app.next(),
                        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => app.previous(),
                        (KeyCode::PageDown, _) => app.scroll_detail_down(5),
                        (KeyCode::PageUp, _) => app.scroll_detail_up(5),
                        _ => {}
                    },
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
