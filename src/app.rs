use std::{collections::HashSet, fs, path::PathBuf, u64};

use anyhow::{anyhow, Context};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cache::Cached;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct App {
    pub download_path: PathBuf,
    pub downloaded: HashSet<u64>,
    pub tile_path: PathBuf,
    pub tiles: Vec<TileInstance>,
    pub selected_tile: Uuid,
    pub monitors: Vec<(u32, u32)>,
}

impl App {
    pub fn new(monitors: Vec<(u32, u32)>) -> anyhow::Result<Self> {
        let mut app = App::default();
        match app.load() {
            Ok(_) => Ok(app),
            Err(_) => {
                app.monitors = monitors;
                let mut home = home_dir().ok_or(anyhow!("couldn't get home dir"))?;
                home.push("rusty-splash");
                home.push("downloads");
                if !home.exists() {
                    fs::create_dir_all(&home)
                        .with_context(|| anyhow!("failed to create missing downloads dir"))?;
                }
                app.download_path = home.clone();

                home.pop();
                home.push("tiles");
                if !home.exists() {
                    fs::create_dir_all(&home)
                        .with_context(|| anyhow!("failed to create missing downloads dir"))?;
                }
                app.tile_path = home;
                app.downloads();
                app.save()?;
                Ok(app)
            }
        }
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

    pub fn tile_select(&mut self, tile_id: Uuid) -> anyhow::Result<()> {
        match self.selected_tile == tile_id {
            true => Ok(()),
            false => match self.tiles.iter().any(|tile| tile.id == tile_id) {
                true => {
                    self.selected_tile = tile_id;
                    self.save()?;
                    Ok(())
                }
                false => Err(anyhow!("tile {} not found", tile_id)),
            },
        }
    }

    pub fn tile_new<S>(&mut self, name: S) -> anyhow::Result<Uuid>
    where
        S: Into<String>,
    {
        let new_tile = TileInstance::new(name);
        let id = new_tile.id;
        self.tiles.push(new_tile);
        self.save()?;
        Ok(id)
    }

    pub fn tile_update_name<S>(&mut self, id: Uuid, name: S) -> anyhow::Result<()>
    where
        S: Into<String>,
    {
        let tile = self.tiles.iter_mut().find(|tile| tile.id == id);
        match tile {
            Some(tile) => {
                tile.name = name.into();
                self.save()?;
                Ok(())
            }
            None => Err(anyhow!(
                "couldn't update the name of tile {:?}, because it doesn't exist",
                id
            )),
        }
    }

    pub fn tile_delete(&mut self, id: Uuid) -> anyhow::Result<()> {
        let index = self.tiles.iter().position(|tile| tile.id == id);
        if let Some(index) = index {
            self.tiles.remove(index);
            if self.selected_tile == id {
                self.selected_tile = Uuid::default();
            }
            self.save()?;
            Ok(())
        } else {
            Err(anyhow!("tile {} not found!", id))
        }
    }

    pub fn tile_add_splash(&mut self, id: &Uuid, splash_id: &u64) -> anyhow::Result<()> {
        match self.tiles.iter_mut().find(|tile| tile.id == *id) {
            Some(tile) => {
                tile.splash_ids.insert(*splash_id);
                self.save()?;
                Ok(())
            }
            None => Err(anyhow!(
                "couldn't add splashes to tile {:?}, because it doesn't exst",
                id
            )),
        }
    }

    pub fn tile_remove_splashes(
        &mut self,
        id: Uuid,
        splash_ids: &HashSet<u64>,
    ) -> anyhow::Result<()> {
        match self.tiles.iter_mut().find(|tile| tile.id == id) {
            Some(tile) => {
                tile.remove_splashes(splash_ids);
                self.save()?;
                return Ok(());
            }
            None => Err(anyhow!("no tile {:?} found", id)),
        }
    }
}

impl Cached for App {
    fn cache_name() -> String {
        String::from("app")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileInstance {
    pub id: Uuid,
    pub name: String,
    pub splash_ids: HashSet<u64>,
    path: PathBuf,
}

impl TileInstance {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            id: Uuid::new_v4(),
            ..Self::default()
        }
    }
    pub fn add_splashes(&mut self, ids: &HashSet<u64>) {
        ids.iter().for_each(|id| {
            self.splash_ids.insert(*id);
        });
    }

    pub fn remove_splashes(&mut self, ids: &HashSet<u64>) {
        for id in ids {
            let _ = self.splash_ids.remove(id);
        }
    }

    pub fn set_name<S>(&mut self, new_name: S)
    where
        S: Into<String>,
    {
        self.name = new_name.into();
    }
}

#[cfg(test)]
mod test {
    use display_info::DisplayInfo;

    use crate::app::App;

    #[test]
    fn load_app() {
        let monitors = DisplayInfo::all()
            .unwrap()
            .iter()
            .map(|monitor| (monitor.width, monitor.height))
            .collect();
        let app = App::new(monitors);
        assert!(app.is_ok());
    }

    #[test]
    fn load_app_no_internet() {
        let monitors = DisplayInfo::all()
            .unwrap()
            .iter()
            .map(|monitor| (monitor.width, monitor.height))
            .collect();
        let app = App::new(monitors);
        assert!(app.is_err())
    }

    #[test]
    fn add_tile() {
        let mut app = App::default();
        assert!(app.tile_new(String::from("testy")).is_ok());
        assert!(app
            .tiles
            .iter()
            .find(|tile| tile.name == String::from("testy"))
            .is_some());
    }
}
