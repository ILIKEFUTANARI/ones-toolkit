use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    models::AuthContext,
    ones::raw::{
        IdentifierResponse, IssueFormFieldsResponse, LoginResponse, MessagesResponse,
        OnesqlResponse, QuickActionResponse, TaskInfoRaw, ViewDetailResponse, ViewListResponse,
    },
};
use reqwest::{Client, Method};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportRetry {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone)]
pub(crate) struct Session {
    pub(crate) token: String,
    pub(crate) user_uuid: String,
}

#[derive(Debug, Clone)]
pub struct OnesClient {
    http: Client,
    config: AppConfig,
    sessions: Arc<RwLock<HashMap<AuthContext, Session>>>,
    login_lock: Arc<Mutex<()>>,
}

impl OnesClient {
    pub fn new(config: AppConfig) -> Self {
        Self {
            http: Client::builder()
                .build()
                .expect("reqwest client should build"),
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            login_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn user_filter_views_list(&self, auth: &AuthContext) -> AppResult<ViewListResponse> {
        self.request_json(
            Method::GET,
            auth,
            self.project_team_url("user_filter_views/list"),
            None::<&()>,
        )
        .await
    }

    pub async fn user_filter_views_detail(
        &self,
        auth: &AuthContext,
        view_uuid: &str,
    ) -> AppResult<ViewDetailResponse> {
        self.request_json(
            Method::POST,
            auth,
            self.project_team_url("user_filter_views/detail"),
            Some(&serde_json::json!({ "view_uuid": view_uuid })),
        )
        .await
    }

    pub async fn resolve_display_id(
        &self,
        auth: &AuthContext,
        display_id: &str,
    ) -> AppResult<IdentifierResponse> {
        self.request_json(
            Method::POST,
            auth,
            self.ones_project_team_url("tasks/identifier"),
            Some(&serde_json::json!({ "display_id_path": display_id })),
        )
        .await
    }

    pub async fn issue_form_fields(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<IssueFormFieldsResponse> {
        self.request_json(
            Method::POST,
            auth,
            self.ones_project_team_url("issue_form/fields"),
            Some(&serde_json::json!({ "issue_context": { "issue_uuid": issue_uuid } })),
        )
        .await
    }

    pub async fn team_fields(&self, auth: &AuthContext) -> AppResult<IssueFormFieldsResponse> {
        // Prefer public legacy `…/project/team/:teamUUID/fields`; fall back to
        // internal `…/ones-project/team/:teamUUID/v2/field/list` on 404/405.
        self.request_json_with_legacy_fallback(
            Method::GET,
            auth,
            [
                self.project_team_url("fields"),
                self.ones_project_team_url("v2/field/list"),
            ],
            None::<&()>,
        )
        .await
    }

    pub async fn workitems_onesql(
        &self,
        auth: &AuthContext,
        body: &Value,
    ) -> AppResult<OnesqlResponse> {
        // ONESQL is a separate request shape from `items/graphql`; the public
        // GraphQL endpoint is not an equivalent fallback for this internal one.
        self.request_json(
            Method::POST,
            auth,
            self.ones_project_team_url("workitems/onesql"),
            Some(body),
        )
        .await
    }

    pub async fn task_messages(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<MessagesResponse> {
        self.request_json(
            Method::GET,
            auth,
            self.project_team_url(&format!("task/{issue_uuid}/messages")),
            None::<&()>,
        )
        .await
    }

    /// Public legacy `GET /project/api/project/team/:teamUUID/task/:taskUUIDOrNumber/info`.
    /// Returns the flat task record documented for the public legacy endpoint.
    /// No fallback: the internal `workitems/onesql` returns a richer GraphQL
    /// shape, so callers needing that should keep using `workitems_onesql`.
    pub async fn task_info(
        &self,
        auth: &AuthContext,
        task_uuid_or_number: &str,
    ) -> AppResult<crate::models::LegacyTaskInfo> {
        let response: TaskInfoRaw = self
            .request_json(
                Method::GET,
                auth,
                self.project_team_url(&format!("task/{task_uuid_or_number}/info")),
                None::<&()>,
            )
            .await?;
        Ok(response.into())
    }

    pub async fn quick_action_task_info(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<QuickActionResponse> {
        let query = include_str!("graphql/quick_action_task_info.graphql");
        self.request_json(
            Method::POST,
            auth,
            self.project_team_url("items/graphql?t=quick_action_task_info"),
            Some(&serde_json::json!({
                "query": query,
                "variables": { "key": format!("task-{issue_uuid}") }
            })),
        )
        .await
    }

    pub(crate) async fn request_json<T, B>(
        &self,
        method: Method,
        auth: &AuthContext,
        url: String,
        body: Option<&B>,
    ) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request_json_with_retry(method, auth, url, body, TransportRetry::Enabled)
            .await
    }

    pub(crate) async fn request_json_once<T, B>(
        &self,
        method: Method,
        auth: &AuthContext,
        url: String,
        body: Option<&B>,
    ) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request_json_with_retry(method, auth, url, body, TransportRetry::Disabled)
            .await
    }

    /// Try the provided URLs in order. Fallback fires only on HTTP 404 / 405
    /// (documented legacy-unavailable signals). Other statuses, JSON failures,
    /// and transport errors propagate so we never mask real upstream problems.
    pub(crate) async fn request_json_with_legacy_fallback<T, B, I>(
        &self,
        method: Method,
        auth: &AuthContext,
        urls: I,
        body: Option<&B>,
    ) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize + ?Sized,
        I: IntoIterator<Item = String>,
    {
        let mut last_fallback: Option<AppError> = None;
        for url in urls {
            match self
                .request_json_with_retry(
                    method.clone(),
                    auth,
                    url.clone(),
                    body,
                    TransportRetry::Enabled,
                )
                .await
            {
                Ok(value) => return Ok(value),
                Err(error) if is_legacy_fallback_signal(&error) => {
                    last_fallback = Some(error);
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        Err(last_fallback
            .unwrap_or_else(|| AppError::invalid_input("legacy fallback list was empty")))
    }

    async fn request_json_with_retry<T, B>(
        &self,
        method: Method,
        auth: &AuthContext,
        url: String,
        body: Option<&B>,
        retry: TransportRetry,
    ) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let value = self.request_value(method, auth, url, body, retry).await?;
        serde_json::from_value(value).map_err(AppError::from)
    }

    async fn request_value<B>(
        &self,
        method: Method,
        auth: &AuthContext,
        url: String,
        body: Option<&B>,
        retry: TransportRetry,
    ) -> AppResult<Value>
    where
        B: Serialize + ?Sized,
    {
        let first_session = self.ensure_session(auth).await?;
        let mut response = match self
            .send_with_session(&method, &url, body, &first_session)
            .await
        {
            Ok(response) => response,
            Err(AppError::Http(error))
                if retry == TransportRetry::Enabled && is_retryable_transport_error(&error) =>
            {
                self.send_with_session(&method, &url, body, &first_session)
                    .await?
            }
            Err(error) => return Err(error),
        };

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let session = self.login(auth).await?;
            {
                let mut guard = self.sessions.write().await;
                guard.insert(auth.clone(), session.clone());
            }
            response = self
                .send_with_session(&method, &url, body, &session)
                .await?;
        }

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::upstream_status(
                status,
                format!("{method} {url}"),
                text,
            ));
        }

        let value = response.json::<Value>().await?;
        Ok(value)
    }

    pub(crate) async fn ensure_session(&self, auth: &AuthContext) -> AppResult<Session> {
        if let Some(session) = self.sessions.read().await.get(auth).cloned() {
            return Ok(session);
        }
        let _guard = self.login_lock.lock().await;
        if let Some(session) = self.sessions.read().await.get(auth).cloned() {
            return Ok(session);
        }
        let session = self.login(auth).await?;
        self.sessions
            .write()
            .await
            .insert(auth.clone(), session.clone());
        Ok(session)
    }

    async fn login(&self, auth: &AuthContext) -> AppResult<Session> {
        let url = format!(
            "{}/project/api/project/auth/login",
            self.config.ones_base_url
        );
        let response = self
            .http
            .post(&url)
            .json(&serde_json::json!({
                "email": auth.email,
                "password": auth.password,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::upstream_status(
                status,
                format!("POST {url}"),
                text,
            ));
        }

        let body: LoginResponse = response.json().await?;
        Ok(Session {
            token: body.user.token,
            user_uuid: body.user.uuid,
        })
    }

    async fn send_with_session<B>(
        &self,
        method: &Method,
        url: &str,
        body: Option<&B>,
        session: &Session,
    ) -> AppResult<reqwest::Response>
    where
        B: Serialize + ?Sized,
    {
        let mut request = self
            .http
            .request(method.clone(), url)
            .header("Ones-Auth-Token", &session.token)
            .header("Ones-User-Id", &session.user_uuid)
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            request = request.json(body);
        }

        request.send().await.map_err(AppError::from)
    }

    pub(crate) fn project_team_url(&self, suffix: &str) -> String {
        format!(
            "{}/project/api/project/team/{}/{}",
            self.config.ones_base_url, self.config.team_uuid, suffix
        )
    }

    pub(crate) fn ones_project_team_url(&self, suffix: &str) -> String {
        format!(
            "{}/project/api/ones-project/team/{}/{}",
            self.config.ones_base_url, self.config.team_uuid, suffix
        )
    }
}

fn is_retryable_transport_error(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_request() || error.is_timeout()
}

/// Whether the error signals that a legacy public endpoint is unavailable on
/// the current host. We only treat HTTP 404 / 405 as compatibility signals —
/// 400, 401, 403, 429, and 5xx always propagate so callers see the real status.
fn is_legacy_fallback_signal(error: &AppError) -> bool {
    matches!(
        error,
        AppError::UpstreamStatus {
            status: 404 | 405,
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn caches_sessions_by_credentials() {
        let client = OnesClient::new(AppConfig {
            ones_base_url: "http://127.0.0.1".to_string(),
            team_uuid: "team".to_string(),
        });
        client.sessions.write().await.insert(
            AuthContext {
                email: "a".to_string(),
                password: "b".to_string(),
            },
            Session {
                token: "token".to_string(),
                user_uuid: "user".to_string(),
            },
        );
        let session = client
            .ensure_session(&AuthContext {
                email: "a".to_string(),
                password: "b".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(session.token, "token");
    }
}
