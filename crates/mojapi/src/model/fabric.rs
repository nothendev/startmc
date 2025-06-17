use super::*;

pub type FabricVersionsLoader = Vec<LoaderFabricVersion>;
pub type FabricVersionsGame = Vec<FabricVersionGame>;

#[derive(Deserialize, Debug)]
pub struct FabricVersionGame {
    pub loader: LoaderFabricVersion,
    pub intermediary: FabricVersion,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FabricVerisonGameLoader {
    pub loader: FabricVersion,
    pub intermediary: FabricVersion,
    pub launcher_meta: FabricLauncherMeta,
}

#[derive(Deserialize, Debug)]
pub struct LoaderFabricVersion {
    #[serde(flatten)]
    pub version: FabricVersion,
    pub separator: String, // usually `+build.`
    pub build: u32
}

#[derive(Deserialize, Debug)]
pub struct FabricVersion {
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Deserialize, Debug)]
pub struct FabricLauncherMeta {
    pub version: u8,
    // for some god forsaken reason this is snake_case and mainClass is camelCase...
    pub min_java_version: u8,
    pub libraries: FabricLibraries,
    #[serde(rename = "mainClass")]
    pub main_class: FabricMainClasses,
}

#[derive(Deserialize, Debug)]
pub struct FabricLibraries {
    pub common: Vec<FabricLibrary>,
    pub client: Vec<FabricLibrary>,
    pub server: Vec<FabricLibrary>,
}

#[derive(Deserialize, Debug)]
pub struct FabricLibrary {
    pub name: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
}

#[derive(Deserialize, Debug)]
pub struct FabricMainClasses {
    pub client: String,
    pub server: String,
}

pub const FABRIC_MAVEN: &str = "https://maven.fabricmc.net/";
pub const FABRIC_VERSIONS_LOADER: &str = "https://meta.fabricmc.net/v2/versions/loader";

/// Concat with `/GAMEVERSION/LOADERVERSION` to get [`FabricVersionsGameLoader`]
///
/// Or concat with `/GAMEVERSION` to get [`FabricVersionsGame`]
pub const FABRIC_VERSIONS_GAME: &str = "https://meta.fabricmc.net/v2/versions/loader";
