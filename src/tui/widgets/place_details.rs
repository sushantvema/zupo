use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::api::types::{price_level_display, Place};
use crate::tui::app::App;

pub fn render_place_details(area: Rect, buf: &mut Buffer, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Details ");

    let place = match &app.detail {
        Some(p) => p,
        None => {
            let empty =
                Paragraph::new("  Select a place to view details.")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(block);
            empty.render(area, buf);
            return;
        }
    };

    let lines = build_detail_lines(place);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    paragraph.render(area, buf);
}

fn build_detail_lines(place: &Place) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Name
    let name = place
        .display_name
        .as_ref()
        .map(|n| n.text.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    lines.push(Line::from(Span::styled(
        name,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));

    // Type
    if let Some(ref pt) = place.primary_type_display_name {
        lines.push(Line::from(Span::styled(
            pt.text.clone(),
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));

    // Rating
    if let Some(rating) = place.rating {
        let full = rating.floor() as usize;
        let half = if rating - rating.floor() >= 0.5 { 1 } else { 0 };
        let empty = 5usize.saturating_sub(full + half);
        let stars = format!(
            "{}{}{}",
            "★".repeat(full),
            "⯪".repeat(half),
            "☆".repeat(empty)
        );
        let count = place.user_rating_count.unwrap_or(0);
        lines.push(Line::from(vec![
            Span::styled("Rating: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(stars, Style::default().fg(Color::Yellow)),
            Span::raw(format!(" {} ({} reviews)", rating, count)),
        ]));
    }

    // Price
    if let Some(ref price) = place.price_level {
        lines.push(Line::from(vec![
            Span::styled("Price:  ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(price_level_display(price).to_string()),
        ]));
    }

    // Status
    if let Some(ref status) = place.business_status {
        let (display, color) = if status == "OPERATIONAL" {
            ("Open", Color::Green)
        } else {
            (status.as_str(), Color::Red)
        };
        lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(display.to_string(), Style::default().fg(color)),
        ]));
    }

    // Opening hours: open_now
    if let Some(ref hours) = place.current_opening_hours {
        if let Some(open) = hours.open_now {
            let (label, color) = if open {
                ("Open now", Color::Green)
            } else {
                ("Closed", Color::Red)
            };
            lines.push(Line::from(vec![
                Span::styled("        ", Style::default()),
                Span::styled(label.to_string(), Style::default().fg(color)),
            ]));
        }
    }

    lines.push(Line::from(""));

    // Address
    if let Some(ref addr) = place.formatted_address {
        lines.push(Line::from(vec![
            Span::styled("Address: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(addr.clone()),
        ]));
    }

    // Phone
    let phone = place
        .international_phone_number
        .as_deref()
        .or(place.national_phone_number.as_deref());
    if let Some(ph) = phone {
        lines.push(Line::from(vec![
            Span::styled("Phone:   ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(ph.to_string()),
        ]));
    }

    // Website
    if let Some(ref uri) = place.website_uri {
        lines.push(Line::from(vec![
            Span::styled("Website: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(uri.clone(), Style::default().fg(Color::Blue)),
        ]));
    }

    // Google Maps
    if let Some(ref uri) = place.google_maps_uri {
        lines.push(Line::from(vec![
            Span::styled("Maps:    ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(uri.clone(), Style::default().fg(Color::Blue)),
        ]));
    }

    // Editorial summary
    if let Some(ref summary) = place.editorial_summary {
        if let Some(ref text) = summary.text {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Summary",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(text.clone()));
        }
    }

    // Opening hours schedule
    let hours_source = place
        .current_opening_hours
        .as_ref()
        .or(place.regular_opening_hours.as_ref());
    if let Some(hours) = hours_source {
        if let Some(ref descs) = hours.weekday_descriptions {
            if !descs.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Hours",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                for desc in descs {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", desc),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }
    }

    // Reviews
    if let Some(ref reviews) = place.reviews {
        if !reviews.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("Reviews ({})", reviews.len()),
                Style::default().add_modifier(Modifier::BOLD),
            )));
            for (i, review) in reviews.iter().take(3).enumerate() {
                let author = review
                    .author_attribution
                    .as_ref()
                    .map(|a| a.display_name.clone())
                    .unwrap_or_else(|| "Anonymous".to_string());
                let rating = review.rating.unwrap_or(0.0);
                let time = review
                    .relative_publish_time_description
                    .as_deref()
                    .unwrap_or("");
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {}. {} ", i + 1, author),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("{:.1}★", rating), Style::default().fg(Color::Yellow)),
                    Span::styled(format!("  {}", time), Style::default().fg(Color::DarkGray)),
                ]));
                if let Some(ref text) = review.text {
                    let truncated = if text.text.len() > 150 {
                        format!("{}...", &text.text[..150])
                    } else {
                        text.text.clone()
                    };
                    lines.push(Line::from(Span::styled(
                        format!("     {}", truncated),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }
    }

    // Place ID
    if !place.id.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(place.id.clone(), Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines
}
