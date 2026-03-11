use crate::core::resolver::loader::ResolvedLoader;
use crate::core::workflow::Workflow;
use crate::domain::source::Hash;
use crate::errors::{ConduitError, ConduitResult};
use crate::schemas::lock::HashKind;

impl Workflow {
    pub async fn download_loader(
        &self,
        resolved: &ResolvedLoader,
    ) -> ConduitResult<(String, HashKind)> {
        let mut hash_obj = Hash {
            sha1: None,
            sha256: None,
            sha512: None,
        };

        if !resolved.hash.is_empty() {
            if resolved.hash.len() == 64 {
                hash_obj.sha256 = Some(resolved.hash.clone());
            } else {
                hash_obj.sha1 = Some(resolved.hash.clone());
            }
        }

        let download_hash = if resolved.hash.is_empty() {
            None
        } else {
            Some(&hash_obj)
        };

        self.ctx
            .downloader
            .download_to_store(&resolved.url, download_hash)
            .await
            .map_err(ConduitError::from)
    }
}
