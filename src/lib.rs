use std::path::Path;

use pyo3::prelude::*;
use sha256::try_digest;

mod backend;
mod local;
pub mod cache;
mod location;
pub mod networking;
mod openweathermap_json;
mod updater;
pub mod weather_file;
mod status;


/// returns the sha-256 of the file
#[pyfunction]
fn hash_file(filename: String) -> String {
    let input = Path::new(&filename);
    try_digest(input).unwrap()
}

/// corelib module for weather cli, implemented in Rust.
#[pymodule]
fn core(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(location::get_location, module)?)?;
    module.add_function(wrap_pyfunction!(hash_file, module)?)?;
    module.add_class::<weather_file::WeatherFile>()?;
    module.add_class::<status::Status>()?;
    backend::register_backend_module(py, module)?;
    cache::register_caching_module(py, module)?;
    networking::register_networking_module(py, module)?;
    updater::register_updater_module(py, module)?;
    py.run("\
    import sys\
    ;sys.modules['core.backend'] = backend\
    ;sys.modules['core.caching'] = caching\
    ;sys.modules['core.networking'] = networking\
    ;sys.modules['core.updater'] = updater", None, Some(module.dict()))?;
    Ok(())
}
