use core::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error::Error, fs, path::PathBuf, sync::mpsc, thread, u32};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Champion {
    skins: Vec<Skin>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Skin {
    id: String,
    pub name: String,
    chromas: bool,
    num: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChampionSkins {
    version: String,
    cache_file_path: PathBuf,
    champions: HashMap<String, Champion>,
}

impl Default for ChampionSkins {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| panic!("Couldn't get home dir"));
        let mut cache_file_path = [home.to_str().unwrap(), "rusty-splash"]
            .iter()
            .collect::<PathBuf>();
        fs::create_dir_all(&cache_file_path).unwrap();
        cache_file_path.push("cache.json");
        let versions = fetch_versions().unwrap();
        let versions: Vec<String> = serde_json::from_str(&versions).unwrap();
        let latest_version = &versions[0];
        ChampionSkins {
            champions: HashMap::new(),
            cache_file_path,
            version: latest_version.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct VersionOutOfDate;

impl fmt::Display for VersionOutOfDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cache is out of date")
    }
}

impl ChampionSkins {
    pub fn skins_for(&self, champ_name: &str) -> Vec<(u32, String)> {
        let res = &self
            .champions
            .get(champ_name)
            .expect("Cound't find that champion")
            .skins
            .iter()
            .map(|skin| (skin.num, skin.name.to_string()))
            .collect::<Vec<(u32, String)>>();
        res.to_vec()
    }

    fn save_cache(&self) {
        let data = serde_json::to_string(self)
            .unwrap_or_else(|error| panic!("Couldn't serialize data: {error}"));
        fs::write(self.cache_file_path.to_str().unwrap(), data).unwrap_or_else(|error| {
            panic!(
                "Couldn't write cache to {:?}: {error}",
                self.cache_file_path.to_str()
            )
        });
    }

    fn load_cache(&self) -> Result<ChampionSkins, Box<dyn Error>> {
        let cache: String = fs::read_to_string(&self.cache_file_path)?.parse()?;
        let data: ChampionSkins = serde_json::from_str(&cache.to_owned())?;
        Ok(data)
    }

    fn test_cache(&mut self) -> Result<(), VersionOutOfDate> {
        let data = self.load_cache().unwrap_or(ChampionSkins {
            version: String::from(""),
            ..Default::default()
        });
        match data.version == self.version {
            true => {
                *self = data;
                Ok(())
            }
            false => Err(VersionOutOfDate),
        }
    }

    pub fn load() -> ChampionSkins {
        let mut data = ChampionSkins {
            ..Default::default()
        };
        match data.test_cache() {
            Ok(_) => data,
            Err(_) => {
                let champs = fetch_champs()
                    .unwrap_or_else(|error| panic!("Couldn't fetch champion data: {error}"));
                data.champions = load_champs(champs);
                data.save_cache();
                data
            }
        }
    }

    pub fn new() -> ChampionSkins {
        ChampionSkins::default()
    }
}

fn load_champs(champs: String) -> HashMap<String, Champion> {
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
        thread::spawn(move || {
            tx.send((champ.to_string(), populate_champ(&champ)))
                .unwrap();
        });
    }
    drop(tx);
    for received in rx {
        champ_map.insert(received.0.to_string(), received.1);
    }
    println!("Finished");
    champ_map
}

fn populate_champ(champ_name: &str) -> Champion {
    let result = fetch_champ(champ_name).unwrap();
    let root: Value = serde_json::from_str(&result).unwrap_or_else(|error| {
        panic!("Couldn't parse {champ_name}: {error}");
    });
    let skins = root
        .get("data")
        .and_then(|val| val.get(champ_name))
        .and_then(|val| val.get("skins"))
        .unwrap();
    let skin_data: Vec<Skin> = serde_json::from_str(&skins.to_string())
        .unwrap_or_else(|error| panic!("Failed to deserialize skins for {champ_name}: {error}"));

    Champion { skins: skin_data }
}

fn fetch_versions() -> Result<String, reqwest::Error> {
    let url = String::from("https://ddragon.leagueoflegends.com/api/versions.json");
    let res = reqwest::blocking::get(url).unwrap_or_else(|error| {
        panic!("Couldn't fetch version: {error}");
    });
    let result = res.text().unwrap_or_else(|error| {
        panic!("Couldn't get version value from fetched data: {error}");
    });
    Ok(result)
}

fn fetch_champ(champ_name: &str) -> Result<String, reqwest::Error> {
    let base_url =
        String::from("https://ddragon.leagueoflegends.com/cdn/13.24.1/data/en_US/champion/");
    let res = reqwest::blocking::get(format!("{}{}.json", base_url, champ_name)).unwrap_or_else(
        |error| {
            panic!("Couldn't fetch data for {}: {}", champ_name, error);
        },
    );
    let result = res.text().unwrap_or_else(|error| {
        panic!("Couldn't get value for {champ_name}: {error}");
    });
    Ok(result)
}

fn fetch_champs() -> Result<String, reqwest::Error> {
    let res = reqwest::blocking::get(
        "https://ddragon.leagueoflegends.com/cdn/13.24.1/data/en_US/champion.json",
    )?;

    let result = res.text()?;
    Ok(result)
}
