use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::app::{App, Focus};

pub fn render_status_bar(area: Rect, buf: &mut Buffer, app: &App) {
    let keybinds = match app.focus {
        Focus::SearchInput => "Enter: search  Tab: filters  ↓: suggestions  Esc: results",
        Focus::AutocompleteList => "j/↓: next  k/↑: prev  Enter: select  Esc: back",
        Focus::ResultsList => "j/↓: next  k/↑: prev  Enter: details  /: search  Tab/f: filters",
        Focus::FilterPanel => "j/↓/k/↑: navigate  Enter: edit/toggle  0-4: price  Tab: results  /: search",
        Focus::FilterEditing => "type value, Enter/Esc: confirm",
    };

    let mut spans = vec![Span::styled(
        format!(" {} ", keybinds),
        Style::default().fg(Color::DarkGray),
    )];

    if let Some((ref msg, is_error)) = app.status {
        spans.push(Span::raw(" │ "));
        let style = if is_error {
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        spans.push(Span::styled(msg.clone(), style));
    }

    if app.loading {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            "⟳",
            Style::default().fg(Color::Yellow),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Rgb(30, 30, 30)));

    paragraph.render(area, buf);
}
