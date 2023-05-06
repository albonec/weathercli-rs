use crate::backend::nws::json::{NWSPointJSON, NWSJSON};
use crate::location::Coordinates;
use crate::networking;

mod current;
pub mod forecast;
mod json;

pub fn nws_get_api_url(location: Coordinates, _metric: bool) -> crate::Result<String> {
    let get_point = networking::get_url(
        format!(
            "https://api.weather.gov/points/{},{}",
            location.latitude, location.longitude
        ),
        None,
        None,
        None,
    )?
    .text;
    let point_json: NWSPointJSON =
        serde_json::from_str(&get_point)?;
    Ok(point_json.properties.forecast_grid_data)
}

pub fn nws_get_combined_data_formatted(
    location: Coordinates,
    metric: bool,
) -> crate::Result<NWSJSON> {
    let raw_data = networking::get_url(nws_get_api_url(location, metric)?, None, None, None)?;
    let data: NWSJSON = serde_json::from_str(&raw_data.text)?;
    Ok(data)
}
