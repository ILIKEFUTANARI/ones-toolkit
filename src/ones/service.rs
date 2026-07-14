use crate::{
    config::AppConfig,
    ones::{client::OnesClient, identity::IdentityRegistry},
};
use std::sync::Arc;

mod attachments;
mod code;
mod fields;
mod issue;
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
}
