use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, u32, u64};

#[derive(Debug, Serialize, Deserialize)]
struct CDragon {
    latest_date: String,
    champions: HashMap<String, Champion>,
    plugins: Vec<Plugin>,
}

impl CDragon {
    pub fn fetch_plugins() -> String {
        fetch("https://raw.communitydragon.org/json/latest/plugins/".to_string())
    }

    pub fn get_plugins() -> Result<Vec<Plugin>, serde_json::Error> {
        let plugin_res = Self::fetch_plugins();
        let plugins = serde_json::from_str::<Vec<Plugin>>(&plugin_res)?;
        Ok(plugins)
    }

    pub fn fetch_champions() -> String {
        fetch("https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json".to_string())
    }

    pub fn get_champions() -> Result<HashMap<u64, Champion>, serde_json::Error> {
        let champ_res = Self::fetch_champions();
        let data: Value = serde_json::from_str(&champ_res)?;
        let champions: HashMap<u64, Champion> = data
            .as_array()
            .unwrap()
            .iter()
            .skip(1)
            .map(|value| {
                let id: u64 = value.get("id").unwrap().as_u64().unwrap();
                let name: String = value.get("name").unwrap().to_string().replace('\"', "");
                let alias: String = value.get("alias").unwrap().to_string().replace('\"', "");
                (
                    id,
                    Champion {
                        name,
                        alias,
                        skins: HashMap::new(),
                    },
                )
            })
            .collect();
        dbg!(&champions);
        Ok(champions)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Champion {
    name: String,
    alias: String,
    skins: HashMap<String, Skin>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Skin {
    id: u32,
    is_base: bool,
    name: String,
    splash_path: String,
    uncentered_splash_path: String,
    skin_type: String,
    rarity: String,
    is_legacy: bool,
    chroma_path: Option<String>,
    skin_lines: Option<Vec<SkinLine>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SkinLine {
    id: u32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Plugin {
    name: String,
    #[serde(with = "mtime_format")]
    mtime: DateTime<Utc>,
}

mod mtime_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%a, %d %b %Y %H:%M:%S %Z";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}
pub fn fetch(url: String) -> String {
    let res = reqwest::blocking::get(url.clone())
        .unwrap_or_else(|error| panic!("Error in request {url}: {error}"));

    res.text()
        .unwrap_or_else(|error| panic!("Error in request {url}: {error}"))
}

#[cfg(test)]
#[test]
fn get_plugins() {
    let plugins = CDragon::get_plugins();
    assert!(plugins.is_ok())
}

#[test]
fn get_champions() {
    let champions = CDragon::get_champions();
    assert!(champions.is_ok())
}
