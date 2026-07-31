use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stream {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "ytId", skip_serializing_if = "Option::is_none")]
    pub yt_id: Option<String>,
    #[serde(rename = "infoHash", skip_serializing_if = "Option::is_none")]
    pub info_hash: Option<String>,
    #[serde(rename = "fileIdx", skip_serializing_if = "Option::is_none")]
    pub file_idx: Option<u32>,
    #[serde(rename = "externalUrl", skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamResponse {
    pub streams: Vec<Stream>,
}
