use core::panic;
use std::{collections::HashSet, error::Error, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::splashes::Splashes;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub path: PathBuf,
    pub data_file: PathBuf,
    pub patch_file: PathBuf,
    pub app_state_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub tile_imgs: HashSet<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tile_imgs: HashSet::new(),
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| panic!("Couldn't get home dir"));
        let file_path = [home.to_str().unwrap(), "rusty-splash"]
            .iter()
            .collect::<PathBuf>();
        fs::create_dir_all(&file_path).unwrap();
        Self {
            path: file_path.clone(),
            data_file: {
                let mut path = file_path.clone();
                path.push("cache.json");
                path
            },
            patch_file: {
                let mut path = file_path.clone();
                path.push("patch.json");
                path
            },
            app_state_file: {
                let mut path = file_path.clone();
                path.push("app.json");
                path
            },
        }
    }
}

impl Cache {
    pub fn save_patch_version(&self, version: &str) {
        let version = serde_json::to_string(version)
            .unwrap_or_else(|error| panic!("Couldn't serialize data: {error}"));
        fs::write(self.patch_file.clone(), version).unwrap_or_else(|error| {
            panic!("Couldn't write version to {:?}: {error}", self.patch_file)
        });
    }

    pub fn save_data(&self, data: &Splashes) {
        let data = serde_json::to_string(&data)
            .unwrap_or_else(|error| panic!("Couldn't serialize data: {error}"));
        fs::write(self.data_file.clone(), data).unwrap_or_else(|error| {
            panic!("Couldn't write cache to {:?}: {error}", self.data_file)
        });
    }

    pub fn get_patch_version(&mut self) -> Result<String, Box<dyn Error>> {
        let cached_version = fs::read_to_string(&self.patch_file)?;
        let mut version: String = cached_version.parse()?;
        version = version.replace('\"', "");
        Ok(version)
    }

    pub fn get_data(&self) -> Result<Splashes, Box<dyn Error>> {
        let cache: String = fs::read_to_string(&self.data_file)?.parse()?;
        let data: Splashes = serde_json::from_str(&cache.to_owned())?;
        Ok(data)
    }

    pub fn save_app_state(&self, app: &App) {
        let app = serde_json::to_string(app)
            .unwrap_or_else(|error| panic!("Couldn't serialize app_state: {error}"));
        fs::write(self.app_state_file.clone(), app).unwrap_or_else(|error| {
            panic!(
                "Couldn't write app_state to {:?}: {error}",
                self.app_state_file
            )
        });
    }

    pub fn get_app_state(&self) -> Result<App, Box<dyn Error>> {
        let app_state: String = fs::read_to_string(&self.app_state_file)?.parse()?;
        let app_state: App = serde_json::from_str(&app_state.to_owned())?;
        Ok(app_state)
    }
}
