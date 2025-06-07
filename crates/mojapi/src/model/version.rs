use super::*;

/// `https://piston-meta.mojang.com/v1/packages/<WHATEVER>/<VERSION>.json`
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionPackage {
    pub arguments: VersionArguments,
    pub asset_index: VersionAssetIndex,
    // pub assets: String, // == asset_index.id
    pub downloads: VersionDownloads,
    pub id: String,
    pub java_version: JavaVersion,
    pub libraries: Vec<VersionLibrary>,
    pub logging: VersionLogging,
    pub main_class: String,
    #[serde(rename = "type")]
    pub kind: VersionType,
}

#[derive(Deserialize, Debug)]
pub struct VersionArguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Deserialize, Debug)]
pub struct VersionLogging {
    pub client: VersionLoggingClient,
}

#[derive(Deserialize, Debug)]
pub struct VersionLoggingClient {
    pub argument: String,
    pub file: MojapiFile,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionAssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: u64,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct VersionDownloads {
    pub client: BaseMojapiFile,
    pub client_mappings: BaseMojapiFile,
    pub server: BaseMojapiFile,
    pub server_mappings: BaseMojapiFile,
}

#[derive(Deserialize, Debug)]
pub struct VersionLibrary {
    pub downloads: VersionLibraryDownloads,
    pub name: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Debug)]
pub struct VersionLibraryDownloads {
    pub artifact: MojapiArtifact,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u64,
}
