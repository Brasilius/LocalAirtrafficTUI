use crate::api::{AircraftTypeInfo, Location, fetch_aircraft_info, fetch_nearby_aircraft};
use crate::geo::haversine_miles;
use chrono::{DateTime, Local};
use std::collections::HashMap;

// ── Display model ────────────────────────────────────────────────────────────

pub struct AircraftDisplay {
    pub icao24: String,
    pub callsign: String,
    pub registration: Option<String>,
    /// Full type string (e.g. "Boeing 737-800" or ICAO code "B738").
    pub aircraft_type: Option<String>,
    pub distance_miles: f64,
    /// Barometric altitude in feet.
    pub altitude_ft: Option<f64>,
    /// Ground speed in knots.
    pub speed_knots: Option<f64>,
    /// True track in degrees clockwise from north.
    pub heading_deg: Option<f64>,
    pub on_ground: bool,
    /// Vertical speed in feet per minute.
    pub vertical_rate_fpm: Option<f64>,
}

// ── Application state ────────────────────────────────────────────────────────

pub struct App {
    pub location: Location,
    pub radius_miles: f64,
    pub aircraft: Vec<AircraftDisplay>,
    pub last_update: Option<DateTime<Local>>,
    /// Short status / error message shown in the header.
    pub status: String,
    pub is_loading: bool,
    /// Scroll offset for the aircraft table.
    pub scroll: usize,
    /// Cache: icao24 → type info (None = confirmed unknown).
    aircraft_info_cache: HashMap<String, Option<AircraftTypeInfo>>,
}

impl App {
    pub fn new(location: Location, radius_miles: f64) -> Self {
        Self {
            location,
            radius_miles,
            aircraft: Vec::new(),
            last_update: None,
            status: "Fetching data…".to_string(),
            is_loading: true,
            scroll: 0,
            aircraft_info_cache: HashMap::new(),
        }
    }

    pub async fn refresh(&mut self) {
        self.is_loading = true;
        self.status = "Refreshing…".to_string();

        match fetch_nearby_aircraft(self.location.lat, self.location.lon, self.radius_miles).await {
            Ok(raw_list) => {
                // Identify ICAO24 codes not yet resolved.
                let new_icao24s: Vec<String> = raw_list
                    .iter()
                    .filter(|a| !self.aircraft_info_cache.contains_key(&a.icao24))
                    .map(|a| a.icao24.clone())
                    .collect();

                // Fetch type info concurrently via tokio tasks.
                if !new_icao24s.is_empty() {
                    let handles: Vec<_> = new_icao24s
                        .into_iter()
                        .map(|icao| {
                            tokio::spawn(async move {
                                let info = fetch_aircraft_info(&icao).await;
                                (icao, info)
                            })
                        })
                        .collect();

                    for handle in handles {
                        if let Ok((icao, info)) = handle.await {
                            self.aircraft_info_cache.insert(icao, info);
                        }
                    }
                }

                let user_lat = self.location.lat;
                let user_lon = self.location.lon;
                let radius = self.radius_miles;

                let mut display: Vec<AircraftDisplay> = raw_list
                    .into_iter()
                    .filter_map(|raw| {
                        let dist = haversine_miles(user_lat, user_lon, raw.lat, raw.lon);
                        // The bounding box is a square; trim to a proper circle.
                        if dist > radius {
                            return None;
                        }

                        let info = self
                            .aircraft_info_cache
                            .get(&raw.icao24)
                            .and_then(|o| o.as_ref());

                        Some(AircraftDisplay {
                            callsign: raw
                                .callsign
                                .unwrap_or_else(|| raw.icao24.to_uppercase()),
                            icao24: raw.icao24,
                            registration: info
                                .map(|i| i.registration.clone())
                                .filter(|s: &String| !s.is_empty()),
                            aircraft_type: info
                                .map(|i| i.aircraft_type.clone())
                                .filter(|s| !s.is_empty()),
                            distance_miles: dist,
                            altitude_ft: raw.baro_altitude_m.map(|m| m * 3.280_84),
                            speed_knots: raw.velocity_ms.map(|ms| ms * 1.943_84),
                            heading_deg: raw.true_track,
                            on_ground: raw.on_ground,
                            // m/s → ft/min: × 196.85
                            vertical_rate_fpm: raw.vertical_rate_ms.map(|ms| ms * 196.85),
                        })
                    })
                    .collect();

                display.sort_by(|a, b| {
                    a.distance_miles
                        .partial_cmp(&b.distance_miles)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let count = display.len();
                self.aircraft = display;
                self.last_update = Some(Local::now());
                self.status = if count == 0 {
                    "No aircraft detected".to_string()
                } else {
                    format!("{count} aircraft in range")
                };
                // Keep scroll in bounds after refresh.
                if self.scroll >= count.max(1) {
                    self.scroll = count.saturating_sub(1);
                }
            }
            Err(e) => {
                self.status = format!("Error: {e}");
            }
        }

        self.is_loading = false;
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll + 1 < self.aircraft.len() {
            self.scroll += 1;
        }
    }
}
