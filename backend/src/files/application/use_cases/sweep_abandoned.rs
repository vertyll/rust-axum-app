use jiff::Timestamp;

use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileRepository, FilesError};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
    /// Removes the uploads allowed before `before` that never arrived, with whatever
    /// part of them reached the storage: nothing here learns whether the browser
    /// finished, so a pending upload past its window is an abandoned one.
    pub async fn sweep_abandoned(&self, before: Timestamp) -> Result<usize, FilesError> {
        let abandoned = self.repository.pending_before(before).await?;
        for file in &abandoned {
            self.storage.remove(file.storage, &file.object_key).await?;
            self.repository.delete(file.id).await?;
        }
        Ok(abandoned.len())
    }
}

#[cfg(test)]
mod tests {
    use jiff::{SignedDuration, Timestamp};

    use crate::files::application::testing::{files_harness, request, uploaded};

    #[tokio::test]
    async fn removes_only_the_uploads_still_pending() {
        let (service, repository, storage) = files_harness();
        let (abandoned, _) = service.request_upload(request("lost.txt", 5)).await.unwrap();
        let kept = uploaded(&service, "kept.txt").await;

        let removed = service
            .sweep_abandoned(Timestamp::now() + SignedDuration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(removed, 1);
        assert!(repository.get_any(abandoned.id).is_none());
        assert!(repository.get_any(kept.id).is_some());
        assert_eq!(storage.removed(), vec![abandoned.object_key]);
    }
}
