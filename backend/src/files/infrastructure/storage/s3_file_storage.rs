//! An S3-compatible object store as a storage (Garage locally). The browser sends
//! and fetches the bytes with pre-signed requests; this back-end only signs them,
//! asks the size of what arrived and removes objects.

use std::time::Duration;

use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region, RequestChecksumCalculation};
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};
use jiff::{SignedDuration, Timestamp};
use secrecy::{ExposeSecret, SecretString};

use super::{DOWNLOAD_LINK_TTL, UPLOAD_LINK_TTL, attachment};
use crate::files::application::{ObjectDownload, ObjectUpload, SignedLink};
use crate::files::domain::FilesError;

/// Where the bucket is and the credentials to it.
#[derive(Clone)]
pub struct S3Settings {
    /// The address this back-end reaches the store at.
    pub endpoint: String,
    /// The address the browser reaches the store at, which a signature covers.
    pub public_endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: SecretString,
}

#[derive(Clone)]
pub struct S3FileStorage {
    /// Talks to the store from this back-end.
    client: Client,
    /// Signs the requests the browser makes, for the address it reaches the store at.
    signer: Client,
    bucket: String,
}

impl S3FileStorage {
    #[must_use]
    pub fn new(settings: &S3Settings) -> Self {
        Self {
            client: client(settings, &settings.endpoint),
            signer: client(settings, &settings.public_endpoint),
            bucket: settings.bucket.clone(),
        }
    }

    pub async fn upload_link(&self, upload: &ObjectUpload) -> Result<SignedLink, FilesError> {
        let size = i64::try_from(upload.size).map_err(|_| FilesError::TooLarge { max: upload.size })?;
        let request = self
            .signer
            .put_object()
            .bucket(&self.bucket)
            .key(&upload.key)
            .content_type(&upload.mime_type)
            .content_length(size)
            .content_disposition(attachment(&upload.file_name))
            .presigned(presigning(UPLOAD_LINK_TTL)?)
            .await
            .map_err(|err| failure("sign an upload", &err))?;
        signed("PUT", &request, UPLOAD_LINK_TTL)
    }

    pub async fn download_link(&self, download: &ObjectDownload) -> Result<SignedLink, FilesError> {
        let request = self
            .signer
            .get_object()
            .bucket(&self.bucket)
            .key(&download.key)
            .response_content_disposition(attachment(&download.file_name))
            .response_content_type(&download.mime_type)
            .presigned(presigning(DOWNLOAD_LINK_TTL)?)
            .await
            .map_err(|err| failure("sign a download", &err))?;
        signed("GET", &request, DOWNLOAD_LINK_TTL)
    }

    pub async fn object_size(&self, key: &str) -> Result<Option<u64>, FilesError> {
        match self.client.head_object().bucket(&self.bucket).key(key).send().await {
            Ok(object) => Ok(object.content_length().and_then(|length| u64::try_from(length).ok())),
            Err(err) if err.as_service_error().is_some_and(HeadObjectError::is_not_found) => Ok(None),
            Err(err) => Err(failure("read an object's size", &err)),
        }
    }

    pub async fn remove(&self, key: &str) -> Result<(), FilesError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|err| failure("delete an object", &err))?;
        Ok(())
    }
}

/// A client for one address. Requests use path-style addressing, as a store such
/// as Garage serves no bucket subdomains without wildcard DNS, and carry no
/// checksum the SDK would add on its own, which a browser could not repeat.
fn client(settings: &S3Settings, endpoint: &str) -> Client {
    let credentials = Credentials::new(
        settings.access_key.clone(),
        settings.secret_key.expose_secret(),
        None,
        None,
        "rust-axum-app",
    );
    let config = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(settings.region.clone()))
        .endpoint_url(endpoint)
        .credentials_provider(credentials)
        .force_path_style(true)
        .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
        .build();
    Client::from_conf(config)
}

fn presigning(ttl: Duration) -> Result<PresigningConfig, FilesError> {
    PresigningConfig::expires_in(ttl).map_err(|err| failure("configure a signature", &err))
}

/// The browser's part of a signed request: the address and the headers it has to
/// send. `host` and `content-length` are left out, as the browser sets them itself
/// and a script may not.
fn signed(method: &'static str, request: &PresignedRequest, ttl: Duration) -> Result<SignedLink, FilesError> {
    let expires_at = SignedDuration::try_from(ttl)
        .ok()
        .and_then(|ttl| Timestamp::now().checked_add(ttl).ok())
        .ok_or_else(|| FilesError::StorageFailure("link expiry out of range".into()))?;
    Ok(SignedLink {
        method,
        url: request.uri().to_string(),
        headers: request
            .headers()
            .filter(|(name, _)| !name.eq_ignore_ascii_case("host") && !name.eq_ignore_ascii_case("content-length"))
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect(),
        expires_at,
    })
}

