use std::collections::HashMap;
use crate::backend::openweathermap::openweathermap_json::OpenWeatherMapForecastItemJson;
use crate::backend::weather_condition::WeatherCondition;
use crate::backend::weather_data::{get_conditions_sentence, WeatherData};
use crate::backend::WindData;
use crate::local::weather_file::WeatherFile;
use crate::now;

pub fn get_openweathermap_future(
    data: OpenWeatherMapForecastItemJson,
    weather_codes: HashMap<String, Vec<String>>
) -> crate::Result<WeatherData> {
    let mut conditions: Vec<WeatherCondition> = Vec::new();
    for condition in data.weather.clone() {
        conditions.push(WeatherCondition::new(
            condition.id as u16,
            &weather_codes,
        )?);
    }
    Ok(WeatherData {
        time: now() as i128,
        temperature: data.main.temp as f32,
        min_temp: data.main.temp_min as f32,
        max_temp: data.main.temp_max as f32,
        wind: WindData {
            speed: data.wind.speed,
            heading: data.wind.deg,
        },
        raw_data: serde_json::to_string_pretty(&data).expect("dump to string failed"),
        dewpoint: data.main.humidity as f32,
        feels_like: data.main.feels_like as f32,
        aqi: data.clouds.all,
        cloud_cover: 0,
        conditions: conditions.clone(),
        condition_sentence: get_conditions_sentence(conditions.clone()),
    })
}
