use std::io::Cursor;

use colored::Colorize;
use image::ImageReader;
use viuer::{print as viuer_print, Config as ViuerConfig};

use crate::api::types::*;

/// Render a list of places (used by search, nearby, resolve)
pub fn render_places(places: &[Place], label: &str) {
    if places.is_empty() {
        println!("{}", "No results found.".yellow());
        return;
    }

    println!(
        "{} {} {}",
        label.bold(),
        format!("({})", places.len()).dimmed(),
        "─".repeat(40).dimmed()
    );
    println!();

    for (i, place) in places.iter().enumerate() {
        render_place_summary(i + 1, place);
    }
}

/// Render a single place summary (one-line style for lists)
fn render_place_summary(index: usize, place: &Place) {
    let name = place
        .display_name
        .as_ref()
        .map(|n| n.text.as_str())
        .unwrap_or("Unknown");

    // Line 1: index + name + type
    let type_str = place
        .primary_type_display_name
        .as_ref()
        .map(|t| t.text.clone())
        .or_else(|| place.primary_type.clone())
        .unwrap_or_default();

    print!("  {} {}", format!("{}.", index).dimmed(), name.bold().cyan());
    if !type_str.is_empty() {
        print!("  {}", type_str.dimmed());
    }
    println!();

    // Line 2: rating + price + status
    let mut meta_parts: Vec<String> = Vec::new();
    if let Some(rating) = place.rating {
        let stars = star_string(rating);
        let count = place.user_rating_count.unwrap_or(0);
        meta_parts.push(format!("{} {} ({})", stars, rating, count));
    }
    if let Some(ref price) = place.price_level {
        meta_parts.push(price_level_display(price).to_string());
    }
    if let Some(ref status) = place.business_status {
        if status != "OPERATIONAL" {
            meta_parts.push(status.red().to_string());
        }
    }
    if !meta_parts.is_empty() {
        println!("     {}", meta_parts.join("  ·  "));
    }

    // Line 3: address
    if let Some(ref addr) = place.formatted_address {
        println!("     {}", addr.dimmed());
    }

    // Line 4: ID
    if !place.id.is_empty() {
        println!("     {} {}", "ID:".dimmed(), place.id.dimmed());
    }

    println!();
}

