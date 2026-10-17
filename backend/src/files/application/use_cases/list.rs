use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileRepository, FilesError, StoredFile};
use crate::shared_kernel::pagination::{PageRequest, PageResult};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
    pub async fn list(&self, request: PageRequest) -> Result<PageResult<StoredFile>, FilesError> {
        self.repository.page_uploaded(request).await
    }
}
#[cfg(test)]
mod tests {
    use crate::files::application::testing::{files_harness, request, uploaded};
    use crate::shared_kernel::pagination::PageRequest;

    #[tokio::test]
    async fn lists_neither_soft_deleted_files_nor_pending_uploads() {
        let (service, _, _) = files_harness();
        let kept = uploaded(&service, "keep.txt").await;
        let gone = uploaded(&service, "gone.txt").await;
        service.soft_delete(gone.id, 7).await.unwrap();
        service.request_upload(request("pending.txt", 5)).await.unwrap();

        let listed = service.list(PageRequest::new(0, 20).unwrap()).await.unwrap();

        assert_eq!(listed.total, 1);
        assert_eq!(listed.items[0].id, kept.id);
    }

    #[tokio::test]
    async fn lists_the_newest_files_first_a_page_at_a_time() {
        let (service, _, _) = files_harness();
        for name in ["a.txt", "b.txt", "c.txt"] {
            uploaded(&service, name).await;
        }

        let second = service.list(PageRequest::new(1, 2).unwrap()).await.unwrap();

        assert_eq!(
            second
                .items
                .iter()
                .map(|file| file.original_name.as_str())
                .collect::<Vec<_>>(),
            ["a.txt"]
        );
        assert_eq!((second.total, second.total_pages()), (3, 2));
    }
}
