use hightorrent::{
    InfoHash, MultiTarget, SingleTarget, ToTorrent, ToTorrentContent, Torrent, TorrentContent,
    TorrentID, TorrentList, Tracker, TryIntoTracker,
};
use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::{Client, ClientBuilder, Response, StatusCode, Url};
use serde::de::DeserializeOwned;
use snafu::ResultExt;
use tokio::{fs::File, io::AsyncReadExt};

use std::borrow::Borrow;

use crate::{
    api::*,
    api_error::{ApiError as Error, *},
    qbittorrent::{QBittorrentTorrent, QBittorrentTorrentContent, QBittorrentTracker},
};

#[derive(Clone)]
pub struct QBittorrentClient {
    host: String,
    user: String,
    password: String,
    client: Client,
}

impl QBittorrentClient {
    /// Returns the URL to an endpoint without params
    pub fn _endpoint(&self, path: &str) -> Url {
        Url::parse(&format!("{}/api/v2/{}", self.host, path))
            .expect("PROGRAMMING ERROR: invalid api URL")
    }

    /// Returns the URL to an endpoint with custom query params
    pub fn _endpoint_params<I, K, V>(&self, endpoint: &str, args: I) -> Url
    where
        I: IntoIterator,
        K: AsRef<str>,
        V: AsRef<str>,
        <I as IntoIterator>::Item: Borrow<(K, V)>,
    {
        Url::parse_with_params(&format!("{}/api/v2/{}", self.host, endpoint), args)
            .expect("PROGRAMMING ERROR: invalid api URL")
    }

    pub async fn _post_multipart(&self, endpoint: Url, form: Form) -> Result<Response, Error> {
        self.keepalive().await?;
        self.client
            .post(endpoint)
            .multipart(form)
            .send()
            .await
            .boxed()
            .context(HttpError)
    }

    pub async fn _post(&self, endpoint: Url) -> Result<Response, Error> {
        self.keepalive().await?;
        self.client
            .post(endpoint)
            .send()
            .await
            .boxed()
            .context(HttpError)
    }

    pub async fn _get(&self, endpoint: Url) -> Result<Response, Error> {
        self.keepalive().await?;
        self.client
            .get(endpoint)
            .send()
            .await
            .boxed()
            .context(HttpError)
    }

    /// Keeps the current session alive even if QBittorrent restarted
    ///
    /// Called before making a "real" API call in other methods. Not ideal because it now takes
    /// two round-trips (3 when reconnecting), but better than failing because cookie expired.
    pub async fn keepalive(&self) -> Result<(), Error> {
        let res = self
            .client
            .get(self._endpoint("app/version"))
            .send()
            .await
            .boxed()
            .context(HttpError)?;
        if res.status() == StatusCode::FORBIDDEN {
            // We have been disconnected. Reconnect now!
            // TODO: check if reconnect was successful?
            self.reconnect().await?;
        }

        Ok(())
    }

    /// Triggers a reconnection to the QBittorrent API.
    ///
    // TODO: check if reconnect was successful?
    pub async fn reconnect(&self) -> Result<(), Error> {
        let form = Form::new()
            .text("username", self.user.to_string())
            .text("password", self.password.to_string());

        let _ = self
            .client
            .post(self._endpoint("auth/login"))
            .multipart(form)
            .send()
            .await
            .boxed()
            .context(HttpError)?;

        Ok(())
    }

    pub async fn _json<U: DeserializeOwned>(&self, res: Response) -> Result<U, Error> {
        let full = res.bytes().await.boxed().context(HttpError)?;
        serde_json::from_slice(&full).context(DeserializationError)
    }

