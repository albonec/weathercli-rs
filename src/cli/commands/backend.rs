use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use crate::color;
use crate::dynamic_loader::is_valid_ext;
use crate::local::dirs::custom_backends_dir;
use crate::local::settings::Settings;
use crate::util::list_dir;

pub fn install(path: String) -> crate::Result<()> { // TODO: Add validity checks (prompt user for permission first)
    let real_path = PathBuf::from_str(&path).unwrap();
    let file_name = real_path.file_name().ok_or("Not a file")?.to_str().unwrap();
    if !is_valid_ext(file_name) {
        return Err("Not a valid system extension, aborting")?
    }
    fs::copy(&real_path, custom_backends_dir()?.join(file_name))?;
    Ok(())
}

pub fn list(settings: Settings) -> crate::Result<()> {
    let paths = list_dir(custom_backends_dir()?)?;
    for path in paths { // TODO: Check which ones are valid (hard to do)
        let file_name = &*path;
        if is_valid_ext(file_name) {
            let valid = settings.internal.enable_custom_backends;
            if valid {
                println!("{}{file_name}", color::FORE_GREEN)
            } else {
                println!("{}{file_name}", color::FORE_RED)
            }
        }
    }
    Ok(())
}

pub fn select(settings: Settings) -> crate::Result<()> {
    let selected = settings.internal.default_backend;
    let mut settings = Settings::new()?;
    let choices = ["openweathermap", "meteo", "nws"];
    let selected_usize = choices.iter().position(|&i| i == selected).unwrap_or(0);
    let choice = crate::prompt::choice(&choices, selected_usize, None)?;
    settings.internal.default_backend = choices[choice].to_string();
    settings.write()?;
    Ok(())
}

pub fn delete() -> crate::Result<()> {
    let paths = list_dir(custom_backends_dir()?)?;
    let choice = paths[crate::prompt::choice(&paths, 0, None)?].to_string();
    fs::remove_file(custom_backends_dir()?.join(&*choice))?;
    Ok(())
}
