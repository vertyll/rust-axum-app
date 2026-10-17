//! The local disk as a storage. It hands out the same kind of signed links as an
//! object store, pointing at this back-end's own `/uploads`, so the browser sends
//! and fetches the bytes the same way whichever storage holds them.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use futures_util::{Stream, StreamExt};
use hmac::{Hmac, KeyInit, Mac};
use jiff::{SignedDuration, Timestamp};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sha2::Sha256;
use tokio::io::AsyncWriteExt;

use super::{DOWNLOAD_LINK_TTL, UPLOAD_LINK_TTL, attachment, is_object_key};
use crate::files::application::{ObjectDownload, ObjectUpload, SignedLink};
use crate::files::domain::FilesError;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct LocalFileStorage {
    upload_dir: PathBuf,
    base_url: String,
    signing_key: SecretString,
}

/// The parameters of an upload link, signed together with the key.
#[derive(Debug, Deserialize)]
pub struct UploadParams {
    pub expires: i64,
    pub size: u64,
    #[serde(rename = "type")]
    pub mime_type: String,
    pub signature: String,
}

/// The parameters of a download link, signed together with the key.
#[derive(Debug, Deserialize)]
pub struct DownloadParams {
    pub expires: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub mime_type: String,
    pub signature: String,
}

