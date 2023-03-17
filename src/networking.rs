use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use pyo3::{pyclass, pyfunction, PyResult, Python, wrap_pyfunction};
use pyo3::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reqwest::cookie::Jar;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Url;

#[pyclass]
#[derive(Clone)]
pub struct Resp {
    #[pyo3(get)]
    pub url: String,
    #[pyo3(get)]
    pub status: u16,
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub bytes: Vec<u8>,
}

/// :param url: the url to retrieve
/// :param user_agent: the user agent to use, weathercli/1 by default
/// :param headers: optional dictionary with headers in it
/// :param cookies: optional list of cookies
#[pyfunction]
pub fn get_url(
    url: String,
    user_agent: Option<String>,
    headers: Option<HashMap<String, String>>,
    cookies: Option<HashMap<String, String>>,
) -> Resp {
    let jar: Jar = Jar::default();
    if let Some(cookies) = cookies {
        let mut formatted_cookies: Vec<String> = Vec::new();
        for (key, value) in cookies {
            formatted_cookies.push(key + "=" + &value);
        }
        for cookie in formatted_cookies {
            jar.add_cookie_str(&cookie, &url.parse::<Url>().unwrap());
        }
    }
    let mut app_user_agent = "weathercli/1".to_string();
    if let Some(user_agent) = user_agent {
        app_user_agent = user_agent
    }
    let client_pre = reqwest::blocking::Client::builder().user_agent(app_user_agent);
    let mut header_map = HeaderMap::new();
    let mut heads = HashMap::new();
    if let Some(h) = headers {
        heads = h
    }
    for (k, v) in heads {
        header_map.insert(
            HeaderName::from_str(&k).expect(""),
            HeaderValue::from_str(&v).expect(""),
        );
    }
    let client = client_pre
        .default_headers(header_map)
        .cookie_store(true)
        .cookie_provider::<Jar>(Arc::new(jar))
        .build()
        .unwrap();
    let data = client.get(url).send().expect("Url Get failed");
    let url = data.url().to_string();
    let status = data.status().as_u16();
    let bytes = data.bytes().unwrap().to_vec();
    let mut text = String::from("");
    for byte in bytes.clone() {
        text += &(byte as char).to_string();
    }
    Resp {
        url,
        status,
        text,
        bytes,
    }
}

/// Async retrival of multiple urls
/// :param urls: the urls to retrieve
#[pyfunction]
pub fn get_urls(urls: Vec<String>, cookies: Option<Vec<String>>) -> Vec<Resp> {
    let jar: Jar = Jar::default();
    if let Some(cookies) = cookies {
        for cookie in cookies {
            for url in urls.clone() {
                jar.add_cookie_str(&cookie, &url.parse::<Url>().unwrap());
            }
        }
    }
    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .cookie_provider::<Jar>(Arc::new(jar))
        .build()
        .unwrap();
    let data: Vec<_> = urls
        .par_iter()
        .map(|url| {
            let data = client.get(url).send().unwrap();
            let url = data.url().to_string();
            let status = data.status().as_u16();
            let bytes = data.bytes().unwrap().to_vec();
            let mut text = String::from("");
            for byte in bytes.clone() {
                text += &(byte as char).to_string();
            }
            Resp {
                url,
                status,
                text,
                bytes,
            }
        })
        .collect();
    data
}

pub fn register_networking_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(py, "networking")?;
    child_module.add_function(wrap_pyfunction!(get_url, child_module)?)?;
    child_module.add_function(wrap_pyfunction!(get_urls, child_module)?)?;
    parent_module.add_submodule(child_module)?;
    Ok(())
}
