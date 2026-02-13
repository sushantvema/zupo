use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, StatefulWidget, Widget};

use crate::api::types::{price_level_display, Place};
use crate::tui::app::{App, Focus};

pub fn render_places_list(area: Rect, buf: &mut Buffer, app: &mut App) {
    let is_focused = app.focus == Focus::ResultsList;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if app.results.is_empty() {
        " Results ".to_string()
    } else {
        format!(" Results ({}) ", app.results.len())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    if app.results.is_empty() {
        let empty = ratatui::widgets::Paragraph::new("  No results yet. Type a query and press Enter.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        empty.render(area, buf);
        return;
    }

    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, place)| place_to_list_item(i, place))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    StatefulWidget::render(list, area, buf, &mut app.results_state);
}

fn place_to_list_item(index: usize, place: &Place) -> ListItem<'static> {
    let name = place
        .display_name
        .as_ref()
        .map(|n| n.text.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let type_str = place
        .primary_type_display_name
        .as_ref()
        .map(|t| t.text.clone())
        .or_else(|| place.primary_type.clone())
        .unwrap_or_default();

    // Line 1: name + type
    let mut line1_spans = vec![
        Span::styled(
            format!("{}. ", index + 1),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ];
    if !type_str.is_empty() {
        line1_spans.push(Span::raw("  "));
        line1_spans.push(Span::styled(type_str, Style::default().fg(Color::DarkGray)));
    }

    // Line 2: rating + price
    let mut meta_parts: Vec<Span> = Vec::new();
    if let Some(rating) = place.rating {
        let stars = tui_star_string(rating);
        let count = place.user_rating_count.unwrap_or(0);
        meta_parts.push(Span::styled(stars, Style::default().fg(Color::Yellow)));
        meta_parts.push(Span::raw(format!(" {} ({})", rating, count)));
    }
    if let Some(ref price) = place.price_level {
        if !meta_parts.is_empty() {
            meta_parts.push(Span::styled("  ·  ", Style::default().fg(Color::DarkGray)));
        }
        meta_parts.push(Span::raw(price_level_display(price).to_string()));
    }

    // Line 3: address
    let addr = place
        .formatted_address
        .as_deref()
        .or(place.short_formatted_address.as_deref())
        .unwrap_or("");

    let mut lines = vec![Line::from(line1_spans)];
    if !meta_parts.is_empty() {
        lines.push(Line::from(meta_parts));
    }
    if !addr.is_empty() {
        lines.push(Line::from(Span::styled(
            addr.to_string(),
            Style::default().fg(Color::DarkGray),
        )));
    }
    // Blank line separator
    lines.push(Line::from(""));

    ListItem::new(lines)
}

fn tui_star_string(rating: f64) -> String {
    let full = rating.floor() as usize;
    let half = if rating - rating.floor() >= 0.5 { 1 } else { 0 };
    let empty = 5usize.saturating_sub(full + half);
    format!(
        "{}{}{}",
        "★".repeat(full),
        "⯪".repeat(half),
        "☆".repeat(empty)
    )
}
