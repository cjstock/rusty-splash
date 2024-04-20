use core::panic;
use std::{error::Error, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

pub trait Cached: Serialize + for<'a> Deserialize<'a> + Default {
    fn cache_name(&self) -> String;

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let cache_str = serde_json::to_string_pretty(&self)?;
        fs::write(self.cache(), cache_str)?;
        Ok(())
    }

    fn load(&mut self) {
        let cached_data: String = fs::read_to_string(self.cache())
            .unwrap_or_default()
            .parse()
            .unwrap_or_default();

        *self = serde_json::from_str(&cached_data.to_owned()).unwrap_or_default();
    }

    fn cache(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| panic!("Couldn't get home dir"));
        let mut file_path = [home.to_str().unwrap(), "rusty-splash"]
            .iter()
            .collect::<PathBuf>();
        if !file_path.exists() {
            fs::create_dir_all(&file_path).unwrap();
            println!("created: {:?}", file_path);
        }
        file_path.push(format!("{}.json", self.cache_name()));
        file_path
    }
}
