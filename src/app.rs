use core::panic;
use std::{collections::HashSet, fs, path::PathBuf};

use dirs::home_dir;
use serde::{Deserialize, Serialize};

use crate::cache::Cached;

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub download_path: PathBuf,
    pub downloaded: HashSet<u64>,
    pub tile_path: PathBuf,
    pub tiles: Vec<TileInstance>,
    pub monitors: Vec<(u32, u32)>,
}

impl Default for App {
    fn default() -> Self {
        let mut res = Self {
            download_path: PathBuf::default(),
            downloaded: HashSet::default(),
            tiles: vec![],
            tile_path: PathBuf::default(),
            monitors: Vec::default(),
        };
        let _ = res.load();
        res
    }
}

impl App {
    pub fn new(monitors: Vec<(u32, u32)>) -> Self {
        let mut app = App::default();
        app.monitors = monitors;
        app.download_path = home_dir().map_or(PathBuf::default(), |mut home| {
            home.push("rusty-splash");
            home.push("downloads");
            if !home.exists() {
                fs::create_dir_all(&home).unwrap();
            }
            home
        });
        app.tile_path = home_dir().map_or(PathBuf::default(), |mut home| {
            home.push("rusty-splash");
            home.push("tiles");
            if !home.exists() {
                fs::create_dir_all(&home).unwrap();
            }
            home
        });
        app.downloads();
        let _ = app.save();
        app
    }
    fn downloads(&mut self) {
        if self.download_path.try_exists().is_ok() {
            self.downloaded = self
                .download_path
                .read_dir()
                .unwrap_or_else(|err| panic!("Couldn't read downloads dir: {err}"))
                .into_iter()
                .filter_map(|dir| dir.ok())
                .map(|entry| {
                    let file_name = entry
                        .path()
                        .file_stem()
                        .unwrap_or_else(|| {
                            panic!("invalid path: {}", entry.path().to_string_lossy())
                        })
                        .to_string_lossy()
                        .to_string();
                    file_name.parse::<u64>().unwrap_or_else(|err| {
                        panic!(
                            "Invalid file name: {}, error: {err}",
                            entry.path().to_string_lossy()
                        )
                    })
                })
                .collect();
        }
    }
    pub fn refresh_downloads(&mut self) {
        self.downloads();
        let _ = self.save();
    }
}

impl Cached for App {
    fn cache_name(&self) -> String {
        "app".to_string()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TileInstance {
    name: String,
    splash_ids: HashSet<u64>,
    path: PathBuf,
}

impl TileInstance {
    pub fn build(&self) {}
}
