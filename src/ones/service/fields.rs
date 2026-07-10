use super::OnesService;
use crate::{
    error::AppResult,
    models::AuthContext,
    ones::{
        field_catalog::{FieldCatalog, FieldDefinition, public_definitions},
        raw::IssueFieldDefinition,
    },
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::Instant};

const FIELD_CATALOG_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Default)]
pub(super) struct FieldCatalogCache {
    entries: RwLock<HashMap<AuthContext, CachedCatalog>>,
}

#[derive(Debug, Clone)]
struct CachedCatalog {
    catalog: Arc<FieldCatalog>,
    expires_at: Instant,
}

impl FieldCatalogCache {
    async fn get(&self, auth: &AuthContext) -> Option<Arc<FieldCatalog>> {
        let entry = self.entries.read().await.get(auth).cloned()?;
        (entry.expires_at > Instant::now()).then_some(entry.catalog)
    }

    async fn insert(&self, auth: AuthContext, catalog: Arc<FieldCatalog>) {
        self.entries.write().await.insert(
            auth,
            CachedCatalog {
                catalog,
                expires_at: Instant::now() + FIELD_CATALOG_TTL,
            },
        );
    }
}

impl OnesService {
    pub(crate) async fn team_field_catalog(
        &self,
        auth: &AuthContext,
    ) -> AppResult<Arc<FieldCatalog>> {
        if let Some(catalog) = self.field_catalogs.get(auth).await {
            return Ok(catalog);
        }
        let fields = self.client.team_fields(auth).await?.fields;
        let catalog = Arc::new(FieldCatalog::new(fields)?);
        self.field_catalogs
            .insert(auth.clone(), catalog.clone())
            .await;
        Ok(catalog)
    }

    pub async fn list_team_fields(&self, auth: &AuthContext) -> AppResult<Vec<FieldDefinition>> {
        Ok(self.team_field_catalog(auth).await?.public_fields())
    }

    pub async fn list_issue_fields(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<Vec<FieldDefinition>> {
        let fields = self
            .client
            .issue_form_fields(auth, issue_uuid)
            .await?
            .fields;
        Ok(public_definitions(&fields))
    }

    pub(crate) async fn issue_field_definitions(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<Vec<IssueFieldDefinition>> {
        Ok(self
            .client
            .issue_form_fields(auth, issue_uuid)
            .await?
            .fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ones::raw::IssueFormFieldsResponse;

    fn auth(email: &str) -> AuthContext {
        AuthContext {
            email: email.to_string(),
            password: "password".to_string(),
        }
    }

    #[tokio::test]
    async fn cache_is_auth_scoped_and_expires() {
        let fields = serde_json::from_str::<IssueFormFieldsResponse>(include_str!(
            "../../../tests/fixtures/team_fields.json"
        ))
        .unwrap()
        .fields;
        let cache = FieldCatalogCache::default();
        let first_auth = auth("first@example.test");
        cache
            .insert(
                first_auth.clone(),
                Arc::new(FieldCatalog::new(fields).unwrap()),
            )
            .await;
        assert!(cache.get(&first_auth).await.is_some());
        assert!(cache.get(&auth("second@example.test")).await.is_none());

        cache
            .entries
            .write()
            .await
            .get_mut(&first_auth)
            .unwrap()
            .expires_at = Instant::now();
        assert!(cache.get(&first_auth).await.is_none());
    }
}
