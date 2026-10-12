//! Core-only surface checks.
//!
//! These run when neither adapter feature is enabled, so they prove that the
//! shared business DTOs and the extracted credential helper stay usable without
//! any schema derive or HTTP/MCP adapter in the build.
#![cfg(not(any(feature = "http", feature = "mcp")))]

use http::HeaderMap;
use ones::{
    AppError,
    api::{CodeLinksResponse, FieldsResponse, IssueResponse, ViewsResponse},
    attachments::IssueAttachmentPage,
    auth::auth_from_headers,
    code::{CodeBranch, CodeRepository},
    config::AppConfig,
    models::{IssueDetail, WorkbenchViewSummary},
    ones::field_catalog::FieldDefinition,
    workflow::{TransitionFieldValue, TransitionValues},
};

fn headers(email: Option<&str>, password: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Some(email) = email {
        headers.insert("x-ones-email", email.parse().unwrap());
    }
    if let Some(password) = password {
        headers.insert("x-ones-password", password.parse().unwrap());
    }
    headers
}

#[test]
fn resolves_credentials_from_current_and_legacy_headers() {
    let auth =
        auth_from_headers(&headers(Some("alex@example.test"), Some("dummy-password"))).unwrap();
    assert_eq!(auth.email, "alex@example.test");
    assert_eq!(auth.password, "dummy-password");

    let mut legacy = HeaderMap::new();
    legacy.insert("x-user-email", "alex@example.test".parse().unwrap());
    legacy.insert("x-user-password", "dummy-password".parse().unwrap());
    let auth = auth_from_headers(&legacy).unwrap();
    assert_eq!(auth.email, "alex@example.test");
    assert_eq!(auth.password, "dummy-password");

    assert!(matches!(
        auth_from_headers(&headers(None, Some("dummy-password"))),
        Err(AppError::MissingAuth)
    ));
    assert!(matches!(
        auth_from_headers(&headers(Some("alex@example.test"), None)),
        Err(AppError::MissingAuth)
    ));
}

#[test]
fn shared_dtos_round_trip_without_schema_derives() {
    let views = serde_json::json!({
        "views": [{ "uuid": "view-demo-001", "name": "Demo view" }]
    });
    let parsed: ViewsResponse = serde_json::from_value(views.clone()).unwrap();
    assert_eq!(serde_json::to_value(&parsed).unwrap(), views);

    let issue = serde_json::json!({
        "issue": {
            "uuid": "issue-demo-001",
            "display_id": "DEMO-1234",
            "title": "Demo issue",
            "status": { "name": "In Progress", "category": "started" },
            "custom_fields": [{ "name": "Story Point", "value": "3" }]
        }
    });
    let parsed: IssueResponse = serde_json::from_value(issue.clone()).unwrap();
    assert_eq!(serde_json::to_value(&parsed).unwrap(), issue);
    assert_eq!(parsed.issue.display_id.as_deref(), Some("DEMO-1234"));
    assert_eq!(
        parsed
            .issue
            .status
            .as_ref()
            .map(|status| status.name.as_str()),
        Some("In Progress")
    );
    assert_eq!(
        serde_json::to_value(parsed.issue.custom_fields).unwrap(),
        serde_json::json!([{ "name": "Story Point", "value": "3" }])
    );

    let fields = serde_json::json!({
        "fields": [{
            "uuid": "field001",
            "name": "Title",
            "built_in": true,
            "type_uuid": "name",
            "value_type": "text",
            "can_filter": true,
            "can_sort": true,
            "can_group": false
        }]
    });
    let parsed: FieldsResponse = serde_json::from_value(fields).unwrap();
    assert_eq!(parsed.fields.len(), 1);
    assert!(parsed.fields[0].can_filter);
    assert!(!parsed.fields[0].can_group);
    let _: FieldDefinition = parsed.fields[0].clone();

    let attachments = serde_json::json!({
        "attachments": [{
            "uuid": "attachment-demo-001",
            "name": "spec.pdf",
            "mime": "application/pdf",
            "size": 1024
        }],
        "page_info": { "has_next_page": false }
    });
    let parsed: IssueAttachmentPage = serde_json::from_value(attachments.clone()).unwrap();
    assert_eq!(serde_json::to_value(&parsed).unwrap(), attachments);
    assert_eq!(parsed.page_info.has_next_page, false);
    assert_eq!(parsed.page_info.next_cursor, None);

    let links = serde_json::json!({
        "repositories": [{ "uuid": "repo-demo-001", "name": "demo", "namespace": "demo", "scm_type": "Git" }],
        "branches": [{ "uuid": "branch-demo-001", "name": "feature/demo", "repo": "demo/demo", "scm_type": "Git" }],
        "commits": [],
        "pull_requests": []
    });
    let parsed: CodeLinksResponse = serde_json::from_value(links).unwrap();
    assert_eq!(parsed.repositories[0].name, "demo");
    assert_eq!(parsed.branches[0].name, "feature/demo");
    let _ = (
        CodeRepository {
            uuid: "repo-demo-001".to_string(),
            name: "demo".to_string(),
            namespace: "demo".to_string(),
            scm_type: "Git".to_string(),
            is_public: None,
        },
        CodeBranch {
            uuid: "branch-demo-001".to_string(),
            name: "feature/demo".to_string(),
            repo: "demo/demo".to_string(),
            scm_type: "Git".to_string(),
            url: None,
        },
    );

    let transitions = serde_json::json!({
        "transition_uuid": "transition-demo-001",
        "field_values": [{
            "field_uuid": "field057",
            "value": [{ "uuid": "comment-demo-001", "content": "regression note" }]
        }]
    });
    let parsed: TransitionValues = serde_json::from_value(transitions.clone()).unwrap();
    assert_eq!(serde_json::to_value(&parsed).unwrap(), transitions);
    assert!(matches!(
        parsed.field_values[0],
        TransitionFieldValue { .. }
    ));
}

#[test]
fn core_config_builds_a_service_without_adapters() {
    let config = AppConfig {
        ones_base_url: "https://ones.example.test/project/api".to_string(),
        team_uuid: "team-demo-001".to_string(),
    };
    let service = ones::OnesService::new(config.clone());
    assert_eq!(config.team_uuid, "team-demo-001");
    let _clone = service.clone();

    let error = AppError::upstream_status(
        reqwest::StatusCode::BAD_GATEWAY,
        "/project/api/demo/endpoint",
        "upstream said no",
    );
    assert_eq!(
        error.to_string(),
        "upstream /project/api/demo/endpoint returned status 502: upstream said no"
    );

    let _: WorkbenchViewSummary = WorkbenchViewSummary {
        uuid: "view-demo-001".to_string(),
        name: "Demo view".to_string(),
    };
    let _: IssueDetail = IssueDetail {
        uuid: "issue-demo-001".to_string(),
        display_id: None,
        title: None,
        issue_type: None,
        status: None,
        project: None,
        creator: None,
        assignee: None,
        watchers: None,
        priority: None,
        sprint: None,
        modules: None,
        due_date: None,
        created_at: None,
        updated_at: None,
        description: None,
        custom_fields: None,
        related_wiki_pages: None,
        related_cases: None,
    };
}
