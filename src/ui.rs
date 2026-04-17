use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::app::{App, AircraftDisplay};

// ── Entry point ──────────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),    // aircraft table
            Constraint::Length(1), // footer / key hints
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_table(f, app, chunks[1]);
    render_footer(f, chunks[2]);
}

// ── Header ───────────────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let update_str = app
        .last_update
        .map(|t| t.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "—".to_string());

    let spinner = if app.is_loading { " ⟳" } else { "" };

    let content = Line::from(vec![
        Span::styled(
            " ✈  LOCAL AIR TRAFFIC",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}, {}", app.location.city, app.location.country),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({:.4}°, {:.4}°)", app.location.lat, app.location.lon),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("r={:.0} mi", app.radius_miles),
            Style::default().fg(Color::Green),
        ),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.status, Style::default().fg(Color::White)),
        Span::styled(spinner, Style::default().fg(Color::Magenta)),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled("updated ", Style::default().fg(Color::DarkGray)),
        Span::styled(update_str, Style::default().fg(Color::Gray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(Paragraph::new(content).block(block), area);
}

// ── Aircraft table ───────────────────────────────────────────────────────────

/// Colour-code a row by proximity.  On-ground aircraft are always dim.
fn dist_color(ac: &AircraftDisplay) -> Color {
    if ac.on_ground {
        Color::DarkGray
    } else if ac.distance_miles < 3.0 {
        Color::LightGreen
    } else if ac.distance_miles < 6.0 {
        Color::Yellow
    } else {
        Color::White
    }
}

/// Unicode compass arrow for a heading in degrees.
fn heading_arrow(deg: f64) -> char {
    let idx = ((deg + 22.5) / 45.0) as usize % 8;
    ['↑', '↗', '→', '↘', '↓', '↙', '←', '↖'][idx]
}

fn render_table(f: &mut Frame, app: &App, area: Rect) {
    // Column headers
    let header = Row::new([
        Cell::from("DIST").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("CALLSIGN").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("REG").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("TYPE").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("ALTITUDE").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("SPEED").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("HDG").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("V/S").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .aircraft
        .iter()
        .map(|ac| build_row(ac))
        .collect();

    let widths = [
        Constraint::Length(8),  // DIST
        Constraint::Length(10), // CALLSIGN
        Constraint::Length(8),  // REG
        Constraint::Fill(1),    // TYPE  — elastic, takes spare width
        Constraint::Length(11), // ALTITUDE
        Constraint::Length(8),  // SPEED
        Constraint::Length(8),  // HDG
        Constraint::Length(9),  // V/S
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Aircraft ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::DarkGray));

    if app.aircraft.is_empty() {
        let msg = if app.is_loading {
            "Fetching live traffic…"
        } else {
            "No aircraft detected within range."
        };
        f.render_widget(
            Paragraph::new(msg)
                .block(block)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .column_spacing(1);

    let mut state = TableState::default().with_selected(Some(app.scroll));
    f.render_stateful_widget(table, area, &mut state);
}

fn build_row(ac: &AircraftDisplay) -> Row<'static> {
    let color = dist_color(ac);
    let base = Style::default().fg(color);

    // Distance
    let dist = format!("{:.1} mi", ac.distance_miles);

    // Callsign — always bright blue
    let callsign = ac.callsign.clone();

    // Registration
    let reg = ac.registration.clone().unwrap_or_else(|| "—".to_string());

    // Type — prefer full type, fall back to ICAO hex
    let ac_type = ac
        .aircraft_type
        .clone()
        .unwrap_or_else(|| ac.icao24.to_uppercase());

    // Altitude
    let altitude = if ac.on_ground {
        "Ground".to_string()
    } else {
        match ac.altitude_ft {
            Some(ft) if ft > 0.0 => format!("{ft:.0} ft"),
            _ => "—".to_string(),
        }
    };

    // Speed
    let speed = match ac.speed_knots {
        Some(kt) => format!("{kt:.0} kt"),
        None => "—".to_string(),
    };

    // Heading
    let heading = match ac.heading_deg {
        Some(d) => format!("{} {d:.0}°", heading_arrow(d)),
        None => "—".to_string(),
    };

    // Vertical speed
    let (vs_str, vs_color) = if ac.on_ground {
        ("—".to_string(), Color::DarkGray)
    } else {
        match ac.vertical_rate_fpm {
            Some(fpm) if fpm > 100.0 => (
                format!("↑{:.0}", fpm),
                Color::LightGreen,
            ),
            Some(fpm) if fpm < -100.0 => (
                format!("↓{:.0}", fpm.abs()),
                Color::LightRed,
            ),
            Some(_) => ("→ Level".to_string(), Color::Gray),
            None => ("—".to_string(), Color::DarkGray),
        }
    };

    Row::new([
        Cell::from(dist).style(base.add_modifier(Modifier::BOLD)),
        Cell::from(callsign).style(
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(reg).style(Style::default().fg(Color::Gray)),
        Cell::from(ac_type).style(Style::default().fg(Color::Magenta)),
        Cell::from(altitude).style(base),
        Cell::from(speed).style(base),
        Cell::from(heading).style(Style::default().fg(Color::Yellow)),
        Cell::from(vs_str).style(Style::default().fg(vs_color)),
    ])
}

// ── Footer ───────────────────────────────────────────────────────────────────

fn render_footer(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" Quit  "),
        Span::styled(" r ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" Refresh  "),
        Span::styled(" ↑ k ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" ↓ j ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::raw(" Scroll  "),
        Span::styled(
            "  Auto-refresh every 30 min",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
