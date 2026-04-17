use anyhow::{anyhow, Result};
use serde::Deserialize;

// ── Public types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub city: String,
    pub country: String,
}

#[derive(Debug, Clone)]
pub struct RawAircraft {
    pub icao24: String,
    pub callsign: Option<String>,
    pub lat: f64,
    pub lon: f64,
    /// Barometric altitude in metres (can be negative near sea level).
    pub baro_altitude_m: Option<f64>,
    /// Ground speed in m/s.
    pub velocity_ms: Option<f64>,
    /// True track in degrees clockwise from north.
    pub true_track: Option<f64>,
    pub on_ground: bool,
    /// Vertical rate in m/s (positive = climbing).
    pub vertical_rate_ms: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AircraftTypeInfo {
    pub registration: String,
    pub aircraft_type: String,
}

// ── ip-api.com ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct IpApiResponse {
    status: String,
    lat: f64,
    lon: f64,
    city: String,
    country: String,
}

pub async fn get_user_location() -> Result<Location> {
    let resp: IpApiResponse = reqwest::Client::new()
        .get("http://ip-api.com/json/")
        .send()
        .await?
        .json()
        .await?;

    if resp.status != "success" {
        return Err(anyhow!("ip-api.com geolocation failed"));
    }

    Ok(Location {
        lat: resp.lat,
        lon: resp.lon,
        city: resp.city,
        country: resp.country,
    })
}

// ── OpenSky Network ──────────────────────────────────────────────────────────
//
// State vector field indices:
//   0  icao24          string
//   1  callsign        string | null
//   2  origin_country  string
//   3  time_position   int    | null
//   4  last_contact    int
//   5  longitude       float  | null
//   6  latitude        float  | null
//   7  baro_altitude   float  | null  (metres)
//   8  on_ground       bool
//   9  velocity        float  | null  (m/s)
//  10  true_track      float  | null  (deg, clockwise from N)
//  11  vertical_rate   float  | null  (m/s)
//  12  sensors         array  | null
//  13  geo_altitude    float  | null  (metres)
//  14  squawk          string | null
//  15  spi             bool
//  16  position_source int

#[derive(Deserialize)]
struct OpenSkyResponse {
    states: Option<Vec<serde_json::Value>>,
}

pub async fn fetch_nearby_aircraft(lat: f64, lon: f64, radius_miles: f64) -> Result<Vec<RawAircraft>> {
    use crate::geo::bounding_box;

    let (lat_min, lon_min, lat_max, lon_max) = bounding_box(lat, lon, radius_miles);

    let url = format!(
        "https://opensky-network.org/api/states/all?lamin={lat_min}&lomin={lon_min}&lamax={lat_max}&lomax={lon_max}"
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()?;

    let mut request = client.get(&url);
    if let (Ok(user), Ok(pass)) = (
        std::env::var("OPENSKY_USER"),
        std::env::var("OPENSKY_PASS"),
    ) {
        request = request.basic_auth(user, Some(pass));
    }

    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(anyhow!("OpenSky returned HTTP {status}"));
    }
    let resp: OpenSkyResponse = response.json().await?;

    let mut aircraft = Vec::new();

    if let Some(states) = resp.states {
        for state in states {
            let arr = match state.as_array() {
                Some(a) => a,
                None => continue,
            };

            let icao24 = match arr.first().and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            // Skip aircraft without a known position.
            let ac_lat = match arr.get(6).and_then(|v| v.as_f64()) {
                Some(v) => v,
                None => continue,
            };
            let ac_lon = match arr.get(5).and_then(|v| v.as_f64()) {
                Some(v) => v,
                None => continue,
            };

            let callsign = arr
                .get(1)
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            aircraft.push(RawAircraft {
                icao24,
                callsign,
                lat: ac_lat,
                lon: ac_lon,
                baro_altitude_m: arr.get(7).and_then(|v| v.as_f64()),
                velocity_ms: arr.get(9).and_then(|v| v.as_f64()),
                true_track: arr.get(10).and_then(|v| v.as_f64()),
                on_ground: arr.get(8).and_then(|v| v.as_bool()).unwrap_or(false),
                vertical_rate_ms: arr.get(11).and_then(|v| v.as_f64()),
            });
        }
    }

    Ok(aircraft)
}

// ── hexdb.io aircraft type lookup ───────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(default)]
struct HexDbResponse {
    #[serde(rename = "Registration")]
    registration: String,
    #[serde(rename = "Manufacturer")]
    manufacturer: String,
    #[serde(rename = "Type")]
    aircraft_type: String,
    #[serde(rename = "ICAOTypeCode")]
    icao_type_code: String,
}

pub async fn fetch_aircraft_info(icao24: &str) -> Option<AircraftTypeInfo> {
    let url = format!("https://hexdb.io/api/v1/aircraft/{icao24}");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()
        .ok()?;

    let resp = client.get(&url).send().await.ok()?;

    // 404 = unknown aircraft
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return None;
    }

    let data: HexDbResponse = resp.json().await.ok()?;

    // Prefer the full type description, fall back to ICAO type code.
    let aircraft_type = if !data.aircraft_type.is_empty() {
        data.aircraft_type
    } else {
        data.icao_type_code
    };

    if data.registration.is_empty() && aircraft_type.is_empty() {
        return None;
    }

    Some(AircraftTypeInfo {
        registration: data.registration,
        aircraft_type,
    })
}