impl LocalFileStorage {
    pub fn new(upload_dir: impl Into<PathBuf>, base_url: impl Into<String>, signing_key: SecretString) -> Self {
        Self {
            upload_dir: upload_dir.into(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            signing_key,
        }
    }

    pub fn upload_link(&self, upload: &ObjectUpload) -> Result<SignedLink, FilesError> {
        let expires_at = expiry(UPLOAD_LINK_TTL)?;
        let expires = expires_at.as_second();
        let signature = self.sign(&upload_message(&upload.key, expires, upload.size, &upload.mime_type));
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("expires", &expires.to_string())
            .append_pair("size", &upload.size.to_string())
            .append_pair("type", &upload.mime_type)
            .append_pair("signature", &signature)
            .finish();
        Ok(SignedLink {
            method: "PUT",
            url: format!("{}/{}?{query}", self.base_url, upload.key),
            headers: vec![("content-type".to_string(), upload.mime_type.clone())],
            expires_at,
        })
    }

    pub fn download_link(&self, download: &ObjectDownload) -> Result<SignedLink, FilesError> {
        let expires_at = expiry(DOWNLOAD_LINK_TTL)?;
        let expires = expires_at.as_second();
        let signature = self.sign(&download_message(
            &download.key,
            expires,
            &download.file_name,
            &download.mime_type,
        ));
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("expires", &expires.to_string())
            .append_pair("name", &download.file_name)
            .append_pair("type", &download.mime_type)
            .append_pair("signature", &signature)
            .finish();
        Ok(SignedLink {
            method: "GET",
            url: format!("{}/{}?{query}", self.base_url, download.key),
            headers: Vec::new(),
            expires_at,
        })
    }

    /// Checks that an upload link was handed out by this application for `key` and has not expired.
    ///
    /// # Errors
    ///
    /// `FilesError::InvalidLink` for a key, signature or expiry that does not hold.
    pub fn verify_upload(&self, key: &str, params: &UploadParams) -> Result<(), FilesError> {
        self.verify(
            key,
            params.expires,
            &upload_message(key, params.expires, params.size, &params.mime_type),
            &params.signature,
        )
    }

    /// Checks that a download link was handed out by this application for `key` and has not expired.
    ///
    /// # Errors
    ///
    /// `FilesError::InvalidLink` for a key, signature or expiry that does not hold.
    pub fn verify_download(&self, key: &str, params: &DownloadParams) -> Result<(), FilesError> {
        self.verify(
            key,
            params.expires,
            &download_message(key, params.expires, &params.name, &params.mime_type),
            &params.signature,
        )
    }

    /// Writes the bytes of an upload, which has to be exactly the size its link allowed;
    /// anything else leaves nothing behind.
    ///
    /// # Errors
    ///
    /// `FilesError::NotUploaded` for a body of another size, `FilesError::StorageFailure` when the disk fails.
    pub async fn write<S, E>(&self, key: &str, size: u64, mut body: S) -> Result<(), FilesError>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Unpin,
        E: std::fmt::Display,
    {
        tokio::fs::create_dir_all(&self.upload_dir)
            .await
            .map_err(|err| storage("create upload directory", &err))?;
        let path = self.path(key)?;
        let partial = path.with_extension("partial");
        let mut file = tokio::fs::File::create(&partial)
            .await
            .map_err(|err| storage("create file", &err))?;
        let mut written: u64 = 0;
        while let Some(chunk) = body.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(err) => {
                    tracing::warn!("upload body broke off: {err}");
                    discard(&partial).await;
                    return Err(FilesError::NotUploaded);
                }
            };
            written += chunk.len() as u64;
            if written > size {
                discard(&partial).await;
                return Err(FilesError::NotUploaded);
            }
            file.write_all(&chunk)
                .await
                .map_err(|err| storage("write file", &err))?;
        }
        if written != size {
            discard(&partial).await;
            return Err(FilesError::NotUploaded);
        }
        file.flush().await.map_err(|err| storage("write file", &err))?;
        tokio::fs::rename(&partial, &path)
            .await
            .map_err(|err| storage("store file", &err))
    }

    /// The stored file, opened for reading.
    ///
    /// # Errors
    ///
    /// `FilesError::NotFound` when nothing is stored under `key`.
    pub async fn open(&self, key: &str) -> Result<tokio::fs::File, FilesError> {
        match tokio::fs::File::open(self.path(key)?).await {
            Ok(file) => Ok(file),
            Err(err) if err.kind() == ErrorKind::NotFound => Err(FilesError::NotFound),
            Err(err) => Err(storage("open file", &err)),
        }
    }

    pub async fn object_size(&self, key: &str) -> Result<Option<u64>, FilesError> {
        match tokio::fs::metadata(self.path(key)?).await {
            Ok(metadata) => Ok(Some(metadata.len())),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(storage("read file size", &err)),
        }
    }

    pub async fn remove(&self, key: &str) -> Result<(), FilesError> {
        match tokio::fs::remove_file(self.path(key)?).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) => Err(storage("delete file", &err)),
        }
    }

    fn path(&self, key: &str) -> Result<PathBuf, FilesError> {
        if is_object_key(key) {
            Ok(self.upload_dir.join(key))
        } else {
            Err(FilesError::InvalidLink)
        }
    }

    fn sign(&self, message: &str) -> String {
        URL_SAFE_NO_PAD.encode(self.mac(message).finalize().into_bytes())
    }

    fn mac(&self, message: &str) -> HmacSha256 {
        let mut mac = HmacSha256::new_from_slice(self.signing_key.expose_secret().as_bytes())
            .expect("HMAC takes a key of any length");
        mac.update(message.as_bytes());
        mac
    }

    fn verify(&self, key: &str, expires: i64, message: &str, signature: &str) -> Result<(), FilesError> {
        if !is_object_key(key) || Timestamp::now().as_second() > expires {
            return Err(FilesError::InvalidLink);
        }
        let signature = URL_SAFE_NO_PAD.decode(signature).map_err(|_| FilesError::InvalidLink)?;
        self.mac(message)
            .verify_slice(&signature)
            .map_err(|_| FilesError::InvalidLink)
    }
}

fn upload_message(key: &str, expires: i64, size: u64, mime_type: &str) -> String {
    format!("PUT\n{key}\n{expires}\n{size}\n{mime_type}")
}

fn download_message(key: &str, expires: i64, name: &str, mime_type: &str) -> String {
    format!("GET\n{key}\n{expires}\n{name}\n{mime_type}")
}

