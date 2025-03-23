use std::boxed::Box;
use std::path::{Path, PathBuf};

use crate::api_error::*;

#[async_trait]
/// ApiAdd is implemented by torrent API clients to add new torrents/magnets to the Bittorrent client
pub trait ApiAdd<'a>: Send + Sync {
    async fn api_add_send(&self, add: AddBuilder<'a, AddSource>) -> Result<(), ApiError>;
}

pub struct NoAddSource;
pub enum AddSource {
    MagnetStr(String),
    MagnetFile(PathBuf),
    TorrentFile(PathBuf),
}

impl AddSource {
    pub fn magnet(s: &str) -> AddSource {
        AddSource::MagnetStr(s.to_string())
    }

    pub fn magnet_file(p: &Path) -> AddSource {
        AddSource::MagnetFile(p.to_path_buf())
    }

    pub fn torrent_file(p: &Path) -> AddSource {
        AddSource::TorrentFile(p.to_path_buf())
    }
}

pub struct AddBuilder<'a, T> {
    //api: &'a Box<dyn ApiAdd<'a>>,
    api: &'a dyn ApiAdd<'a>,
    #[allow(dead_code)]
    pub source: T,
    pub save_path: Option<String>,
    pub paused: Option<bool>,
    pub tags: Option<Vec<String>>,
}

impl<'a> AddBuilder<'a, NoAddSource> {
    //pub fn new(api: &'a dyn ApiAdd<'a>) -> Self {
    // pub fn new(api: Box<&'a dyn ApiAdd<'a>>) -> Self {
    pub fn new(api: &'a dyn ApiAdd<'a>) -> Self {
        AddBuilder {
            // api: Box::new(api),
            api,
            source: NoAddSource,
            save_path: None,
            paused: None,
            tags: None,
        }
    }
}

impl<'a> AddBuilder<'a, NoAddSource> {
    pub fn magnet(self, s: &'a str) -> AddBuilder<'a, AddSource> {
        let Self {
            api,
            save_path,
            paused,
            tags,
            ..
        } = self;
        AddBuilder {
            api,
            source: AddSource::magnet(s),
            save_path,
            paused,
            tags,
        }
    }

    pub fn magnet_file(self, s: &Path) -> AddBuilder<'a, AddSource> {
        let Self {
            api,
            save_path,
            paused,
            tags,
            ..
        } = self;
        AddBuilder {
            api,
            source: AddSource::magnet_file(s),
            save_path,
            paused,
            tags,
        }
    }

    pub fn torrent_file(self, s: &Path) -> AddBuilder<'a, AddSource> {
        let Self {
            api,
            save_path,
            paused,
            tags,
            ..
        } = self;
        AddBuilder {
            api,
            source: AddSource::torrent_file(s),
            save_path,
            paused,
            tags,
        }
    }
}

impl<'a, S> AddBuilder<'a, S> {
    pub fn paused(mut self, p: bool) -> AddBuilder<'a, S> {
        self.paused = Some(p);
        self
    }

    pub fn tags(mut self, t: Vec<String>) -> AddBuilder<'a, S> {
        self.tags = Some(t);
        self
    }

    pub fn save_path(mut self, s: &str) -> AddBuilder<'a, S> {
        self.save_path = Some(s.to_string());
        self
    }
}

impl AddBuilder<'_, AddSource> {
    pub async fn send(self) -> Result<(), ApiError> {
        self.api.api_add_send(self).await
    }
}
