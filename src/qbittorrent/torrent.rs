use hightorrent::{
    InfoHash, ToTorrent, ToTorrentContent, Torrent, TorrentContent, TorrentID, Tracker,
    TrackerError, TryIntoTracker,
};
use serde::{Deserialize, Deserializer, Serialize};

use std::path::PathBuf;

/// Deserializes from the 'info' endpoint of QBittorrent API
/// [See QBittorrent API docs](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#get-torrent-list)
#[derive(Clone, Debug, Deserialize)]
pub struct QBittorrentTorrent {
    pub name: String,
    #[serde(rename = "save_path")]
    pub path: String,
    #[serde(rename = "added_on")]
    pub date_start: i64,
    #[serde(rename = "completion_on")]
    pub date_end: i64,
    pub progress: f32,
    #[serde(rename = "total_size")]
    pub size: i64,
    pub state: String,
    #[serde(deserialize_with = "load_tags")]
    pub tags: Vec<String>,
    #[serde(rename = "hash")]
    pub id: TorrentID,
    pub infohash_v1: String,
    pub infohash_v2: String,
}

impl ToTorrent for QBittorrentTorrent {
    fn to_torrent(&self) -> Torrent {
        Torrent {
            name: self.name.to_string(),
            path: self.path.to_string(),
            date_start: self.date_start,
            date_end: self.date_end,
            progress: (self.progress * 100.0) as u8,
            size: self.size,
            state: self.state.to_string(),
            tags: self.tags.clone(),
            id: self.id.clone(),
            hash: self.hash(),
        }
    }
}

impl QBittorrentTorrent {
    fn hash(&self) -> InfoHash {
        match (&self.infohash_v1.is_empty(), &self.infohash_v2.is_empty()) {
            (true, true) => {
                panic!("API returned torrent without v1/v2 hash!");
            }
            (true, false) => InfoHash::V2(self.infohash_v2.to_string()),
            (false, true) => InfoHash::V1(self.infohash_v1.to_string()),
            (false, false) => {
                InfoHash::Hybrid((self.infohash_v1.to_string(), self.infohash_v2.to_string()))
            }
        }
    }
}

fn load_tags<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .map(|s| s.split(',').map(|tag| tag.trim().to_string()).collect())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QBittorrentTracker {
    pub url: String,
    pub status: usize,
    pub msg: String,
}

impl TryIntoTracker for QBittorrentTracker {
    fn try_into_tracker(&self) -> Result<Tracker, TrackerError> {
        Tracker::new(&self.url)
    }
}

impl PartialEq for QBittorrentTracker {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QBittorrentTorrentContent {
    #[serde(rename = "name")]
    pub path: PathBuf,
    pub size: u32,
    pub progress: f32,
    #[serde(default)]
    pub is_seed: bool,
}

impl ToTorrentContent for QBittorrentTorrentContent {
    fn to_torrent_content(&self) -> TorrentContent {
        TorrentContent {
            path: self.path.clone(),
            size: self.size as u64,
        }
    }
}
