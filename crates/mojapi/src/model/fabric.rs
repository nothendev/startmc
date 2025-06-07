use super::*;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FabricVerisonGameLoader {
    pub loader: FabricVersion,
    pub intermediary: FabricVersion,
    pub launcher_meta: FabricLauncherMeta,
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
    pub min_java_version: u8,
    pub libraries: FabricLibraries,
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

impl FabricLibrary {
    pub fn get_path(&self) -> String {
        let temp = self.name.split(':').collect::<Vec<_>>();
        let [.., name, version] = temp.as_slice() else {
            panic!("invalid library name: {:?}", self.name);
        };
        format!(
            "{}/{name}-{version}.jar",
            self.name
                .split(':')
                .map(|it| it.split('.').collect::<Vec<_>>().join("/"))
                .collect::<Vec<_>>()
                .join("/")
        )
    }
    pub fn get_url(&self) -> String {
        format!(
            "{}/{}",
            self.url, // always = https://maven.fabricmc.net/ (note the trailing slash)
            self.get_path()
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct FabricMainClasses {
    pub client: String,
    pub server: String,
}
