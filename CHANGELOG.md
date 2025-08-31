# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## UNRELEASED (YYYY-MM-DD)

### Added

- `QBittorrentClient::qbittorrent_version` returns the qbittorrent daemon version

## Version 0.2.1 (2025-08-28)

This is a minor release only updating the docs, to specify the qBittorrent API versions supported.

## Version 0.2.0 (2025-08-27)

This is a small release focusing on listing torrent files from API, and supporting the latest QBittorrent releases.

### Added

- `QBittorrentClient` now implements `Debug`
- `QBittorrentClient::get_files` lists files in given torrent (only Bittorrent v1 supported at the moment)
- the hightorrent crate is now directly re-exported to avoid version mismatch

### Changed

- **Breaking change:** QBittorrent API v2.12 is now the minimum supported API version, despite being still
  undocumented upstream at the time of writing ([upstream pull request](https://github.com/qbittorrent/wiki/pull/29))

## Version 0.1.0 (2025-03-23)

### Added

- Initial release
