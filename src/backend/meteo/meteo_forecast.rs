use crate::backend::meteo::meteo_current::get_meteo_weather_data;
use crate::backend::meteo::meteo_get_combined_data_formatted;
use crate::backend::meteo::meteo_json::MeteoForecastJson;
use crate::backend::status::Status;
use crate::backend::weather_data::WeatherDataRS;
use crate::backend::weather_forecast::get_location;
use crate::backend::weather_forecast::WeatherForecastRS;
use crate::local::settings::Settings;

fn get_forecast_sentence(
    data: Vec<WeatherDataRS>,
    raw_data: MeteoForecastJson,
    start: usize,
) -> String {
    let mut rain = raw_data
        .hourly
        .rain
        .iter()
        .map(|x| x != &0.0)
        .collect::<Vec<bool>>();
    let mut snow = raw_data
        .hourly
        .snowfall
        .iter()
        .map(|x| x != &0.0)
        .collect::<Vec<bool>>();
    for _i in 0..start {
        rain.remove(0);
        snow.remove(0);
    } // TODO: Convert
    if data[0]
        .conditions
        .clone()
        .into_iter()
        .map(|condition| condition.condition_id / 100 == 5)
        .collect::<Vec<bool>>()
        .contains(&true)
    {
        let mut t: u8 = 0;
        for i in rain {
            if !i {
                break;
            }
            t += 1;
        }
        return format!("It will continue raining for {} hours.", t);
    }
    if data[0]
        .conditions
        .clone()
        .into_iter()
        .map(|condition| condition.condition_id / 100 == 6)
        .collect::<Vec<bool>>()
        .contains(&true)
    {
        let mut t: u8 = 0;
        for i in snow {
            if !i {
                break;
            }
            t += 1;
        }
        return format!("It will continue snowing for {} hours.", t);
    } else {
        let rain_start = rain.clone().into_iter().position(|x| x);
        let snow_start = snow.clone().into_iter().position(|x| x);

        if rain_start.is_none() && snow_start.is_none() {
            return "Conditions are predicted to be clear for the next 7 days.".to_string();
        }
        rain.reverse();
        snow.reverse();
        let rain_end = rain.into_iter().position(|x| x);
        let snow_end = snow.into_iter().position(|x| x);
        if rain_start.is_some() {
            return format!(
                "It will rain in {} hours for {} hours",
                rain_start.unwrap(),
                rain_end.unwrap() - rain_start.unwrap()
            );
        }
        if snow_start.is_some() {
            return format!(
                "It will snow in {} hours for {} hours",
                snow_start.unwrap(),
                snow_end.unwrap() - snow_start.unwrap()
            );
        }
    }
    String::from("Conditions are predicted to be clear for the next 7 days.")
}

pub fn get_meteo_forecast(coordinates: Vec<String>, settings: Settings) -> crate::Result<WeatherForecastRS> {
    let data =
        meteo_get_combined_data_formatted(coordinates.clone(), settings.internal.metric_default)?;
    let mut forecast: Vec<WeatherDataRS> = Vec::new();
    let now = data
        .weather
        .hourly
        .time
        .iter()
        .position(|r| *r == data.weather.current_weather.time)
        .expect("now not found");
    let current = get_meteo_weather_data(
        data.weather.clone(),
        data.air_quality.clone(),
        now,
        settings.internal.metric_default,
    )?;
    forecast.push(current);
    for i in now + 1..data.weather.hourly.time.len() - 1 {
        forecast.push(get_meteo_weather_data(
            data.weather.clone(),
            data.air_quality.clone(),
            i,
            settings.internal.metric_default,
        )?);
    }
    let region_country = get_location(coordinates)?;
    let forecast_sentence = get_forecast_sentence(forecast.clone(), data.weather, now);
    let f = WeatherForecastRS {
        status: Status::OK,
        region: region_country[0].clone(),
        country: region_country[1].clone(),
        forecast: forecast.clone(),
        current_weather: forecast.into_iter().next().unwrap(),
        forecast_sentence,
        raw_data: None,
    };
    Ok(f)
}
