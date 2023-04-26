use std::{collections::HashMap, ffi::OsStr, io, rc::Rc};
use std::fmt::format;

use libloading::Library;
use log::{debug, error, trace};
use serde::de;

use crate::backend::weather_forecast::WeatherForecastRS;
use crate::custom_backend;
use crate::custom_backend::{InvocationError, PluginDeclaration, WeatherForecastPlugin};
use crate::local::settings::Settings;
use crate::util::Error;

pub fn load(paths: Vec<String>) -> ExternalBackends {
    let mut functions = ExternalBackends::new();
    unsafe {
        for path in paths {
            trace!("Loading {}", path);
            let l  = functions.load(&path);
            match l {
                Ok(()) => trace!("Loaded {} successfully", &path),
                Err(e) => error!("Failed to load external backend at {}: {}", &path, e.to_string())
            }
        }
    }
    functions
}

pub fn run(
    functions: ExternalBackends,
    name: String,
    coordinates: [&str; 2],
    settings: Settings,
) -> WeatherForecastRS {
    functions
        .call(name, coordinates, settings)
        .expect("Invocation failed")
}

/// A map of all externally provided functions.
#[derive(Default)]
pub struct ExternalBackends {
    functions: HashMap<String, BackendWrapper>,
    libraries: Vec<Rc<Library>>,
}

impl ExternalBackends {
    pub fn new() -> ExternalBackends {
        ExternalBackends::default()
    }

    pub fn call(
        &self,
        name: String,
        coordinates: [&str; 2],
        settings: Settings,
    ) -> crate::Result<WeatherForecastRS> {
        debug!("Calling function {}", name);
        self.functions
            .get(&*name)
            .ok_or_else(|| Error::InvocationError(InvocationError::NotFound))?
            .call(coordinates, settings)
    }

    /// Load a plugin library and add all contained functions to the internal
    /// function table.
    ///
    /// # Safety
    ///
    /// A plugin library **must** be implemented using the
    /// [`plugins_core::plugin_declaration!()`] macro. Trying manually implement
    /// a plugin without going through that macro will result in undefined
    /// behaviour.
    pub unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> crate::Result<()> {
        let path = library_path.as_ref().to_str().ok_or_else(|| "Failed to get library path")?;
        // load the library into memory
        let library = Rc::new(Library::new(path).map_err(|e| format!("Could not load library at {}, details: {}", path, e.to_string()))?);

        // get a pointer to the plugin_declaration symbol.
        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")
            .expect("plugin decl failed")
            .read();

        // version checks to prevent accidental ABI incompatibilities
        if decl.core_version != custom_backend::CORE_VERSION {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      format!("Plugin version mismatch, found {}, but expected {}", decl.core_version, custom_backend::CORE_VERSION)
            ))?;
        }
        if decl.rustc_version != custom_backend::RUSTC_VERSION {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      format!("Rustc version mismatch, found {}, but expected {}", decl.rustc_version, custom_backend::RUSTC_VERSION)
            ))?;
        }

        let mut registrar = PluginRegistrar::new(Rc::clone(&library));

        (decl.register)(&mut registrar);

        // add all loaded plugins to the functions map
        self.functions.extend(registrar.functions);
        // and make sure ExternalFunctions keeps a reference to the library
        self.libraries.push(library);

        Ok(())
    }
}

struct PluginRegistrar {
    functions: HashMap<String, BackendWrapper>,
    lib: Rc<Library>,
}

impl PluginRegistrar {
    fn new(lib: Rc<Library>) -> PluginRegistrar {
        PluginRegistrar {
            lib,
            functions: HashMap::default(),
        }
    }
}

impl custom_backend::PluginRegistrar for PluginRegistrar {
    fn register_function(&mut self, name: &str, backend: Box<dyn WeatherForecastPlugin>) {
        let proxy = BackendWrapper {
            backend,
            _lib: Rc::clone(&self.lib),
        };
        self.functions.insert(name.to_string(), proxy);
    }
}

/// A proxy object which wraps a [`WeatherForecastPlugin`] and makes sure it can't outlive
/// the library it came from.
pub struct BackendWrapper {
    backend: Box<dyn WeatherForecastPlugin>,
    _lib: Rc<Library>,
}

impl WeatherForecastPlugin for BackendWrapper {
    fn call(
        &self,
        coordinates: [&str; 2],
        settings: Settings,
    ) -> crate::Result<WeatherForecastRS> {
        self.backend.call(coordinates, settings)
    }

    fn name(&self) -> Option<&str> {
        self.backend.name()
    }

    fn help(&self) -> Option<&str> {
        self.backend.help()
    }
}
