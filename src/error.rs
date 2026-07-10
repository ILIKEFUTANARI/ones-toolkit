use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("missing config: {0}")]
    MissingConfig(&'static str),
    #[error("missing ONES credentials")]
    MissingAuth,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("upstream request failed: {0}")]
    Upstream(String),
    #[error("upstream {endpoint} returned status {status}: {message}")]
    UpstreamStatus {
        status: u16,
        endpoint: String,
        message: String,
    },
    #[error("failed to parse upstream response: {0}")]
    InvalidUpstream(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl AppError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn invalid_upstream(message: impl Into<String>) -> Self {
        Self::InvalidUpstream(message.into())
    }

    pub fn upstream_status(
        status: reqwest::StatusCode,
        endpoint: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::UpstreamStatus {
            status: status.as_u16(),
            endpoint: endpoint.into(),
            message: message.into(),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingConfig(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingAuth => StatusCode::UNAUTHORIZED,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Upstream(_) | Self::UpstreamStatus { .. } | Self::InvalidUpstream(_) => {
                StatusCode::BAD_GATEWAY
            }
            Self::Http(_) | Self::Json(_) | Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.to_string();
        (status, Json(ErrorBody { error: &message })).into_response()
    }
}
