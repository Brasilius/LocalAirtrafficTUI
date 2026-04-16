mod api;
mod app;
mod geo;
mod ui;

use anyhow::{Context, Result};
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};

const REFRESH_SECS: u64 = 15;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Resolve location: CLI flags beat IP geolocation.
    let location = match location_from_args(&args) {
        Some(loc) => loc,
        None => {
            eprintln!("Detecting location via IP geolocation…");
            api::get_user_location()
                .await
                .context("Could not detect location. Pass --lat <deg> --lon <deg> to override.")?
        }
    };

    let radius: f64 = radius_from_args(&args).unwrap_or(10.0);

    // Install a panic hook that restores the terminal before printing the
    // backtrace so the shell isn't left in raw/alternate-screen mode.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(info);
    }));

    // Set up the terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(location, radius);
    // First fetch.
    app.refresh().await;

    let tick = Duration::from_secs(REFRESH_SECS);
    let mut last_refresh = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        // Poll for input; wake up at the next refresh deadline at the latest.
        let wait = tick.saturating_sub(last_refresh.elapsed());
        if event::poll(wait)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,

                    // Manual refresh
                    KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::F(5) => {
                        app.refresh().await;
                        last_refresh = Instant::now();
                    }

                    // Scroll
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),

                    _ => {}
                }
            }
        }

        // Auto-refresh on schedule.
        if last_refresh.elapsed() >= tick {
            app.refresh().await;
            last_refresh = Instant::now();
        }
    }

    // Restore terminal.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

// ── CLI helpers ──────────────────────────────────────────────────────────────

fn location_from_args(args: &[String]) -> Option<api::Location> {
    let lat = flag_value(args, "--lat")?.parse::<f64>().ok()?;
    let lon = flag_value(args, "--lon")?.parse::<f64>().ok()?;
    Some(api::Location {
        lat,
        lon,
        city: format!("{lat:.4}°, {lon:.4}°"),
        country: "manual".to_string(),
    })
}

fn radius_from_args(args: &[String]) -> Option<f64> {
    flag_value(args, "--radius")?.parse::<f64>().ok()
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let pos = args.iter().position(|a| a == flag)?;
    args.get(pos + 1).map(String::as_str)
}
