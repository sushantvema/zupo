mod app;
mod event;
mod place_types;
mod ui;
mod widgets;

use std::sync::Arc;
use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tracing::{error, info};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use app::{App, FilterField, Focus};
use event::Action;

use crate::api::client::Client;
use crate::config::Config;

/// How quickly two Ctrl+C presses must occur to quit (ms)
const DOUBLE_CTRL_C_MS: u128 = 500;

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = dirs::config_dir()
        .map(|d| d.join("zupo"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let file_appender = tracing_appender::rolling::never(&log_dir, "tui.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .init();
    guard
}

pub async fn run(client: Client, config: Config) -> anyhow::Result<()> {
    let _log_guard = init_logging();
    info!("TUI started");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

    let client = Arc::new(client);
    let mut app = App::new(client, config, action_tx);

    let mut event_stream = crossterm::event::EventStream::new();

    // Main event loop
    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        tokio::select! {
            Some(Ok(evt)) = event_stream.next() => {
                handle_crossterm_event(evt, &mut app);
            }
            Some(action) = action_rx.recv() => {
                handle_action(action, &mut app);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_crossterm_event(evt: Event, app: &mut App) {
    if let Event::Key(key) = evt {
        // Only handle key press events (not release/repeat)
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Double Ctrl+C to quit
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            let now = Instant::now();
            if let Some(last) = app.last_ctrl_c {
                if now.duration_since(last).as_millis() < DOUBLE_CTRL_C_MS {
                    app.should_quit = true;
                    return;
                }
            }
            app.last_ctrl_c = Some(now);
            app.status = Some(("Press Ctrl+C again to quit".to_string(), false));
            return;
        }

        // Any other key clears the Ctrl+C state
        app.last_ctrl_c = None;

        match app.focus {
            Focus::SearchInput => handle_search_input(key, app),
            Focus::AutocompleteList => handle_autocomplete_nav(key, app),
            Focus::ResultsList => handle_results_nav(key, app),
            Focus::FilterPanel => handle_filter_panel(key, app),
            Focus::FilterEditing => handle_filter_editing(key, app),
        }
    }
}

fn handle_search_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Enter => {
            let query = app.input.value().to_string();
            app.execute_search(query);
            app.focus = Focus::ResultsList;
        }
        KeyCode::Esc => {
            if !app.autocomplete.is_empty() {
                app.autocomplete.clear();
                app.ac_selected = 0;
            } else if !app.results.is_empty() {
                app.focus = Focus::ResultsList;
            }
        }
        KeyCode::Down => {
            if !app.autocomplete.is_empty() {
                app.focus = Focus::AutocompleteList;
                app.ac_selected = 0;
            } else {
                app.focus = Focus::FilterPanel;
            }
        }
        KeyCode::Tab => {
            // Tab always goes to filter panel (skip autocomplete)
            app.autocomplete.clear();
            app.ac_selected = 0;
            app.focus = Focus::FilterPanel;
        }
        KeyCode::Char('q') if app.input.value().is_empty() => {
            app.should_quit = true;
        }
        _ => {
            app.input.handle_event(&Event::Key(key));
            app.trigger_autocomplete();
        }
    }
}

fn handle_autocomplete_nav(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if app.ac_selected + 1 < app.autocomplete.len() {
                app.ac_selected += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.ac_selected = app.ac_selected.saturating_sub(1);
        }
        KeyCode::Enter => {
            if let Some(suggestion) = app.autocomplete.get(app.ac_selected) {
                let query = if let Some(ref pp) = suggestion.place_prediction {
                    pp.text
                        .as_ref()
                        .map(|t| t.text.clone())
                        .unwrap_or_default()
                } else if let Some(ref qp) = suggestion.query_prediction {
                    qp.text
                        .as_ref()
                        .map(|t| t.text.clone())
                        .unwrap_or_default()
                } else {
                    String::new()
                };

                if !query.is_empty() {
                    app.input = tui_input::Input::new(query.clone());
                    app.execute_search(query);
                    app.focus = Focus::ResultsList;
                }
            }
        }
        KeyCode::Esc => {
            app.autocomplete.clear();
            app.ac_selected = 0;
            app.focus = Focus::SearchInput;
        }
        _ => {}
    }
}

fn handle_results_nav(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('/') => {
            app.focus = Focus::SearchInput;
        }
        KeyCode::Tab | KeyCode::Char('f') => {
            app.focus = Focus::FilterPanel;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_result();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev_result();
        }
        KeyCode::Enter => {
            app.fetch_details();
        }
        KeyCode::Char('g') => {
            app.detail_scroll = app.detail_scroll.saturating_sub(3);
        }
        KeyCode::Char('G') => {
            app.detail_scroll = app.detail_scroll.saturating_add(3);
        }
        _ => {}
    }
}