    pub fn add(&self) -> AddBuilder<'_, NoAddSource> {
        AddBuilder::new(self)
    }

    pub async fn list_target(&self, target: &MultiTarget) -> Result<TorrentList, Error> {
        match target {
            MultiTarget::All => Ok(self.list().await?),
            MultiTarget::Hash(single_target) => {
                if let Some(t) = self.get(single_target).await? {
                    Ok(TorrentList::from_vec(vec![t]))
                } else {
                    Err(Error::MissingTorrent {
                        hash: single_target.to_string(),
                    })
                }
            }
        }
    }

    /// Returns a TorrentID for the requested SingleTarget
    ///
    /// TODO: This is a workaround until QBittorrent finally supports v1 hybrid hashes and full v2
    /// hashes in its API... This has HUGE performance implications.
    /// See [this issue ](https://github.com/qbittorrent/qBittorrent/issues/18185) for more info.
    pub async fn id(&self, target: &SingleTarget) -> Result<Option<TorrentID>, Error> {
        Ok(self.get(target).await?.map(|torrent| torrent.id.clone()))
    }

    /// Returns a list of torrents as a vector of a custom type
    pub async fn list_as<T: DeserializeOwned + AsRef<InfoHash>>(&self) -> Result<Vec<T>, Error> {
        let res = self._get(self._endpoint("torrents/info")).await?;
        self._json(res).await
    }

    /// Returns a single torrent as a custom type
    pub async fn get_as<T: DeserializeOwned + AsRef<InfoHash>>(
        &self,
        target: &SingleTarget,
    ) -> Result<Option<T>, Error> {
        self.list_as::<T>().await.map(|list| {
            list.into_iter()
                .find(|torrent| target.matches_hash(torrent.as_ref()))
        })
    }

    pub async fn set_location(&self, target: &SingleTarget, location: &str) -> Result<(), Error> {
        if let Some(id) = self.id(target).await? {
            let form = Form::new()
                .text("hashes", id.to_string())
                .text("location", location.to_string());
            self._post_multipart(self._endpoint("torrents/setLocation"), form)
                .await?;

            Ok(())
        } else {
            Err(Error::MissingTorrent {
                hash: target.to_string(),
            })
        }
    }
}

#[async_trait]
impl Api for QBittorrentClient {
    fn host(&self) -> String {
        self.host.to_string()
    }

    fn user(&self) -> String {
        self.user.to_string()
    }

    fn password(&self) -> String {
        self.user.to_string()
    }

    async fn login(host: &str, user: &str, password: &str) -> Result<Self, Error> {
        let client = ClientBuilder::new()
            .cookie_store(true)
            .build()
            .boxed()
            .context(HttpError)?;

        let form = Form::new()
            .text("username", user.to_string())
            .text("password", password.to_string());

        let res = client
            .post(format!("{}/api/v2/auth/login", host))
            .multipart(form)
            .send()
            .await
            .boxed()
            .context(HttpError)?;

        if res.headers().get("set-cookie").is_some() {
            Ok(Self {
                host: host.to_string(),
                user: user.to_string(),
                password: password.to_string(),
                client,
            })
        } else {
            Err(Error::InvalidLogin {
                host: host.to_string(),
                user: user.to_string(),
            })
        }
    }

    async fn list(&self) -> Result<TorrentList, Error> {
        let res = self._get(self._endpoint("torrents/info")).await?;
        let concrete: Vec<QBittorrentTorrent> = self._json(res).await?;
        Ok(concrete.iter().map(|t| t.to_torrent()).collect())
    }

    async fn get(&self, target: &SingleTarget) -> Result<Option<Torrent>, Error> {
        Ok(self.list().await?.get(target))
    }

    async fn remove(&self, target: &SingleTarget, delete_files: bool) -> Result<(), Error> {
        if let Some(id) = self.id(target).await? {
            let mut form = Form::new();
            form = form.text("hashes", id.as_str().to_string());
            form = form.text("deleteFiles", delete_files.to_string());

            self._post_multipart(self._endpoint("torrents/delete"), form)
                .await?;
        }

        Ok(())
    }

    async fn get_trackers(&self, target: &SingleTarget) -> Result<Vec<Tracker>, Error> {
        let truncated = target.truncated();
        let res = self
            ._post(self._endpoint_params("torrents/trackers", vec![("hash", truncated)]))
            .await?;

        let trackers = self
            ._json::<Vec<QBittorrentTracker>>(res)
            .await?
            .into_iter()
            .filter_map(|tracker| {
                // Dismiss non-tracker types (DHT/PEX/LSD)
                tracker.try_into_tracker().ok()
            })
            .collect();

        Ok(trackers)
    }