fn failure(action: &str, err: &dyn std::error::Error) -> FilesError {
    tracing::error!("object storage: failed to {action}: {err}");
    FilesError::StorageFailure(format!("{action}: {err}"))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use secrecy::SecretString;
    use testcontainers_modules::testcontainers::core::{ContainerPort, WaitFor};
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use testcontainers_modules::testcontainers::{ContainerAsync, GenericImage, ImageExt};

    use super::{S3FileStorage, S3Settings};
    use crate::files::application::{ObjectDownload, ObjectUpload, SignedLink};

    const ACCESS_KEY: &str = "GK0123456789abcdef01234567";
    const SECRET_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const BUCKET: &str = "files";
    const CONFIG: &str = r#"
metadata_dir = "/var/lib/garage/meta"
data_dir = "/var/lib/garage/data"
db_engine = "lmdb"
replication_factor = 1
rpc_bind_addr = "[::]:3901"
rpc_public_addr = "127.0.0.1:3901"

[s3_api]
api_bind_addr = "[::]:3900"
s3_region = "garage"
root_domain = ".s3.localhost"
"#;

    async fn garage() -> (ContainerAsync<GenericImage>, S3FileStorage) {
        let container = GenericImage::new("docker.io/dxflrs/garage", "v2.4.1")
            .with_exposed_port(ContainerPort::Tcp(3900))
            .with_wait_for(WaitFor::Nothing)
            .with_cmd(["/garage", "server", "--single-node", "--default-bucket"])
            .with_env_var("GARAGE_RPC_SECRET", SECRET_KEY)
            .with_env_var("GARAGE_DEFAULT_ACCESS_KEY", ACCESS_KEY)
            .with_env_var("GARAGE_DEFAULT_SECRET_KEY", SECRET_KEY)
            .with_env_var("GARAGE_DEFAULT_BUCKET", BUCKET)
            .with_copy_to("/etc/garage.toml", CONFIG.as_bytes().to_vec())
            .start()
            .await
            .unwrap();
        let endpoint = format!("http://127.0.0.1:{}", container.get_host_port_ipv4(3900).await.unwrap());
        let storage = S3FileStorage::new(&S3Settings {
            endpoint: endpoint.clone(),
            public_endpoint: endpoint,
            region: "garage".into(),
            bucket: BUCKET.into(),
            access_key: ACCESS_KEY.into(),
            secret_key: SecretString::from(SECRET_KEY),
        });
        for _ in 0..60 {
            if storage.object_size("probe").await.is_ok() {
                return (container, storage);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        panic!("Garage did not start");
    }

    fn upload(size: u64) -> ObjectUpload {
        ObjectUpload {
            key: "0192a4f0.txt".into(),
            file_name: "zażółć.txt".into(),
            mime_type: "text/plain".into(),
            size,
        }
    }

    async fn send(link: &SignedLink, body: &'static [u8]) -> reqwest::Response {
        let mut request = reqwest::Client::new().put(&link.url).body(body);
        for (name, value) in &link.headers {
            request = request.header(name, value);
        }
        request.send().await.unwrap()
    }

    #[tokio::test]
    async fn the_browser_uploads_and_downloads_with_signed_links() {
        let (_container, storage) = garage().await;

        let upload_link = storage.upload_link(&upload(5)).await.unwrap();
        assert!(send(&upload_link, b"hello").await.status().is_success());
        assert_eq!(storage.object_size("0192a4f0.txt").await.unwrap(), Some(5));

        let download_link = storage
            .download_link(&ObjectDownload {
                key: "0192a4f0.txt".into(),
                file_name: "zażółć.txt".into(),
                mime_type: "text/plain".into(),
            })
            .await
            .unwrap();
        let download = reqwest::get(&download_link.url).await.unwrap();
        let disposition = download.headers()["content-disposition"].to_str().unwrap().to_owned();
        assert_eq!(download.text().await.unwrap(), "hello");
        assert!(
            disposition.contains("filename*=UTF-8''za%C5%BC%C3%B3%C5%82%C4%87.txt"),
            "{disposition}"
        );

        storage.remove("0192a4f0.txt").await.unwrap();
        assert_eq!(storage.object_size("0192a4f0.txt").await.unwrap(), None);
    }

    #[tokio::test]
    async fn an_upload_of_another_size_is_refused() {
        let (_container, storage) = garage().await;

        let link = storage.upload_link(&upload(5)).await.unwrap();

        assert_eq!(
            send(&link, b"hello world").await.status(),
            reqwest::StatusCode::FORBIDDEN
        );
        assert_eq!(storage.object_size("0192a4f0.txt").await.unwrap(), None);
    }
}
