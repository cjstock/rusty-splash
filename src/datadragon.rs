use std::{io::Error, sync::mpsc, thread};

use crate::splashes::Skin;

pub fn get_latest_version() -> String {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let v = fetch_latest_version();
        let _ = tx.send(v);
    });
    rx.recv().expect("Problem getting latest version").unwrap()
}

pub fn splash_url(skin: &Skin) -> String {
    format!(
        "https://ddragon.leagueoflegends.com/cdn/img/champion/splash/{}_{}.jpg",
        if skin.id == "9027" {
            "FiddleSticks".to_string()
        } else {
            skin.champ.clone()
        },
        skin.num
    )
}

pub fn preview_splash(skin: &Skin) -> Result<(), Error> {
    let url = splash_url(skin);
    open::that(url)?;
    Ok(())
}

pub fn request_champs(patch: &str) -> Result<String, reqwest::Error> {
    let res = reqwest::blocking::get(format!(
        "https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json",
        patch
    ))?;

    let result = res.text()?;
    Ok(result)
}

pub fn fetch_latest_version() -> Option<String> {
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

pub fn fetch_versions() -> Result<String, reqwest::Error> {
    let url = String::from("https://ddragon.leagueoflegends.com/api/versions.json");
    let res = reqwest::blocking::get(url)?;
    let result = res.text()?;
    Ok(result)
}
