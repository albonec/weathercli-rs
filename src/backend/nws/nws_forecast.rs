use crate::backend::nws::nws_current::get_nws_current;
use crate::backend::nws::nws_get_combined_data_formatted;
use crate::backend::Status;
use crate::backend::weather_forecast::WeatherForecastRS;
use crate::local::settings::Settings;

pub fn get_nws_forecast(coordinates: Vec<String>, settings: Settings) -> crate::Result<WeatherForecastRS> {
    let data =
        nws_get_combined_data_formatted(coordinates, settings.internal.metric_default)?;
    let current = get_nws_current(data, settings.internal.metric_default)?;
    Ok(WeatherForecastRS {
        status: Status::OK,
        region: "WIP".to_string(),
        country: "WIP".to_string(),
        forecast: vec![current.clone()],
        current_weather: current,
        forecast_sentence: "".to_string(),
        raw_data: None,
    })
}
