use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    models::AuthContext,
    ones::raw::{
        IdentifierResponse, IssueFormFieldsResponse, LoginResponse, MessagesResponse,
        OnesqlResponse, QuickActionResponse, ViewConfigResponse, ViewDetailResponse,
        ViewListResponse,
    },
};
use reqwest::{Client, Method};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone)]
struct Session {
    token: String,
    user_uuid: String,
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

    pub async fn user_filter_views_config(
        &self,
        auth: &AuthContext,
    ) -> AppResult<ViewConfigResponse> {
        self.request_json(
            Method::GET,
            auth,
            self.project_team_url("user_filter_views/config"),
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

    pub async fn issue_detail_form(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<Value> {
        self.request_value(
            Method::GET,
            auth,
            self.ones_project_team_url(&format!("issue/{issue_uuid}/detail_form")),
            None::<&()>,
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

    pub async fn workitems_onesql(
        &self,
        auth: &AuthContext,
        body: &Value,
    ) -> AppResult<OnesqlResponse> {
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

    pub async fn quick_action_task_info(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<QuickActionResponse> {
        let query = "query Task { task(key: $key) { key path uuid displayId name project { uuid isSample isArchive } issueType { uuid detailType } alternativeIssueType { uuid detailType name } canEdit(attachPermission:{ permissions:[\"update_tasks\"], }) sprint { name uuid } relatedWikiPages { uuid title referenceType ref_type: referenceType subReferenceType sub_ref_type: subReferenceType errorMessage } relatedWikiPagesCount relatedCases { uuid } allRelatedCases(stubRelatedCases: {}){ uuid name id path testcaseLibrary { uuid name } } } }";
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

    async fn request_json<T, B>(
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
        let value = self.request_value(method, auth, url, body).await?;
        serde_json::from_value(value).map_err(AppError::from)
    }

    async fn request_value<B>(
        &self,
        method: Method,
        auth: &AuthContext,
        url: String,
        body: Option<&B>,
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
            Err(AppError::Http(error)) if is_retryable_transport_error(&error) => {
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
            return Err(AppError::upstream_status(status, text));
        }

        let value = response.json::<Value>().await?;
        Ok(value)
    }

    async fn ensure_session(&self, auth: &AuthContext) -> AppResult<Session> {
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
            .post(url)
            .json(&serde_json::json!({
                "email": auth.email,
                "password": auth.password,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::upstream_status(status, text));
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

    fn project_team_url(&self, suffix: &str) -> String {
        format!(
            "{}/project/api/project/team/{}/{}",
            self.config.ones_base_url, self.config.team_uuid, suffix
        )
    }

    fn ones_project_team_url(&self, suffix: &str) -> String {
        format!(
            "{}/project/api/ones-project/team/{}/{}",
            self.config.ones_base_url, self.config.team_uuid, suffix
        )
    }
}

fn is_retryable_transport_error(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_request() || error.is_timeout()
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
