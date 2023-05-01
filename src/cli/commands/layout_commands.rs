use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use crate::color;
use crate::local::dirs::layouts_dir;
use crate::local::settings::Settings;
use crate::util::list_dir;

pub fn install(path: String) -> crate::Result<()> { // TODO: Add validity checks
    let real_path = PathBuf::from_str(&*path).unwrap();
    let file_name = real_path.file_name().ok_or_else(|| "Not a file")?.to_str().unwrap();
    if file_name == "default.json" || file_name == "default.res" {
        return Err("File name cannot be default.json or default.res")?;
    }
    fs::copy(&real_path, layouts_dir()?.join(file_name))?;
    Ok(())
}

pub fn list() -> crate::Result<()> {
    let paths = fs::read_dir(layouts_dir()?)?;
    let settings = Settings::new()?; // TODO: Optimize excess read
    let current_layout = settings.internal.layout_file;
    for path in paths {
        let tmp = path?.file_name();
        let file_name = tmp.to_str().unwrap();
        if file_name == current_layout {
            println!("{}*{} {file_name}{}", color::FORE_MAGENTA, color::FORE_GREEN, color::FORE_RESET)
        }
        else {
            println!("{}  {file_name}", color::FORE_BLUE)
        }
    }
    Ok(())
}

pub fn select() -> crate::Result<()> {
    let paths = list_dir(layouts_dir()?)?;
    let mut settings = Settings::new()?; // TODO: Optimize excess read
    let current = &*settings.internal.layout_file;
    let current_index = paths.iter().position(|c| c == current).unwrap_or(0); // TODO: make it default.res
    let choice = crate::prompt::choice(&*paths, current_index, None)?;
    settings.internal.layout_file = paths[choice].to_string();
    settings.write()?;
    Ok(())
}

pub fn delete() -> crate::Result<()> {
    let paths = list_dir(layouts_dir()?)?;
    let settings = Settings::new()?; // TODO: Optimize excess read
    let current = &*settings.internal.layout_file;
    let current_index = paths.iter().position(|c| c == current).unwrap_or(0); // TODO: make it default.res
    let choice = paths[crate::prompt::choice(&*paths, current_index, None)?].to_string();
    fs::remove_file(layouts_dir()?.join(&*choice))?;
    if choice == current {
        println!("Please select a new default layout");
        select()?;
    }
    Ok(())
}
