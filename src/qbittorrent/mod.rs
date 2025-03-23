mod api;
pub use api::QBittorrentClient;

mod torrent;
pub use torrent::{QBittorrentTorrent, QBittorrentTracker};
