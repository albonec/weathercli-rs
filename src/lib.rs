use std::{fs, thread};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::ValueEnum;
use sha2::Digest;

use crate::backend::meteo::meteo_forecast::get_meteo_forecast;
use crate::backend::nws::nws_forecast::get_nws_forecast;
use crate::backend::openweathermap::openweathermap_forecast::get_openweathermap_forecast;
use crate::backend::status::Status;
use crate::backend::weather_forecast::WeatherForecastRS;
use crate::layout::LayoutFile;
use crate::local::settings::Settings;

#[cfg(feature = "support")]
pub mod autolaunch;
pub mod backend;
#[cfg(feature = "support")]
pub mod bin_common;
pub mod color;
pub mod component_updater;
mod layout;
pub mod local;
pub mod location;
pub mod networking;
pub mod prompt;
#[cfg(feature = "gui")]
mod settings_app;

pub fn now() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect(
        "Time went backwards :( or there is an overflow error of some sort and stuff broke",
    );
    since_the_epoch.as_millis()
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// returns the sha-256 of the file
pub fn hash_file(filename: &str) -> String {
    let input = Path::new(filename);
    let bytes = fs::read(input).expect("File read failed");
    hex::encode(sha2::Sha256::digest(bytes))
}

#[cfg(feature = "gui")]
pub fn open_settings_app() {
    settings_app::run_settings_app().unwrap();
}

#[cfg(not(feature = "gui"))]
pub fn open_settings_app() {
    println!("GUI support not enabled!");
}

pub struct Config {
    pub weather_file_name: String,
    pub weather_dfile_name: String,
    pub updater_file_name: String,
}

#[cfg(target_os = "windows")]
impl Config {
    pub fn new() -> Self {
        Config {
            weather_file_name: "weather.exe".to_string(),
            weather_dfile_name: "weatherd.exe".to_string(),
            updater_file_name: "updater.exe".to_string(),
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl Config {
    pub fn new() -> Self {
        Config {
            weather_file_name: "weather".to_string(),
            weather_dfile_name: "weatherd".to_string(),
            updater_file_name: "updater".to_string(),
        }
    }
}

fn get_data_from_datasource(
    datasource: Datasource,
    coordinates: [String; 2],
    settings: Settings,
) -> WeatherForecastRS {
    let mut dir = crate::local::dirs::home_dir().expect("Home dir get failed");
    dir.push(".weathercli/resources");
    let mut f1 = dir.clone();
    f1.push("weather_codes.json");
    let mut f2 = dir.clone();
    f2.push("weather_ascii_images.json");
    if !(Path::exists(&dir) && Path::exists(&f1) && Path::exists(&f2)) {
        println!("Forcing downloading of web resources");
        component_updater::update_web_resources(settings.internal.development.unwrap(), None)
    } else if settings.internal.auto_update_internet_resources.unwrap() {
        thread::spawn(move || {
            component_updater::update_web_resources(settings.internal.development.unwrap(), None);
        });
    }

    match datasource {
        Datasource::Openweathermap => get_openweathermap_forecast(Vec::from(coordinates), settings),
        Datasource::NWS => get_nws_forecast(Vec::from(coordinates), settings),
        Datasource::Meteo => get_meteo_forecast(Vec::from(coordinates), settings),
    }
}

fn print_out(layout_file: Option<String>, data: WeatherForecastRS, json: bool, metric: bool) {
    if json {
        println!("{:#?}", data.raw_data.expect("No raw data to print"))
    } else if data.status == Status::OK {
        let out = LayoutFile::new(layout_file);
        println!("{}", out.to_string(data, metric));
    } else {
        println!(
            "{}Something went wrong when requesting data!{}",
            color::FORE_RED,
            color::FORE_RESET
        )
    }
}

#[derive(Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum Datasource {
    Meteo,
    Openweathermap,
    NWS
}

pub fn datasource_from_str(s: &str) -> Datasource {
    match &*s.to_lowercase() {
        "nws" => Datasource::NWS,
        "openweathermap" => Datasource::Openweathermap,
        _ => Datasource::Meteo
    }
}

pub fn weather(
    datasource: Datasource,
    coordinates: [String; 2],
    settings: Settings,
    true_metric: bool,
    json: bool,
) {
    let mut s = settings.clone();
    s.internal.metric_default = Some(true_metric);
    let data = get_data_from_datasource(datasource, coordinates, s);
    print_out(settings.internal.layout_file, data, json, true_metric);
}
