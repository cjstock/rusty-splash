use anyhow::{anyhow, Context, Ok};
use chrono::{DateTime, Utc};
use core::panic;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs, io, path::PathBuf, u32, u64};

use crate::cache::Cached;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CDragon {
    pub latest_date: DateTime<Utc>,
    pub champions: HashMap<u64, Champion>,
    pub plugins: Vec<Plugin>,
}

impl Cached for CDragon {
    fn cache_name(&self) -> String {
        String::from("cdragon")
    }
}

impl CDragon {
    pub fn new() -> anyhow::Result<Self> {
        let mut cdragon = Self::default();
        cdragon.load().unwrap_or_default();

        let plugins = Self::get_plugins()?;

        let game_data_plugin = plugins
            .iter()
            .find(|p| p.name == PluginName::RcpBeLolGameData);
        if let Some(plug) = game_data_plugin {
            let latest_date = plug.mtime;
            if cdragon.latest_date.lt(&latest_date) {
                print!("New version found! Updating...");
                let mut champions = Self::get_champions()?;
                champions.par_iter_mut().for_each(|champ| {
                    champ.1.skins = Self::get_skins(*champ.0)
                        .with_context(|| "failed to load skins")
                        .unwrap();
                });
                cdragon = CDragon {
                    champions,
                    latest_date,
                    plugins,
                };
                print!("Done!");
                cdragon.save()?;
            }
        } else {
            print!("Failed to check for latest CommunityDragon version!");
        }
        Ok(cdragon)
    }

    fn fetch_plugins() -> String {
        fetch("https://raw.communitydragon.org/json/latest/plugins/".to_string())
            .unwrap_or_else(|err| panic!("error fetching plugins: {err}"))
            .text()
            .unwrap_or_else(|err| panic!("error in plugin response text: {err}"))
    }

    fn get_plugins() -> anyhow::Result<Vec<Plugin>> {
        let plugin_res = Self::fetch_plugins();
        serde_json::from_str(&plugin_res).with_context(|| "failed to deserialize plugins")
    }

    fn fetch_champions() -> anyhow::Result<String> {
        fetch("https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json".to_string())
        .with_context(|| format!("error fetching champions"))?
            .text()
            .with_context(|| format!("error in champions response text"))
    }

    fn get_champions() -> anyhow::Result<HashMap<u64, Champion>> {
        let champ_res = Self::fetch_champions().with_context(|| "failed to fetch champions")?;
        let data: Value = serde_json::from_str(&champ_res)
            .with_context(|| "failed to deserialize champions response")?;
        let champions: anyhow::Result<HashMap<u64, Champion>> = data
            .as_array()
            .with_context(|| "failed to deserialize the champions into an array")?
            .iter()
            .skip(1)
            .map(|value| {
                let id: u64 = value
                    .get("id")
                    .with_context(|| "failed to get the id for champion")?
                    .as_u64()
                    .with_context(|| "failed to cast the id to u64")?;
                let name: String = value
                    .get("name")
                    .with_context(|| "no name found ")?
                    .to_string()
                    .replace('\"', "");
                let alias: String = value
                    .get("alias")
                    .with_context(|| "failed to get alias for champion")?
                    .to_string()
                    .replace('\"', "");
                Ok((
                    id,
                    Champion {
                        id,
                        name,
                        alias,
                        skins: HashMap::new(),
                    },
                ))
            })
            .collect();
        champions
    }

    pub fn fetch_champion(id: u64) -> anyhow::Result<String> {
        fetch(format!("https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champions/{}.json", id).to_string())
            .with_context(|| "error fetching champion")?
            .text()
            .with_context(|| "error in champion response text")
    }

