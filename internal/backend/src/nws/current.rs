use crate::nws::json::NWSJSON;
use crate::WeatherCondition;
use crate::WindData;
use crate::{get_conditions_sentence, WeatherData};
use local::now;
use local::weather_file::WeatherFile;
use std::collections::HashMap;

fn convert_temp(value: f64, metric: bool) -> f64 {
    if metric {
        value
    } else {
        value * 9.0 / 5.0 + 32.0
    }
}

fn convert_speed(value: f64, metric: bool) -> f64 {
    if metric {
        value
    } else {
        value * 0.62
    }
}

fn get_conditions(
    data: NWSJSON,
    metric: bool,
    index: usize,
    cloud_cover: u8,
) -> crate::Result<Vec<WeatherCondition>> {
    let weather_file = WeatherFile::weather_codes()?;
    let weather_codes: HashMap<String, Vec<String>> = bincode::deserialize(&weather_file.data)?;
    let mut conditions: Vec<WeatherCondition> = Vec::new();
    if cloud_cover == 0 {
        conditions.push(WeatherCondition::new(800, &weather_codes)?);
    } else if cloud_cover < 25 {
        conditions.push(WeatherCondition::new(801, &weather_codes)?);
    } else if cloud_cover < 50 {
        conditions.push(WeatherCondition::new(802, &weather_codes)?);
    } else if cloud_cover < 85 {
        conditions.push(WeatherCondition::new(803, &weather_codes)?);
    } else {
        conditions.push(WeatherCondition::new(804, &weather_codes)?);
    }
    if data.properties.quantitative_precipitation.values[index].value != 0.0 {
        let rain = data.properties.quantitative_precipitation.values[index].value;
        let metric = metric;
        if (0.0 < rain && rain < 0.098 && !metric) || (0.0 < rain && rain < 2.5 && metric) {
            conditions.push(WeatherCondition::new(500, &weather_codes)?);
        } else if (rain < 0.39 && !metric) || (rain < 10.0 && metric) {
            conditions.push(WeatherCondition::new(501, &weather_codes)?);
        } else if (rain < 2.0 && !metric) || (rain < 50.0 && metric) {
            conditions.push(WeatherCondition::new(502, &weather_codes)?);
        } else if rain != 0.0 {
            conditions.push(WeatherCondition::new(503, &weather_codes)?);
        }
    }
    if data.properties.snowfall_amount.values[index].value != 0.0 {
        conditions.push(WeatherCondition::new(601, &weather_codes)?);
    }
    Ok(conditions)
}

pub fn get_current(data: NWSJSON, metric: bool) -> crate::Result<WeatherData> {
    let cloud_cover = data.properties.sky_cover.values[0].value as u8;
    let conditions = get_conditions(data.clone(), metric, 0, cloud_cover)?;
    let d = WeatherData {
        time: now() as i128,
        temperature: convert_temp(data.properties.temperature.values[0].value, metric) as f32,
        min_temp: convert_temp(data.properties.min_temperature.values[0].value, metric) as f32,
        max_temp: convert_temp(data.properties.max_temperature.values[0].value, metric) as f32,
        wind: WindData {
            speed: convert_speed(data.properties.wind_speed.values[0].value, metric),
            heading: data.properties.wind_direction.values[0].value as u16,
        },
        raw_data: String::new(),
        dewpoint: convert_temp(data.properties.dewpoint.values[0].value, metric) as f32,
        feels_like: convert_temp(data.properties.apparent_temperature.values[0].value, metric)
            as f32,
        aqi: 0,
        cloud_cover,
        conditions: vec![],
        condition_sentence: get_conditions_sentence(conditions),
    };
    Ok(d)
}
