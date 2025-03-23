//! hightorrent_api provides clients for various Torrenting software APIs. Only [QBittorrent](https://qbittorrent.org/) is supported at the moment.
//!
//!**Note that hightorrent_api is not a torrenting library. It will not provide any utilities
//!for querying the DHT and/or downloading torrents. It is merely an interface with actual Torrent clients.**
//!
//! It provides the [Api] trait which various backends can implement, as well as
//! the [ApiError] struct to represent their error cases. By default, it is
//! built with the qbittorrent feature flag, providing [QBittorrentClient] API client.
//!
//! ```no_run
//! use hightorrent_api::{Api, QBittorrentClient};
//!
//! # async fn run() -> Result<(), hightorrent_api::ApiError> {
//! let client = QBittorrentClient::login(
//!   "http://localhost:8080",
//!   "admin",
//!   "adminadmin",
//! ).await?;
//!
//! for torrent in client.list().await? {
//!   println!("Torrent: {}", &torrent.name);
//! }
//! # Ok(())
//! # }
//! ```
//! # Supported backends
//!
//! - [x] QBittorrent
//! - [ ] Transmission
//!
//! ## QBittorrent notes
//!
//! The QBittorrent API exists, but is fragile...
//!
//! - sometimes returns JSON, sometimes plaintext
//! - may return HTTP 200 "Fails", or 400 "Bad Request"
//! - does not return the same information in list/get endpoints (issue [#18188](https://github.com/qbittorrent/qBittorrent/issues/18188))
//! - behaves unexpectedly with v2/hybrid hashes (issue [#18185](https://github.com/qbittorrent/qBittorrent/issues/18185))
//! - [sometimes changes methods](https://github.com/qbittorrent/qBittorrent/issues/18097#issuecomment-1336194151) on endpoints without bumping the API version to a new major (semantic versioning)
//!
//! Bittorrent v2 is only supported since v4.4.0 release (January 6th 2022).
//!
//! # Supported features
//!
//! - [x] List torrents
//! - [x] Get torrent detailed information
//! - [x] List, add, and remove trackers to a torrent
//! - [x] Remove torrents
//! - [x] Add torrents by magnet link or torrent file
//!
//! # Interacting with a torrent
//!
//! When interacting with a torrent, you need to use a [SingleTarget](hightorrent::SingleTarget) instance. This may be produced from a parsed [InfoHash](hightorrent::InfoHash) or from a stringy hash. This SingleTarget may be a full infohash (v1/v2) or a TorrentID (truncated v2 hash or complete v1 hash).
//!
//! ```no_run
//! # use hightorrent_api::QBittorrentClient;
//! # use std::str::FromStr;
//! use hightorrent::SingleTarget;
//! use hightorrent_api::Api;
//!
//! # async fn run() -> Result<(), hightorrent_api::ApiError> {
//! # let client = QBittorrentClient::login(
//! #   "http://localhost:8080",
//! #   "admin",
//! #   "adminadmin",
//! # ).await?;
//!
//! let target = SingleTarget::from_str("E74C8AEB6F23A0BAEB6563CCF83E52B7094DB18E").unwrap();
//! if let Some(torrent) = client.get(&target).await? {
//!   println!("{}", &torrent.name);
//! }
//! # Ok(())
//! # }
//! ```

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate snafu;

pub mod api;
pub use api::Api;

pub mod api_error;
pub use api_error::ApiError;

#[cfg(feature = "qbittorrent")]
pub mod qbittorrent;
#[cfg(feature = "qbittorrent")]
pub use qbittorrent::QBittorrentClient;