    pub fn get_skins(champion_id: u64) -> anyhow::Result<HashMap<u64, Skin>> {
        let champ_res = Self::fetch_champion(champion_id)?;
        let data: Value = serde_json::from_str(&champ_res)
            .with_context(|| "failed to convert the response text to a Json value")?;
        data.as_object()
            .with_context(|| "invalid champion structure")?
            .get("skins")
            .with_context(|| "failed to get the skins for the champion")?
            .as_array()
            .with_context(|| "failed to convert skins objec to array")?
            .iter()
            .map(|value| {
                let id: u64 = value
                    .get("id")
                    .with_context(|| "failed to get id for skin")?
                    .as_u64()
                    .with_context(|| "failed to cast id for skin to u64")?;
                let mut skin: Skin = serde_json::from_value(value.to_owned())
                    .with_context(|| "failed to deserialize skin json")?;
                skin.uncentered_splash_path = skin
                    .uncentered_splash_path
                    .clone()
                    .components()
                    .skip(3)
                    .collect();
                Ok((id, skin))
            })
            .collect()
    }

    /// Get a `Skin` by id
    pub fn skin(&self, id: u64) -> Option<&Skin> {
        self.champions.par_iter().find_map_first(|champion| {
            champion
                .1
                .skins
                .par_iter()
                .find_map_first(|skin| match skin.0.eq(&id) {
                    true => Some(skin.1),
                    false => None,
                })
        })
    }

    pub fn query(&self, query: impl Into<String>) -> anyhow::Result<Vec<&Skin>> {
        let query: String = query.into();
        let result: Vec<&Skin> = self
            .champions
            .iter()
            .flat_map(|champ| {
                champ.1.skins.iter().filter_map(|skin| {
                    skin.1
                        .name
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                        .then_some(skin.1)
                })
            })
            .collect();
        if result.is_empty() {
            Err(anyhow!("no skins found!"))
        } else {
            Ok(result)
        }
    }

    /// Download an uncentered splash
    ///
    /// # Arguments
    /// * `save_path` - the target directory
    pub fn download_splash(skin: &Skin, save_path: &PathBuf) -> anyhow::Result<()> {
        let mut file_path = save_path.clone();
        file_path.push(format!("{}.jpg", skin.id.to_string()));
        let mut file = fs::File::create(&file_path)
            .with_context(|| format!("error creating file for skin {}", skin.id))?;

        let url = PathBuf::from(
            "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default",
        )
        .join(&skin.uncentered_splash_path);
        let image = fetch(url.to_str().unwrap())
            .with_context(|| format!("error fetching skin {}", skin.id))?
            .bytes()
            .with_context(|| format!("error in reponse bytes for skin {}", skin.id))?;

        io::copy(&mut image.as_ref(), &mut file)
            .with_context(|| format!("error saving image {:?}", file_path))?;

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Champion {
    id: u64,
    name: String,
    alias: String,
    skins: HashMap<u64, Skin>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub id: u64,
    pub name: String,
    pub is_base: bool,
    pub splash_path: String,
    /// The relative path to the uncentered (full) splash art for the skin
    ///
    /// # Example
    /// Annie's base splash will have an `uncentered_splash_path` of `/lol-game-data/assets/v1/champion-splashes/uncentered/1/1000.jpg`.
    /// However, the actual full path to the file is at `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-splashes/uncentered/1/1000.jpg`
    pub uncentered_splash_path: PathBuf,
    pub skin_type: String,
    pub rarity: String,
    pub is_legacy: bool,
    pub skin_lines: Option<Vec<SkinLine>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SkinLine {
    id: u32,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
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
pub struct Plugin {
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
pub fn fetch(url: impl Into<String>) -> reqwest::Result<reqwest::blocking::Response> {
    reqwest::blocking::get(url.into())
}

#[cfg(test)]
#[test]
fn get_plugins() {
    let plugins = CDragon::get_plugins();
    dbg!(&plugins);
    assert!(plugins.len() > 0)
}

#[test]
fn get_champions() {
    let champions = CDragon::get_champions();
    assert!(champions.len() > 0)
}

#[test]
fn populate_skins() {
    let mut champions = CDragon::get_champions();
    let annie = champions.get_mut(&1);
    if let Some(annie) = annie {
        annie.skins = CDragon::get_skins(annie.id);
        dbg!(&annie);
        assert!(annie.skins.len() > 0);
    }
}

#[test]
fn save_cache() {
    let data = CDragon::new();
    assert!(data.champions.get(&1).is_some())
}
