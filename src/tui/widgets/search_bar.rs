use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::tui::app::{App, Focus};

pub fn render_search_bar(area: Rect, buf: &mut Buffer, app: &App) {
    let is_focused = app.focus == Focus::SearchInput;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Search ");

    let input_value = app.input.value();
    let cursor_pos = app.input.visual_cursor();

    // Build the input line with a visible cursor
    let (before, after) = input_value.split_at(
        input_value
            .char_indices()
            .nth(cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(input_value.len()),
    );

    let spans = if is_focused {
        let cursor_char = after.chars().next().unwrap_or(' ');
        let rest = if after.len() > cursor_char.len_utf8() {
            &after[cursor_char.len_utf8()..]
        } else {
            ""
        };
        vec![
            Span::raw(before.to_string()),
            Span::styled(
                cursor_char.to_string(),
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(rest.to_string()),
        ]
    } else {
        vec![Span::raw(input_value.to_string())]
    };

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    paragraph.render(area, buf);
}

pub fn render_autocomplete_dropdown(area: Rect, buf: &mut Buffer, app: &App) {
    if app.autocomplete.is_empty() || app.input.value().is_empty() {
        return;
    }

    let items: Vec<String> = app
        .autocomplete
        .iter()
        .map(|s| {
            if let Some(ref pp) = s.place_prediction {
                let main = pp
                    .structured_format
                    .as_ref()
                    .and_then(|sf| sf.main_text.as_ref())
                    .map(|t| t.text.as_str())
                    .unwrap_or("");
                let secondary = pp
                    .structured_format
                    .as_ref()
                    .and_then(|sf| sf.secondary_text.as_ref())
                    .map(|t| t.text.as_str())
                    .unwrap_or("");
                if secondary.is_empty() {
                    main.to_string()
                } else {
                    format!("{} — {}", main, secondary)
                }
            } else if let Some(ref qp) = s.query_prediction {
                let text = qp.text.as_ref().map(|t| t.text.as_str()).unwrap_or("?");
                format!("🔍 {}", text)
            } else {
                String::new()
            }
        })
        .collect();

    let count = items.len().min(5);
    if count == 0 {
        return;
    }

    // Position dropdown below the search bar
    let dropdown_height = count as u16 + 2; // +2 for borders
    let dropdown = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: dropdown_height.min(area.height),
    };

    // Clear the area first
    Clear.render(dropdown, buf);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Suggestions ");

    let inner = block.inner(dropdown);
    block.render(dropdown, buf);

    let is_ac_focused = app.focus == Focus::AutocompleteList;

    for (i, item) in items.iter().take(inner.height as usize).enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let style = if is_ac_focused && i == app.ac_selected {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default().fg(Color::White)
        };

        let line_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        };

        let truncated = if item.len() > inner.width as usize {
            &item[..inner.width as usize]
        } else {
            item
        };

        Paragraph::new(truncated.to_string())
            .style(style)
            .render(line_area, buf);
    }
}
