use crate::backend::openweathermap::openweathermap_json::OpenWeatherMapForecastItemJson;
use crate::backend::weather_condition::WeatherCondition;
use crate::backend::weather_data::{get_conditions_sentence, WeatherDataRS};
use crate::backend::wind_data::WindData;
use crate::now;

pub struct OpenWeatherMapFuture {
    data: OpenWeatherMapForecastItemJson,
}
impl OpenWeatherMapFuture {
    pub fn new(data: OpenWeatherMapForecastItemJson) -> Self {
        OpenWeatherMapFuture { data }
    }
}
impl WeatherDataRS for OpenWeatherMapFuture {
    fn get_time(&self) -> i128 {
        now() as i128 // TODO: Fix
    }

    fn get_temperature(&self) -> f32 {
        self.data.main.temp as f32
    }

    fn get_min_temp(&self) -> f32 {
        self.data.main.temp_min as f32
    }

    fn get_max_temp(&self) -> f32 {
        self.data.main.temp_max as f32
    }

    fn get_wind(&self) -> WindData {
        WindData {
            speed: self.data.wind.speed,
            heading: self.data.wind.deg,
        }
    }

    fn get_raw_data(&self) -> String {
        serde_json::to_string_pretty(&self.data).expect("dump to string failed")
    }

    fn get_dewpoint(&self) -> f32 {
        self.data.main.humidity as f32
    }

    fn get_feels_like(&self) -> f32 {
        self.data.main.feels_like as f32
    }

    fn get_aqi(&self) -> u8 {
        0
    }

    fn get_cloud_cover(&self) -> u8 {
        self.data.clouds.all
    }

    fn get_conditions(&self) -> Vec<WeatherCondition> {
        let mut conditions: Vec<WeatherCondition> = Vec::new();
        for condition in self.data.weather.clone() {
            conditions.push(WeatherCondition::new(condition.id as u16))
        }
        conditions
    }

    fn get_condition_sentence(&self) -> String {
        get_conditions_sentence(self.get_conditions())
    }
}
