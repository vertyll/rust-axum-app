use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::Serialize;

use super::FileResponse;
use crate::files::application::SignedLink;
use crate::files::domain::StoredFile;

/// A request the browser makes on its own, and the headers it has to send with it.
#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub method: &'static str,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub expires_at: Timestamp,
}

impl From<SignedLink> for LinkResponse {
    fn from(link: SignedLink) -> Self {
        Self {
            method: link.method,
            url: link.url,
            headers: link.headers.into_iter().collect(),
            expires_at: link.expires_at,
        }
    }
}

/// An allowed upload: the pending file, and where its bytes go.
#[derive(Debug, Serialize)]
pub struct UploadTicketResponse {
    pub file: FileResponse,
    pub upload: LinkResponse,
}

impl From<(StoredFile, SignedLink)> for UploadTicketResponse {
    fn from((file, link): (StoredFile, SignedLink)) -> Self {
        Self {
            file: file.into(),
            upload: link.into(),
        }
    }
}
