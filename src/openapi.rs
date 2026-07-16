use serde_json::Value;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityRequirement, SecurityScheme};

pub fn document() -> Value {
    let mut openapi = crate::http_api::build_api_router().into_openapi();
    finalize(&mut openapi);
    serde_json::to_value(openapi).expect("OpenAPI should serialize")
}

pub(crate) fn finalize(openapi: &mut utoipa::openapi::OpenApi) {
    openapi.info.title = "ONES Toolkit (Unofficial)".to_string();
    openapi.info.description = Some(
        "Unofficial third-party HTTP API for ONES issue management. Independently developed; not affiliated with, sponsored, authorized, or endorsed by ONES or its owners.".to_string(),
    );
    openapi.info.version = env!("CARGO_PKG_VERSION").to_string();

    let components = openapi.components.get_or_insert_with(Default::default);
    components.add_security_scheme(
        "onesEmail",
        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-ones-email"))),
    );
    components.add_security_scheme(
        "onesPassword",
        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-ones-password"))),
    );
    openapi.security = Some(vec![
        SecurityRequirement::new("onesEmail", std::iter::empty::<&str>())
            .add("onesPassword", std::iter::empty::<&str>()),
    ]);
}

#[cfg(test)]
mod tests {
    use super::document;

    #[test]
    fn documents_all_http_capabilities() {
        let document = document();
        let paths = document["paths"].as_object().unwrap();
        let expected = [
            "/api/workbench/views",
            "/api/workbench/views/{view_uuid}",
            "/api/workbench/views/{view_uuid}/groups",
            "/api/workbench/views/{view_uuid}/issues",
            "/api/issues/by-display-id/{display_id}",
            "/api/issues/{issue_uuid}",
            "/api/issues/{issue_uuid}/attachments",
            "/api/issues/{issue_uuid}/code-links",
            "/api/issues/{issue_uuid}/branches",
            "/api/issues/{issue_uuid}/transitions",
            "/api/issues/{issue_uuid}/transition-preview",
            "/api/issues/{issue_uuid}/transition",
            "/api/issues/{issue_uuid}/activity",
        ];

        assert_eq!(paths.len(), expected.len());
        for path in expected {
            assert!(paths.contains_key(path), "missing OpenAPI path {path}");
        }
        assert_eq!(
            document["components"]["securitySchemes"]["onesEmail"]["name"],
            "x-ones-email"
        );
        assert_eq!(
            document["components"]["securitySchemes"]["onesPassword"]["name"],
            "x-ones-password"
        );
    }
}
