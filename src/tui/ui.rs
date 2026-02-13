use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::tui::app::{App, Focus};
use crate::tui::widgets::{filter_panel, place_details, places_list, search_bar, status_bar};

pub fn render(frame: &mut ratatui::Frame, app: &mut App) {
    let area = frame.area();

    // search bar | filter panel | main content | status bar
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search bar
            Constraint::Length(7), // filter panel (5 rows + border)
            Constraint::Min(5),   // main content
            Constraint::Length(1), // status bar
        ])
        .split(area);

    let search_area = vertical[0];
    let filter_area = vertical[1];
    let main_area = vertical[2];
    let status_area = vertical[3];

    // Render search bar
    search_bar::render_search_bar(search_area, frame.buffer_mut(), app);

    // Render filter panel
    filter_panel::render_filter_panel(filter_area, frame.buffer_mut(), app);

    // Main split pane: results list | details
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(55),
        ])
        .split(main_area);

    places_list::render_places_list(horizontal[0], frame.buffer_mut(), app);
    place_details::render_place_details(horizontal[1], frame.buffer_mut(), app);

    // Status bar
    status_bar::render_status_bar(status_area, frame.buffer_mut(), app);

    // Autocomplete dropdown overlay
    if !app.autocomplete.is_empty() && !app.input.value().is_empty() {
        let dropdown_y = search_area.y + search_area.height;
        let dropdown_height = (app.autocomplete.len() as u16 + 2).min(8);
        let dropdown_area = Rect {
            x: search_area.x,
            y: dropdown_y,
            width: search_area.width.min(area.width),
            height: dropdown_height.min(filter_area.height + main_area.height),
        };
        search_bar::render_autocomplete_dropdown(dropdown_area, frame.buffer_mut(), app);
    }

    // Type picker overlay (when editing the type filter)
    if app.focus == Focus::FilterEditing && !app.filter_type_matches.is_empty() {
        // Position below the Type row in the filter panel (row 0 + border = y+1)
        let picker_y = filter_area.y + filter_area.height;
        let picker_height = (app.filter_type_matches.len() as u16 + 2).min(8);
        let picker_width = 45.min(area.width);
        let picker_area = Rect {
            x: filter_area.x,
            y: picker_y,
            width: picker_width,
            height: picker_height.min(main_area.height),
        };
        render_type_picker(picker_area, frame.buffer_mut(), app);
    }
}

fn render_type_picker(area: Rect, buf: &mut ratatui::buffer::Buffer, app: &App) {
    Clear.render(area, buf);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Types (↑↓ select, Enter confirm) ");

    let inner = block.inner(area);
    block.render(area, buf);

    for (i, &type_name) in app
        .filter_type_matches
        .iter()
        .take(inner.height as usize)
        .enumerate()
    {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let style = if i == app.filter_type_match_idx {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default().fg(Color::White)
        };

        let row = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        };

        let display = if type_name.len() > inner.width as usize {
            &type_name[..inner.width as usize]
        } else {
            type_name
        };

        Paragraph::new(format!(" {}", display))
            .style(style)
            .render(row, buf);
    }
}
