use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
pub struct WeatherCondition {
    pub condition_id: u16,
    pub image_url: String,
    pub sentence: String,
    pub image_ascii: String,
}

impl WeatherCondition {
    pub fn new(condition_id: u16, weather_codes: &str) -> crate::Result<Self> {
        let mut data: Value = serde_json::from_str(weather_codes)?;
        let code = data[condition_id.to_string()].as_array_mut().unwrap();
        let sentence = code[3]
            .clone()
            .as_str()
            .ok_or("Sentence expected")?
            .to_string();
        let image_url = String::from("https://openweathermap.org/img/wn/")
            + code[2].clone().as_str().ok_or("image_url expected")?
            + "@4x.png";
        let image_ascii = code[4].clone().as_str().ok_or("image_ascii expected")?.to_string();
        Ok(WeatherCondition {
            condition_id,
            image_url,
            sentence,
            image_ascii,
        })
    }
}
