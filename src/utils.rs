use std::borrow::Cow;
use std::string::FromUtf8Error;

pub fn bytes_to_string(bytes: Cow<'_, [u8]>) -> Result<String, FromUtf8Error> {
    String::from_utf8(bytes.to_vec())
}

pub fn haversine_distance_metres(
    from_lat_long_deg: (f64, f64),
    to_lat_long_deg: (f64, f64),
) -> u64 {
    const EARTH_RADIUS_KILOMETER: f64 = 6371.0_f64;

    let (from_lat_deg, from_long_deg) = from_lat_long_deg;
    let (to_lat_deg, to_long_deg) = to_lat_long_deg;

    let from_latitude = from_lat_deg.to_radians();
    let to_latitude = to_lat_deg.to_radians();

    let delta_latitude = (from_lat_deg - to_lat_deg).to_radians();
    let delta_longitude = (from_long_deg - to_long_deg).to_radians();

    let central_angle_inner = (delta_latitude / 2.0).sin().powi(2)
        + from_latitude.cos() * to_latitude.cos() * (delta_longitude / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    let distance = EARTH_RADIUS_KILOMETER * central_angle;
    (distance * 1000.0) as u64
}
