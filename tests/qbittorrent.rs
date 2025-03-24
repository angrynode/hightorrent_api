use hightorrent::SingleTarget;
use hightorrent_api::{Api, ApiError, QBittorrentClient};
use tokio::sync::OnceCell;

use std::sync::{Mutex, MutexGuard};

// We wrap the API client in a mutex, and ensure we only use one client.
// We don't have hundreds of torrent files to test with,
// so we need to make sure operations are not executed in parallel.
static LOCK: OnceCell<Mutex<QBittorrentClient>> = OnceCell::const_new();

static V2_MAGNET: &str = "magnet:?xt=urn:btmh:1220caf1e1c30e81cb361b9ee167c4aa64228a7fa4fa9f6105232b28ad099f3a302e&dn=bittorrent-v2-test";
static V2_TORRENT: &[u8] = include_bytes!("bittorrent-v2-test.torrent");
static V2_V2HASH: &str = "caf1e1c30e81cb361b9ee167c4aa64228a7fa4fa9f6105232b28ad099f3a302e";
static V2_ID: &str = "caf1e1c30e81cb361b9ee167c4aa64228a7fa4fa";
static V2_NAME: &str = "bittorrent-v2-test";

static HYBRID_MAGNET: &str = "magnet:?xt=urn:btih:631a31dd0a46257d5078c0dee4e66e26f73e42ac&xt=urn:btmh:1220d8dd32ac93357c368556af3ac1d95c9d76bd0dff6fa9833ecdac3d53134efabb&dn=bittorrent-v1-v2-test";
static HYBRID_TORRENT: &[u8] = include_bytes!("bittorrent-v2-hybrid-test.torrent");
static HYBRID_V2HASH: &str = "d8dd32ac93357c368556af3ac1d95c9d76bd0dff6fa9833ecdac3d53134efabb";
static HYBRID_V1HASH: &str = "631a31dd0a46257d5078c0dee4e66e26f73e42ac";
static HYBRID_ID: &str = "d8dd32ac93357c368556af3ac1d95c9d76bd0dff";
static HYBRID_NAME: &str = "bittorrent-v1-v2-hybrid-test";

static V1_MAGNET: &str = "magnet:?xt=urn:btih:2c6e17017f6bb87125b2ba98c56a67f8ffe7e02c&dn=tails-amd64-5.6-img&tr=udp%3a%2f%2ftracker.torrent.eu.org%3a451&tr=udp%3a%2f%2ftracker.coppersurfer.tk%3a6969";
static V1_TORRENT: &[u8] = include_bytes!("tails-amd64-5.6.img.torrent");
static V1_V1HASH: &str = "2c6e17017f6bb87125b2ba98c56a67f8ffe7e02c";
static V1_ID: &str = "2c6e17017f6bb87125b2ba98c56a67f8ffe7e02c";
static V1_NAME: &str = "tails-amd64-5.6.img";


async fn client() -> MutexGuard<'static, QBittorrentClient> {
    LOCK.get_or_init(|| async {
        Mutex::new(QBittorrentClient::login("http://localhost:8080", "admin", "adminadmin").await.unwrap())
    }).await.lock().unwrap()
}

#[tokio::test]
async fn list() -> Result<(), ApiError> {
    let api = client().await;
    api.list().await?;
    Ok(())
}

#[tokio::test]
async fn magnet_v1() -> Result<(), ApiError> {
    let api = client().await;
    let target = SingleTarget::new(V1_V1HASH).unwrap();

    // Check torrent does not exist
    let list = api.list().await?;
    let entry = list.get(&target);
    assert!(entry.is_none());

    // Add torrent
    api.add().magnet(V1_MAGNET).paused(true).send().await?;

    // Check torrent does exist now
    let list = api.list().await?;
    let entry = list.get(&target);
    assert!(entry.is_some());

    // Remove torrent
    api.remove(&target, true).await?;

    // Check torrent does not exist anymore
    let list = api.list().await?;
    let entry = list.get(&target);
    assert!(entry.is_none());

    Ok(())
}
