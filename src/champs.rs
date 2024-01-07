use core::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error::Error, fs, path::PathBuf, sync::mpsc, thread, u32};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    path: PathBuf,
    data_file: PathBuf,
    version_file: PathBuf,
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
            version_file: {
                let mut path = file_path.clone();
                path.push("version.json");
                path
            },
        }
    }
}

impl Cache {
    pub fn save_version(&self, version: &str) {
        let version = serde_json::to_string(version)
            .unwrap_or_else(|error| panic!("Couldn't serialize data: {error}"));
        fs::write(self.version_file.clone(), version).unwrap_or_else(|error| {
            panic!("Couldn't write version to {:?}: {error}", self.version_file)
        });
    }

    pub fn save_data(&self, data: &Splashes) {
        let data = serde_json::to_string(&data)
            .unwrap_or_else(|error| panic!("Couldn't serialize data: {error}"));
        fs::write(self.data_file.clone(), data).unwrap_or_else(|error| {
            panic!("Couldn't write cache to {:?}: {error}", self.data_file)
        });
    }

    fn get_version(&mut self) -> Result<String, Box<dyn Error>> {
        let cached_version = fs::read_to_string(&self.version_file)?;
        let mut version: String = cached_version.parse()?;
        version = version.replace('\"', "");
        Ok(version)
    }

    fn get_data(&self) -> Result<Splashes, Box<dyn Error>> {
        let cache: String = fs::read_to_string(&self.data_file)?.parse()?;
        let data: Splashes = serde_json::from_str(&cache.to_owned())?;
        Ok(data)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Champion {
    name: String,
    skins: Vec<Skin>,
}

impl Champion {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            skins: vec![],
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
    id: String,
    pub name: String,
    chromas: bool,
    num: u32,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Splashes {
    champions: HashMap<String, Champion>,
    pub cache: Cache,
    patch: String,
}

#[derive(Debug, Clone)]
struct VersionOutOfDate;

impl fmt::Display for VersionOutOfDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cache is out of date")
    }
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

    pub fn new() -> Splashes {
        let mut splashes = Splashes::default();
        let version = splashes.cache.get_version().unwrap_or_else(|_| {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let v = get_latest_version();
                let _ = tx.send(v);
            });
            rx.recv().expect("Problem getting latest version").unwrap()
        });
        splashes.patch = version.to_string();
        splashes.cache.save_version(&version);
        splashes
    }

    pub fn load(&mut self) {
        if let Ok(splashes) = self.cache.get_data() {
            println!("Found cached data...");
            self.champions = splashes.champions;
        } else {
            println!("Fetching champions...");
            self.champions = load_champs(&self.patch);
        }
    }

    pub fn save_data(&self) {
        self.cache.save_data(self);
    }
}
fn fetch_champs(patch: &str) -> Result<String, reqwest::Error> {
    let res = reqwest::blocking::get(format!(
        "https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json",
        patch
    ))?;

    let result = res.text()?;
    Ok(result)
}
fn load_champs(patch: &str) -> HashMap<String, Champion> {
    let champs = fetch_champs(patch).unwrap();
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
    println!("Finished");
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
    let skin_data: Vec<Skin> = serde_json::from_str(&skins.to_string())
        .unwrap_or_else(|error| panic!("Failed to deserialize skins for {champ_name}: {error}"));

    champion.skins = skin_data;
    champion
}

fn get_latest_version() -> Option<String> {
    let versions = fetch_versions().unwrap_or_else(|error| {
        panic!("Couldn't fetch versions: {error}");
    });
    let versions: Vec<String> = serde_json::from_str(&versions).unwrap_or_else(|error| {
        panic!("Invalid response: {error}");
    });
    let versions: Vec<String> = versions
        .iter()
        .map(|version| version.replace('\"', ""))
        .collect();
    versions.first().cloned()
}

fn fetch_versions() -> Result<String, reqwest::Error> {
    let url = String::from("https://ddragon.leagueoflegends.com/api/versions.json");
    let res = reqwest::blocking::get(url)?;
    let result = res.text()?;
    Ok(result)
}
