use std::collections::HashMap;

use super::*;

#[derive(Deserialize, Debug)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize, Debug)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}
