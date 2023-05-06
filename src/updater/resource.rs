use log::{debug, trace};
use serde_json::Value;

use crate::local::weather_file::WeatherFile;
use crate::util::hash_file;
use crate::{color, networking};

/// Updates the web resource at `$weathercli_dir/$local_path` if the hash of the local file does not match with
/// the hash at index.json of the index name, if the hashes do not match it download a copy and replaces the existing file
/// :param dev: if true the hashes will be printed if they do not match
fn update_web_resource(
    local_path: String,
    web_resp: Value,
    web_path: &str,
    name: &str,
    out_name: &str,
    quiet: bool,
) -> crate::Result<()> {
    trace!("Checking for update for {name} ");
    let mut f = WeatherFile::new(&local_path)?;
    let file_hash = hash_file(&f.path.display().to_string())?;
    let web_json: Value = web_resp;
    let web_hash: String = web_json[name]
        .as_str()
        .ok_or("Failed to get hash from web")?
        .to_string();
    if web_hash != file_hash {
        debug!("updating {name} web: {web_hash} file: {file_hash}");
        if !quiet {
            if f.exists {
                println!("{}Downloading update for{out_name}", color::FORE_YELLOW);
            } else {
                println!("{}Downloading {out_name}", color::FORE_YELLOW);
            }
        }
        let data = networking::get_url(web_path, None, None, None)?.text;
        f.data = Vec::from(data);
        f.write()?;
    }
    Ok(())
}

/// Updates all the web resources, run on a separate thread as there is no return value
/// :param dev: gets passed `update_web_resource`, if true `update_web_resource` will print the hashes if they don't match
pub fn update_web_resources(server: String, quiet: Option<bool>) -> crate::Result<()> {
    debug!("updating web resources");
    let real_quiet = quiet.unwrap_or(false);
    let resp = networking::get_url(format!("{server}index.json"), None, None, None)?;
    unsafe {
        if resp.status == 200 {
            let mut web_text = resp.text;
            let web_json: Value = simd_json::serde::from_str(&mut web_text)?; // Real unsafe here
            update_web_resource(
                String::from("resources/weather_codes.res"),
                web_json.clone(),
                &(server.clone() + "weather_codes.res"),
                "weather-codes-hash",
                "weather codes",
                real_quiet,
            )?;
            update_web_resource(
                "resources/weather_ascii_images.res".to_string(),
                web_json.clone(),
                &(server.clone() + "weather_ascii_images.res"),
                "weather-ascii-images-hash",
                "ascii images",
                real_quiet,
            )?;
            update_web_resource(
                "layouts/default.res".to_string(),
                web_json,
                &(server + "default_layout.res"),
                "default-layout-hash",
                "default layout",
                real_quiet,
            )?;
            return Ok(());
        }
    }
    Err(crate::error::Error::NetworkError(
        "Status not 200".to_string(),
    ))
}