/// Render full place details with optional inline photo previews
pub fn render_place_details(place: &Place, photo_images: Option<&[Vec<u8>]>) {
    let name = place
        .display_name
        .as_ref()
        .map(|n| n.text.as_str())
        .unwrap_or("Unknown");

    println!("{}", "━".repeat(60).dimmed());
    println!("  {}", name.bold().cyan());

    if let Some(ref primary) = place.primary_type_display_name {
        println!("  {}", primary.text.dimmed());
    }
    println!("{}", "━".repeat(60).dimmed());

    // Rating
    if let Some(rating) = place.rating {
        let stars = star_string(rating);
        let count = place.user_rating_count.unwrap_or(0);
        println!("  {} {} {} {}", "Rating:".bold(), stars, rating, format!("({} reviews)", count).dimmed());
    }

    // Price level
    if let Some(ref price) = place.price_level {
        println!("  {} {}", "Price:".bold(), price_level_display(price));
    }

    // Status
    if let Some(ref status) = place.business_status {
        let colored_status = if status == "OPERATIONAL" {
            "Open".green().to_string()
        } else {
            status.red().to_string()
        };
        println!("  {} {}", "Status:".bold(), colored_status);
    }

    // Address
    if let Some(ref addr) = place.formatted_address {
        println!("  {} {}", "Address:".bold(), addr);
    }

    // Location
    if let Some(ref loc) = place.location {
        println!(
            "  {} {}, {}",
            "Location:".bold(),
            loc.latitude,
            loc.longitude
        );
    }

    // Phone
    if let Some(ref phone) = place.international_phone_number {
        println!("  {} {}", "Phone:".bold(), phone);
    } else if let Some(ref phone) = place.national_phone_number {
        println!("  {} {}", "Phone:".bold(), phone);
    }

    // Website
    if let Some(ref uri) = place.website_uri {
        println!("  {} {}", "Website:".bold(), uri.underline());
    }

    // Google Maps
    if let Some(ref uri) = place.google_maps_uri {
        println!("  {} {}", "Maps:".bold(), uri.underline());
    }

    // Editorial summary
    if let Some(ref summary) = place.editorial_summary {
        if let Some(ref text) = summary.text {
            println!();
            println!("  {}", "Summary".bold());
            println!("  {}", text);
        }
    }

    // Opening hours
    if let Some(ref hours) = place.current_opening_hours {
        if let Some(open) = hours.open_now {
            println!();
            let status = if open {
                "Open now".green().to_string()
            } else {
                "Closed".red().to_string()
            };
            println!("  {} {}", "Hours:".bold(), status);
        }
    }
    let hours_source = place
        .current_opening_hours
        .as_ref()
        .or(place.regular_opening_hours.as_ref());
    if let Some(hours) = hours_source {
        if let Some(ref descs) = hours.weekday_descriptions {
            for desc in descs {
                println!("    {}", desc.dimmed());
            }
        }
    }

    // Reviews
    if let Some(ref reviews) = place.reviews {
        if !reviews.is_empty() {
            println!();
            println!(
                "  {} {}",
                "Reviews".bold(),
                format!("({})", reviews.len()).dimmed()
            );
            for (i, review) in reviews.iter().take(3).enumerate() {
                render_review(i + 1, review);
            }
            if reviews.len() > 3 {
                println!(
                    "  {}",
                    format!("  ... and {} more reviews", reviews.len() - 3).dimmed()
                );
            }
        }
    }

    // Photos
    if let Some(ref photos) = place.photos {
        if !photos.is_empty() {
            println!();
            println!(
                "  {} {}",
                "Photos".bold(),
                format!("({})", photos.len()).dimmed()
            );
            for photo in photos.iter().take(3) {
                println!("    {}", photo.name.dimmed());
                if let Some(ref authors) = photo.author_attributions {
                    for author in authors {
                        println!("      by {}", author.display_name.dimmed());
                    }
                }
            }
            if photos.len() > 3 {
                println!(
                    "  {}",
                    format!("  ... and {} more photos", photos.len() - 3).dimmed()
                );
            }
        }
    }

    // Inline photo previews (if provided)
    if let Some(ref image_data) = photo_images {
        if !image_data.is_empty() {
            println!();
            println!("  {}", "Photo Previews".bold());
            for (i, bytes) in image_data.iter().enumerate() {
                println!("  {} {}", format!("Photo {}:", i + 1).dimmed(), "─".repeat(30).dimmed());
                render_image_bytes(bytes, 60, 15);
                println!();
            }
        }
    }

    // Place ID
    if !place.id.is_empty() {
        println!();
        println!("  {} {}", "Place ID:".dimmed(), place.id.dimmed());
    }

    println!();
}

fn render_review(index: usize, review: &Review) {
    let author = review
        .author_attribution
        .as_ref()
        .map(|a| a.display_name.as_str())
        .unwrap_or("Anonymous");
    let rating = review.rating.unwrap_or(0.0);
    let time = review
        .relative_publish_time_description
        .as_deref()
        .unwrap_or("");

    println!(
        "    {}. {} {} {}",
        index,
        author.bold(),
        star_string(rating),
        time.dimmed()
    );

    if let Some(ref text) = review.text {
        let display = truncate(&text.text, 200);
        println!("       {}", display);
    }
}

/// Render autocomplete suggestions
pub fn render_autocomplete(response: &AutocompleteResponse) {
    if response.suggestions.is_empty() {
        println!("{}", "No suggestions found.".yellow());
        return;
    }

    println!(
        "{} {} {}",
        "Suggestions".bold(),
        format!("({})", response.suggestions.len()).dimmed(),
        "─".repeat(40).dimmed()
    );
    println!();

    for (i, suggestion) in response.suggestions.iter().enumerate() {
        if let Some(ref place_pred) = suggestion.place_prediction {
            let text = place_pred
                .text
                .as_ref()
                .map(|t| t.text.as_str())
                .unwrap_or("?");
            let main = place_pred
                .structured_format
                .as_ref()
                .and_then(|sf| sf.main_text.as_ref())
                .map(|t| t.text.as_str());
            let secondary = place_pred
                .structured_format
                .as_ref()
                .and_then(|sf| sf.secondary_text.as_ref())
                .map(|t| t.text.as_str());

            print!("  {} ", format!("{}.", i + 1).dimmed());
            if let Some(main_text) = main {
                print!("{}", main_text.bold().cyan());
                if let Some(sec) = secondary {
                    print!("  {}", sec.dimmed());
                }
            } else {
                print!("{}", text.bold().cyan());
            }

            // Show types if available
            if let Some(ref types) = place_pred.types {
                let type_str = types.iter().take(2).cloned().collect::<Vec<_>>().join(", ");
                if !type_str.is_empty() {
                    print!("  [{}]", type_str.dimmed());
                }
            }

            println!();

            // Show place ID
            if let Some(ref pid) = place_pred.place_id {
                println!("     {} {}", "ID:".dimmed(), pid.dimmed());
            }
        } else if let Some(ref query_pred) = suggestion.query_prediction {
            let text = query_pred
                .text
                .as_ref()
                .map(|t| t.text.as_str())
                .unwrap_or("?");
            println!(
                "  {} {} {}",
                format!("{}.", i + 1).dimmed(),
                "🔍".dimmed(),
                text.bold()
            );
        }
        println!();
    }
}

