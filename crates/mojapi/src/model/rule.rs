use std::collections::HashMap;

use super::*;

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub action: RuleAction,
    pub os: Option<HashMap<String, String>>,
    pub features: Option<HashMap<String, bool>>,
}

impl Rule {
    pub fn check(&self) -> bool {
        match self
            .os
            .as_ref()
            .and_then(|it| it.get("name"))
            .map(|s| s.as_str())
        {
            Some("linux") => cfg!(target_os = "linux"),
            Some("windows") => cfg!(target_os = "windows"),
            Some("osx") => cfg!(target_os = "macos"),
            _ => true,
        }
        // TODO: features
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
}