fn expiry(ttl: std::time::Duration) -> Result<Timestamp, FilesError> {
    SignedDuration::try_from(ttl)
        .ok()
        .and_then(|ttl| Timestamp::now().checked_add(ttl).ok())
        .ok_or_else(|| FilesError::StorageFailure("link expiry out of range".into()))
}

/// The `Content-Disposition` a local download is served with.
#[must_use]
pub fn local_disposition(name: &str) -> String {
    attachment(name)
}

async fn discard(path: &Path) {
    if let Err(err) = tokio::fs::remove_file(path).await
        && err.kind() != ErrorKind::NotFound
    {
        tracing::error!("could not remove a partial upload {}: {err}", path.display());
    }
}

fn storage(action: &str, err: &std::io::Error) -> FilesError {
    tracing::error!("storage: failed to {action}: {err}");
    FilesError::StorageFailure(format!("{action}: {err}"))
}

#[cfg(test)]
mod tests {
    use futures_util::stream;

    use super::*;

    fn storage(dir: &Path) -> LocalFileStorage {
        LocalFileStorage::new(dir, "/uploads", SecretString::from("test-signing-key"))
    }

    fn upload() -> ObjectUpload {
        ObjectUpload {
            key: "0192a4f0.txt".into(),
            file_name: "note.txt".into(),
            mime_type: "text/plain".into(),
            size: 5,
        }
    }

    fn params_of(url: &str) -> UploadParams {
        let query = url.split_once('?').unwrap().1;
        serde_urlencoded::from_str(query).unwrap()
    }

    #[test]
    fn a_link_handed_out_here_verifies() {
        let local = storage(Path::new("unused"));
        let link = local.upload_link(&upload()).unwrap();

        local.verify_upload("0192a4f0.txt", &params_of(&link.url)).unwrap();
    }

    #[test]
    fn a_link_for_another_size_does_not_verify() {
        let local = storage(Path::new("unused"));
        let mut params = params_of(&local.upload_link(&upload()).unwrap().url);
        params.size = 6;

        assert_eq!(
            local.verify_upload("0192a4f0.txt", &params),
            Err(FilesError::InvalidLink)
        );
    }

    #[test]
    fn an_expired_link_does_not_verify() {
        let local = storage(Path::new("unused"));
        let mut params = params_of(&local.upload_link(&upload()).unwrap().url);
        params.expires = Timestamp::now().as_second() - 1;

        assert_eq!(
            local.verify_upload("0192a4f0.txt", &params),
            Err(FilesError::InvalidLink)
        );
    }

    #[test]
    fn a_link_signed_with_another_key_does_not_verify() {
        let other = LocalFileStorage::new("unused", "/uploads", SecretString::from("another-key"));
        let params = params_of(&other.upload_link(&upload()).unwrap().url);

        assert_eq!(
            storage(Path::new("unused")).verify_upload("0192a4f0.txt", &params),
            Err(FilesError::InvalidLink)
        );
    }

    #[tokio::test]
    async fn a_body_of_the_allowed_size_is_stored() {
        let dir = tempfile::tempdir().unwrap();
        let local = storage(dir.path());
        let body = stream::iter([Ok::<_, std::io::Error>(bytes::Bytes::from_static(b"hello"))]);

        local.write("0192a4f0.txt", 5, body).await.unwrap();

        assert_eq!(local.object_size("0192a4f0.txt").await.unwrap(), Some(5));
    }

    #[tokio::test]
    async fn a_body_of_another_size_leaves_nothing_behind() {
        let dir = tempfile::tempdir().unwrap();
        let local = storage(dir.path());
        let body = stream::iter([Ok::<_, std::io::Error>(bytes::Bytes::from_static(b"hello world"))]);

        let err = local.write("0192a4f0.txt", 5, body).await.unwrap_err();

        assert_eq!(err, FilesError::NotUploaded);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