/// Render photo media result, optionally displaying the image inline
pub fn render_photo(response: &PhotoMediaResponse, image_bytes: Option<&[u8]>) {
    println!("{}", "Photo".bold());
    println!("  {} {}", "Name:".bold(), response.name);
    println!("  {} {}", "URL:".bold(), response.photo_uri.underline());

    if let Some(bytes) = image_bytes {
        println!();
        render_image_bytes(bytes, 60, 15);
    }
}

/// Render image bytes inline using Unicode half-blocks (works in Alacritty + tmux)
pub fn render_image_bytes(bytes: &[u8], width: u32, height: u32) {
    let cursor = Cursor::new(bytes);
    let reader = match ImageReader::new(cursor).with_guessed_format() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  {}", format!("Could not read image: {}", e).dimmed());
            return;
        }
    };
    let img = match reader.decode() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("  {}", format!("Could not decode image: {}", e).dimmed());
            return;
        }
    };

    let conf = ViuerConfig {
        width: Some(width),
        height: Some(height),
        absolute_offset: false,
        use_kitty: false,
        use_iterm: false,
        truecolor: true,
        ..Default::default()
    };

    if let Err(e) = viuer_print(&img, &conf) {
        eprintln!("  {}", format!("Could not render image: {}", e).dimmed());
    }
}

/// Render route search results
pub fn render_route(response: &RouteSearchResponse) {
    println!(
        "{} {} {} {} {} {} {}",
        "Route".bold(),
        response.from.cyan(),
        "→".dimmed(),
        response.to.cyan(),
        format!("({})", response.travel_mode).dimmed(),
        "─".repeat(20).dimmed(),
        format!("{} waypoints", response.waypoints.len()).dimmed()
    );
    println!();

    for wp_result in &response.waypoints {
        println!(
            "  {} {} ({:.4}, {:.4})",
            format!("Waypoint {}:", wp_result.waypoint_index + 1).bold().yellow(),
            "📍",
            wp_result.waypoint.latitude,
            wp_result.waypoint.longitude
        );

        if wp_result.places.is_empty() {
            println!("    {}", "No places found near this waypoint.".dimmed());
        } else {
            for (j, place) in wp_result.places.iter().enumerate() {
                let name = place
                    .display_name
                    .as_ref()
                    .map(|n| n.text.as_str())
                    .unwrap_or("Unknown");
                let addr = place
                    .short_formatted_address
                    .as_deref()
                    .or(place.formatted_address.as_deref())
                    .unwrap_or("");

                print!("    {} {}", format!("{}.", j + 1).dimmed(), name.cyan());
                if let Some(rating) = place.rating {
                    print!("  {}", star_string(rating));
                }
                println!();
                if !addr.is_empty() {
                    println!("       {}", addr.dimmed());
                }
            }
        }
        println!();
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn star_string(rating: f64) -> String {
    let full = rating.floor() as usize;
    let half = if rating - rating.floor() >= 0.5 { 1 } else { 0 };
    let empty = 5usize.saturating_sub(full + half);
    format!(
        "{}{}{}",
        "★".repeat(full).yellow(),
        "⯪".repeat(half).yellow(),
        "☆".repeat(empty).dimmed()
    )
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        let end = s
            .char_indices()
            .take_while(|(i, _)| *i <= max_len)
            .last()
            .map(|(i, _)| i)
            .unwrap_or(max_len);
        &s[..end]
    }
}
