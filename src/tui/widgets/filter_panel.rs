use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::app::{App, FilterField, Focus};

pub fn render_filter_panel(area: Rect, buf: &mut Buffer, app: &App) {
    let is_focused = app.focus == Focus::FilterPanel || app.focus == Focus::FilterEditing;

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Filters (0-4: toggle price) ");

    let inner = block.inner(area);
    block.render(area, buf);

    let rows = [
        render_type_row(app),
        render_radius_row(app),
        render_min_rating_row(app),
        render_price_row(app),
        render_open_now_row(app),
    ];

    for (i, line) in rows.iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }
        let row_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        };

        let is_selected = is_focused && app.filter_selected == i;
        let base_style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Render selection indicator
        let indicator = if is_selected { "▶ " } else { "  " };
        let mut spans = vec![Span::styled(
            indicator.to_string(),
            Style::default().fg(Color::Yellow),
        )];
        spans.extend(line.clone());

        Paragraph::new(Line::from(spans))
            .style(base_style)
            .render(row_area, buf);
    }
}

fn render_type_row(app: &App) -> Vec<Span<'static>> {
    let is_editing = app.focus == Focus::FilterEditing
        && FilterField::from_index(app.filter_selected) == FilterField::Type;
    let val = app.filter_type_input.value();

    let mut spans = vec![
        Span::styled(
            "Type:       ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];

    if is_editing {
        let cursor_pos = app.filter_type_input.visual_cursor();
        let (before, after) = val.split_at(
            val.char_indices()
                .nth(cursor_pos)
                .map(|(i, _)| i)
                .unwrap_or(val.len()),
        );
        let cursor_char = after.chars().next().unwrap_or(' ');
        let rest = if after.len() > cursor_char.len_utf8() {
            &after[cursor_char.len_utf8()..]
        } else {
            ""
        };
        spans.push(Span::raw(before.to_string()));
        spans.push(Span::styled(
            cursor_char.to_string(),
            Style::default().bg(Color::White).fg(Color::Black),
        ));
        spans.push(Span::raw(rest.to_string()));
    } else if val.is_empty() {
        spans.push(Span::styled(
            "any (e.g. restaurant, cafe, bar, thai_restaurant)".to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        spans.push(Span::styled(val.to_string(), Style::default().fg(Color::Cyan)));
    }

    spans
}

fn render_radius_row(app: &App) -> Vec<Span<'static>> {
    let r = app.filter_radius;
    let display = if r >= 1000.0 {
        format!("{:.0} km", r / 1000.0)
    } else {
        format!("{:.0} m", r)
    };

    vec![
        Span::styled(
            "Radius:     ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(display, Style::default().fg(Color::Cyan)),
        Span::styled(
            "  (Enter to cycle)".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ]
}

fn render_min_rating_row(app: &App) -> Vec<Span<'static>> {
    let display = match app.filter_min_rating {
        None => "any".to_string(),
        Some(r) => format!("{:.1}+", r),
    };

    let color = if app.filter_min_rating.is_some() {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    vec![
        Span::styled(
            "Min Rating: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(display, Style::default().fg(color)),
        Span::styled(
            "  (Enter to cycle)".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ]
}

fn render_price_row(app: &App) -> Vec<Span<'static>> {
    let labels = ["Free", "$", "$$", "$$$", "$$$$"];
    let mut spans = vec![
        Span::styled(
            "Price:      ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];

    let any_active = app.filter_price_levels.iter().any(|&v| v);

    if !any_active {
        spans.push(Span::styled(
            "any".to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    for (i, &label) in labels.iter().enumerate() {
        if any_active {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            if app.filter_price_levels[i] {
                spans.push(Span::styled(
                    format!("[{}]", label),
                    Style::default().fg(Color::Green),
                ));
            } else {
                spans.push(Span::styled(
                    format!(" {} ", label),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
    }

    // Show toggle hint even when "any"
    if !any_active {
        spans.push(Span::styled(
            "  (0-4 to toggle)".to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    spans
}

fn render_open_now_row(app: &App) -> Vec<Span<'static>> {
    let (display, color) = if app.filter_open_now {
        ("Yes", Color::Green)
    } else {
        ("No", Color::DarkGray)
    };

    vec![
        Span::styled(
            "Open Now:   ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(display.to_string(), Style::default().fg(color)),
        Span::styled(
            "  (Enter to toggle)".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ]
}
