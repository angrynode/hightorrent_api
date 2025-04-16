use crate::ApiError;
use hightorrent::{SingleTarget, Torrent, TorrentContent, TorrentList, Tracker};

mod add;
pub use add::*;

#[async_trait]
pub trait Api: Send + Sync + for<'a> ApiAdd<'a> {
    // Build Api client
    async fn login(host: &str, user: &str, password: &str) -> Result<Self, ApiError>
    where
        Self: Sized;

    // Configuration values used to bootstrap API
    fn host(&self) -> String;
    fn user(&self) -> String;
    fn password(&self) -> String;

    // Torrent information
    async fn list(&self) -> Result<TorrentList, ApiError>;
    async fn get(&self, hash: &SingleTarget) -> Result<Option<Torrent>, ApiError>;
    async fn remove(&self, hash: &SingleTarget, delete_files: bool) -> Result<(), ApiError>;

    // Tracker manipulation
    // TODO: change to Result<Option<Vec<Tracker>> to express torrent not found
    async fn get_trackers(&self, hash: &SingleTarget) -> Result<Vec<Tracker>, ApiError>;
    async fn add_tracker(&self, hash: &SingleTarget, tracker: &str) -> Result<(), ApiError>;
    async fn remove_tracker(&self, hash: &SingleTarget, tracker: &str) -> Result<(), ApiError>;

    async fn get_files(&self, hash: &SingleTarget) -> Result<Vec<TorrentContent>, ApiError>;
}
