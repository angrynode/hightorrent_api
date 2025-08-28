//! hightorrent_api provides clients for various Torrenting software APIs. Only [QBittorrent](https://qbittorrent.org/) is supported at the moment.
//!
//!**Note that hightorrent_api is not a torrenting library. It will not provide any utilities
//!for querying the DHT and/or downloading torrents. It is merely an interface with actual Torrent clients.**
//!
//! It provides the [Api](Api) trait which various backends can implement, as well as
//! the [ApiError](ApiError) struct to represent their error cases. By default, it is
//! built with the qbittorrent feature flag, providing [QBittorrentClient](QBittorrentClient) API client.
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
//! - [x] QBittorrent (v5.0.x, v5.1.x)
//! - [ ] Transmission
//!
//! ## qBittorrent notes
//!
//! Only the following qBittorrent releases are supported and tested in CI:
//!
//! - v5.1.2 (2 July 2025)
//! - v5.0.5 (13 Aprli 2025)
//!
//! qBittorrent v4.6.x is known not to work properly due to the ever changing API. Checking support in newer releases only requires changing the [CI configuration](.github/workflows/ci.yml) (pull requests welcome). We will not add support for older qBittorrent releases (Debian 13 Trixies packages qBittorrent v5.1.x), but contributions for this are welcome. Bittorrent v2 is only supported since v4.4.0 (6 January 2022) so it's unlikely we'll ever support an older release.
//!
//! The qBittorrent API is surprising (to say the least):
//!
//! - some responses are JSON, some are plaintext
//! - an error may be HTTP 200 with plaintext "Fails", or HTTP 400 "Bad Request"
//! - client requests [may not be chunked](https://github.com/qbittorrent/qBittorrent/issues/17353), despite being the default when uploading files in many HTTP clients
//! - does not return the same information in list/get endpoints (issue [#18188](https://github.com/qbittorrent/qBittorrent/issues/18188))
//! - behaves unexpectedly with v2/hybrid hashes (issue [#18185](https://github.com/qbittorrent/qBittorrent/issues/18185))
//! - [sometimes changes methods](https://github.com/qbittorrent/qBittorrent/issues/18097#issuecomment-1336194151) on endpoints without bumping the API version to a new major (semantic versioning)
//! - may change form field names in API [without updating the docs](https://github.com/qbittorrent/qBittorrent/pull/20532) ([upstream docs PR](https://github.com/qbittorrent/wiki/pull/29))
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

#![allow(rustdoc::redundant_explicit_links)]

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate snafu;

// Reexpose hightorrent to avoid version mismatches
pub use hightorrent;

pub mod api;
pub use api::Api;

pub mod api_error;
pub use api_error::ApiError;

#[cfg(feature = "qbittorrent")]
pub mod qbittorrent;
#[cfg(feature = "qbittorrent")]
pub use qbittorrent::QBittorrentClient;
