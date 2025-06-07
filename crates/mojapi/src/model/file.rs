use super::*;

#[derive(Deserialize, Debug)]
pub struct MojapiFile {
    #[serde(flatten)]
    pub base: BaseMojapiFile,
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct BaseMojapiFile {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct MojapiArtifact {
    #[serde(flatten)]
    pub base: MojapiFile,
    pub path: String,
}
