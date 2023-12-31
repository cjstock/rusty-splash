use serde::{Deserialize, Serialize};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let champs = fetch_champs()
        .await
        .unwrap_or_else(|error| panic!("Couldn't fetch champion data: {error}"));
    let champ_ids = champion_ids(&champs);
    let champs: Vec<Champion> = champ_ids
        .iter()
        .map(|id| Champion { id: id.to_string() })
        .collect();
    println!("{:?}", champs)
}

#[derive(Serialize, Deserialize, Debug)]
struct Champion {
    id: String,
}

fn champion_ids(champs: &str) -> Vec<String> {
    let root: Value = serde_json::from_str(champs)
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
    champs
}

async fn fetch_champs() -> Result<String, reqwest::Error> {
    let res =
        reqwest::get("https://ddragon.leagueoflegends.com/cdn/13.24.1/data/en_US/champion.json")
            .await?;

    let result = res.text().await?;
    Ok(result)
}
