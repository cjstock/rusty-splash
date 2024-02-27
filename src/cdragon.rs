use chrono::{DateTime, Utc};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, default, fmt::Display, path::PathBuf, u32, u64};

use crate::cache::Cached;

#[derive(Debug, Default, Serialize, Deserialize)]
struct CDragon {
    latest_date: DateTime<Utc>,
    champions: HashMap<u64, Champion>,
    plugins: Vec<Plugin>,
}

impl Cached for CDragon {
    fn cache_name(&self) -> String {
        String::from("c_dragon")
    }
}

impl CDragon {
    pub fn new() -> Self {
        let cdragon = Self::default();

        let plugins =
            Self::get_plugins().unwrap_or_else(|error| panic!("Couldn't get plugins: {error}"));

        let latest_date = plugins
            .first()
            .unwrap_or_else(|| panic!("Couldn't find first plugin"))
            .mtime;

        if let Err(err) = cdragon.load() {
            panic!("Error loading c_dragon cache: {err}");
        }

        let mut champions =
            Self::get_champions().unwrap_or_else(|error| panic!("Couldn't get champions: {error}"));
        for mut champion in &mut champions {
            Self::populate_skins(&mut champion.1);
        }
        CDragon {
            plugins,
            champions,
            latest_date,
        }
    }

    fn fetch_plugins() -> String {
        fetch("https://raw.communitydragon.org/json/latest/plugins/".to_string())
    }

    fn get_plugins() -> Result<Vec<Plugin>, serde_json::Error> {
        let plugin_res = Self::fetch_plugins();
        let plugins: Vec<Plugin> = serde_json::from_str(&plugin_res)?;
        Ok(plugins)
    }

    fn fetch_champions() -> String {
        fetch("https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json".to_string())
    }

    fn get_champions() -> Result<HashMap<u64, Champion>, serde_json::Error> {
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
                        id,
                        name,
                        alias,
                        skins: HashMap::new(),
                    },
                )
            })
            .collect();
        Ok(champions)
    }

    pub fn fetch_champion(id: u64) -> String {
        fetch(format!("https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champions/{}.json", id).to_string())
    }

    pub fn populate_skins(champion: &mut Champion) -> Result<(), serde_json::Error> {
        let champ_res = Self::fetch_champion(champion.id);
        let data: Value = serde_json::from_str(&champ_res)?;
        let skins: HashMap<u64, Skin> = data
            .as_object()
            .unwrap()
            .get("skins")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|value| {
                let id: u64 = value.get("id").unwrap().as_u64().unwrap();
                let skin: Skin = serde_json::from_value(value.to_owned()).unwrap();
                (id, skin)
            })
            .collect();
        champion.skins = skins;
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Champion {
    id: u64,
    name: String,
    alias: String,
    skins: HashMap<u64, Skin>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Skin {
    id: u64,
    name: String,
    is_base: bool,
    splash_path: String,
    uncentered_splash_path: String,
    skin_type: String,
    rarity: String,
    is_legacy: bool,
    skin_lines: Option<Vec<SkinLine>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SkinLine {
    id: u32,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum PluginName {
    #[default]
    None,
    RcpBeLolGameData,
    RcpBeLolLicenseAgreement,
    RcpBeSanitizer,
    RcpFeAudio,
    RcpFeCommonLibs,
    RcpFeEmberLibs,
    RcpFeLolCareerStats,
    RcpFeLolChampSelect,
    RcpFeLolChampionDetails,
    RcpFeLolChampionStatistics,
    RcpFeLolClash,
    RcpFeLolCollections,
    RcpFeLolEsportsSpectate,
    RcpFeLolEventHub,
    RcpFeLolEventShop,
    RcpFeLolHighlights,
    RcpFeLolHonor,
    RcpFeLolKickout,
    RcpFeLolL10n,
    RcpFeLolLeagues,
    RcpFeLolLockAndLoad,
    RcpFeLolLoot,
    RcpFeLolMatchHistory,
    RcpFeLolNavigation,
    RcpFeLolNewPlayerExperience,
    RcpFeLolNpeRewards,
    RcpFeLolParties,
    RcpFeLolPaw,
    RcpFeLolPft,
    RcpFeLolPostgame,
    RcpFeLolPremadeVoice,
    RcpFeLolProfiles,
    RcpFeLolSettings,
    RcpFeLolSharedComponents,
    RcpFeLolSkinsPicker,
    RcpFeLolSocial,
    RcpFeLolStartup,
    RcpFeLolStaticAssets,
    RcpFeLolStore,
    RcpFeLolTft,
    RcpFeLolTftTeamPlanner,
    RcpFeLolTftTroves,
    RcpFeLolTypekit,
    RcpFeLolUikit,
    RcpFeLolYourshop,
    RcpFePluginRunner,
    #[serde(other)]
    PluginManifest,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Plugin {
    #[serde(rename(deserialize = "name"))]
    name: PluginName,
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
    dbg!(&plugins);
    assert!(plugins.is_ok())
}

#[test]
fn get_champions() {
    let champions = CDragon::get_champions();
    assert!(champions.is_ok())
}

#[test]
fn populate_skins() {
    let mut champions = CDragon::get_champions().unwrap();
    let annie = champions.get_mut(&1);
    if let Some(annie) = annie {
        assert!(CDragon::populate_skins(annie).is_ok());
    }
}

#[test]
fn save_cache() {
    let data = CDragon::new();
    dbg!(&data);
    assert!(data.champions.get(&1).is_some())
}