fn handle_filter_panel(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => {
            app.focus = Focus::SearchInput;
        }
        KeyCode::Tab => {
            app.focus = Focus::ResultsList;
        }
        KeyCode::Char('/') => {
            app.focus = Focus::SearchInput;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.filter_selected = (app.filter_selected + 1) % FilterField::ALL.len();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.filter_selected == 0 {
                app.filter_selected = FilterField::ALL.len() - 1;
            } else {
                app.filter_selected -= 1;
            }
        }
        KeyCode::Enter => {
            let field = FilterField::from_index(app.filter_selected);
            match field {
                FilterField::Type => {
                    app.focus = Focus::FilterEditing;
                }
                FilterField::Radius => {
                    app.cycle_radius();
                }
                FilterField::MinRating => {
                    app.cycle_min_rating();
                }
                FilterField::Price => {
                    let active_count = app.filter_price_levels.iter().filter(|&&v| v).count();
                    if active_count == 0 {
                        app.filter_price_levels[0] = true;
                    } else {
                        let highest = app
                            .filter_price_levels
                            .iter()
                            .rposition(|&v| v)
                            .unwrap_or(0);
                        app.filter_price_levels = [false; 5];
                        if highest + 1 < 5 {
                            app.filter_price_levels[highest + 1] = true;
                        }
                    }
                }
                FilterField::OpenNow => {
                    app.filter_open_now = !app.filter_open_now;
                }
            }
        }
        KeyCode::Char(c @ '0'..='4') => {
            let idx = (c as u8 - b'0') as usize;
            app.filter_price_levels[idx] = !app.filter_price_levels[idx];
        }
        _ => {}
    }
}

fn handle_filter_editing(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => {
            app.filter_type_matches.clear();
            app.filter_type_match_idx = 0;
            app.focus = Focus::FilterPanel;
        }
        KeyCode::Enter => {
            // If there are matches and one is highlighted, pick it
            if !app.filter_type_matches.is_empty() {
                let selected = app
                    .filter_type_matches
                    .get(app.filter_type_match_idx)
                    .copied()
                    .unwrap_or("");
                if !selected.is_empty() {
                    app.filter_type_input = Input::new(selected.to_string());
                }
            }
            app.filter_type_matches.clear();
            app.filter_type_match_idx = 0;
            app.focus = Focus::FilterPanel;
        }
        KeyCode::Down | KeyCode::Tab => {
            if !app.filter_type_matches.is_empty() {
                if app.filter_type_match_idx + 1 < app.filter_type_matches.len() {
                    app.filter_type_match_idx += 1;
                }
            }
        }
        KeyCode::Up => {
            app.filter_type_match_idx = app.filter_type_match_idx.saturating_sub(1);
        }
        _ => {
            app.filter_type_input.handle_event(&Event::Key(key));
            app.update_type_matches();
        }
    }
}

fn handle_action(action: Action, app: &mut App) {
    match action {
        Action::AutocompleteResult(Ok(resp)) => {
            info!("Autocomplete: {} suggestions", resp.suggestions.len());
            app.autocomplete = resp.suggestions;
            app.ac_selected = 0;
        }
        Action::AutocompleteResult(Err(e)) => {
            error!("Autocomplete error: {}", e);
            app.autocomplete.clear();
            app.status = Some((format!("Autocomplete error: {}", e), true));
        }
        Action::SearchResult(Ok(resp)) => {
            info!("Search: {} results", resp.places.len());
            app.loading = false;
            if resp.places.is_empty() {
                app.status = Some(("No results found.".to_string(), false));
                app.results.clear();
                app.results_state.select(None);
                app.detail = None;
            } else {
                app.status = Some((format!("{} results", resp.places.len()), false));
                app.results = resp.places;
                app.results_state.select(Some(0));
                app.detail = app.results.first().cloned();
                app.detail_scroll = 0;
            }
        }
        Action::SearchResult(Err(e)) => {
            error!("Search error: {}", e);
            app.loading = false;
            app.status = Some((format!("Search error: {}", e), true));
        }
        Action::DetailsResult(Ok(place)) => {
            let name = place
                .display_name
                .as_ref()
                .map(|n| n.text.as_str())
                .unwrap_or("?");
            info!("Details loaded: {}", name);
            app.loading = false;
            app.status = Some(("Details loaded.".to_string(), false));
            app.detail = Some(place);
            app.detail_scroll = 0;
        }
        Action::DetailsResult(Err(e)) => {
            error!("Details error: {}", e);
            app.loading = false;
            app.status = Some((format!("Details error: {}", e), true));
        }
    }
}
