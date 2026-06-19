use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum AliasEntry {
    Plain(String),
    Full(Alias),
}

impl AliasEntry {
    pub fn into_alias(self) -> Alias {
        match self {
            AliasEntry::Plain(cmd) => Alias { cmd, shell: None, desc: None },
            AliasEntry::Full(a) => a,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Alias {
    pub cmd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
}

impl Alias {
    #[allow(dead_code)]
    pub fn new(cmd: impl Into<String>) -> Self {
        Alias { cmd: cmd.into(), shell: None, desc: None }
    }
}
