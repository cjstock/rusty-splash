use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error::Error, fs, path::PathBuf, sync::mpsc, thread, u32};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    path: PathBuf,
    data_file: PathBuf,
    patch_file: PathBuf,
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

    fn get_patch_version(&mut self) -> Result<String, Box<dyn Error>> {
        let cached_version = fs::read_to_string(&self.patch_file)?;
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
        let latest_version = get_latest_version();
        if let Ok(cached_patch) = splashes.cache.get_patch_version() {
            if latest_version != cached_patch {
                println!("Found a newer version: {latest_version}");
                println!("Would you like to update?");
                let mut input = String::default();
                let _ = std::io::stdin().read_line(&mut input);
                match input.to_lowercase().as_ref() {
                    "y" | "yes" => splashes.update(&latest_version),
                    _ => (),
                }
            }
        } else {
            splashes.update(&latest_version);
        }
        splashes
    }

    pub fn load(&mut self) {
        if let Ok(splashes) = self.cache.get_data() {
            self.champions = splashes.champions;
        } else {
            self.champions = map_champs(&self.patch);
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

fn get_latest_version() -> String {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let v = fetch_latest_version();
        let _ = tx.send(v);
    });
    rx.recv().expect("Problem getting latest version").unwrap()
}

fn request_champs(patch: &str) -> Result<String, reqwest::Error> {
    let res = reqwest::blocking::get(format!(
        "https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json",
        patch
    ))?;

    let result = res.text()?;
    Ok(result)
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
    let skin_data: Vec<Skin> = serde_json::from_str(&skins.to_string())
        .unwrap_or_else(|error| panic!("Failed to deserialize skins for {champ_name}: {error}"));

    champion.skins = skin_data;
    champion
}

fn fetch_latest_version() -> Option<String> {
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
