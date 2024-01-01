use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Champion {
    skins: Vec<Skin>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Skin {
    id: String,
    name: String,
    chromas: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChampionSkins {
    champions: HashMap<String, Champion>,
}

impl ChampionSkins {
    pub async fn new() -> ChampionSkins {
        let champs = fetch_champs()
            .await
            .unwrap_or_else(|error| panic!("Couldn't fetch champion data: {error}"));

        ChampionSkins {
            champions: load_champs(champs).await,
        }
    }
}

async fn load_champs(champs: String) -> HashMap<String, Champion> {
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
    for champ in champs {
        champ_map.insert(champ.to_string(), populate_champ(&champ).await);
    }
    champ_map
}

async fn populate_champ(champ_name: &str) -> Champion {
    let base_url =
        String::from("https://ddragon.leagueoflegends.com/cdn/13.24.1/data/en_US/champion/");
    let res = reqwest::get(format!("{}{}.json", base_url, champ_name))
        .await
        .unwrap_or_else(|error| {
            panic!("Couldn't fetch data for {}: {}", champ_name, error);
        });
    let result = res.text().await.unwrap_or_else(|error| {
        panic!("Couldn't get value for {champ_name}: {error}");
    });
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

async fn fetch_champs() -> Result<String, reqwest::Error> {
    let res =
        reqwest::get("https://ddragon.leagueoflegends.com/cdn/13.24.1/data/en_US/champion.json")
            .await?;

    let result = res.text().await?;
    Ok(result)
}