    async fn remove_tracker(&self, target: &SingleTarget, tracker: &str) -> Result<(), Error> {
        //.context(InfoHashError as <ToSingleTarget::Error>)?;
        let truncated = target.truncated();
        let res = self
            ._post(self._endpoint_params(
                "torrent/removeTrackers",
                vec![("hash", truncated), ("urls", tracker)],
            ))
            .await?;

        match res.status() {
            StatusCode::NOT_FOUND => Err(Error::MissingTorrent {
                hash: target.to_string(),
            }),
            StatusCode::CONFLICT => {
                // Tracker URL was not found
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn add_tracker(&self, target: &SingleTarget, tracker: &str) -> Result<(), Error> {
        let truncated = target.truncated();
        let res = self
            ._post(self._endpoint_params(
                "torrent/addTrackers",
                vec![("hash", truncated), ("urls", tracker)],
            ))
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Error::MissingTorrent {
                hash: target.as_str().to_string(),
            })
        }
    }

    async fn get_files(&self, target: &SingleTarget) -> Result<Vec<TorrentContent>, Error> {
        let Some(id) = self.id(target).await? else {
            return Err(Error::MissingTorrent {
                hash: target.as_str().to_string(),
            });
        };

        let mut form = Form::new();
        form = form.text("hash", id.as_str().to_string());
        let res = self
            ._post_multipart(self._endpoint("torrents/files"), form)
            .await?;

        if res.status().is_success() {
            let concrete: Vec<QBittorrentTorrentContent> = self._json(res).await?;
            Ok(concrete.iter().map(|t| t.to_torrent_content()).collect())
        } else {
            Err(Error::MissingTorrent {
                hash: target.as_str().to_string(),
            })
        }
    }
}

#[async_trait]
impl<'a> ApiAdd<'a> for QBittorrentClient {
    async fn api_add_send(&self, add: AddBuilder<'a, AddSource>) -> Result<(), ApiError> {
        match add.source {
            AddSource::MagnetStr(url) => {
                let mut form = Form::new();

                if let Some(save_path) = add.save_path {
                    form = form.text("savepath", save_path);
                }

                if let Some(paused) = add.paused {
                    form = form.text("stopped", paused.to_string());
                }

                if let Some(tags) = add.tags {
                    form = form.text("tags", tags.join(","));
                }
                form = form.text("urls", url);
                let res = self
                    ._post_multipart(self._endpoint("torrents/add"), form)
                    .await?;
                add_success(res).await
            }
            AddSource::MagnetFile(path) => {
                let mut form = Form::new();

                if let Some(save_path) = add.save_path {
                    form = form.text("savepath", save_path);
                }

                if let Some(paused) = add.paused {
                    form = form.text("stopped", paused.to_string());
                }

                if let Some(tags) = add.tags {
                    form = form.text("tags", tags.join(","));
                }
                let content = std::fs::read_to_string(&path).context(FailedReadTorrentError {
                    path: path.to_path_buf(),
                })?;
                form = form.text("urls", content);
                let res = self
                    ._post_multipart(self._endpoint("torrents/add"), form)
                    .await?;
                add_success(res).await
            }
            AddSource::TorrentFile(path) => {
                // Form.file() is not supported in async reqwest::multipart::Form
                // Snippet copied from https://github.com/seanmonstar/reqwest/issues/646
                let file_name = path
                    .file_name()
                    .map(|val| val.to_string_lossy().to_string())
                    .unwrap_or_default();
                let mut file = File::open(&path).await.context(FailedReadTorrentError {
                    path: path.to_path_buf(),
                })?;
                let mut file_bytes: Vec<u8> = Vec::new();
                file.read_to_end(&mut file_bytes)
                    .await
                    .context(FailedReadTorrentError {
                        path: path.to_path_buf(),
                    })?;
                //let reader = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));

                let mut form = Form::new()
                    //.part("torrents", Part::stream(reader).file_name(file_name));
                    .part("torrents", Part::bytes(file_bytes).file_name(file_name));

                if let Some(paused) = add.paused {
                    form = form.text("stopped", paused.to_string());
                }

                if let Some(tags) = add.tags {
                    form = form.text("tags", tags.join(","));
                }

                if let Some(save_path) = add.save_path {
                    form = form.text("savepath", save_path);
                }

                let res = self
                    ._post_multipart(self._endpoint("torrents/add"), form)
                    .await?;
                add_success(res).await
            }
        }
    }
}

async fn add_success(res: reqwest::Response) -> Result<(), Error> {
    if res.status().is_success() {
        if res
            .text()
            .await
            .boxed()
            .context(HttpError)?
            .starts_with("Fail")
        {
            Err(Error::RejectedTorrent)
        } else {
            Ok(())
        }
    } else {
        Err(Error::RejectedTorrent)
    }
}
