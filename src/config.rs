// Copyright 2022 the dancelist authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use eyre::{bail, Report, WrapErr};
use serde::Deserialize;
use std::{
    fs::read_to_string,
    net::SocketAddr,
    path::{Path, PathBuf},
};

/// Paths at which to look for the config file. They are searched in order, and the first one that
/// exists is used.
const CONFIG_FILENAMES: [&str; 2] = ["dancelist.toml", "/etc/dancelist.toml"];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_public_dir")]
    pub public_dir: PathBuf,
    #[serde(default = "default_events")]
    pub events: String,
    #[serde(default = "default_bind_address")]
    pub bind_address: SocketAddr,
    #[serde(default)]
    pub reload_token: String,
}

impl Config {
    pub fn from_file() -> Result<Config, Report> {
        for filename in &CONFIG_FILENAMES {
            if Path::new(filename).is_file() {
                return Config::read(filename);
            }
        }
        bail!(
            "Unable to find config file in any of {:?}",
            &CONFIG_FILENAMES
        );
    }

    fn read(filename: &str) -> Result<Config, Report> {
        let config_file =
            read_to_string(filename).wrap_err_with(|| format!("Reading {}", filename))?;
        Ok(toml::from_str(&config_file)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

fn default_public_dir() -> PathBuf {
    Path::new("public").to_path_buf()
}

fn default_events() -> String {
    "https://raw.githubusercontent.com/qwandor/dancelist-data/release/events.yaml".to_string()
}

fn default_bind_address() -> SocketAddr {
    "0.0.0.0:3002".parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parsing the example config file should not give any errors.
    #[test]
    fn example_config() {
        Config::read("dancelist.example.toml").unwrap();
    }

    /// Parsing an empty config file should not give any errors.
    #[test]
    fn empty_config() {
        toml::from_str::<Config>("").unwrap();
    }
}
