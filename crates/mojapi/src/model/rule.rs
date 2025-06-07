use std::collections::HashMap;

use super::*;

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub action: RuleAction,
    pub os: Option<HashMap<String, String>>,
    pub features: Option<HashMap<String, bool>>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow
}
