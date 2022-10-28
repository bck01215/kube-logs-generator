use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Condition {
    pub key: String,
    pub value: String,
    pub matching: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Pods {
    pub kind: String,
    pub api_version: String,
    pub items: Vec<Pod>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]

pub struct Pod {
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]

pub struct Metadata {
    pub name: String,
    pub namespace: String,
    pub self_link: String,
    pub creation_timestamp: String,
    pub labels: Option<HashMap<String, String>>,
    pub annotations: Option<HashMap<String, String>>,
}
