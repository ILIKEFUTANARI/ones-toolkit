use crate::{
    error::{AppError, AppResult},
    models::AuthContext,
};
use http::HeaderMap;

/// Resolve ONES credentials from request headers.
///
/// The HTTP API and the MCP Streamable HTTP transport share this contract: the
/// `x-ones-*` headers are canonical and the legacy `x-user-*` spellings stay
/// accepted as a fallback.
pub fn auth_from_headers(headers: &HeaderMap) -> AppResult<AuthContext> {
    let email = header_value(headers, "x-ones-email")
        .or_else(|| header_value(headers, "x-user-email"))
        .ok_or(AppError::MissingAuth)?;
    let password = header_value(headers, "x-ones-password")
        .or_else(|| header_value(headers, "x-user-password"))
        .ok_or(AppError::MissingAuth)?;
    Ok(AuthContext { email, password })
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}
