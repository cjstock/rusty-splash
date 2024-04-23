use std::{any, fs, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

pub trait Cached: Serialize + for<'a> Deserialize<'a> + Default {
    fn cache_name(&self) -> String;

    fn save(&self) -> anyhow::Result<()> {
        let cache_str = serde_json::to_string_pretty(&self)?;
        fs::write(self.cache()?, cache_str)?;
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        let cached_data: String = fs::read_to_string(self.cache()?)
            .with_context(|| format!("failed to read from {:?}", self.cache()))?
            .parse()
            .with_context(|| {
                format!(
                    "failed to parse the deserialized type: {:?}",
                    any::type_name::<Self>()
                )
            })?;

        *self = serde_json::from_str(&cached_data.to_owned()).with_context(|| {
            format!("failed to deserialize type: {:?}", any::type_name::<Self>())
        })?;

        Ok(())
    }

    fn cache(&self) -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir().with_context(|| "Couldn't get home dir")?;
        let mut file_path = [home.to_str().unwrap(), "rusty-splash"]
            .iter()
            .collect::<PathBuf>();
        if !file_path.exists() {
            fs::create_dir_all(&file_path)
                .with_context(|| format!("failed to create dir path: {:?}", file_path))?;
        }
        file_path.push(format!("{}.json", self.cache_name()));
        Ok(file_path)
    }
}
