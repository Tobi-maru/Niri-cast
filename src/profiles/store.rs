use std::fs;
use std::path::PathBuf;

use anyhow::Context;

use crate::profiles::model::{ProfileCollection, TvProfile};

#[derive(Debug, Clone)]
pub struct ProfileStore {
    path: PathBuf,
}

impl ProfileStore {
    pub fn new() -> anyhow::Result<Self> {
        let mut dir = dirs::config_dir().context("could not resolve XDG config dir")?;
        dir.push("niri-cast");
        fs::create_dir_all(&dir).context("failed to create config directory")?;

        let mut path = dir;
        path.push("profiles.json");

        if !path.exists() {
            let initial = serde_json::to_string_pretty(&ProfileCollection::default())?;
            fs::write(&path, initial).context("failed to initialize profile store")?;
        }

        Ok(Self { path })
    }

    pub fn save_profile(&self, profile: TvProfile) -> anyhow::Result<()> {
        let mut collection = self.read_collection()?;
        if let Some(existing) = collection
            .profiles
            .iter_mut()
            .find(|current| current.name == profile.name)
        {
            *existing = profile;
        } else {
            collection.profiles.push(profile);
        }

        self.write_collection(&collection)
    }

    pub fn load_profile(&self, name: &str) -> anyhow::Result<Option<TvProfile>> {
        let collection = self.read_collection()?;
        Ok(collection
            .profiles
            .into_iter()
            .find(|profile| profile.name == name))
    }

    fn read_collection(&self) -> anyhow::Result<ProfileCollection> {
        let content = fs::read_to_string(&self.path).context("failed to read profile store")?;
        let parsed = serde_json::from_str(&content).context("failed to parse profiles.json")?;
        Ok(parsed)
    }

    fn write_collection(&self, collection: &ProfileCollection) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(collection)?;
        fs::write(&self.path, content).context("failed to write profile store")?;
        Ok(())
    }
}
