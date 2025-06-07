use serde::Deserialize;

mod assets;
mod argument;
mod fabric;
mod file;
mod rule;
mod version;
pub use assets::*;
pub use argument::*;
pub use fabric::*;
pub use file::*;
pub use rule::*;
pub use version::*;

/// `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
#[derive(Deserialize, Debug)]
pub struct VersionManifestV2 {
    pub latest: VersionManifestLatest,
    pub versions: Vec<VersionManifestVersion>,
}

impl VersionManifestV2 {
    pub async fn fetch(rq: &reqwest::Client) -> Result<Self, reqwest::Error> {
        rq.get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
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
