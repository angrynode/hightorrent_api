use std::boxed::Box;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)), visibility(pub))]
pub enum ApiError {
    #[snafu(display("Request to API backend failed:\n{source}"))]
    Http {
        source: Box<dyn std::error::Error + 'static + Send + Sync>,
    },
    #[snafu(display("Failed to parse response from API backend:\n{source}"))]
    Deserialization { source: serde_json::Error },
    #[snafu(display("Invalid login on API backend {host} with username {user}"))]
    InvalidLogin { host: String, user: String },
    #[snafu(display("API backend rejected the torrent as invalid"))]
    RejectedTorrent,
    #[snafu(display("Torrent hash not found {hash}"))]
    MissingTorrent { hash: String },
    #[snafu(display("Failed to read torrent file from path {}:\n{source}", path.display()))]
    FailedReadTorrent {
        source: std::io::Error,
        path: std::path::PathBuf,
    },
    #[snafu(display("Invalid infohash: {source}"))]
    InfoHash { source: hightorrent::InfoHashError },
}
