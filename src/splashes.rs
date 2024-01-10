pub use image::EncodableLayout;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error::Error, fs, io, path::PathBuf, sync::mpsc, thread, u32};

use crate::{
    cache::Cache,
    datadragon::{get_latest_version, request_champs},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Champion {
    pub name: String,
    pub skins: Vec<Skin>,
    pub tags: Vec<String>,
}

impl Champion {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            skins: vec![],
            tags: vec![],
        }
    }

    pub fn fetch_champ(&self, patch: &str) -> Result<String, reqwest::Error> {
        let base_url = format!(
            "https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion/",
            patch
        );
        let res = reqwest::blocking::get(format!("{}{}.json", base_url, self.name)).unwrap_or_else(
            |error| {
                panic!("Couldn't fetch data for {}: {}", self.name, error);
            },
        );
        let result = res.text().unwrap_or_else(|error| {
            panic!("Couldn't get value for {}: {error}", self.name);
        });
        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Skin {
    pub id: String,
    pub name: String,
    pub chromas: bool,
    pub num: u32,
    pub champ: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Splashes {
    pub champions: HashMap<String, Champion>,
    pub cache: Cache,
    pub patch: String,
    pub save_dir: PathBuf,
}

impl Splashes {
    pub fn splashes_for_champ(&self, name: &str) -> Vec<(u32, String)> {
        let res = &self
            .champions
            .get(name)
            .expect("Cound't find that champion")
            .skins
            .iter()
            .map(|skin| (skin.num, skin.name.to_string()))
            .collect::<Vec<(u32, String)>>();
        res.to_vec()
    }

    pub fn skin(&self, name: &str) -> Option<&Skin> {
        for (_, champ) in self.champions.iter() {
            if let Some(skin) = champ.skins.iter().find(|skin| skin.name == name) {
                return Some(skin);
            }
        }
        None
    }

    pub fn download(&self, skin: &Skin) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "https://ddragon.leagueoflegends.com/cdn/img/champion/splash/{}_{}.jpg",
            skin.champ.clone().unwrap(),
            skin.num
        );
        let response = reqwest::blocking::get(url)?;
        let image_data = response.bytes()?;
        let save_path =
            self.save_dir
                .join(format!("{}_{}.jpg", skin.champ.clone().unwrap(), skin.num));
        io::copy(
            &mut image_data.as_bytes(),
            &mut fs::File::create(save_path)?,
        )?;
        Ok(())
    }

    pub fn all_tags(&self) -> Vec<String> {
        let mut all_tags: Vec<String> = self
            .champions
            .iter()
            .flat_map(|champion| champion.1.tags.clone())
            .collect();
        all_tags.sort_unstable();
        all_tags.dedup();
        all_tags
    }

    pub fn search_skins(&self, query: &str) -> Vec<&Skin> {
        self.champions
            .iter()
            .flat_map(|champ| &champ.1.skins)
            .filter(|skin| skin.name.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&Skin>>()
    }

    pub fn new() -> Splashes {
        let mut splashes = Splashes::default();
        let latest_version = get_latest_version();
        if let Ok(cached_patch) = splashes.cache.get_patch_version() {
            if latest_version != cached_patch {
                println!("Found a newer version: {latest_version}");
                println!("Would you like to update?");
                let mut input = String::default();
                let _ = std::io::stdin().read_line(&mut input);
                match input.to_lowercase().trim() {
                    "y" | "yes" => splashes.update(&latest_version),
                    _ => (),
                }
            }
            splashes.load();
        } else {
            println!("Getting data for patch: {latest_version}");
            splashes.update(&latest_version);
        }
        splashes.save_dir = splashes.cache.path.join("splashes");
        let _ = fs::create_dir_all(&splashes.save_dir);
        splashes
    }

    pub fn load(&mut self) {
        if let Ok(splashes) = self.cache.get_data() {
            self.champions = splashes.champions;
        }
    }

    pub fn save_data(&self) {
        self.cache.save_data(self);
    }

    pub fn update(&mut self, to_patch: &str) {
        self.patch = to_patch.to_string();
        self.cache.save_patch_version(to_patch);
        let new_data = map_champs(to_patch);
        self.champions = new_data;
        self.cache.save_data(self);
    }
}

fn map_champs(patch: &str) -> HashMap<String, Champion> {
    let champs = request_champs(patch).unwrap();
    let root: Value = serde_json::from_str(&champs)
        .unwrap_or_else(|error| panic!("Couldn't parse champions json: {error}"));
    let champs = root
        .get("data")
        .expect("Couldn't find 'data' entry")
        .as_object()
        .unwrap();
    let champs: Vec<String> = champs
        .values()
        .map(|val| {
            val.as_object()
                .unwrap()
                .get("id")
                .expect("Couldn't find 'id' entry")
                .to_string()
                .replace('\"', "")
        })
        .collect();
    let mut champ_map: HashMap<String, Champion> = HashMap::new();
    let (tx, rx) = mpsc::channel();
    for champ in champs {
        let tx = tx.clone();
        let patch = patch.to_string();
        thread::spawn(move || {
            tx.send((champ.to_string(), populate_champ(patch, &champ)))
                .unwrap();
        });
    }
    drop(tx);
    for received in rx {
        champ_map.insert(received.0.to_string(), received.1);
    }
    champ_map
}

fn populate_champ(patch: String, champ_name: &str) -> Champion {
    let mut champion = Champion::new(champ_name);
    let result = champion.fetch_champ(&patch).unwrap();
    let root: Value = serde_json::from_str(&result).unwrap_or_else(|error| {
        panic!("Couldn't parse {champ_name}: {error}");
    });

    let skins = root
        .get("data")
        .and_then(|val| val.get(champ_name))
        .and_then(|val| val.get("skins"))
        .unwrap();
    let mut skin_data: Vec<Skin> = serde_json::from_str(&skins.to_string())
        .unwrap_or_else(|error| panic!("Failed to deserialize skins for {champ_name}: {error}"));
    for skin in skin_data.iter_mut() {
        skin.champ = Some(champ_name.to_string());
    }
    let default_splash = skin_data
        .iter_mut()
        .find(|skin| skin.name == "default")
        .unwrap();
    default_splash.name = champ_name.to_string();

    let tags = root
        .get("data")
        .and_then(|val| val.get(champ_name))
        .and_then(|val| val.get("tags"))
        .unwrap();
    let tag_data: Vec<String> = serde_json::from_str(&tags.to_string())
        .unwrap_or_else(|error| panic!("Failed to deserialize tags for {champ_name}: {error}"));

    champion.skins = skin_data;
    champion.tags = tag_data;
    champion
}

#[cfg(test)]
#[test]
fn open_preview() {
    use crate::datadragon;

    let data = Splashes::new();
    let _ = datadragon::preview_splash(data.skin("Aatrox").unwrap());
}

#[test]
fn champ_data() {
    let data = Splashes::new();
    let aatrox = data.champions.get("Aatrox").unwrap();
    let data = serde_json::to_string_pretty(&aatrox).unwrap();
    let _ = fs::write("aatrox_object.json", data);
}

#[test]
fn champ_string() {
    let latest_patch = get_latest_version();
    let data = Champion::new("Aatrox");
    let data = data.fetch_champ(&latest_patch).unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    let champ = serde_json::to_string_pretty(&data).unwrap();
    let _ = fs::write("aatrox.json", champ);
}
