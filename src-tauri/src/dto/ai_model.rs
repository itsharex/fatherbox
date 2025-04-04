use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateBody {
    pub name:  String,
    pub source_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBody {
    pub id: String,
    pub name:  String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EnableBody {
    pub id: String,
    pub enable:  bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncBody {
    pub id: String,
    pub sync:  bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListBody {
    pub source_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommonBody {
    pub id: String,
}