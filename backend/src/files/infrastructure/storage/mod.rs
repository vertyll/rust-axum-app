//! The storages of file bytes behind the `FileStorage` port: the local disk and
//! an S3-compatible object store, combined by `Storages`.

mod local_file_storage;
mod s3_file_storage;
mod storages;

pub use local_file_storage::{DownloadParams, LocalFileStorage, UploadParams, local_disposition};
pub use s3_file_storage::{S3FileStorage, S3Settings};
pub use storages::Storages;

use std::fmt::Write;
use std::time::Duration;

/// How long the browser has to send the bytes, enough for a slow connection to finish a large file.
const UPLOAD_LINK_TTL: Duration = Duration::from_mins(15);

/// How long a download link works: long enough to start the download, short enough that a
/// link pasted elsewhere stops working before it travels far.
const DOWNLOAD_LINK_TTL: Duration = Duration::from_mins(5);

/// `Content-Disposition` for saving a file under `name` (RFC 6266): an ASCII
/// `filename` for old clients and the full name, UTF-8 and percent-encoded, in
/// `filename*` (RFC 8187).
fn attachment(name: &str) -> String {
    let fallback: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_graphic() && c != '"' && c != '\\' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut encoded = String::new();
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || b"!#$&+-.^_`|~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    format!("attachment; filename=\"{fallback}\"; filename*=UTF-8''{encoded}")
}

/// Whether `key` is one this application could have made: a UUID with an optional
/// plain extension, so a key from a request can never name a path outside the storage.
fn is_object_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && !key.starts_with('.')
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.')
}

#[cfg(test)]
mod tests {
    use super::{attachment, is_object_key};

    #[test]
    fn a_name_outside_ascii_keeps_its_letters_in_the_extended_parameter() {
        assert_eq!(
            attachment("zażółć.txt"),
            "attachment; filename=\"za____.txt\"; filename*=UTF-8''za%C5%BC%C3%B3%C5%82%C4%87.txt"
        );
    }

    #[test]
    fn only_keys_this_application_makes_are_accepted() {
        assert!(is_object_key("0192a4f0-7b1c-7d2e-9f3a-4b5c6d7e8f90.pdf"));
        assert!(!is_object_key("../secrets"));
        assert!(!is_object_key("a/b"));
        assert!(!is_object_key(".hidden"));
    }
}
