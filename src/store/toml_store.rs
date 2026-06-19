use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::model::AliasEntry;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RawStore {
    #[serde(default)]
    pub aliases: HashMap<String, AliasEntry>,
}
