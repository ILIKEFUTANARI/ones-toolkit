use crate::{
    config::AppConfig,
    error::AppResult,
    models::{AuthContext, LegacyTaskInfo},
    ones::{client::OnesClient, identity::IdentityRegistry},
};
use std::sync::Arc;

mod attachments;
mod code;
mod fields;
mod issue;
mod work_hours;
mod workbench;
mod workflow;

#[derive(Debug, Clone)]
pub struct OnesService {
    client: Arc<OnesClient>,
    identities: Arc<IdentityRegistry>,
    field_catalogs: Arc<fields::FieldCatalogCache>,
}

impl OnesService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            client: Arc::new(OnesClient::new(config)),
            identities: Arc::new(IdentityRegistry::default()),
            field_catalogs: Arc::new(fields::FieldCatalogCache::default()),
        }
    }

    /// Read the public legacy `GET …/task/:taskUUIDOrNumber/info` endpoint and
    /// return the public-facing [`LegacyTaskInfo`] DTO. Shared by HTTP, MCP, and
    /// CLI so all three transports emit the same shape from the same service
    /// call.
    pub async fn legacy_task_info(
        &self,
        auth: &AuthContext,
        task_uuid_or_number: &str,
    ) -> AppResult<LegacyTaskInfo> {
        self.client.task_info(auth, task_uuid_or_number).await
    }
}
