use std::collections::HashMap;

use scraper::Html;

use weather_plugin::{export_plugin, networking};
use weather_plugin::{WeatherCondition, WeatherData, WeatherForecast, WindData};
use weather_plugin::custom_backend::PluginRegistrar;
use weather_plugin::custom_backend::WeatherForecastPlugin;
use weather_plugin::location;
use weather_plugin::location::Coordinates;
use weather_plugin::now;
use weather_plugin::settings::Settings;

fn get_the_weather_channel_current(weather_soup: Html, forecast_soup: Html, air_quality_soup: Html) -> WeatherData {
    WeatherData {
        time: now() as i128,
        temperature: 0.0,
        min_temp: 0.0,
        max_temp: 0.0,
        wind: WindData {
            speed: 0.0,
            heading: 0,
        },
        raw_data: String::new(),
        dewpoint: 0.0,
        feels_like: 0.0,
        aqi: 0,
        cloud_cover: 0,
        conditions: vec![],
        condition_sentence: "WIP".to_string(),
    }
}

fn get_the_weather_channel_forecast(coordinates: [&str; 2], settings: Settings) -> weather_plugin::Result<WeatherForecast> {
    let region_country = weather_plugin::location::reverse_geocode(Coordinates {
        latitude: coordinates[0].parse().unwrap(),
        longitude: coordinates[1].parse().unwrap()
    })?;
    let mut cookies = HashMap::new();
    if !settings.metric_default {
        cookies.insert("unitOfMeasurement".to_string(), "e".to_string());
    } else {
        cookies.insert("unitOfMeasurement".to_string(), "m".to_string());
    }
    let r1 = networking::get_url(format!("https://weather.com/weather/today/l/{},{}", &coordinates[0], &coordinates[1]),
                                 None, None, Some(cookies.clone()))?;
    let r2 = networking::get_url(format!("https://weather.com/weather/hourbyhour/l/{},{}", &coordinates[0], &coordinates[1]),
                                 None, None, Some(cookies.clone()))?;
    let r3 = networking::get_url(format!("https://weather.com/weather/air-quality/l/{},{}", &coordinates[0], &coordinates[1]) + &coordinates[0] + "," + &coordinates[1],
                                 None, None, Some(cookies.clone()))?;
    let weather_soup = Html::parse_document(&r1.text);
    let forecast_soup = Html::parse_document(&r2.text);
    let air_quality_soup = Html::parse_document(&r3.text);
    let current = get_the_weather_channel_current(weather_soup, forecast_soup, air_quality_soup);
    let forecast = vec![current.clone()];
    let region = &region_country.clone()[0];
    let country = &region_country.clone()[1];
    Ok(WeatherForecast {
        region: region.to_string(),
        country: country.to_string(),
        forecast,
        current_weather: current,
        forecast_sentence: "WIP".to_string(),
        raw_data: None,
    })
}

export_plugin!(register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function("theweatherchannel", Box::new(TheWeatherChannel));
}

#[derive(Debug, Clone, PartialEq)]
pub struct TheWeatherChannel;

impl WeatherForecastPlugin for TheWeatherChannel {
    fn call(&self, coordinates: [&str; 2], settings: Settings) -> weather_plugin::Result<WeatherForecast> {
        get_the_weather_channel_forecast(coordinates, settings)
    }

    fn name(&self) -> Option<&str> {
        Some("theweatherchannel")
    }

    fn help(&self) -> Option<&str> {
        Some("A weather channel scraper (weather.com)")
    }
}