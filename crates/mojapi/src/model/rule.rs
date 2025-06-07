use std::collections::HashMap;

use super::*;

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub action: RuleAction,
    pub os: Option<RuleOs>,
    #[serde(default, deserialize_with = "deserialize_features")]
    pub features: Vec<String>
}

fn deserialize_features<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error> where D: serde::Deserializer<'de> {
    let features = Option::<HashMap<String, bool>>::deserialize(deserializer)?;
    if let Some(features) = features {
        Ok(features.into_iter().filter(|(_, v)| *v).map(|(k, _)| k).collect())
    } else {
        Ok(Vec::new())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow
}

#[derive(Deserialize, Debug)]
pub struct RuleOs {
    pub name: RuleOsKind
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RuleOsKind {
    Windows,
    Osx,
    Linux,
}
