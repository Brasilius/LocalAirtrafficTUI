/// Haversine great-circle distance between two lat/lon points, returned in miles.
pub fn haversine_miles(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_km = 6371.0_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r_km * c * 0.621371
}

/// Returns a (lat_min, lon_min, lat_max, lon_max) bounding box around a point,
/// large enough to contain a circle of `radius_miles`.
pub fn bounding_box(lat: f64, lon: f64, radius_miles: f64) -> (f64, f64, f64, f64) {
    let radius_km = radius_miles * 1.60934;
    let lat_delta = radius_km / 111.0;
    let lon_delta = radius_km / (111.0 * lat.to_radians().cos().abs());
    (lat - lat_delta, lon - lon_delta, lat + lat_delta, lon + lon_delta)
}
