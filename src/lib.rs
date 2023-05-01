use std::path::Path;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use log::{LevelFilter, warn};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::Handle;

use crate::backend::meteo::meteo_forecast::get_meteo_forecast;
use crate::backend::nws::nws_forecast::get_nws_forecast;
use crate::backend::openweathermap::openweathermap_forecast::get_openweathermap_forecast;
use crate::backend::weather_forecast::WeatherForecast;
use crate::cli::Datasource;
use crate::dynamic_loader::ExternalBackends;
use crate::local::dirs::weathercli_dir;
use crate::local::settings::Settings;
#[cfg(feature = "gui")]
use crate::local::settings_app;
use crate::location::Coordinates;
use crate::util::Config;

pub mod backend;
#[cfg(feature = "support")]
pub mod bin_common;
pub mod cli;
pub mod color;
pub mod dynamic_loader;
pub mod error;
pub mod layout;
pub mod local;
pub mod location;
pub mod networking;
pub mod prompt;
pub mod util;
pub mod updater;

pub type Result<T> = std::result::Result<T, error::Error>;

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

#[cfg(feature = "gui")]
pub fn open_settings_app() {
    settings_app::run_settings_app().expect("App Failed");
}

#[cfg(not(feature = "gui"))]
pub fn open_settings_app() {
    panic!("GUI support not enabled!");
}

#[cfg(target_os = "windows")]
pub const CONFIG: Config<'static> = Config {
    weather_file_name: "weather.exe",
    weather_d_file_name: "weatherd.exe",
    updater_file_name: "updater.exe",
};

#[cfg(not(target_os = "windows"))]
pub const CONFIG: Config<'static> = Config {
    weather_file_name: "weather",
    weather_d_file_name: "weatherd",
    updater_file_name: "updater",
};

pub fn get_data_from_datasource(
    datasource: Datasource,
    coordinates: Coordinates,
    settings: Settings,
    custom_backends: ExternalBackends,
) -> Result<WeatherForecast> {
    let dir = local::dirs::weathercli_dir()?.join("resources");
    let f1 = dir.join("weather_codes.res");
    let f2 = dir.join("weather_ascii_images.res");
    if !(Path::exists(&dir) && Path::exists(&f1) && Path::exists(&f2)) {
        warn!("Forcing downloading of web resources");
        updater::resource_updater::update_web_resources(None)?;
    } else if settings.internal.auto_update_internet_resources {
        thread::spawn(move || {
            updater::resource_updater::update_web_resources(None).unwrap_or(());
        });
    }
    let conv_coords = [&*coordinates.latitude.to_string(), &*coordinates.longitude.to_string()];
    match datasource {
        Datasource::Openweathermap => get_openweathermap_forecast(coordinates, settings),
        Datasource::NWS => get_nws_forecast(coordinates, settings),
        Datasource::Meteo => get_meteo_forecast(coordinates, settings),
        Datasource::Other(s) => custom_backends.call(&s, conv_coords, settings),
    }
}

pub fn init_logging() -> crate::Result<Handle> {
    let level = LevelFilter::Info;
    let mut file_path = weathercli_dir()?.join("logs");
    file_path.push(format!("{}.log", now()));
    // Build a stderr logger.
    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new("{m}\n")))
        .build();
    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("[{l} {M} {d}] {m}\n")))
        .build(file_path.as_os_str().to_str().unwrap())
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    Ok(log4rs::init_config(config).unwrap())
}

