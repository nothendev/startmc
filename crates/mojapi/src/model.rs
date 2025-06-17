use serde::Deserialize;

mod argument;
mod assets;
mod fabric;
mod file;
mod rule;
mod version;
pub use argument::*;
pub use assets::*;
pub use fabric::*;
pub use file::*;
pub use rule::*;
pub use version::*;

pub const VERSION_MANIFEST_V2: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
#[derive(Deserialize, Debug)]
pub struct VersionManifestV2 {
    pub latest: VersionManifestLatest,
    pub versions: Vec<VersionManifestVersion>,
}

impl VersionManifestV2 {
    pub async fn fetch(rq: &reqwest::Client) -> Result<Self, reqwest::Error> {
        rq.get(VERSION_MANIFEST_V2)
            .send()
            .await?
            .json()
            .await
    }
}

#[derive(Deserialize, Debug)]
pub struct VersionManifestLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifestVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: VersionType,
    pub url: String,
}

impl VersionManifestVersion {
    pub async fn fetch(&self, rq: &reqwest::Client) -> Result<VersionPackage, reqwest::Error> {
        rq.get(&self.url).send().await?.json().await
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}
